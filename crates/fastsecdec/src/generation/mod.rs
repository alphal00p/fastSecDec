//! Direct monomial substitution, exact endpoint subtraction, and Laurent vectors.
//!
//! Symbolica owns all polynomial manipulation, derivatives and series expansions.
//! This module owns the integral-specific order of these operations.
mod coefficient_first;
mod coefficients;
mod conditioning;
mod context;
mod domain;
mod geometry;
mod laurent;
mod mapping;
mod metadata;
mod metadata_display;
pub use crate::status::GeometryReuseStatus;
pub use context::{GenerationContext, GenerationEvent};
pub use fastsecdec_sectors::{
    DecompositionPhase, DecompositionProgress, GeometryCompletion, GeometryDispatch, GeometryJob,
    GeometryJobId, GeometryWorkError, SectorError,
};
pub use metadata_display::MetadataView;
#[cfg(test)]
pub(crate) mod profiling;
mod subtraction;
mod support;
mod symmetry;
mod types;
pub(crate) use domain::check_factors;
pub(crate) use mapping::coordinates_from_parts;

pub use metadata::{
    BranchPolicy, ChartRecord, CoordinateMap, DomainAssessment, EndpointPower, FactorAssessment,
    FactorCertificate, GenerationMetadata, PreSubtractionMetadata, PreSubtractionTerm,
};
pub use types::{
    CoefficientExpansionMethod, CoefficientExpansionOptions, CoefficientExpansionStage,
    CoefficientRequestCounts, ConditioningBasis, GeneratedIntegral, GeneratedSector,
    GenerationError, GenerationOptions, GenerationPhase, GenerationProgress, SubtractionStrategy,
};

use crate::parametric::ParametricIntegrand;
use context::emit;
use fastsecdec_sectors::PolynomialSupport;
use geometry::{Geometry, GeometrySource};
use std::{collections::BTreeMap, ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore, AtomView},
    symbol,
};

/// Test-only access to the real Laurent stage, without manufacturing a partial
/// generated integral or its domain/chart metadata.
#[cfg(test)]
pub(crate) struct CapturedLaurent {
    pub template: Atom,
    pub coefficients: BTreeMap<i32, symbolica::atom::AliasedAtom>,
}

#[cfg(test)]
pub(crate) fn captured_coefficients(
    expression: &Atom,
    parameters: &[symbolica::atom::Symbol],
    regulator: symbolica::atom::Symbol,
    maximum: i32,
) -> Result<CapturedLaurent, GenerationError> {
    let mut cache = laurent::TemplateCache::default();
    let coefficients = laurent::expand(expression, parameters, regulator, maximum, &mut cache)?;
    Ok(CapturedLaurent {
        template: cache.only_template().clone(),
        coefficients,
    })
}

/// Test-only replay of already admitted, source-bound mapped inputs.
/// Tuple order is (prefactor, regular factor, coordinate powers).
#[cfg(test)]
pub(crate) fn captured_subtraction(
    terms: Vec<(Atom, Atom, Vec<Atom>)>,
    parameters: &[symbolica::atom::Symbol],
    regulator: symbolica::atom::Symbol,
    options: &GenerationOptions,
) -> Result<(Atom, usize, Vec<Vec<usize>>), GenerationError> {
    subtraction::subtract(
        terms
            .into_iter()
            .map(|(prefactor, regular, powers)| mapping::MappedTerm {
                powers,
                prefactor,
                regular,
            })
            .collect(),
        parameters,
        regulator,
        options,
    )
}

pub fn generate(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<GeneratedIntegral, GenerationError> {
    generate_inner(
        input,
        options,
        GeometrySource::Uncached,
        |event| match event {
            GenerationEvent::Progress(status) => progress(status),
            GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
        },
    )
}

fn generate_inner(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    geometry_source: GeometrySource<'_, '_>,
    mut progress: impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<GeneratedIntegral, GenerationError> {
    let started = Instant::now();
    let domain = domain::check(input, options.assume_no_threshold)?;
    emit(
        &mut progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Domain,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    let started = Instant::now();
    let mut source_supports = support::SupportCache::new(input.parameters());
    let mut supports = Vec::new();
    for term in input.terms() {
        for factor in term.factors() {
            if domain::is_singular(factor) {
                let support = source_supports.get(factor)?;
                if !supports.contains(support) {
                    supports.push(support.clone());
                }
            }
        }
    }
    if supports.is_empty() {
        supports.push(PolynomialSupport::new(vec![vec![
            0;
            input.parameters().len()
        ]])?);
    }
    let mut decomposition = if input.terms().is_empty() {
        None
    } else {
        Some(Geometry::compute(
            input.domain(),
            &supports,
            &options.decomposition,
            geometry_source,
            &mut progress,
        )?)
    };
    let total = decomposition.as_ref().map_or(0, Geometry::len);
    emit(
        &mut progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Geometry,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    let mut source_symbols = input.density().get_all_symbols(true);
    // Unused input coordinates still belong to the source chart and must not
    // be reused as target symbols merely because the density omits them.
    source_symbols.extend(input.parameters().iter().copied());
    source_symbols.extend(std::iter::once(input.regulator()));
    let mut pending = Vec::new();
    let mut exact = BTreeMap::<i32, Atom>::new();
    let mut minimum = options.max_order.min(0);
    let mut templates = laurent::TemplateCache::default();
    let mut registry = symmetry::SymmetryRegistry::default();
    let mut representatives = BTreeMap::new();
    let mut charts = Vec::new();
    for (index, map) in decomposition
        .iter_mut()
        .flat_map(Geometry::maps)
        .enumerate()
    {
        emit(
            &mut progress,
            GenerationProgress::Factorization {
                sector: index,
                total,
            },
        )?;
        let mut namespace = 0usize;
        let parameters = loop {
            let candidates = (0..map.dimension())
                .map(|axis| symbol!(format!("fastsecdec::sector_{namespace}::t{axis}")))
                .collect::<Vec<_>>();
            if candidates
                .iter()
                .all(|symbol| !source_symbols.contains(symbol))
            {
                break candidates;
            }
            namespace += 1;
        };
        let started = Instant::now();
        let coordinates = mapping::coordinates(input, &map, &parameters);
        let mapped = mapping::map_terms(input, &map, &coordinates, &mut source_supports)?;
        let pre_subtraction = Some(PreSubtractionMetadata::capture(
            &mapped,
            input.regulator(),
            options.max_subtractions_per_axis,
        )?);
        emit(
            &mut progress,
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::Mapping,
                seconds: started.elapsed().as_secs_f64(),
            },
        )?;
        let started = Instant::now();
        #[cfg(test)]
        {
            symmetry::profile::chart(index);
            symmetry::profile::trace(
                "Assembly",
                "begin",
                serde_json::json!({"mapped_terms": mapped.len(), "parameters": parameters.len()}),
            );
        }
        let density = mapped
            .iter()
            .map(|term| {
                &term.prefactor
                    * &term.regular
                    * parameters
                        .iter()
                        .zip(&term.powers)
                        .map(|(parameter, power)| Atom::var(*parameter).pow(power))
                        .product::<Atom>()
            })
            .sum::<Atom>();
        #[cfg(test)]
        symmetry::profile::trace(
            "Assembly",
            "end",
            serde_json::json!({"density_bytes": density.as_view().get_byte_size()}),
        );
        let matched = registry.register(index, &parameters, &density)?;
        #[cfg(test)]
        symmetry::profile::trace(
            "Registration",
            "end",
            serde_json::json!({"representative": matched.representative, "matched": matched.representative != index}),
        );
        debug_assert_eq!(matched.permutation.len(), parameters.len());
        charts.push(ChartRecord {
            source_index: index,
            representative: matched.representative,
            representative_permutation: matched.permutation.clone(),
            kernel_sector: None,
            coordinates,
            geometry: map.as_ref().clone(),
            pre_subtraction,
        });
        if matched.representative == index {
            representatives.insert(index, (map.into_owned(), parameters, mapped, 1usize));
        } else {
            let representative = representatives
                .get_mut(&matched.representative)
                .ok_or_else(|| {
                    GenerationError::Invariant("missing symmetry representative".into())
                })?;
            representative.3 += 1;
        }
        emit(
            &mut progress,
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::Symmetry,
                seconds: started.elapsed().as_secs_f64(),
            },
        )?;
    }
    let total = representatives.len();
    let mut kernel_indices = BTreeMap::new();
    for (index, (representative_index, (map, parameters, mapped, multiplicity))) in
        representatives.into_iter().enumerate()
    {
        #[cfg(test)]
        {
            laurent::profiling::context(index, representative_index, multiplicity);
            laurent::profiling::before_subtraction(
                &mapped,
                &parameters,
                input.regulator(),
                options.max_order,
            )?;
        }
        let output = coefficients::expand(
            mapped,
            coefficients::Representative {
                parameters: &parameters,
                regulator: input.regulator(),
                index,
                total,
            },
            options,
            &mut templates,
            &mut progress,
        )?;
        let coefficients = output
            .coefficients
            .into_iter()
            .map(|(order, coefficient)| {
                (
                    order,
                    coefficient.map_root(|root| root * Atom::num(multiplicity)),
                )
            })
            .collect::<BTreeMap<_, _>>();
        emit(
            &mut progress,
            GenerationProgress::PhaseTiming {
                phase: output.phase,
                seconds: output.phase_started.elapsed().as_secs_f64(),
            },
        )?;
        let conditioning = output.conditioning;
        if let Some(order) = coefficients.keys().next() {
            minimum = minimum.min(*order);
        }
        if coefficients.values().all(|coefficient| {
            let symbols = coefficient.get_root().get_all_symbols(true);
            parameters.iter().all(|p| {
                let parameter = Atom::var(*p);
                !symbols.contains(p)
                    && coefficient.get_aliases().iter().all(|(handle, body)| {
                        let AtomView::Var(handle) = handle.as_view() else {
                            unreachable!("Laurent images are native symbols")
                        };
                        !symbols.contains(&handle.get_symbol())
                            || !body.contains(parameter.as_view())
                    })
            })
        }) {
            for (order, coefficient) in coefficients {
                *exact.entry(order).or_insert(Atom::Zero) += coefficient.into_inner();
            }
        } else {
            kernel_indices.insert(representative_index, pending.len());
            pending.push((map, parameters, coefficients, conditioning));
        }
    }
    #[cfg(test)]
    laurent::profiling::reject_uncaptured_result()?;
    for chart in &mut charts {
        chart.kernel_sector = kernel_indices.get(&chart.representative).copied();
    }
    let orders = (minimum..=options.max_order).collect::<Vec<_>>();
    let sectors = pending
        .into_iter()
        .map(
            |(map, parameters, coefficients, conditioning)| GeneratedSector {
                cancellation_degree: conditioning.degree,
                cancellation_terms: conditioning.rows,
                conditioning_basis: conditioning.basis,
                parameters,
                map,
                materialized: Default::default(),
                coefficients: orders
                    .iter()
                    .map(|order| coefficients.get(order).cloned().unwrap_or_default())
                    .collect(),
            },
        )
        .collect();
    let exact_coefficients = orders
        .iter()
        .map(|order| exact.get(order).cloned().unwrap_or(Atom::Zero))
        .collect();
    let result = GeneratedIntegral {
        metadata: GenerationMetadata { domain, charts },
        orders,
        sectors,
        exact_coefficients,
    };
    emit(
        &mut progress,
        GenerationProgress::Complete {
            sectors: result.sectors.len(),
            orders: result.orders.clone(),
        },
    )?;
    Ok(result)
}
// Test-only native named-coefficient program controls.
#[cfg(test)]
type CapturedNamedCoefficients = (
    std::collections::BTreeMap<i32, symbolica::atom::AliasedAtom>,
    serde_json::Value,
);

#[cfg(test)]
pub(crate) fn captured_named_coefficients(
    terms: Vec<(Atom, Atom, Vec<Atom>)>,
    parameters: &[symbolica::atom::Symbol],
    regulator: symbolica::atom::Symbol,
    options: &GenerationOptions,
) -> Result<CapturedNamedCoefficients, GenerationError> {
    captured_named_coefficients_with_faces(terms, parameters, regulator, options, false)
}

#[cfg(test)]
pub(crate) fn captured_named_coefficients_with_faces(
    terms: Vec<(Atom, Atom, Vec<Atom>)>,
    parameters: &[symbolica::atom::Symbol],
    regulator: symbolica::atom::Symbol,
    options: &GenerationOptions,
    interleaved: bool,
) -> Result<CapturedNamedCoefficients, GenerationError> {
    let terms = terms
        .into_iter()
        .map(|(prefactor, regular, powers)| mapping::MappedTerm {
            prefactor,
            regular,
            powers,
        })
        .collect::<Vec<_>>();
    let (coefficients, attempts, statistics) = if interleaved {
        subtraction::series_first::expand_named_with_resolution(
            &terms,
            parameters,
            regulator,
            options,
            subtraction::series_first::Resolution::InterleavedFaces,
        )?
    } else {
        subtraction::series_first::expand_named(&terms, parameters, regulator, options)?
    };
    let attempts = attempts
        .into_iter()
        .map(|attempt| {
            serde_json::json!({
                "route":attempt.route,"width":attempt.width,"absolute_bound":attempt.absolute_bound,
                "pieces":attempt.pieces,"seconds":attempt.seconds,
            })
        })
        .collect::<Vec<_>>();
    Ok((
        coefficients,
        serde_json::json!({"coefficient_representation":"native_named", "resolution":if interleaved {"interleaved_faces"} else {"original"}, "attempts":attempts,"statistics":statistics}),
    ))
}
