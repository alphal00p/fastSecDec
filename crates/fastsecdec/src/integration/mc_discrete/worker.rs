use crate::integration::{IntegrationError, Result, mc::training_envelope};
use numerica::{
    domains::float::{DoubleFloat, RealLike},
    numerical_integration::{DiscreteGrid, MonteCarloRng, Sample, StatisticsAccumulator},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Instant};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HavanaDiscreteTask {
    pub(crate) batch: u32,
    pub(crate) points: usize,
    pub(crate) grid_id: [u8; 32],
    pub(crate) rng_state: [u8; 32],
}
impl HavanaDiscreteTask {
    pub fn batch(&self) -> u32 {
        self.batch
    }
    pub fn point_count(&self) -> usize {
        self.points
    }
}
#[derive(Clone, Debug)]
pub struct HavanaDiscreteReturn {
    pub(crate) task: HavanaDiscreteTask,
    pub(crate) mean: Vec<f64>,
    pub(crate) sector_means: Vec<Vec<f64>>,
    pub(crate) counts: Vec<u64>,
    pub(crate) sector_seconds: Vec<f64>,
    pub(crate) worker_seconds: f64,
    pub(crate) training: Option<DiscreteGrid<f64>>,
}
impl HavanaDiscreteReturn {
    pub fn task(&self) -> &HavanaDiscreteTask {
        &self.task
    }
    pub fn mean(&self) -> &[f64] {
        &self.mean
    }
    /// Actual sector selections in the integration problem's sector order.
    pub fn sector_counts(&self) -> &[u64] {
        &self.counts
    }
    pub fn worker_seconds(&self) -> f64 {
        self.worker_seconds
    }
}
#[derive(Clone, Debug)]
pub struct HavanaDiscreteWorker {
    pub(crate) sector_ids: Vec<u64>,
    pub(crate) grid_id: [u8; 32],
    pub(crate) grid: DiscreteGrid<f64>,
    pub(crate) points: usize,
    pub(crate) outputs: usize,
    pub(crate) training: bool,
}
struct Accumulator {
    values: Vec<StatisticsAccumulator<DoubleFloat>>,
    origin: Vec<f64>,
    count: u64,
}
impl Accumulator {
    fn new(outputs: usize) -> Self {
        Self {
            values: vec![StatisticsAccumulator::new(); outputs],
            origin: vec![0.0; outputs],
            count: 0,
        }
    }
    fn add(&mut self, values: &[f64]) {
        if self.count == 0 {
            self.origin.copy_from_slice(values);
        }
        for ((acc, origin), value) in self.values.iter_mut().zip(&self.origin).zip(values) {
            acc.add_sample(DoubleFloat::from(*value) - DoubleFloat::from(*origin), None);
        }
        self.count += 1;
    }
    fn mean(mut self, global_count: usize) -> Vec<f64> {
        if self.count == 0 {
            return vec![0.0; self.origin.len()];
        }
        self.values
            .iter_mut()
            .zip(self.origin)
            .map(|(acc, origin)| {
                acc.update_iter(false);
                ((DoubleFloat::from(origin) + acc.avg) * DoubleFloat::from(self.count as f64)
                    / DoubleFloat::from(global_count as f64))
                .to_f64()
            })
            .collect()
    }
}
impl HavanaDiscreteWorker {
    pub fn evaluate<E: Display>(
        &mut self,
        task: HavanaDiscreteTask,
        mut evaluate: impl FnMut(u64, &[f64], &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaDiscreteReturn> {
        self.evaluate_inner(task, false, |id, x, _, out| evaluate(id, x, out))
    }
    /// The callback receives the full inverse sector/coordinate probability.
    /// It writes unweighted coefficients; this worker applies the weight once.
    pub fn evaluate_with_weight<E: Display>(
        &mut self,
        task: HavanaDiscreteTask,
        evaluate: impl FnMut(u64, &[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaDiscreteReturn> {
        self.evaluate_inner(task, false, evaluate)
    }
    /// The callback writes final importance-weighted coefficients. The root
    /// native sampling weight is never applied to these values again.
    pub fn evaluate_weighted<E: Display>(
        &mut self,
        task: HavanaDiscreteTask,
        evaluate: impl FnMut(u64, &[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaDiscreteReturn> {
        self.evaluate_inner(task, true, evaluate)
    }
    fn evaluate_inner<E: Display>(
        &mut self,
        task: HavanaDiscreteTask,
        weighted: bool,
        mut evaluate: impl FnMut(u64, &[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<HavanaDiscreteReturn> {
        if task.grid_id != self.grid_id || task.points != self.points || task.rng_state == [0; 32] {
            return Err(IntegrationError::InvalidReturn(
                "discrete Havana task has a different grid or batch design".into(),
            ));
        }
        let started = Instant::now();
        let mut grid = self.grid.clone_without_samples();
        let mut rng = MonteCarloRng::import(task.rng_state);
        let mut sample = Sample::new();
        let mut values = vec![f64::NAN; self.outputs];
        let mut total = Accumulator::new(self.outputs);
        let mut sectors = (0..self.sector_ids.len())
            .map(|_| Accumulator::new(self.outputs))
            .collect::<Vec<_>>();
        let mut counts = vec![0u64; self.sector_ids.len()];
        let mut seconds = vec![0.0; self.sector_ids.len()];
        for _ in 0..task.points {
            let point_started = Instant::now();
            grid.sample(&mut rng, &mut sample);
            let Sample::Discrete(weight, index, Some(child)) = &sample else {
                unreachable!("native nested discrete grid sample")
            };
            let Sample::Continuous(_, point) = child.as_ref() else {
                unreachable!("native continuous child sample")
            };
            if !weight.is_finite() || *weight <= 0.0 {
                return Err(IntegrationError::Evaluation(
                    "native discrete importance weight must be finite and positive".into(),
                ));
            }
            values.fill(f64::NAN);
            evaluate(self.sector_ids[*index], point, *weight, &mut values)
                .map_err(|e| IntegrationError::Evaluation(e.to_string()))?;
            if values
                .iter()
                .any(|value| !value.is_finite() || (!weighted && !(value * weight).is_finite()))
            {
                return Err(IntegrationError::Evaluation(
                    "nonfinite discrete importance-weighted coefficient".into(),
                ));
            }
            if self.training {
                let envelope = values.iter().map(|v| v.abs()).fold(0.0, f64::max);
                grid.add_training_sample(
                    &sample,
                    if weighted {
                        training_envelope(envelope, *weight)?
                    } else {
                        envelope
                    },
                )
                .map_err(IntegrationError::Evaluation)?;
            }
            if !weighted {
                for value in &mut values {
                    *value *= weight;
                }
            }
            total.add(&values);
            sectors[*index].add(&values);
            counts[*index] += 1;
            seconds[*index] += point_started.elapsed().as_secs_f64();
        }
        let mean = total.mean(task.points);
        let sector_means = sectors
            .into_iter()
            .map(|acc| acc.mean(task.points))
            .collect::<Vec<_>>();
        if mean
            .iter()
            .chain(sector_means.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(IntegrationError::Evaluation(
                "nonfinite discrete batch mean".into(),
            ));
        }
        Ok(HavanaDiscreteReturn {
            task,
            mean,
            sector_means,
            counts,
            sector_seconds: seconds,
            worker_seconds: started.elapsed().as_secs_f64(),
            training: self.training.then_some(grid),
        })
    }
}
