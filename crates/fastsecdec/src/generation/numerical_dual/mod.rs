//! Exact endpoint recipes with deferred native map and Taylor-jet evaluation.
mod inspection;
mod mapping;
pub(crate) mod native;
mod subtraction;
mod valuation;
pub(super) use valuation::ValuationCache;

use super::{
    ChartRecord, DomainAssessment, GeneratedIntegral, GeneratedSector, GenerationError,
    GenerationEvent, GenerationMetadata, GenerationOptions, GenerationPhase, GenerationProgress,
    PreSubtractionMetadata, SymbolicDispatch, coefficients, conditioning::Profile, context::emit,
    laurent, support::SupportCache,
};
use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::SectorMap;
use std::{ops::ControlFlow, sync::Arc, time::Instant};
use symbolica::atom::{Atom, AtomCore, Symbol};

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
    pub recipe: subtraction::Recipe,
    pub orders: Vec<i32>,
}

pub(super) struct PreparedChart {
    pub chart: ChartRecord,
    pub sector: GeneratedSector,
    pub orders: Vec<i32>,
}

pub(super) struct Sources<'a> {
    pub input: &'a ParametricIntegrand,
    pub options: &'a GenerationOptions,
    pub programs: &'a Arc<native::SourcePrograms>,
    pub valuations: &'a Arc<ValuationCache>,
}

pub(super) fn prepare(
    sources: Sources<'_>,
    map: SectorMap,
    parameters: Vec<Symbol>,
    index: usize,
    total: usize,
    supports: &mut SupportCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<PreparedChart, GenerationError> {
    let Sources {
        input,
        options,
        programs,
        valuations,
    } = sources;
    emit(
        progress,
        GenerationProgress::NumericalMapping {
            sector: index,
            total,
        },
    )?;
    let started = Instant::now();
    let coordinates = super::mapping::coordinates(input, &map, &parameters);
    let (mapped, terms) = mapping::prepare(input, &map, &coordinates, supports, valuations)?;
    let mut pre_subtraction = Some(PreSubtractionMetadata::capture(
        &mapped,
        input.regulator(),
        options.max_subtractions_per_axis,
    )?);
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    let started = Instant::now();
    let symbols = input
        .density()
        .get_all_symbols(true)
        .into_iter()
        .collect::<Vec<_>>();
    let mut observation = coefficients::Observation::default();
    let representative = coefficients::Representative {
        parameters: &parameters,
        regulator: input.regulator(),
        index,
        total,
    };
    let epsilon = Atom::var(input.regulator());
    let polynomial_in_epsilon = |value: &Atom| {
        !value.contains(epsilon.as_view())
            || value.is_polynomial(true, false).is_some_and(|variables| {
                variables.iter().all(|variable| {
                    *variable == epsilon.as_view() || !variable.contains(epsilon.as_view())
                })
            })
    };
    let recipe = if parameters.is_empty()
        || map.exponent_matrix.iter().flatten().any(|power| power < &0)
        || input
            .terms()
            .iter()
            .flat_map(|term| term.factors())
            .any(|factor| {
                (super::domain::is_singular(factor)
                    && factor.polynomial().contains(epsilon.as_view()))
                    || !polynomial_in_epsilon(factor.polynomial())
                    || !polynomial_in_epsilon(factor.exponent())
            }) {
        Err(subtraction::RecipeError::NeedsExactAdmission { term: 0, axis: 0 })
    } else {
        subtraction::expand(
            &mapped,
            &parameters,
            input.regulator(),
            options,
            &symbols,
            &mut |event| {
                let stage = observation.observe(event);
                match emit(progress, observation.event(&representative, stage)) {
                    Ok(()) => ControlFlow::Continue(()),
                    Err(_) => ControlFlow::Break(()),
                }
            },
        )
    };
    let numerical_recipe = recipe.is_ok();
    let (coefficients, profile, deferred) = match recipe {
        Ok(recipe) => {
            let coefficients = recipe.coefficients.clone();
            let profile = Profile::mapped(&mapped, parameters.len(), input.regulator(), options)?;
            let orders = coefficients.keys().copied().collect();
            (
                coefficients,
                profile,
                Some(Arc::new(DualSector {
                    programs: programs.clone(),
                    source_parameters: input.parameters().to_vec(),
                    regulator: input.regulator(),
                    parameters: parameters.clone(),
                    map: map.clone(),
                    terms,
                    recipe,
                    orders,
                })),
            )
        }
        Err(subtraction::RecipeError::Generation(error)) => return Err(error),
        Err(subtraction::RecipeError::NeedsExactAdmission { .. }) => {
            // A numerically vanishing jet cannot prove convergence of an
            // unregulated endpoint. Retain the existing exact admission path.
            emit(
                progress,
                observation.event(
                    &representative,
                    super::CoefficientExpansionStage::PhysicalFallback,
                ),
            )?;
            let mapping_started = Instant::now();
            let physical = super::mapping::map_terms(input, &map, &coordinates, supports)?;
            pre_subtraction = Some(PreSubtractionMetadata::capture(
                &physical,
                input.regulator(),
                options.max_subtractions_per_axis,
            )?);
            emit(
                progress,
                GenerationProgress::PhaseTiming {
                    phase: GenerationPhase::Mapping,
                    seconds: mapping_started.elapsed().as_secs_f64(),
                },
            )?;
            let output = coefficients::expand(
                physical,
                representative,
                options,
                &mut laurent::TemplateCache::default(),
                progress,
            )?;
            // The physical route already reports subtraction separately.
            emit(
                progress,
                GenerationProgress::PhaseTiming {
                    phase: output.phase,
                    seconds: output.phase_started.elapsed().as_secs_f64(),
                },
            )?;
            (output.coefficients, output.conditioning, None)
        }
    };
    let orders = coefficients.keys().copied().collect::<Vec<_>>();
    let sector = GeneratedSector {
        cancellation_degree: profile.degree,
        cancellation_terms: profile.rows,
        endpoint_profiles: profile.endpoint_profiles,
        conditioning_basis: profile.basis,
        parameters: parameters.clone(),
        coefficients: coefficients.into_values().collect(),
        materialized: Default::default(),
        map: map.clone(),
        deferred,
    };
    let chart = ChartRecord {
        source_index: index,
        representative: index,
        representative_permutation: (0..parameters.len()).collect(),
        kernel_sector: Some(index),
        coordinates,
        geometry: map,
        pre_subtraction,
    };
    if numerical_recipe {
        emit(
            progress,
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::CoefficientExpansion,
                seconds: started.elapsed().as_secs_f64(),
            },
        )?;
    }
    Ok(PreparedChart {
        chart,
        sector,
        orders,
    })
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
    let prepared = if let Some(dispatch) = dispatch {
        super::work::numerical_dual_dispatched(
            input,
            options,
            maps,
            &parameters,
            supports,
            dispatch,
            progress,
        )?
    } else {
        let total = maps.len();
        let mut supports = supports.clone();
        let programs = Arc::new(native::SourcePrograms::default());
        let valuations = Arc::new(ValuationCache::new(input.parameters()));
        maps.into_iter()
            .enumerate()
            .map(|(index, map)| {
                prepare(
                    Sources {
                        input,
                        options,
                        programs: &programs,
                        valuations: &valuations,
                    },
                    map,
                    parameters.clone(),
                    index,
                    total,
                    &mut supports,
                    progress,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
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
