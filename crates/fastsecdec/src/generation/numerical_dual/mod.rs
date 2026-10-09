//! Exact endpoint recipes with deferred native map and Taylor-jet evaluation.
pub(in crate::generation) mod chart;
pub(in crate::generation) mod formula;
mod inspection;
mod mapping;
pub(crate) mod native;
pub(super) mod pipeline;
pub(super) mod subtraction;
mod valuation;
pub(super) use valuation::ValuationCache;

#[cfg(test)]
mod contour_local;

use super::{
    ChartRecord, DomainAssessment, GeneratedIntegral, GeneratedSector, GenerationError,
    GenerationEvent, GenerationMetadata, GenerationOptions, GenerationProgress, SymbolicDispatch,
    context::emit, support::SupportCache,
};
use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::SectorMap;
use std::{ops::ControlFlow, sync::Arc};
use symbolica::atom::{Atom, Symbol};

#[derive(Clone, Debug)]
pub(crate) struct DualFactor {
    pub polynomial: Atom,
    pub exponent: Atom,
    pub valuation: Vec<i32>,
}

#[derive(Clone, Debug)]
pub(crate) struct DualTerm {
    pub factors: Vec<DualFactor>,
}

/// A generation recipe, not a second numerical interpreter. Compilation lowers
/// this owner to Symbolica's ordinary exact scalar IR using its native jets.
#[derive(Clone, Debug)]
pub(crate) struct DualSector {
    pub programs: Arc<native::SourcePrograms>,
    pub source_parameters: Vec<Symbol>,
    pub regulator: Symbol,
    pub parameters: Vec<Symbol>,
    pub map: SectorMap,
    pub terms: Vec<DualTerm>,
    /// Contours are nonlinear maps: differentiate the full mapped smooth
    /// density with native jets, without monomial-specific source zero masks.
    pub mapped_regular: Option<Vec<Atom>>,
    pub recipe: Arc<subtraction::Recipe>,
    pub orders: Vec<i32>,
}

pub(super) struct PreparedChart {
    pub chart: ChartRecord,
    pub sector: GeneratedSector,
    pub orders: Vec<i32>,
}

pub(super) fn finish(
    domain: DomainAssessment,
    prepared: Vec<PreparedChart>,
    max_order: i32,
) -> GeneratedIntegral {
    let minimum = prepared
        .iter()
        .flat_map(|chart| &chart.orders)
        .copied()
        .min()
        .unwrap_or(0)
        .min(max_order.min(0));
    let orders = (minimum..=max_order).collect::<Vec<_>>();
    let mut charts = Vec::with_capacity(prepared.len());
    let mut sectors = Vec::with_capacity(prepared.len());
    let mut exact_coefficients = vec![Atom::Zero; orders.len()];
    for mut prepared in prepared {
        let coefficients = prepared
            .orders
            .into_iter()
            .zip(prepared.sector.coefficients)
            .collect::<std::collections::BTreeMap<_, _>>();
        prepared.sector.coefficients = orders
            .iter()
            .map(|order| coefficients.get(order).cloned().unwrap_or_default())
            .collect();
        if let Some(deferred) = &mut prepared.sector.deferred {
            Arc::make_mut(deferred).orders = orders.clone();
        }
        if prepared.sector.dimension() == 0 {
            for (exact, coefficient) in exact_coefficients
                .iter_mut()
                .zip(prepared.sector.coefficients)
            {
                *exact += coefficient.into_inner();
            }
            prepared.chart.kernel_sector = None;
            charts.push(prepared.chart);
            continue;
        }
        prepared.chart.kernel_sector = Some(sectors.len());
        charts.push(prepared.chart);
        sectors.push(prepared.sector);
    }
    GeneratedIntegral {
        exact_coefficients,
        orders,
        sectors,
        metadata: GenerationMetadata { domain, charts },
    }
}

pub(super) struct PreparedMaps {
    pub domain: DomainAssessment,
    pub maps: Vec<SectorMap>,
    pub parameters: Vec<Symbol>,
}

pub(super) fn generate(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    prepared_maps: PreparedMaps,
    supports: &SupportCache,
    dispatch: Option<&mut SymbolicDispatch<'_>>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<GeneratedIntegral, GenerationError> {
    let PreparedMaps {
        domain,
        maps,
        parameters,
    } = prepared_maps;
    let context = Arc::new(pipeline::Context::prepare(
        input, options, parameters, progress,
    )?);
    let prepared = if let Some(dispatch) = dispatch {
        pipeline::dispatched(context, maps, supports, dispatch, progress)?
    } else {
        let mut pipeline = pipeline::Pipeline::new(context, maps);
        let mut supports = supports.clone();
        while !pipeline.is_complete() {
            pipeline.step(&mut supports, progress)?;
        }
        pipeline.take_result()
    };
    let result = finish(domain, prepared, options.max_order);
    emit(
        progress,
        GenerationProgress::Complete {
            sectors: result.sectors.len(),
            orders: result.orders.clone(),
        },
    )?;
    Ok(result)
}
