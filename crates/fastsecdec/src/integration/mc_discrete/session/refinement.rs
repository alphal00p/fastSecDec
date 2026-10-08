use super::*;

impl HavanaDiscreteSession {
    pub(super) fn batch_draws(&self) -> Result<u64> {
        // Native discrete choice uses one extra draw before the continuous
        // child (whose uniform-density branch already has a +1 allowance).
        let dimension = self
            .problem
            .sectors
            .iter()
            .map(|s| s.dimension)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("discrete MC dimension overflow".into()))?;
        mc_draws(self.settings.batch.points_per_batch as u64, dimension)
    }

    /// Continue on the identical frozen discrete/continuous proposal. Earlier
    /// global batch means remain part of the complete correlated Laurent vector.
    pub fn extend_production_batches(&mut self, batches: u32) -> Result<()> {
        if self.stage != IntegrationStage::Production
            || !self.is_complete()
            || !self.pending.is_empty()
            || batches <= self.settings.batch.batches
        {
            return Err(IntegrationError::Invalid("discrete batch extension requires complete drained production and a larger batch target".into()));
        }
        let mut next = self.clone();
        let previous = next.settings.batch.batches;
        next.settings.batch.batches = batches;
        next.settings.validate()?;
        next.epoch = next
            .epoch
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("discrete MC epoch overflow".into()))?;
        next.grid_id = next.identity()?;
        for record in next.records.values_mut() {
            record.task.grid_id = next.grid_id;
        }
        let draws = next.batch_draws()?;
        for _ in previous..batches {
            next.seeds.push(next.streams.reserve(draws)?.1);
        }
        *self = next;
        Ok(())
    }
}
