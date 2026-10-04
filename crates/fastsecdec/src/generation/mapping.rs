use super::GenerationError;
use crate::parametric::{FactorRole, ParametricIntegrand};
use fastsecdec_sectors::SectorMap;
use std::collections::BTreeMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::integer::Integer,
    id::Replacement,
};

pub(super) struct MappedTerm {
    pub powers: Vec<Atom>,
    pub regular: Atom,
}

/// Extract monomials using exact Newton support, leaving nonnegative powers in
/// every residual polynomial. Numerator factors share the map but do not refine
/// the fan. Symbolica owns substitution, expansion and coefficient arithmetic.
pub(super) fn map_terms(
    input: &ParametricIntegrand,
    map: &SectorMap,
    parameters: &[Symbol],
) -> Result<Vec<MappedTerm>, GenerationError> {
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let images = map
        .exponent_matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(&variables)
                .map(|(power, variable)| variable.pow(Atom::num(power.clone())))
                .product::<Atom>()
        })
        .collect::<Vec<_>>();
    let mut combined = BTreeMap::<Vec<Atom>, Atom>::new();
    for term in input.terms() {
        let mut powers = map
            .jacobian_powers
            .iter()
            .cloned()
            .map(Atom::num)
            .collect::<Vec<_>>();
        for (row, power) in map.exponent_matrix.iter().zip(term.monomial_powers()) {
            for (current, exponent) in powers.iter_mut().zip(row) {
                *current += power * Atom::num(exponent.clone());
            }
        }
        let mut regular = term.prefactor() * Atom::num(map.determinant.clone());
        for factor in term.factors() {
            if factor.exponent().is_zero() {
                continue;
            }
            let support = factor.support(input.parameters())?;
            let transformed = support
                .exponents()
                .iter()
                .map(|row| {
                    (0..parameters.len())
                        .map(|j| {
                            row.iter()
                                .zip(&map.exponent_matrix)
                                .fold(Integer::from(0), |sum, (a, m)| sum + a * &m[j])
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let minima = (0..parameters.len())
                .map(|j| transformed.iter().map(|row| &row[j]).min().unwrap().clone())
                .collect::<Vec<_>>();
            let mapped =
                factor
                    .polynomial()
                    .replace_multiple(input.parameters().iter().zip(&images).map(
                        |(source, target)| {
                            Replacement::new(Atom::var(*source).to_pattern(), target.to_pattern())
                        },
                    ));
            let inverse_monomial = variables
                .iter()
                .zip(&minima)
                .map(|(variable, power)| variable.pow(Atom::num(-power)))
                .product::<Atom>();
            let residual = (mapped * inverse_monomial).expand();
            // Check the actual expression after substitution, rather than trusting
            // exponent supports when coefficients might have cancelled.
            crate::parametric::polynomial_support(&residual, parameters)?;
            let regular_polynomial = super::subtraction::rational(factor.exponent())
                .is_some_and(|power| power.is_integer() && power >= 0);
            if factor.role() == FactorRole::Singularity && !regular_polynomial {
                super::domain::check_residual(&residual, parameters)?;
            }
            for (power, valuation) in powers.iter_mut().zip(minima) {
                *power += factor.exponent() * Atom::num(valuation);
            }
            regular *= residual.pow(factor.exponent());
        }
        let powers = powers.into_iter().map(|p| p.expand()).collect();
        *combined.entry(powers).or_insert(Atom::Zero) += regular;
    }
    Ok(combined
        .into_iter()
        .filter(|(_, regular)| !regular.is_zero())
        .map(|(powers, regular)| MappedTerm { powers, regular })
        .collect())
}
