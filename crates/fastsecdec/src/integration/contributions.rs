use numerica::numerical_integration::qmc::QmcEstimate;
use serde::{Deserialize, Serialize};

use super::{IntegrationError, IntegrationProblem, Result, VectorEstimate};
use crate::status::{
    CoefficientComponent, IntegrationMethod, IntegrationSnapshot, IntegrationStage, SectorSnapshot,
    UncertaintyStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaRelation {
    /// Marginal sector covariance matrices cannot be summed to get total error.
    SharedAcrossSectors,
    IndependentAcrossSectors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectorContribution {
    pub progress: SectorSnapshot,
    /// Replicas selected for this view, even if fewer than two are available.
    /// Democratic QMC excludes complete shifts missing in another sector.
    pub used_replicas: usize,
    pub used_points: u64,
    pub uncertainty: UncertaintyStatus,
    /// Marginal contribution, without any share of the exact coefficient vector.
    pub estimate: Option<VectorEstimate>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContributionReport {
    pub method: IntegrationMethod,
    pub stage: IntegrationStage,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub exact_coefficients: Vec<f64>,
    pub replica_relation: ReplicaRelation,
    pub sectors: Vec<SectorContribution>,
    pub uncertainty: UncertaintyStatus,
    /// Authoritative total from the session's existing centered estimator.
    /// Do not reconstruct it from rounded absolute marginal means.
    pub total: Option<VectorEstimate>,
}

impl SectorContribution {
    pub(crate) fn from_replicas(
        problem: &IntegrationProblem,
        stage: IntegrationStage,
        progress: SectorSnapshot,
        means: &[Vec<f64>],
        points_per_replica: u64,
    ) -> Result<Self> {
        let used_replicas = if stage == IntegrationStage::Pilot {
            0
        } else {
            means.len()
        };
        let used_points = (used_replicas as u64)
            .checked_mul(points_per_replica)
            .ok_or_else(|| {
                IntegrationError::Invalid("sector contribution point count overflow".into())
            })?;
        let estimate = if used_replicas < 2 {
            None
        } else {
            Some(VectorEstimate::from_qmc(
                &problem.orders,
                &problem.components,
                QmcEstimate::from_shift_means(means)?,
                used_replicas == progress.planned_replicas
                    && progress.completed_points == progress.planned_points,
            ))
        };
        let uncertainty = if stage == IntegrationStage::Pilot {
            UncertaintyStatus::PilotOnly
        } else if estimate.is_some() {
            UncertaintyStatus::Available
        } else {
            UncertaintyStatus::WaitingForCoverage
        };
        Ok(Self {
            progress,
            used_replicas,
            used_points,
            uncertainty,
            estimate,
        })
    }
}

impl ContributionReport {
    pub(crate) fn from_snapshot(
        problem: &IntegrationProblem,
        snapshot: IntegrationSnapshot,
        replica_relation: ReplicaRelation,
        sectors: Vec<SectorContribution>,
    ) -> Self {
        Self {
            method: snapshot.method,
            stage: snapshot.stage,
            orders: problem.orders.clone(),
            components: problem.components.clone(),
            exact_coefficients: problem.exact_coefficients.clone(),
            replica_relation,
            sectors,
            uncertainty: snapshot.uncertainty,
            total: snapshot.estimate,
        }
    }
}

impl std::fmt::Display for ContributionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_rows(f, self.sectors.iter())
    }
}

impl ContributionReport {
    pub(crate) fn fmt_rows<'a>(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        rows: impl IntoIterator<Item = &'a SectorContribution>,
    ) -> std::fmt::Result {
        let n = self.orders.len();
        let covariance_size = n.checked_mul(n);
        let consistent = n > 0
            && self.components.len() == n
            && self.exact_coefficients.len() == n
            && self
                .sectors
                .iter()
                .filter_map(|row| row.estimate.as_ref())
                .chain(self.total.as_ref())
                .all(|estimate| {
                    estimate.orders == self.orders
                        && estimate.components == self.components
                        && estimate.mean.len() == n
                        && estimate.standard_error.len() == n
                        && covariance_size == Some(estimate.covariance_of_mean.len())
                });
        if !consistent {
            return f.write_str("Sector contributions: invalid coefficient layout");
        }
        writeln!(
            f,
            "Sector contributions: {:?} {:?}; {} sectors",
            self.method,
            self.stage,
            self.sectors.len()
        )?;
        if self.replica_relation == ReplicaRelation::SharedAcrossSectors {
            writeln!(
                f,
                "Shared replicas: marginal sector covariance cannot be summed to obtain total covariance."
            )?;
        }
        for row in rows {
            writeln!(
                f,
                "  sector {}: {}/{} complete replicas used; {}/{} points used",
                row.progress.id,
                row.used_replicas,
                row.progress.complete_replicas,
                row.used_points,
                row.progress.completed_points
            )?;
            if let Some(estimate) = &row.estimate {
                for (i, (&order, component)) in self.orders.iter().zip(&self.components).enumerate()
                {
                    writeln!(
                        f,
                        "    {component:?} eps^{order}: {:.10e} +/- {:.3e} (marginal)",
                        estimate.mean[i], estimate.standard_error[i]
                    )?;
                }
            } else {
                writeln!(f, "    {:?}", row.uncertainty)?;
            }
        }
        writeln!(
            f,
            "Exact contribution is recorded separately; use the authoritative total for final uncertainty."
        )
    }
}
