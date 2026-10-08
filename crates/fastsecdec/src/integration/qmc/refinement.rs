use super::*;
use fastsecdec_qmc::QmcWorkPackage;

impl SectorRun {
    pub(super) fn canonical_work(&self, start: u64, package_points: u64) -> Result<QmcWorkPackage> {
        let interval = self
            .package_boundaries
            .windows(2)
            .find(|range| range[0] <= start && start < range[1])
            .ok_or_else(|| {
                IntegrationError::InvalidReturn("package outside its allocation blocks".into())
            })?;
        if !(start - interval[0]).is_multiple_of(package_points) {
            return Err(IntegrationError::InvalidReturn(
                "package start is not canonical within its allocation block".into(),
            ));
        }
        Ok(self
            .accumulator
            .plan()
            .work(start, package_points.min(interval[1] - start))?)
    }
}

impl QmcSession {
    /// Enlarge a complete frozen production allocation by appending new shifts.
    /// Existing shifts and their full vector covariance remain in the estimate;
    /// adaptive pilots are not rerun and democratic sector shifts stay shared.
    /// This changes neither lattice size nor periodization.
    pub fn extend_production_shifts(&mut self, factor: u32) -> Result<()> {
        if factor < 2
            || self.stage != IntegrationStage::Production
            || !self.is_complete()
            || self.runs.iter().any(|run| !run.pending.is_empty())
        {
            return Err(IntegrationError::Invalid(
                "shift extension requires complete drained production and a factor of at least two"
                    .into(),
            ));
        }
        let mut next = self.clone();
        next.settings.shifts = next
            .settings
            .shifts
            .checked_mul(factor)
            .ok_or_else(|| IntegrationError::Invalid("QMC shift target overflow".into()))?;
        next.epoch = next
            .epoch
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("QMC allocation epoch overflow".into()))?;
        for index in 0..next.runs.len() {
            let allocation = &mut next.allocations[index];
            allocation.shifts = allocation
                .shifts
                .checked_mul(factor)
                .ok_or_else(|| IntegrationError::Invalid("sector shift target overflow".into()))?;
            let points = allocation.points;
            let shifts = allocation.shifts;
            let plan = next.settings.plan(
                next.problem.sectors[index].dimension,
                points,
                shifts,
                next.stream(index),
            )?;
            let end = plan.total_points();
            next.runs[index].accumulator.extend_plan(plan)?;
            next.runs[index].package_boundaries.push(end);
        }
        next.cursor = 0;
        *self = next;
        Ok(())
    }
}
