//! One active native evaluator per caller-owned worker. Sector switches discard
//! compiled buffers and precision caches; accepted replay envelopes live outside
//! the slot and survive eviction. Revisits may rebuild caches, so this bounds
//! ownership without promising a speed improvement.
use crate::{CliResult, driver::replay::AcceptedReplay};
use fastsecdec::{
    integration::{QmcSession, QmcWorker},
    kernel::{KernelSet, WeightedEvaluationContext},
};

#[derive(Default)]
pub(super) struct QmcSlot {
    pub(super) active: Option<ActiveSector>,
}

pub(super) struct ActiveSector {
    id: u64,
    pub(super) worker: QmcWorker,
    pub(super) context: WeightedEvaluationContext,
}

impl QmcSlot {
    pub(super) fn prepare(
        &mut self,
        id: u64,
        kernels: &KernelSet,
        session: &QmcSession,
        replay: &AcceptedReplay,
    ) -> CliResult<()> {
        if self.active.as_ref().is_none_or(|active| active.id != id) {
            // Release the previous large clone before allocating its successor.
            // A failed replacement leaves this slot empty.
            self.active = None;
            let worker = session.worker_context(id)?;
            let context = replay.context(kernels, id)?;
            self.active = Some(ActiveSector {
                id,
                worker,
                context,
            });
        } else {
            self.active
                .as_mut()
                .unwrap()
                .context
                .merge_state(replay.state(usize::try_from(id)?))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
