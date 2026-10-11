//! Ascent from the ACTUAL original coefficient constructor into its ancestor.
use super::super::*;
use super::{
    embedding::SupportEmbedding,
    helpers::{Result, unit},
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OriginalProblemCenter {
    prepared: Arc<PreparedProblemCenter>,
    origin: Arc<EmbeddedChildCycle>,
    embedding: Arc<SupportEmbedding>,
    child: Arc<EmbeddedChildCycle>,
    ideal: Ideal,
    normals: Vec<Poly>,
    clearings: Vec<UnitClearing>,
}
impl OriginalProblemCenter {
    pub fn prepared(&self) -> &Arc<PreparedProblemCenter> {
        &self.prepared
    }
    pub fn origin(&self) -> &Arc<EmbeddedChildCycle> {
        &self.origin
    }
    pub fn embedding(&self) -> &Arc<SupportEmbedding> {
        &self.embedding
    }
    pub fn child_cycle(&self) -> &Arc<EmbeddedChildCycle> {
        &self.child
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        self.origin.center().frame()
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        self.origin.center().parent_source()
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        self.origin
            .center()
            .coefficient()
            .source()
            .restriction()
            .history()
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
}
pub(crate) fn produce_original_problem_center(
    prepared: Arc<PreparedProblemCenter>,
    b: &mut Budget,
) -> Result<EmbeddedProblemProduction> {
    let ProblemConstruction::OriginalCoefficient(origin) = prepared.problem().construction() else {
        return Err(Error::Invalid("original ascent construction owner"));
    };
    if !Arc::ptr_eq(prepared.problem().history(), origin.initial_history())
        || !Arc::ptr_eq(
            prepared.problem().source(),
            origin.center().child().source(),
        )
    {
        return Err(Error::Invalid("original ascent source/history owner"));
    }
    let center = prepared.center();
    let restriction = center.coefficient().source().restriction();
    if !Arc::ptr_eq(restriction.open().source(), origin.initial_history())
        || !unit(
            origin.initial_history().ledger().frame().local(),
            restriction.open().factor(),
            b,
        )?
    {
        return Ok(EmbeddedProblemProduction::NeedsPhysicalLocalization { prepared });
    }
    let base = SupportEmbedding::contact(origin.center().coefficient().contact().clone(), b)?;
    if !Arc::ptr_eq(base.frame(), origin.initial_history().ledger().frame()) {
        return Err(Error::Invalid("original ascent contact/history frame"));
    }
    let embedding = base.restrict_unit_open(restriction.clone(), b)?;
    if !Arc::ptr_eq(embedding.frame(), center.frame()) {
        return Err(Error::Invalid("original ascent actual companion frame"));
    }
    let (ideal, normals, clearings) = super::super::recursive::lift_embedded_geometry(
        origin.center().frame(),
        origin.center().parent_source(),
        center.frame(),
        embedding.extension(),
        embedding.equations().generators().to_vec(),
        center.ideal(),
        center.normals(),
        b,
    )?;
    // The original ancestor companion is an additional admissibility obligation,
    // not an equation which can be discarded when its marked parent has low order.
    let q = origin.center().coefficient().contact();
    let g = Arc::new(MarkedIdeal::new(
        (**q.source().source()).clone(),
        q.source().algebraic_maximum_order(),
        b,
    )?);
    let _ = super::super::recursive::lift_embedded_geometry(
        origin.center().frame(),
        &g,
        center.frame(),
        embedding.extension(),
        embedding.equations().generators().to_vec(),
        center.ideal(),
        center.normals(),
        b,
    )?;
    let child = EmbeddedChildCycle::new(center.clone(), b)?;
    Ok(EmbeddedProblemProduction::OriginalCenter(Arc::new(
        OriginalProblemCenter {
            origin: origin.clone(),
            prepared,
            embedding,
            child,
            ideal,
            normals,
            clearings,
        },
    )))
}
