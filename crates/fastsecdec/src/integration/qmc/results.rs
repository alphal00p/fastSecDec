use super::*;
use crate::{
    integration::VectorEstimate,
    status::{IntegrationSnapshot, SectorSnapshot, UncertaintyStatus},
};
use numerica::numerical_integration::qmc::QmcEstimate;

impl QmcSession {
    pub fn estimate(&self) -> Result<VectorEstimate> {
        if self.stage == IntegrationStage::Pilot {
            return Err(IntegrationError::Unavailable(
                "pilot samples are not production evidence".into(),
            ));
        }
        let n = self.problem.orders.len();
        if self.runs.is_empty() {
            return Ok(VectorEstimate {
                orders: self.problem.orders.clone(),
                components: self.problem.components.clone(),
                mean: self.problem.exact_coefficients.clone(),
                standard_error: vec![0.0; n],
                covariance_of_mean: vec![0.0; n * n],
                production_complete: true,
            });
        }
        let value = if self.method == IntegrationMethod::DemocraticQmc {
            let means: Vec<_> = self
                .runs
                .iter()
                .map(|r| r.accumulator.shift_estimates())
                .collect::<std::result::Result<_, _>>()?;
            let mut complete_rows = Vec::new();
            for first in &means[0] {
                let rows: Option<Vec<_>> = means
                    .iter()
                    .map(|row| row.iter().find(|v| v.shift == first.shift))
                    .collect();
                if let Some(rows) = rows {
                    complete_rows.push(rows);
                }
            }
            let Some(anchor) = complete_rows.first() else {
                return Err(
                    numerica::numerical_integration::qmc::QmcError::InsufficientShifts {
                        complete: 0,
                    }
                    .into(),
                );
            };
            // Center each sector before aggregation. Exact constants enter the
            // absolute mean only, preserving both small covariance and large
            // cancellations involving whole zero-dimensional sectors.
            let aggregate = complete_rows
                .iter()
                .map(|rows| {
                    (0..n)
                        .map(|j| {
                            precise_sum(rows.iter().zip(anchor).map(|(v, a)| v.mean[j] - a.mean[j]))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?;
            let mut value = QmcEstimate::from_shift_means(&aggregate)?;
            for (j, mean) in value.mean.iter_mut().enumerate() {
                *mean = precise_sum(
                    anchor
                        .iter()
                        .map(|a| a.mean[j])
                        .chain([self.problem.exact_coefficients[j], *mean]),
                )?;
            }
            value
        } else {
            let estimates: Vec<_> = self
                .runs
                .iter()
                .map(|r| r.accumulator.estimate())
                .collect::<std::result::Result<_, _>>()?;
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
            QmcEstimate {
                mean,
                standard_error,
                covariance_of_mean,
                complete_shifts: 0,
                completed_points: 0,
                used_points: 0,
            }
        };
        Ok(VectorEstimate::from_qmc(
            &self.problem.orders,
            &self.problem.components,
            value,
            self.is_complete(),
        ))
    }

    pub fn snapshot(&self) -> Result<IntegrationSnapshot> {
        let sectors = self
            .problem
            .sectors
            .iter()
            .zip(&self.runs)
            .map(|(spec, run)| {
                Ok(SectorSnapshot {
                    id: spec.id,
                    dimension: spec.dimension,
                    completed_points: run.accumulator.completed_points(),
                    planned_points: run.accumulator.plan().total_points(),
                    complete_replicas: run.accumulator.shift_estimates()?.len(),
                    planned_replicas: run.accumulator.plan().shift_count(),
                    worker_seconds: run.seconds()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let estimate = match self.estimate() {
            Ok(value) => Some(value),
            Err(IntegrationError::Unavailable(_))
            | Err(IntegrationError::Qmc(
                numerica::numerical_integration::qmc::QmcError::InsufficientShifts { .. },
            )) => None,
            Err(error) => return Err(error),
        };
        let uncertainty = if self.runs.is_empty() {
            UncertaintyStatus::Exact
        } else if self.stage == IntegrationStage::Pilot {
            UncertaintyStatus::PilotOnly
        } else if estimate.is_some() {
            UncertaintyStatus::Available
        } else {
            UncertaintyStatus::WaitingForCoverage
        };
        Ok(IntegrationSnapshot {
            method: self.method,
            stage: self.stage,
            completed_points: checked_count(sectors.iter().map(|s| s.completed_points))?,
            planned_points: checked_count(sectors.iter().map(|s| s.planned_points))?,
            complete_sectors: self
                .runs
                .iter()
                .filter(|r| r.accumulator.is_complete())
                .count(),
            worker_seconds: precise_sum(sectors.iter().map(|s| s.worker_seconds))?,
            sectors,
            uncertainty,
            estimate,
            stop_reason: None,
            evaluation_diagnostics: None,
        })
    }
}

fn checked_count(mut values: impl Iterator<Item = u64>) -> Result<u64> {
    values.try_fold(0u64, |sum, value| {
        sum.checked_add(value)
            .ok_or_else(|| IntegrationError::Invalid("total point count overflow".into()))
    })
}
