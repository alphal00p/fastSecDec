//! Direct monomial substitution, exact endpoint subtraction, and Laurent vectors.
//!
//! Symbolica owns all polynomial manipulation, derivatives and series expansions.
//! This module owns the integral-specific order of these operations.
mod domain;
mod laurent;
mod mapping;
mod metadata;
mod metadata_display;
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
    BranchPolicy, ChartRecord, CoordinateMap, DomainAssessment, FactorAssessment,
    FactorCertificate, GenerationMetadata,
};
pub use types::{
    GeneratedIntegral, GeneratedSector, GenerationError, GenerationOptions, GenerationPhase,
    GenerationProgress, SubtractionStrategy,
};

use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::{PolynomialSupport, decompose};
use std::{collections::BTreeMap, ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore},
    symbol,
};

pub fn generate(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<GeneratedIntegral, GenerationError> {
    let mut emit = |status| {
        if progress(&status).is_break() {
            Err(GenerationError::Cancelled)
        } else {
            Ok(())
        }
    };
    let started = Instant::now();
    let domain = domain::check(input, options.assume_no_threshold)?;
    emit(GenerationProgress::PhaseTiming {
        phase: GenerationPhase::Domain,
        seconds: started.elapsed().as_secs_f64(),
    })?;
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
    let decomposition = if input.terms().is_empty() {
        None
    } else {
        Some(decompose(
            input.domain(),
            &supports,
            &options.decomposition,
            |status| {
                if emit(GenerationProgress::Decomposition(status.clone())).is_err() {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )?)
    };
    let total = decomposition.as_ref().map_or(0, |d| d.sectors.len());
    emit(GenerationProgress::PhaseTiming {
        phase: GenerationPhase::Geometry,
        seconds: started.elapsed().as_secs_f64(),
    })?;
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
        .into_iter()
        .flat_map(|d| d.sectors)
        .enumerate()
    {
        emit(GenerationProgress::Factorization {
            sector: index,
            total,
        })?;
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
        emit(GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        })?;
        let started = Instant::now();
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
        let matched = registry.register(index, &parameters, &density)?;
        debug_assert_eq!(matched.permutation.len(), parameters.len());
        charts.push(ChartRecord {
            source_index: index,
            representative: matched.representative,
            representative_permutation: matched.permutation.clone(),
            kernel_sector: None,
            coordinates,
            geometry: map.clone(),
        });
        if matched.representative == index {
            representatives.insert(index, (map, parameters, mapped, 1usize));
        } else {
            let representative = representatives
                .get_mut(&matched.representative)
                .ok_or_else(|| {
                    GenerationError::Invariant("missing symmetry representative".into())
                })?;
            representative.3 += 1;
        }
        emit(GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Symmetry,
            seconds: started.elapsed().as_secs_f64(),
        })?;
    }
    let total = representatives.len();
    let mut kernel_indices = BTreeMap::new();
    for (index, (representative_index, (map, parameters, mapped, multiplicity))) in
        representatives.into_iter().enumerate()
    {
        let started = Instant::now();
        let (expression, terms, cancellation_terms) =
            subtraction::subtract(mapped, &parameters, input.regulator(), options)?;
        let cancellation_degree = cancellation_terms
            .iter()
            .map(|row| row.iter().sum::<usize>())
            .max()
            .unwrap_or(0);
        emit(GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Subtraction,
            seconds: started.elapsed().as_secs_f64(),
        })?;
        emit(GenerationProgress::Subtraction {
            sector: index,
            total,
            terms,
        })?;
        emit(GenerationProgress::LaurentExpansion {
            sector: index,
            total,
        })?;
        let started = Instant::now();
        #[cfg(test)]
        laurent::profiling::context(index, representative_index, multiplicity);
        let coefficients = laurent::expand(
            &expression,
            &parameters,
            input.regulator(),
            options.max_order,
            &mut templates,
        )?
        .into_iter()
        .map(|(order, coefficient)| (order, coefficient * Atom::num(multiplicity)))
        .collect::<BTreeMap<_, _>>();
        emit(GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Laurent,
            seconds: started.elapsed().as_secs_f64(),
        })?;
        if let Some(order) = coefficients.keys().next() {
            minimum = minimum.min(*order);
        }
        if coefficients.values().all(|coefficient| {
            parameters
                .iter()
                .all(|p| !coefficient.contains(Atom::var(*p).as_view()))
        }) {
            for (order, coefficient) in coefficients {
                *exact.entry(order).or_insert(Atom::Zero) += coefficient;
            }
        } else {
            kernel_indices.insert(representative_index, pending.len());
            pending.push((
                map,
                parameters,
                coefficients,
                cancellation_degree,
                cancellation_terms,
            ));
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
            |(map, parameters, coefficients, cancellation_degree, cancellation_terms)| {
                GeneratedSector {
                    cancellation_degree,
                    cancellation_terms,
                    parameters,
                    map,
                    coefficients: orders
                        .iter()
                        .map(|order| coefficients.get(order).cloned().unwrap_or(Atom::Zero))
                        .collect(),
                }
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
    emit(GenerationProgress::Complete {
        sectors: result.sectors.len(),
        orders: result.orders.clone(),
    })?;
    Ok(result)
}
