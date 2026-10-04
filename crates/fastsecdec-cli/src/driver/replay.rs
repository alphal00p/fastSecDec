//! Only successfully submitted complete packages advance checkpointed replay
//! state. Worker-local failed prefixes are discarded with their contexts.
use crate::CliResult;
use fastsecdec::kernel::{KernelSet, ReplayPolicy, ReplayState, WeightedEvaluationContext};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AcceptedReplay {
    policy: ReplayPolicy,
    states: Vec<ReplayState>,
}

impl AcceptedReplay {
    pub(super) fn new(kernels: &KernelSet, policy: ReplayPolicy) -> CliResult<Self> {
        policy.validate()?;
        Ok(Self {
            states: (0..kernels.sectors().len())
                .map(|sector| kernels.replay_state(sector, policy.clone()))
                .collect::<Result<_, _>>()?,
            policy,
        })
    }

    pub(super) fn validate(&self, kernels: &KernelSet, policy: &ReplayPolicy) -> CliResult<()> {
        policy.validate()?;
        if &self.policy != policy || self.states.len() != kernels.sectors().len() {
            return Err("checkpoint replay policy or sector count differs".into());
        }
        for (sector, state) in self.states.iter().enumerate() {
            kernels.validate_replay_state(sector, policy, state)?;
        }
        Ok(())
    }

    pub(super) fn contexts(
        &self,
        kernels: &KernelSet,
        sectors: &[fastsecdec::integration::SectorSpec],
    ) -> CliResult<std::collections::BTreeMap<u64, WeightedEvaluationContext>> {
        sectors
            .iter()
            .map(|sector| {
                let index = usize::try_from(sector.id)?;
                let state = self.states.get(index).ok_or("unknown replay sector")?;
                Ok((
                    sector.id,
                    kernels.restore_evaluation_context(index, self.policy.clone(), state)?,
                ))
            })
            .collect()
    }

    pub(super) fn state(&self, sector: usize) -> &ReplayState {
        &self.states[sector]
    }

    /// The caller must submit the numerical package successfully first.
    pub(super) fn accept(&mut self, sector: usize, state: &ReplayState) -> CliResult<()> {
        self.states
            .get_mut(sector)
            .ok_or("unknown replay sector")?
            .merge(state)?;
        Ok(())
    }

    pub(super) fn validate_candidate(&self, sector: usize, state: &ReplayState) -> CliResult<()> {
        self.states
            .get(sector)
            .ok_or("unknown replay sector")?
            .clone()
            .merge(state)?;
        Ok(())
    }
}
