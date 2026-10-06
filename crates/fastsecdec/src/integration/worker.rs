use std::{fmt::Display, time::Instant};

use fastsecdec_qmc::{
    Korobov2, Korobov3, QmcPartial, QmcPlan, QmcWorkPackage,
};
use serde::{Deserialize, Serialize};

use super::{IntegrationError, Periodization, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QmcTask {
    pub(crate) content_id: String,
    pub(crate) epoch: u64,
    pub(crate) sector_id: u64,
    pub(crate) periodization: Periodization,
    pub(crate) work: QmcWorkPackage,
}

impl QmcTask {
    pub fn sector_id(&self) -> u64 {
        self.sector_id
    }
    pub fn point_count(&self) -> u64 {
        self.work.point_count()
    }
    pub fn work(&self) -> QmcWorkPackage {
        self.work
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QmcReturn {
    pub(crate) task: QmcTask,
    pub(crate) partial: QmcPartial,
    pub(crate) worker_seconds: f64,
}

impl QmcReturn {
    pub fn from_partial(task: QmcTask, partial: QmcPartial, worker_seconds: f64) -> Result<Self> {
        if !worker_seconds.is_finite() || worker_seconds < 0.0 || task.work != partial.work() {
            return Err(IntegrationError::InvalidReturn(
                "invalid timing or mismatched partial interval".into(),
            ));
        }
        Ok(Self {
            task,
            partial: partial.finish()?,
            worker_seconds,
        })
    }
    pub fn task(&self) -> &QmcTask {
        &self.task
    }
    pub fn worker_seconds(&self) -> f64 {
        self.worker_seconds
    }
}

/// A reusable worker context. Point generation and full-vector evaluation occur
/// here, on whichever thread/process the caller selected. Clone once per worker
/// and sector; no coordinate lattice or worker pool is owned by the session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QmcWorker {
    pub(crate) content_id: String,
    pub(crate) epoch: u64,
    pub(crate) sector_id: u64,
    pub(crate) plan: QmcPlan,
    pub(crate) output_count: usize,
    pub(crate) periodization: Periodization,
    #[serde(skip)]
    pub(crate) point: Vec<f64>,
    #[serde(skip)]
    pub(crate) values: Vec<f64>,
}

impl QmcWorker {
    pub fn plan(&self) -> &QmcPlan {
        &self.plan
    }

    pub fn evaluate<E: Display>(
        &mut self,
        task: QmcTask,
        mut evaluate: impl FnMut(&[f64], &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<QmcReturn> {
        self.evaluate_with_weight(task, |point, _, output| evaluate(point, output))
    }

    /// Expose the known periodization weight for evaluation error budgeting.
    /// The callback writes unweighted coefficients; this worker applies the
    /// supplied weight exactly once after the callback returns.
    pub fn evaluate_with_weight<E: Display>(
        &mut self,
        task: QmcTask,
        evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<QmcReturn> {
        self.evaluate_inner(task, false, evaluate)
    }

    /// Accumulate final weighted coefficients supplied by the callback.
    /// The callback receives the periodization weight and must apply it before
    /// writing the vector. This method never multiplies that vector again.
    pub fn evaluate_weighted<E: Display>(
        &mut self,
        task: QmcTask,
        evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<QmcReturn> {
        self.evaluate_inner(task, true, evaluate)
    }

    fn evaluate_inner<E: Display>(
        &mut self,
        task: QmcTask,
        already_weighted: bool,
        mut evaluate: impl FnMut(&[f64], f64, &mut [f64]) -> std::result::Result<(), E>,
    ) -> Result<QmcReturn> {
        if task.content_id != self.content_id
            || task.epoch != self.epoch
            || task.sector_id != self.sector_id
            || task.periodization != self.periodization
        {
            return Err(IntegrationError::InvalidReturn(
                "task belongs to another integral, sector or allocation".into(),
            ));
        }
        let mut partial = QmcPartial::new(&self.plan, task.work, self.output_count)?;
        self.point.resize(self.plan.dimension(), 0.0);
        self.values.resize(self.output_count, 0.0);
        let started = Instant::now();
        for index in task.work.start()..task.work.start() + task.work.point_count() {
            self.plan.point(index, &mut self.point)?;
            let weight = match self.periodization {
                Periodization::None => 1.0,
                Periodization::Korobov3 => Korobov3::transform_in_place(&mut self.point)?,
                Periodization::Korobov2 => Korobov2::transform_in_place(&mut self.point)?,
            };
            self.values.fill(f64::NAN);
            evaluate(&self.point, weight, &mut self.values)
                .map_err(|error| IntegrationError::Evaluation(error.to_string()))?;
            if !already_weighted {
                for value in &mut self.values {
                    *value *= weight;
                }
            }
            partial.push(&self.values)?;
        }
        QmcReturn::from_partial(task, partial, started.elapsed().as_secs_f64())
    }
}
