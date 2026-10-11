//! Exact ascent of a newly constructed incidence problem's nonmonomial center.
//! Nonunit opens require the explicit physical cover, rather than being erased.
use super::super::*;
use super::{
    helpers::{Result, unit},
    induced::{SupportedCenterLift, lift_supported_center},
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct EmbeddedProblemCenter {
    prepared: Arc<PreparedProblemCenter>,
    source: Arc<EmbeddedPresentation>,
    child_cycle: Arc<EmbeddedChildCycle>,
    lower_ideal: Ideal,
    lower_normals: Vec<Poly>,
    lower_clearings: Vec<UnitClearing>,
    ideal: Ideal,
    normals: Vec<Poly>,
    clearings: Vec<UnitClearing>,
}
impl EmbeddedProblemCenter {
    pub fn prepared(&self) -> &Arc<PreparedProblemCenter> {
        &self.prepared
    }
    pub fn presentation(&self) -> &Arc<EmbeddedPresentation> {
        &self.source
    }
    pub fn child_cycle(&self) -> &Arc<EmbeddedChildCycle> {
        &self.child_cycle
    }
    pub fn lower_ideal(&self) -> &Ideal {
        &self.lower_ideal
    }
    pub fn lower_normals(&self) -> &[Poly] {
        &self.lower_normals
    }
    pub fn lower_clearings(&self) -> &[UnitClearing] {
        &self.lower_clearings
    }
    pub fn ideal(&self) -> &Ideal {
        &self.ideal
    }
    pub fn normals(&self) -> &[Poly] {
        &self.normals
    }
    pub fn clearings(&self) -> &[UnitClearing] {
        &self.clearings
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        self.source.chart().geometry().frame()
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        self.source.chart().history()
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        self.source.chart().parent().target()
    }
}
#[derive(Clone, Debug)]
pub enum EmbeddedProblemProduction {
    Center(Arc<EmbeddedProblemCenter>),
    NeedsPhysicalLocalization {
        prepared: Arc<PreparedProblemCenter>,
    },
    OriginalEmbeddingRequired {
        prepared: Arc<PreparedProblemCenter>,
    },
}
pub(crate) fn produce_embedded_problem_center(
    prepared: Arc<PreparedProblemCenter>,
    b: &mut Budget,
) -> Result<EmbeddedProblemProduction> {
    let ProblemConstruction::IncidenceRedefinition(drop) = prepared.problem().construction() else {
        return Ok(EmbeddedProblemProduction::OriginalEmbeddingRequired { prepared });
    };
    let source = drop.presentation();
    if !Arc::ptr_eq(source.incidence_sum(), prepared.problem().source())
        || !Arc::ptr_eq(source.history(), prepared.problem().history())
    {
        return Err(Error::Invalid("nonmonomial ascent actual incidence source"));
    }
    let center = prepared.center();
    let restriction = center.coefficient().source().restriction();
    if !Arc::ptr_eq(restriction.open().source(), source.history()) {
        return Ok(EmbeddedProblemProduction::NeedsPhysicalLocalization { prepared });
    }
    let frame = source.history().ledger().frame();
    if !unit(frame.local(), restriction.open().factor(), b)? {
        return Ok(EmbeddedProblemProduction::NeedsPhysicalLocalization { prepared });
    }
    // Descend from an actual unit open to its same original lower frame before
    // physical ascent. The helper checks the full center ideal and marked order.
    let (lower_ideal, lower_normals, lower_clearings) =
        super::super::recursive::lift_embedded_geometry(
            frame,
            source.incidence_sum(),
            center.frame(),
            restriction.open().extension(),
            vec![],
            center.ideal(),
            center.normals(),
            b,
        )?;
    let Some((ideal, normals, clearings)) = lift_supported_center(
        SupportedCenterLift {
            frame: source.chart().geometry().frame(),
            parent: source.chart().parent().target(),
            companion: source.companion().target(),
            support: source.support(),
            open: source.open(),
            child: &lower_ideal,
            child_normals: &lower_normals,
        },
        b,
    )?
    else {
        return Ok(EmbeddedProblemProduction::NeedsPhysicalLocalization { prepared });
    };
    // Capture this newly constructed child's ORIGINAL recursion now, before
    // any physical pivot. It does not reset the history-bearing parent J.
    let child_cycle = EmbeddedChildCycle::new(center.clone(), b)?;
    Ok(EmbeddedProblemProduction::Center(Arc::new(
        EmbeddedProblemCenter {
            prepared,
            source,
            child_cycle,
            lower_ideal,
            lower_normals,
            lower_clearings,
            ideal,
            normals,
            clearings,
        },
    )))
}
