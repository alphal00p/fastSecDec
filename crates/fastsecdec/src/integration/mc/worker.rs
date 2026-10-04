use std::{fmt::Display, time::Instant};

use numerica::{
    domains::float::{DoubleFloat, RealLike},
    numerical_integration::{ContinuousGrid, MonteCarloRng, Sample, StatisticsAccumulator},
};
use serde::{Deserialize, Serialize};

use crate::integration::{IntegrationError, Result};

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
        let mut accumulators = vec![StatisticsAccumulator::<DoubleFloat>::new(); self.outputs];
        let mut origin = vec![0.0; self.outputs];
        self.values.resize(self.outputs, f64::NAN);
        for index in 0..task.points {
            grid.sample(&mut rng, &mut self.sample);
            let Sample::Continuous(weight, point) = &self.sample else {
                unreachable!("ContinuousGrid produces continuous samples")
            };
            self.values.fill(f64::NAN);
            evaluate(point, &mut self.values)
                .map_err(|e| IntegrationError::Evaluation(e.to_string()))?;
            if !weight.is_finite()
                || self
                    .values
                    .iter()
                    .any(|v| !v.is_finite() || !(v * weight).is_finite())
            {
                return Err(IntegrationError::Evaluation(
                    "nonfinite importance-weighted coefficient".into(),
                ));
            }
            for (j, (value, accumulator)) in self.values.iter().zip(&mut accumulators).enumerate() {
                let weighted = value * weight;
                if index == 0 {
                    origin[j] = weighted;
                }
                accumulator.add_sample(
                    DoubleFloat::from(weighted) - DoubleFloat::from(origin[j]),
                    None,
                );
            }
            if self.training {
                let envelope = self.values.iter().map(|v| v.abs()).fold(0.0, f64::max);
                grid.add_training_sample(&self.sample, envelope)
                    .map_err(IntegrationError::Evaluation)?;
            }
        }
        let mean = accumulators
            .iter_mut()
            .zip(origin)
            .map(|(accumulator, origin)| {
                accumulator.update_iter(false);
                (DoubleFloat::from(origin) + accumulator.avg).to_f64()
            })
            .collect::<Vec<_>>();
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
