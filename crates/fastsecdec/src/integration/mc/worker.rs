use std::{fmt::Display, time::Instant};

use numerica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use serde::{Deserialize, Serialize};

use crate::integration::{IntegrationError, McLiveView, Result, mc_live::CenteredAccumulator};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HavanaTask {
    pub(crate) sector_id: u64,
    pub(crate) batch: u32,
    pub(crate) points: usize,
    pub(crate) grid_id: [u8; 32],
    pub(crate) rng_state: [u8; 32],
}

impl HavanaTask {
    pub fn sector_id(&self) -> u64 {
        self.sector_id
    }
    pub fn batch(&self) -> u32 {
        self.batch
    }
    pub fn point_count(&self) -> usize {
        self.points
    }
}

#[derive(Clone, Debug)]
pub struct HavanaReturn {
    pub(crate) task: HavanaTask,
    pub(crate) mean: Vec<f64>,
    pub(crate) worker_seconds: f64,
    pub(crate) training: Option<ContinuousGrid<f64>>,
}

impl HavanaReturn {
    pub fn task(&self) -> &HavanaTask {
        &self.task
    }
    pub fn mean(&self) -> &[f64] {
        &self.mean
    }
    pub fn worker_seconds(&self) -> f64 {
        self.worker_seconds
    }
}

#[derive(Clone, Debug)]
pub struct HavanaWorker {
    pub(crate) sector_id: u64,
    pub(crate) grid_id: [u8; 32],
    pub(crate) grid: ContinuousGrid<f64>,
    pub(crate) points: usize,
    pub(crate) outputs: usize,
    pub(crate) training: bool,
    pub(crate) sample: Sample<f64>,
    pub(crate) values: Vec<f64>,
}

impl HavanaWorker {
    pub fn evaluate<E: Display>(
        &mut self,
        task: HavanaTask,
        mut evaluate: impl FnMut(&[f64], &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaReturn> {
        self.evaluate_with_weight(task, |point, _, output| evaluate(point, output))
    }

    /// Expose the sampled importance weight for evaluation error budgeting.
    /// The callback writes unweighted coefficients; the worker applies this
    /// weight exactly once to statistics and retains native training semantics.
    pub fn evaluate_with_weight<E: Display>(
        &mut self,
        task: HavanaTask,
        evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaReturn> {
        self.evaluate_inner(task, false, evaluate, |_| {})
    }

    /// Accumulate final importance-weighted coefficients from the callback.
    /// This method never applies the sampled weight to those coefficients again.
    ///
    /// During pilot training, the native Havana API needs an unweighted scalar
    /// envelope. Reconstructing it must preserve the weighted envelope within
    /// binary64 roundoff. If that inverse scaling is unrepresentable, return
    /// `IntegrationError::UnrepresentablePilotEnvelope` rather than corrupt training. Production
    /// has no such restriction and uses the supplied weighted vector directly.
    pub fn evaluate_weighted<E: Display>(
        &mut self,
        task: HavanaTask,
        evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaReturn> {
        self.evaluate_inner(task, true, evaluate, |_| {})
    }

    /// Observe native prefix statistics without admitting an incomplete batch.
    pub fn evaluate_weighted_observed<E: Display>(
        &mut self,
        task: HavanaTask,
        evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
        observe: impl FnMut(McLiveView<'_>),
    ) -> Result<HavanaReturn> {
        self.evaluate_inner(task, true, evaluate, observe)
    }
    fn evaluate_inner<E: Display>(
        &mut self,
        task: HavanaTask,
        already_weighted: bool,
        mut evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
        mut observe: impl FnMut(McLiveView<'_>),
    ) -> Result<HavanaReturn> {
        if task.sector_id != self.sector_id
            || task.grid_id != self.grid_id
            || task.points != self.points
            || task.rng_state == [0; 32]
        {
            return Err(IntegrationError::InvalidReturn(
                "Havana task belongs to another integral, grid or production design".into(),
            ));
        }
        let started = Instant::now();
        let mut grid = self.grid.clone_without_samples();
        let mut rng = MonteCarloRng::import(task.rng_state);
        let mut accumulator = CenteredAccumulator::new(self.outputs);
        let mut weighted_values = vec![0.0; self.outputs];
        self.values.resize(self.outputs, f64::NAN);
        for index in 0..task.points {
            grid.sample(&mut rng, &mut self.sample);
            let Sample::Continuous(weight, point) = &self.sample else {
                unreachable!("ContinuousGrid produces continuous samples")
            };
            if !weight.is_finite() || *weight <= 0.0 {
                return Err(IntegrationError::Evaluation(
                    "native Havana sampled weight must be finite and strictly positive".into(),
                ));
            }
            self.values.fill(f64::NAN);
            evaluate(point, *weight, &mut self.values)
                .map_err(|e| IntegrationError::Evaluation(e.to_string()))?;
            if self
                .values
                .iter()
                .any(|v| !v.is_finite() || (!already_weighted && !(v * weight).is_finite()))
            {
                return Err(IntegrationError::Evaluation(
                    "nonfinite importance-weighted coefficient".into(),
                ));
            }
            for (out, value) in weighted_values.iter_mut().zip(&self.values) {
                *out = if already_weighted {
                    *value
                } else {
                    value * weight
                };
            }
            accumulator.add(&weighted_values);
            if self.training {
                let mut envelope = self.values.iter().map(|v| v.abs()).fold(0.0, f64::max);
                if already_weighted {
                    envelope = training_envelope(envelope, *weight)?;
                }
                grid.add_training_sample(&self.sample, envelope)
                    .map_err(IntegrationError::Evaluation)?;
            }
            observe(McLiveView {
                batch: task.batch,
                sector_id: Some(self.sector_id),
                points: index as u64 + 1,
                total: None,
                last_point_timing: None,
                sectors: std::slice::from_ref(&accumulator),
                sector_ids: &[self.sector_id],
            });
        }
        let mean = accumulator.conditional_mean();
        if mean.iter().any(|v| !v.is_finite()) {
            return Err(IntegrationError::Evaluation(
                "nonfinite Monte Carlo batch mean".into(),
            ));
        }
        Ok(HavanaReturn {
            task,
            mean,
            worker_seconds: started.elapsed().as_secs_f64(),
            training: self.training.then_some(grid),
        })
    }
}

pub(crate) fn training_envelope(weighted: f64, weight: f64) -> Result<f64> {
    if weighted == 0.0 {
        return Ok(0.0);
    }
    let unweighted = weighted / weight;
    let restored = unweighted * weight;
    if !unweighted.is_finite()
        || unweighted == 0.0
        || !restored.is_finite()
        || (restored / weighted - 1.0).abs() > 8.0 * f64::EPSILON
    {
        return Err(IntegrationError::UnrepresentablePilotEnvelope {
            weighted_envelope: weighted,
            sampled_weight: weight,
        });
    }
    Ok(unweighted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(training: bool) -> (HavanaWorker, HavanaTask) {
        let mut grid = ContinuousGrid::new(1, 4, 4, None, false).unwrap();
        // A valid nonuniform native grid with a broad bin whose inverse
        // sampling density exceeds 2. The exact sampling algorithm stays native.
        grid.continuous_dimensions[0].partitioning = vec![0.0, 0.001, 0.002, 0.003, 1.0];
        let worker = HavanaWorker {
            sector_id: 4,
            grid_id: [1; 32],
            grid,
            points: 256,
            outputs: 2,
            training,
            sample: Sample::new(),
            values: Vec::new(),
        };
        let task = HavanaTask {
            sector_id: 4,
            batch: 0,
            points: 256,
            grid_id: [1; 32],
            rng_state: MonteCarloRng::new(51, 0).export(),
        };
        (worker, task)
    }

    #[test]
    fn weighted_production_preserves_subnormals_that_cannot_train_unweighted() {
        let tiny = f64::from_bits(1);
        let callback = |_: &[f64], _: f64, values: &mut [f64]| {
            values.copy_from_slice(&[tiny, -2.0 * tiny]);
            Ok::<_, String>(())
        };
        let (mut production, task) = fixture(false);
        let value = production.evaluate_weighted(task, callback).unwrap();
        assert_eq!(value.mean(), [tiny, -2.0 * tiny]);
        let (mut pilot, task) = fixture(true);
        let error = pilot.evaluate_weighted(task, callback).unwrap_err();
        assert!(
            matches!(error, IntegrationError::UnrepresentablePilotEnvelope {
            weighted_envelope, sampled_weight
        } if weighted_envelope == 2.0 * tiny && sampled_weight > 2.0)
        );
    }

    #[test]
    fn weighted_and_unweighted_pilots_train_the_same_native_grid() {
        let (mut original, task) = fixture(true);
        let (mut weighted, _) = fixture(true);
        let ordinary = original
            .evaluate(task.clone(), |point, values| {
                values[0] = 0.1 + point[0].powi(5);
                values[1] = -2.0 * values[0];
                Ok::<_, String>(())
            })
            .unwrap();
        let already_weighted = weighted
            .evaluate_weighted(task, |point, weight, values| {
                values[0] = (0.1 + point[0].powi(5)) * weight;
                values[1] = -2.0 * values[0];
                Ok::<_, String>(())
            })
            .unwrap();
        assert_eq!(ordinary.mean(), already_weighted.mean());
        let mut first = ordinary.training.unwrap();
        let mut second = already_weighted.training.unwrap();
        first.update(0.5);
        second.update(0.5);
        for (a, b) in first.continuous_dimensions[0]
            .partitioning
            .iter()
            .zip(&second.continuous_dimensions[0].partitioning)
        {
            assert!((a - b).abs() < 2e-14);
        }
        // Native diagnostic samples retain their actual proposal weight.
        let Some(Sample::Continuous(first_weight, first_point)) =
            first.accumulator.max_eval_positive_xs
        else {
            panic!("native positive sample missing")
        };
        let Some(Sample::Continuous(second_weight, second_point)) =
            second.accumulator.max_eval_positive_xs
        else {
            panic!("native positive sample missing")
        };
        assert_eq!(first_weight, second_weight);
        assert_eq!(first_point, second_point);
    }

    #[test]
    fn invalid_native_sampling_weight_is_rejected_before_callback() {
        let (mut worker, task) = fixture(true);
        worker.grid.continuous_dimensions[0].partitioning.fill(0.0);
        worker.grid.min_probability_density = 0.0;
        let mut called = false;
        let error = worker
            .evaluate_weighted(task, |_, _, values| {
                called = true;
                values.fill(0.0);
                Ok::<_, String>(())
            })
            .unwrap_err();
        assert!(!called);
        assert!(matches!(error, IntegrationError::Evaluation(message)
            if message.contains("strictly positive")));
    }
}
