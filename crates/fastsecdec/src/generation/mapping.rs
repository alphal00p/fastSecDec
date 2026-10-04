use super::{CoordinateMap, GenerationError};
use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::SectorMap;
use std::collections::BTreeMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::integer::Integer,
    id::{Pattern, Replacement},
};

pub(super) struct MappedTerm {
    pub powers: Vec<Atom>,
    pub prefactor: Atom,
    pub regular: Atom,
}

/// Extract monomials using exact Newton support, leaving nonnegative powers in
/// every residual polynomial. Numerator factors share the map but do not refine
/// the fan. Symbolica owns substitution, sparse polynomial collection and
/// coefficient arithmetic; mapped factors stay factored unless a nonzero
/// monomial valuation must actually be removed.
pub(super) fn map_terms(
    input: &ParametricIntegrand,
    map: &SectorMap,
    coordinates: &CoordinateMap,
) -> Result<Vec<MappedTerm>, GenerationError> {
    let parameters = coordinates.target_parameters();
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let images = coordinates.images();
    let nonnegative_map = map
        .exponent_matrix
        .iter()
        .flatten()
        .all(|power| power >= &0);
    let mut combined = BTreeMap::<Vec<Atom>, BTreeMap<Atom, Atom>>::new();
    for term in input.terms() {
        let mut powers = coordinates.measure_powers.clone();
        for (row, power) in map.exponent_matrix.iter().zip(term.monomial_powers()) {
            for (current, exponent) in powers.iter_mut().zip(row) {
                *current += power * Atom::num(exponent.clone());
            }
        }
        let prefactor = term.prefactor() * &coordinates.measure_factor;
        let mut regular = Atom::one();
        for factor in term.factors() {
            if factor.exponent().is_zero() {
                continue;
            }
            let mapped =
                factor
                    .polynomial()
                    .replace_multiple(input.parameters().iter().zip(images).map(
                        |(source, target)| {
                            Replacement::new(
                                Pattern::Literal(Atom::var(*source)),
                                Pattern::Literal(target.clone()),
                            )
                        },
                    ));
            // For a polynomial map, nonvanishing coordinate faces exclude a
            // common coordinate monomial without enumerating dense support.
            // This preserves both (1+x)^10000 and (x+y)^10000. Singular factors
            // still receive exact polynomial residual/domain validation below.
            let zero_valuation = nonnegative_map
                && variables.iter().all(|variable| {
                    !mapped
                        .replace(Pattern::Literal(variable.clone()))
                        .with(Atom::Zero)
                        .is_zero()
                });
            let (minima, residual) = if zero_valuation {
                (vec![Integer::from(0); parameters.len()], mapped)
            } else {
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
                let residual = if minima.iter().all(|power| power == &0) {
                    mapped
                } else {
                    // Signed native polynomial exponents support orthant
                    // infinity charts. Bound conversion before invoking the
                    // fixed-width native collector and monomial shift.
                    let signed = |value: &Integer| {
                        value
                            .to_i64()
                            .and_then(|v| i32::try_from(v).ok())
                            .ok_or(GenerationError::ResourceLimit("mapped polynomial exponent"))
                    };
                    for row in &transformed {
                        for (power, minimum) in row.iter().zip(&minima) {
                            signed(power)?;
                            signed(&(power - minimum))?;
                        }
                    }
                    let shifts = minima
                        .iter()
                        .map(|value| signed(&-value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let polynomial = mapped
                        .to_polynomial_in_vars::<i32>(&variables)
                        .mul_exp(&shifts);
                    if polynomial
                        .exponents_iter()
                        .flatten()
                        .any(|power| *power < 0)
                    {
                        return Err(GenerationError::Invariant(
                            "negative exponent after monomial extraction".into(),
                        ));
                    }
                    polynomial.flatten(false)
                };
                (minima, residual)
            };
            if super::domain::is_singular(factor) {
                super::domain::check_residual(&residual, parameters)?;
            }
            for (power, valuation) in powers.iter_mut().zip(minima) {
                *power += factor.exponent() * Atom::num(valuation);
            }
            regular *= residual.pow(factor.exponent());
        }
        let powers = powers.into_iter().map(|p| p.expand()).collect();
        *combined
            .entry(powers)
            .or_default()
            .entry(prefactor)
            .or_insert(Atom::Zero) += regular;
    }
    Ok(combined
        .into_iter()
        .filter_map(|(powers, mut prefactors)| {
            // Preserve cancellation between terms with the same endpoint powers.
            // Only detach a genuinely common, parameter-independent factor.
            let (prefactor, regular) = if prefactors.len() == 1 {
                prefactors.pop_first().unwrap()
            } else {
                (
                    Atom::one(),
                    prefactors.into_iter().map(|(p, r)| p * r).sum(),
                )
            };
            (!regular.is_zero() && !prefactor.is_zero()).then_some(MappedTerm {
                powers,
                prefactor,
                regular,
            })
        })
        .collect())
}

pub(super) fn coordinates(
    input: &ParametricIntegrand,
    map: &SectorMap,
    parameters: &[Symbol],
) -> CoordinateMap {
    coordinates_from_parts(input.parameters(), input.domain(), map, parameters)
}

pub(crate) fn coordinates_from_parts(
    source: &[Symbol],
    domain: fastsecdec_sectors::ParametricDomain,
    map: &SectorMap,
    parameters: &[Symbol],
) -> CoordinateMap {
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let images = map
        .exponent_matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(&variables)
                .map(|(power, variable)| variable.pow(Atom::num(power.clone())))
                .product()
        })
        .collect();
    let measure_factor = Atom::num(map.determinant.clone());
    let measure_powers = map
        .jacobian_powers
        .iter()
        .cloned()
        .map(Atom::num)
        .collect::<Vec<_>>();
    let measure_jacobian = &measure_factor
        * measure_powers
            .iter()
            .zip(&variables)
            .map(|(power, variable)| variable.pow(power))
            .product::<Atom>();
    CoordinateMap {
        source_parameters: source.to_vec(),
        target_parameters: parameters.to_vec(),
        images,
        measure_jacobian,
        measure_factor,
        measure_powers,
        source_domain: domain,
        projective_fixed_parameter: map.fixed_parameter,
    }
}
