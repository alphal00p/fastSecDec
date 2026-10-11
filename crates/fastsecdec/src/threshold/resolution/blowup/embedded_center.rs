//! Lift the actual current lower problem; never restart its exceptional history.
use super::super::*;
use super::{
    induced::{SupportedCenterLift, lift_supported_center},
    presentation::EmbeddedPresentation,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct EmbeddedMonomialCenter {
    source: Arc<EmbeddedPresentation>,
    factor: Arc<FactorLeaf>,
    child: Arc<ProducedMonomialCenter>,
    ideal: Ideal,
    normals: Vec<Poly>,
    clearings: Vec<UnitClearing>,
}
impl EmbeddedMonomialCenter {
    pub fn presentation(&self) -> &Arc<EmbeddedPresentation> {
        &self.source
    }
    pub fn factor(&self) -> &Arc<FactorLeaf> {
        &self.factor
    }
    pub fn child(&self) -> &Arc<ProducedMonomialCenter> {
        &self.child
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
#[derive(Clone, Debug)]
pub enum EmbeddedCenterProduction {
    Center(Arc<EmbeddedMonomialCenter>),
    Empty(MonomialProduction),
    NeedsLowerRecursion {
        source: Arc<EmbeddedPresentation>,
        factor: Arc<FactorLeaf>,
    },
    NeedsLocalization {
        source: Arc<EmbeddedPresentation>,
        factor: Arc<FactorLeaf>,
    },
}
pub fn produce_embedded_monomial_center(
    source: Arc<EmbeddedPresentation>,
    factor: Arc<FactorLeaf>,
    b: &mut Budget,
) -> Result<EmbeddedCenterProduction> {
    let data = factor.data();
    if !Arc::ptr_eq(data.source(), source.incidence_sum()) {
        return Err(Error::Invalid("embedded center current lower source"));
    }
    if !Arc::ptr_eq(data.history(), source.history()) {
        if !data.history().same_root(source.history())
            || !data
                .history()
                .chart_path()
                .starts_with(source.history().chart_path())
            || data.history().births() != source.history().births()
        {
            return Err(Error::Invalid("embedded center lower history reset"));
        }
        return Ok(EmbeddedCenterProduction::NeedsLocalization { source, factor });
    }
    let local = data.history().ledger().frame().local();
    let residual = Ideal::new(local.ring().clone(), data.quotients().to_vec(), b)?;
    if !local
        .ideal()
        .sum(&residual, b)?
        .contains(&local.ring().one(), local.unit_relations(), b)?
    {
        return Ok(EmbeddedCenterProduction::NeedsLowerRecursion { source, factor });
    }
    let child = match produce_monomial_center(
        data.history().ledger().clone(),
        MonomialWitness {
            source: source.incidence_sum().clone(),
            powers: data.powers().to_vec(),
            quotients: data.quotients().to_vec(),
        },
        b,
    )? {
        MonomialProduction::Center(c) => Arc::new(*c),
        p @ MonomialProduction::EmptyCosupport { .. } => {
            return Ok(EmbeddedCenterProduction::Empty(p));
        }
        MonomialProduction::Incomplete { reason, .. } => {
            return Err(Error::ResourceIncomplete(reason));
        }
    };
    let normals = child
        .indices()
        .iter()
        .map(|i| child.ledger().divisors()[*i].equation.clone())
        .collect::<Vec<_>>();
    let Some((ideal, normals, clearings)) = lift_supported_center(
        SupportedCenterLift {
            frame: source.chart().geometry().frame(),
            parent: source.chart().parent().target(),
            companion: source.companion().target(),
            support: source.support(),
            open: source.open(),
            child: child.center(),
            child_normals: &normals,
        },
        b,
    )?
    else {
        return Ok(EmbeddedCenterProduction::NeedsLocalization { source, factor });
    };
    Ok(EmbeddedCenterProduction::Center(Arc::new(
        EmbeddedMonomialCenter {
            source,
            factor,
            child,
            ideal,
            normals,
            clearings,
        },
    )))
}
