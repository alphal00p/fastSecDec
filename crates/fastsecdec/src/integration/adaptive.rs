use crate::status::{IntegrationMethod, IntegrationStage};

use super::{IntegrationError, ProductionAllocation, QmcSession, Result, estimate::precise_sum};

impl QmcSession {
    /// Recommend a frozen allocation from a complete, independent pilot.
    ///
    /// Each sector receives `minimum_shifts`. Additional shifts follow
    /// `sqrt(variance/cost)` using scaled vector variance and a budget measured
    /// in summed worker seconds. The pilot lattice size is retained: this
    /// recommendation makes no unsupported extrapolation of lattice convergence.
    /// Real execution time is not guaranteed by a pilot cost estimate.
    pub fn recommend_allocation(
        &self,
        worker_seconds: f64,
        minimum_shifts: u32,
        component_scales: &[f64],
    ) -> Result<Vec<ProductionAllocation>> {
        self.require_complete_pilot()?;
        if !worker_seconds.is_finite()
            || worker_seconds <= 0.0
            || minimum_shifts < 2
            || component_scales.len() != self.problem.orders.len()
            || component_scales.iter().any(|s| !s.is_finite() || *s <= 0.0)
        {
            return Err(IntegrationError::Invalid("allocation needs a positive finite worker-time budget, at least two shifts and positive finite output scales".into()));
        }
        let mut costs = Vec::new();
        let mut variances = Vec::new();
        for run in &self.runs {
            let estimate = run.accumulator.estimate()?;
            let cost = run.seconds()? / estimate.complete_shifts as f64;
            if cost <= 0.0 {
                return Err(IntegrationError::Unavailable(
                    "pilot costs must be measured before automatic allocation".into(),
                ));
            }
            let n = component_scales.len();
            let variance = precise_sum((0..n).map(|i| {
                estimate.covariance_of_mean[i * n + i] * estimate.complete_shifts as f64
                    / component_scales[i]
                    / component_scales[i]
            }))?;
            costs.push(cost);
            variances.push(variance);
        }
        let base_cost = precise_sum(costs.iter().map(|c| c * minimum_shifts as f64))?;
        if worker_seconds < base_cost {
            return Err(IntegrationError::Invalid(
                "worker-time budget is below the minimum full-sector coverage allocation".into(),
            ));
        }
        let norm = precise_sum(variances.iter().zip(&costs).map(|(v, c)| (v * c).sqrt()))?;
        self.problem
            .sectors
            .iter()
            .zip(&self.allocations)
            .zip(variances.iter().zip(&costs))
            .map(|((sector, allocation), (variance, cost))| {
                let extra = if norm == 0.0 {
                    0.0
                } else {
                    ((worker_seconds - base_cost) * (variance / cost).sqrt() / norm).floor()
                };
                if !extra.is_finite() || extra < 0.0 || extra > (u32::MAX - minimum_shifts) as f64 {
                    return Err(IntegrationError::Invalid(
                        "recommended shift count is outside the supported range".into(),
                    ));
                }
                Ok(ProductionAllocation {
                    sector_id: sector.id,
                    points: allocation.points,
                    shifts: minimum_shifts + extra as u32,
                })
            })
            .collect()
    }

    fn require_complete_pilot(&self) -> Result<()> {
        if self.method != IntegrationMethod::AdaptiveQmc
            || self.stage != IntegrationStage::Pilot
            || !self.is_complete()
        {
            return Err(IntegrationError::Unavailable(
                "a complete adaptive pilot is required".into(),
            ));
        }
        Ok(())
    }

    /// Start a new production design with independent sector randomizations.
    /// Pilot observations are discarded from production statistics. No later
    /// production samples can alter this allocation or enter another stage.
    pub fn freeze_production(&mut self, allocations: Vec<ProductionAllocation>) -> Result<()> {
        self.require_complete_pilot()?;
        let epoch = self
            .epoch
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("allocation epoch overflow".into()))?;
        let mut next = Self {
            problem: self.problem.clone(),
            settings: self.settings.clone(),
            method: self.method,
            stage: IntegrationStage::Production,
            epoch,
            allocations,
            runs: Vec::new(),
            cursor: 0,
        };
        next.runs = next.make_runs()?;
        *self = next;
        Ok(())
    }
}
