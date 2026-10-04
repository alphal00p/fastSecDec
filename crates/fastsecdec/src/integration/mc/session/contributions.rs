use super::*;
use crate::integration::{ContributionReport, ReplicaRelation, SectorContribution};

impl HavanaSession {
    /// Additive independent-batch contributions, with pilot and incomplete
    /// production coverage kept explicit. Exact coefficients occur only once.
    pub fn contributions(&self) -> Result<ContributionReport> {
        let snapshot = self.snapshot()?;
        let sectors = self
            .runs
            .iter()
            .zip(&snapshot.sectors)
            .map(|(run, progress)| {
                let means = if self.stage == IntegrationStage::Pilot {
                    Vec::new()
                } else {
                    run.records
                        .values()
                        .map(|record| record.mean.clone())
                        .collect()
                };
                SectorContribution::from_replicas(
                    &self.problem,
                    self.stage,
                    progress.clone(),
                    &means,
                    self.settings.points_per_batch as u64,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ContributionReport::from_snapshot(
            &self.problem,
            snapshot,
            ReplicaRelation::IndependentAcrossSectors,
            sectors,
        ))
    }
}
