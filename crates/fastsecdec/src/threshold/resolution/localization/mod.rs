mod cover;
mod geometry;
pub use cover::{
    CoverLocalizationProduction, LocalizedCoverHistory, VerifiedCoverOpen,
    localize_verified_cover_open,
};
// Checked component principal opens. This is localization with unit coordinate
// Jacobian; inverse presentation slots are not integration coordinates.
use super::{
    Budget, Error, EtaleFrame, Poly, RegularOrigin, ResolutionHistory, RingExtension, SncProgress,
    VerifiedComponentSplit, VerifiedRelativeSnc,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Default)]
pub struct LocalizationProgress {
    pub stage: &'static str,
    pub frame: Option<Arc<EtaleFrame>>,
    pub snc: Option<SncProgress>,
    pub old_coordinate_derivatives_checked: usize,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct VerifiedPrincipalOpen {
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    factor: Poly,
    extension: RingExtension,
    ledger: Arc<VerifiedRelativeSnc>,
}
impl VerifiedPrincipalOpen {
    pub fn source(&self) -> &Arc<ResolutionHistory> {
        &self.source
    }
    pub fn split(&self) -> &Arc<VerifiedComponentSplit> {
        &self.split
    }
    pub fn side(&self) -> usize {
        self.side
    }
    pub fn factor(&self) -> &Poly {
        &self.factor
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    /// Identity on the retained relative free coordinates. This is only this
    /// localization's Jacobian; previous projective/blowup maps remain separate.
    pub fn coordinate_jacobian(&self) -> Poly {
        self.ledger.frame().local().ring().one()
    }
}
#[derive(Clone, Debug)]
pub struct LocalizedHistory {
    open: Arc<VerifiedPrincipalOpen>,
    history: Arc<ResolutionHistory>,
    progress: LocalizationProgress,
}
impl LocalizedHistory {
    pub fn open(&self) -> &Arc<VerifiedPrincipalOpen> {
        &self.open
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn progress(&self) -> &LocalizationProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum LocalizationProduction {
    Complete(Box<LocalizedHistory>),
    Empty {
        source: Arc<ResolutionHistory>,
        split: Arc<VerifiedComponentSplit>,
        side: usize,
        factor: Poly,
        progress: LocalizationProgress,
    },
    Incomplete {
        source: Arc<ResolutionHistory>,
        split: Arc<VerifiedComponentSplit>,
        side: usize,
        reason: &'static str,
        progress: LocalizationProgress,
    },
}
pub fn localize_component_open(
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<LocalizationProduction> {
    if side >= 2 {
        return Err(Error::Invalid("component open side"));
    }
    let source_frame = source.ledger().frame();
    match split.owner().origin() {
        RegularOrigin::Ambient(frame) if !Arc::ptr_eq(frame, source_frame) => {
            return Err(Error::Invalid("ambient split frame owner"));
        }
        RegularOrigin::Boundary { ledger, .. } if !Arc::ptr_eq(ledger, source.ledger()) => {
            return Err(Error::Invalid("boundary split ledger owner"));
        }
        _ => {}
    }
    if !Arc::ptr_eq(split.owner().ambient_local(), source_frame.local()) {
        return Err(Error::Invalid("component open original localization"));
    }
    let mut progress = LocalizationProgress::default();
    let result = build(
        source.clone(),
        split.clone(),
        side,
        namespace,
        budget,
        &mut progress,
    );
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(Some(open)) => {
            let open = Arc::new(open);
            match source.restricted(&open, budget) {
                Ok(history) => {
                    progress.stage = "complete";
                    progress.operations = budget.operations();
                    progress.ideal_slots = budget.ideal_slots();
                    Ok(LocalizationProduction::Complete(Box::new(
                        LocalizedHistory {
                            open,
                            history,
                            progress,
                        },
                    )))
                }
                Err(Error::ResourceIncomplete(reason)) => {
                    progress.operations = budget.operations();
                    progress.ideal_slots = budget.ideal_slots();
                    Ok(LocalizationProduction::Incomplete {
                        source,
                        split,
                        side,
                        reason,
                        progress,
                    })
                }
                Err(e) => Err(e),
            }
        }
        Ok(None) => Ok(LocalizationProduction::Empty {
            factor: split.opens()[side].clearing().numerator.clone(),
            source,
            split,
            side,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(LocalizationProduction::Incomplete {
            source,
            split,
            side,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn build(
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    namespace: &str,
    budget: &mut Budget,
    progress: &mut LocalizationProgress,
) -> Result<Option<VerifiedPrincipalOpen>> {
    let factor = split.opens()[side].clearing().numerator.clone();
    Ok(
        geometry::build(source, factor, namespace, budget, progress)?.map(|g| {
            VerifiedPrincipalOpen {
                source: g.source,
                split,
                side,
                factor: g.factor,
                extension: g.extension,
                ledger: g.ledger,
            }
        }),
    )
}

#[cfg(test)]
mod tests;
