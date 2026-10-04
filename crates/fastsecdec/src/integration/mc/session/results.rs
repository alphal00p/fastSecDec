use super::*;
use crate::{
    integration::{VectorEstimate, estimate::precise_sum},
    status::{IntegrationMethod, IntegrationSnapshot, SectorSnapshot, UncertaintyStatus},
};
use numerica::numerical_integration::qmc::QmcEstimate;

impl HavanaSession {
    pub fn estimate(&self) -> Result<VectorEstimate> {
        if self.stage == IntegrationStage::Pilot {
            return Err(IntegrationError::Unavailable(
                "Havana pilot observations are not production samples".into(),
            ));
        }
        let n = self.problem.orders.len();
        let estimates = self
            .runs
            .iter()
            .map(|run| {
                let means = run
                    .records
                    .values()
                    .map(|r| r.mean.clone())
                    .collect::<Vec<_>>();
                QmcEstimate::from_shift_means(&means).map_err(|error| match error {
                    numerica::numerical_integration::qmc::QmcError::InsufficientShifts {
                        ..
                    } => IntegrationError::Unavailable(
                        "each Havana sector needs at least two complete independent batches".into(),
                    ),
                    error => IntegrationError::Invalid(format!(
                        "Monte Carlo batch statistics failed: {error}"
                    )),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mean = (0..n)
            .map(|j| {
                precise_sum(
                    estimates
                        .iter()
                        .map(|v| v.mean[j])
                        .chain([self.problem.exact_coefficients[j]]),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let covariance_of_mean = (0..n * n)
            .map(|j| precise_sum(estimates.iter().map(|v| v.covariance_of_mean[j])))
            .collect::<Result<Vec<_>>>()?;
        let standard_error = (0..n)
            .map(|j| covariance_of_mean[j * n + j].sqrt())
            .collect();
        Ok(VectorEstimate {
            orders: self.problem.orders.clone(),
            components: self.problem.components.clone(),
            mean,
            standard_error,
            covariance_of_mean,
            production_complete: self.is_complete(),
        })
    }

    pub fn snapshot(&self) -> Result<IntegrationSnapshot> {
        let estimate = match self.estimate() {
            Ok(value) => Some(value),
            Err(IntegrationError::Unavailable(_))
            | Err(IntegrationError::Qmc(
                numerica::numerical_integration::qmc::QmcError::InsufficientShifts { .. },
            )) => None,
            Err(error) => return Err(error),
        };
        let sectors = self
            .problem
            .sectors
            .iter()
            .zip(&self.runs)
            .map(|(spec, run)| {
                Ok(SectorSnapshot {
                    id: spec.id,
                    dimension: spec.dimension,
                    completed_points: run.records.len() as u64
                        * self.settings.points_per_batch as u64,
                    planned_points: self.settings.planned_points()?,
                    complete_replicas: run.records.len(),
                    planned_replicas: self.settings.batches as usize,
                    worker_seconds: precise_sum(run.records.values().map(|v| v.worker_seconds))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let total = |values: Vec<u64>| {
            values.into_iter().try_fold(0u64, |sum, value| {
                sum.checked_add(value).ok_or_else(|| {
                    IntegrationError::Invalid("Havana total point count overflow".into())
                })
            })
        };
        Ok(IntegrationSnapshot {
            method: IntegrationMethod::HavanaMc,
            stage: self.stage,
            completed_points: total(sectors.iter().map(|s| s.completed_points).collect())?,
            planned_points: total(sectors.iter().map(|s| s.planned_points).collect())?,
            complete_sectors: self
                .runs
                .iter()
                .filter(|r| r.records.len() == self.settings.batches as usize)
                .count(),
            worker_seconds: precise_sum(sectors.iter().map(|s| s.worker_seconds))?,
            uncertainty: if self.runs.is_empty() {
                UncertaintyStatus::Exact
            } else if self.stage == IntegrationStage::Pilot {
                UncertaintyStatus::PilotOnly
            } else if estimate.is_some() {
                UncertaintyStatus::Available
            } else {
                UncertaintyStatus::WaitingForCoverage
            },
            sectors,
            estimate,
            stop_reason: None,
            evaluation_diagnostics: None,
        })
    }
}
