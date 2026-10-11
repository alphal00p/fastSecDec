//! The original source of one explicitly boundary-free auxiliary recursion.
//! Its history is not an ambient ancestor's physical divisor history.
use super::*;
#[derive(Clone, Debug)]
pub struct AuxiliaryRecursionOrigin {
    frame: Arc<EtaleFrame>,
    source: Arc<MarkedIdeal>,
    history: Arc<ResolutionHistory>,
}
impl AuxiliaryRecursionOrigin {
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    // Called only by the retained original source before its first descent.
    pub(super) fn prepare(
        frame: Arc<EtaleFrame>,
        source: Arc<MarkedIdeal>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        let ledger = match verify_initial_relative_snc(frame.clone(), vec![], b)? {
            SncProduction::Verified(ledger) => ledger,
            SncProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            SncProduction::Unresolved { .. } => {
                return Err(Error::Invalid("auxiliary original empty boundary"));
            }
        };
        let history = ResolutionHistory::initial(ledger)?;
        Ok(Arc::new(Self {
            frame,
            source,
            history,
        }))
    }
}
