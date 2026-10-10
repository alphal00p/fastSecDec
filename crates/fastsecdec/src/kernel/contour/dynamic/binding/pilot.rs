//! Synchronous pilot work. Only execution of one immutable plan can construct
//! its receipt; accepting it consumes both handles. No production lease or
//! sampling identity is introduced.
use super::{Arc, BTreeMap, Coverage, KernelError, Specification, Symbol};

pub(in crate::kernel) struct PilotSector {
    pub sector_index: usize,
    pub point: Vec<f64>,
    pub specification: Arc<Specification>,
}
pub(in crate::kernel) struct PilotExact {
    pub point: Arc<BTreeMap<Symbol, f64>>,
    pub specification: Arc<Specification>,
}
pub(in crate::kernel) enum PilotWork<'a> {
    Sector(&'a PilotSector),
    Exact(&'a PilotExact),
}
pub(in crate::kernel) struct PilotPlan {
    pub(super) chart_index: usize,
    pub(super) sectors: Vec<PilotSector>,
    pub(super) exact: Option<PilotExact>,
    pub(super) homotopy: bool,
    pub(super) epoch: Arc<()>,
    pub(super) nonce: Arc<()>,
}
pub(in crate::kernel) struct PilotReceipt {
    pub(super) sectors: Vec<(usize, Coverage)>,
    pub(super) exact: Option<Coverage>,
    pub(super) nonce: Arc<()>,
}
impl PilotPlan {
    /// Execute exactly this plan's numerical owners and points. A failure
    /// returns no receipt, so partial work cannot unlock pilot readiness.
    pub fn execute(
        &self,
        mut evaluate: impl FnMut(PilotWork<'_>) -> Result<Coverage, KernelError>,
    ) -> Result<PilotReceipt, KernelError> {
        let sectors = self
            .sectors
            .iter()
            .map(|work| {
                evaluate(PilotWork::Sector(work)).map(|coverage| (work.sector_index, coverage))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let exact = self
            .exact
            .as_ref()
            .map(|work| evaluate(PilotWork::Exact(work)))
            .transpose()?;
        Ok(PilotReceipt {
            sectors,
            exact,
            nonce: self.nonce.clone(),
        })
    }
}
