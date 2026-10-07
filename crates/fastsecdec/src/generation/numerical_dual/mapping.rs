//! Exact integer valuations without substituting sector maps into source Atoms.
use super::{DualFactor, DualTerm};
use crate::generation::{
    CoordinateMap, GenerationError, mapping::MappedTerm, support::SupportCache,
};
use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::SectorMap;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::integer::Integer,
};

pub(super) fn prepare(
    input: &ParametricIntegrand,
    map: &SectorMap,
    coordinates: &CoordinateMap,
    supports: &mut SupportCache,
    valuations: &super::ValuationCache,
) -> Result<(Vec<MappedTerm>, Vec<DualTerm>), GenerationError> {
    let dimension = map.dimension();
    let mut mapped = Vec::with_capacity(input.terms().len());
    let mut terms = Vec::with_capacity(input.terms().len());
    for term in input.terms() {
        let mut powers = coordinates.measure_powers.clone();
        for (row, power) in map.exponent_matrix.iter().zip(term.monomial_powers()) {
            for (current, exponent) in powers.iter_mut().zip(row) {
                *current += power * Atom::num(exponent.clone());
            }
        }
        let mut factors = Vec::new();
        let mut regular = Atom::one();
        for factor in term.factors() {
            if factor.exponent().is_zero() {
                continue;
            }
            // A polynomial numerator remains regular under a nonnegative map.
            // Native one-scale leading Series supplies a conservative bound
            // without expanding its potentially enormous factored source support.
            let regular_polynomial_map = !crate::generation::domain::is_singular(factor)
                && map
                    .exponent_matrix
                    .iter()
                    .flatten()
                    .all(|power| power >= &0);
            let transformed = if regular_polynomial_map {
                Vec::new()
            } else {
                supports
                    .get(factor)?
                    .exponents()
                    .iter()
                    .map(|row| {
                        (0..dimension)
                            .map(|axis| {
                                row.iter()
                                    .zip(&map.exponent_matrix)
                                    .fold(Integer::from(0), |sum, (power, map)| {
                                        sum + power * &map[axis]
                                    })
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            };
            let minima = if regular_polynomial_map {
                valuations
                    .for_map(factor.polynomial(), map)?
                    .into_iter()
                    .map(Integer::from)
                    .collect()
            } else {
                (0..dimension)
                    .map(|axis| {
                        transformed
                            .iter()
                            .map(|row| &row[axis])
                            .min()
                            .cloned()
                            .ok_or_else(|| {
                                GenerationError::Invariant("empty source support".into())
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
            if crate::generation::domain::is_singular(factor)
                && !transformed.iter().any(|row| row == &minima)
            {
                return Err(GenerationError::Invariant(
                    "resolved singular polynomial has no constant residual monomial".into(),
                ));
            }
            let valuation = minima
                .iter()
                .map(|power| {
                    power
                        .to_i64()
                        .and_then(|power| i32::try_from(power).ok())
                        .ok_or(GenerationError::ResourceLimit(
                            "numerical dual monomial valuation",
                        ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            for (power, minimum) in powers.iter_mut().zip(minima) {
                *power += factor.exponent() * Atom::num(minimum);
            }
            // Retained only for symbol reservation and compact source metadata.
            // No source parameter is replaced by a sector coordinate here.
            regular *= factor.polynomial().pow(factor.exponent());
            factors.push(DualFactor {
                polynomial: factor.polynomial().clone(),
                exponent: factor.exponent().clone(),
                valuation,
            });
        }
        mapped.push(MappedTerm {
            powers: powers.into_iter().map(|power| power.expand()).collect(),
            prefactor: term.prefactor() * &coordinates.measure_factor,
            regular,
        });
        terms.push(DualTerm { factors });
    }
    Ok((mapped, terms))
}
