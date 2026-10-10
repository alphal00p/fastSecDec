use super::super::{
    Budget, Error, Poly, ResolutionHistory, RingExtension, UnitClearing, VerifiedOpenCover,
    VerifiedRelativeSnc, contact::clear_units,
};
use super::{
    LocalizationProgress,
    geometry::{self, CheckedOpenGeometry},
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
/// One actual native principal open from a checked algebraic locus cover.
/// Its support may only be a marked cosupport, not the full integration domain.
#[derive(Clone, Debug)]
pub struct VerifiedCoverOpen {
    cover: Arc<VerifiedOpenCover>,
    index: usize,
    clearing: UnitClearing,
    geometry: CheckedOpenGeometry,
}
impl VerifiedCoverOpen {
    pub fn cover(&self) -> &Arc<VerifiedOpenCover> {
        &self.cover
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn clearing(&self) -> &UnitClearing {
        &self.clearing
    }
    pub fn source(&self) -> &Arc<ResolutionHistory> {
        &self.geometry.source
    }
    pub fn factor(&self) -> &Poly {
        &self.geometry.factor
    }
    pub fn extension(&self) -> &RingExtension {
        &self.geometry.extension
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.geometry.ledger
    }
    pub fn coordinate_jacobian(&self) -> Poly {
        self.geometry.ledger.frame().local().ring().one()
    }
}
#[derive(Clone, Debug)]
pub struct LocalizedCoverHistory {
    open: Arc<VerifiedCoverOpen>,
    history: Arc<ResolutionHistory>,
    progress: LocalizationProgress,
}
impl LocalizedCoverHistory {
    pub fn open(&self) -> &Arc<VerifiedCoverOpen> {
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
pub enum CoverLocalizationProduction {
    Complete(Box<LocalizedCoverHistory>),
    Empty {
        source: Arc<ResolutionHistory>,
        cover: Arc<VerifiedOpenCover>,
        index: usize,
        clearing: UnitClearing,
        progress: LocalizationProgress,
    },
    Incomplete {
        source: Arc<ResolutionHistory>,
        cover: Arc<VerifiedOpenCover>,
        index: usize,
        reason: &'static str,
        progress: LocalizationProgress,
    },
}
pub fn localize_verified_cover_open(
    source: Arc<ResolutionHistory>,
    cover: Arc<VerifiedOpenCover>,
    index: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<CoverLocalizationProduction> {
    if !Arc::ptr_eq(source.ledger().frame().local(), cover.algebra()) {
        return Err(Error::Invalid("cover localization original owner"));
    }
    let factor = cover
        .opens()
        .get(index)
        .ok_or(Error::Invalid("cover localization open index"))?;
    let mut progress = LocalizationProgress::default();
    let result = (|| {
        let clearing = clear_units(source.ledger().frame().local(), factor, budget)?;
        let geometry = geometry::build(
            source.clone(),
            clearing.numerator.clone(),
            namespace,
            budget,
            &mut progress,
        )?;
        match geometry {
            None => Ok((None, clearing)),
            Some(geometry) => {
                let open = Arc::new(VerifiedCoverOpen {
                    cover: cover.clone(),
                    index,
                    clearing: clearing.clone(),
                    geometry,
                });
                let history = source.restricted_cover(&open, budget)?;
                Ok((Some((open, history)), clearing))
            }
        }
    })();
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok((Some((open, history)), _)) => {
            progress.stage = "complete";
            Ok(CoverLocalizationProduction::Complete(Box::new(
                LocalizedCoverHistory {
                    open,
                    history,
                    progress,
                },
            )))
        }
        Ok((None, clearing)) => Ok(CoverLocalizationProduction::Empty {
            source,
            cover,
            index,
            clearing,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(CoverLocalizationProduction::Incomplete {
            source,
            cover,
            index,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
