use super::*;

impl HavanaSession {
    /// Append independent equal-size production batches to the current frozen
    /// native grid, retaining every accepted vector mean and covariance input.
    /// A new stream is reserved from the persisted frontier for every batch.
    pub fn extend_production_batches(&mut self, batches: u32) -> Result<()> {
        if self.stage != IntegrationStage::Production
            || !self.is_complete()
            || self.runs.iter().any(|run| !run.pending.is_empty())
            || batches <= self.settings.batches
        {
            return Err(IntegrationError::Invalid(
                "batch extension requires complete drained production and a larger batch target"
                    .into(),
            ));
        }
        let mut next = self.clone();
        let previous = self.settings.batches;
        next.settings.batches = batches;
        next.settings.validate()?;
        next.epoch = next
            .epoch
            .checked_add(1)
            .ok_or_else(|| IntegrationError::Invalid("Havana epoch overflow".into()))?;
        for index in 0..next.runs.len() {
            let draws = mc_draws(
                next.settings.points_per_batch as u64,
                next.problem.sectors[index].dimension,
            )?;
            let id = next.grid_id(index, &next.runs[index].grid)?;
            let run = &mut next.runs[index];
            run.grid_id = id;
            for record in run.records.values_mut() {
                record.task.grid_id = id;
            }
            for _ in previous..batches {
                run.seeds.push(next.streams.reserve(draws)?.1);
            }
        }
        next.cursor = 0;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_frontier_refuses_append_without_changing_frozen_production() {
        let problem = IntegrationProblem::new(
            "frontier-limit".into(),
            vec![0],
            vec![crate::integration::SectorSpec {
                id: 0,
                dimension: 1,
            }],
            vec![0.],
        )
        .unwrap();
        let mut session = HavanaSession::production(
            problem,
            HavanaSettings {
                points_per_batch: 4,
                batches: 2,
                bins: 2,
                ..Default::default()
            },
        )
        .unwrap();
        while let Some(task) = session.next_work() {
            let value = session
                .worker_context(0)
                .unwrap()
                .evaluate(task, |x, out| {
                    out[0] = x[0];
                    Ok::<_, String>(())
                })
                .unwrap();
            session.submit(value).unwrap();
        }
        session.streams.next = u64::MAX;
        let before = session.checkpoint().unwrap();
        assert!(session.extend_production_batches(4).is_err());
        assert_eq!(before, session.checkpoint().unwrap());
    }
}
