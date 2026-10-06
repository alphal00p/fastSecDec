use super::*;
#[derive(Serialize, Deserialize)]
struct Checkpoint {
    version: u8,
    problem: IntegrationProblem,
    settings: HavanaDiscreteSettings,
    epoch: u64,
    next_rng: [u8; 32],
    seeds: Vec<[u8; 32]>,
    proposal: Vec<(f64, Vec<Vec<f64>>)>,
    records: Vec<BatchRecord>,
}
impl HavanaDiscreteSession {
    /// Resume frozen production without serializing native mutable training
    /// internals. A pilot can be repeated, but is never restored as production.
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        if self.stage != IntegrationStage::Production {
            return Err(IntegrationError::Unavailable(
                "discrete Havana checkpoints require frozen production".into(),
            ));
        }
        Ok(serde_json::to_vec(&Checkpoint {
            version: 1,
            problem: self.problem.clone(),
            settings: self.settings.clone(),
            epoch: self.epoch,
            next_rng: self.next_rng,
            seeds: self.seeds.clone(),
            proposal: self.proposal(),
            records: self.records.values().cloned().collect(),
        })?)
    }
    pub fn restore(bytes: &[u8], expected_problem: &IntegrationProblem) -> Result<Self> {
        let stored: Checkpoint = serde_json::from_slice(bytes)?;
        if stored.version != 1
            || &stored.problem != expected_problem
            || stored.next_rng == [0; 32]
            || stored.proposal.len() != stored.problem.sectors.len()
            || stored.seeds.len() != stored.settings.batch.batches as usize
        {
            return Err(IntegrationError::Invalid(
                "discrete checkpoint identity or shape differs".into(),
            ));
        }
        let mut seen = BTreeSet::new();
        if stored
            .seeds
            .iter()
            .any(|s| *s == [0; 32] || !seen.insert(*s))
        {
            return Err(IntegrationError::Invalid(
                "invalid or repeated discrete random stream".into(),
            ));
        }
        let mut session = Self::production(stored.problem, stored.settings)?;
        let sum = crate::integration::estimate::precise_sum(stored.proposal.iter().map(|x| x.0))?;
        if !stored.proposal.is_empty() && (sum - 1.0).abs() > 16.0 * f64::EPSILON {
            return Err(IntegrationError::Invalid(
                "checkpoint probabilities do not sum to one".into(),
            ));
        }
        if let Some(grid) = &mut session.grid {
            for (i, (bin, (probability, partitions))) in
                grid.bins.iter_mut().zip(stored.proposal).enumerate()
            {
                if !probability.is_finite()
                    || probability <= 0.0
                    || partitions.len() != session.problem.sectors[i].dimension
                {
                    return Err(IntegrationError::Invalid(
                        "invalid checkpoint sector proposal".into(),
                    ));
                }
                bin.pdf = probability;
                let Some(Grid::Continuous(child)) = &mut bin.sub_grid else {
                    unreachable!()
                };
                for (dimension, partition) in child.continuous_dimensions.iter_mut().zip(partitions)
                {
                    if partition.len() != session.settings.batch.bins + 1
                        || partition.first() != Some(&0.0)
                        || partition.last() != Some(&1.0)
                        || partition
                            .windows(2)
                            .any(|w| !w[0].is_finite() || w[0] >= w[1])
                    {
                        return Err(IntegrationError::Invalid(
                            "invalid checkpoint continuous partition".into(),
                        ));
                    }
                    dimension.partitioning = partition;
                }
            }
        }
        session.epoch = stored.epoch;
        session.next_rng = stored.next_rng;
        session.seeds = stored.seeds;
        session.grid_id = session.identity()?;
        session.training = session
            .grid
            .as_ref()
            .map(clone_without_samples)
            .transpose()?;
        for record in stored.records {
            session.validate_record(&record)?;
            if session.records.insert(record.task.batch, record).is_some() {
                return Err(IntegrationError::Invalid(
                    "duplicate discrete batch record".into(),
                ));
            }
        }
        Ok(session)
    }
}
