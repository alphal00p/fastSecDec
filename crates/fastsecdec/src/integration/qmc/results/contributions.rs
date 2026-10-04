use super::*;
use crate::integration::{ContributionReport, ReplicaRelation, SectorContribution};

impl QmcSession {
    /// Additive marginal estimates with explicit replica dependence and coverage.
    /// Democratic rows use precisely the common complete shifts selected by the
    /// total estimator. Pilot observations never become production estimates.
    pub fn contributions(&self) -> Result<ContributionReport> {
        let snapshot = self.snapshot()?;
        let shared = self.method == IntegrationMethod::DemocraticQmc;
        let common = if shared && self.stage == IntegrationStage::Production {
            self.common_shift_rows()?
        } else {
            Vec::new()
        };
        let sectors = self
            .runs
            .iter()
            .zip(&snapshot.sectors)
            .enumerate()
            .map(|(index, (run, progress))| {
                let means = if self.stage == IntegrationStage::Pilot {
                    Vec::new()
                } else if shared {
                    common
                        .iter()
                        .map(|replica| replica[index].mean.clone())
                        .collect()
                } else {
                    run.accumulator
                        .shift_estimates()?
                        .into_iter()
                        .map(|row| row.mean)
                        .collect()
                };
                SectorContribution::from_replicas(
                    &self.problem,
                    self.stage,
                    progress.clone(),
                    &means,
                    run.accumulator.plan().rule().points(),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ContributionReport::from_snapshot(
            &self.problem,
            snapshot,
            if shared {
                ReplicaRelation::SharedAcrossSectors
            } else {
                ReplicaRelation::IndependentAcrossSectors
            },
            sectors,
        ))
    }
}
