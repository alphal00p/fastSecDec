//! Direct monomial substitution, exact endpoint subtraction, and Laurent vectors.
//!
//! Symbolica owns all polynomial manipulation, derivatives and series expansions.
//! This module owns the integral-specific order of these operations.
mod domain;
mod laurent;
mod mapping;
mod subtraction;
mod types;

pub use types::{
    GeneratedIntegral, GeneratedSector, GenerationError, GenerationOptions, GenerationProgress,
};

use crate::parametric::{FactorRole, ParametricIntegrand};
use fastsecdec_sectors::{PolynomialSupport, decompose};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{
    atom::{Atom, AtomCore},
    symbol,
};

pub fn generate(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<GeneratedIntegral, GenerationError> {
    domain::check(input, options.assume_no_threshold)?;
    let mut emit = |status| {
        if progress(&status).is_break() {
            Err(GenerationError::Cancelled)
        } else {
            Ok(())
        }
    };
    let mut supports = Vec::new();
    for term in input.terms() {
        for factor in term.factors() {
            if factor.role() == FactorRole::Singularity && !factor.exponent().is_zero() {
                let support = factor.support(input.parameters())?;
                if !supports.contains(&support) {
                    supports.push(support);
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
    let source_symbols = input.density().get_all_symbols(true);
    let mut pending = Vec::new();
    let mut exact = BTreeMap::<i32, Atom>::new();
    let mut minimum = options.max_order.min(0);
    let mut templates = laurent::TemplateCache::default();
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
        let mapped = mapping::map_terms(input, &map, &parameters)?;
        let (expression, terms, cancellation_degree) =
            subtraction::subtract(mapped, &parameters, input.regulator(), options)?;
        emit(GenerationProgress::Subtraction {
            sector: index,
            terms,
        })?;
        emit(GenerationProgress::LaurentExpansion {
            sector: index,
            total,
        })?;
        let coefficients = laurent::expand(
            &expression,
            &parameters,
            input.regulator(),
            options.max_order,
            &mut templates,
        )?;
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
            pending.push((map, parameters, coefficients, cancellation_degree));
        }
    }
    let orders = (minimum..=options.max_order).collect::<Vec<_>>();
    let sectors = pending
        .into_iter()
        .map(
            |(map, parameters, coefficients, cancellation_degree)| GeneratedSector {
                cancellation_degree,
                parameters,
                map,
                coefficients: orders
                    .iter()
                    .map(|order| coefficients.get(order).cloned().unwrap_or(Atom::Zero))
                    .collect(),
            },
        )
        .collect();
    let exact_coefficients = orders
        .iter()
        .map(|order| exact.get(order).cloned().unwrap_or(Atom::Zero))
        .collect();
    let result = GeneratedIntegral {
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
