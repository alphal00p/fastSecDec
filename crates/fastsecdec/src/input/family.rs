//! Input orchestration around HEPKit's native family and Symbolica substitutions.

use std::collections::{BTreeMap, HashSet};

use feynkit_graph::{IntegralFamily, IntegralFamilyError};
use feynkit_kinematics::Kinematics;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
};

use crate::{Error, Result};

pub(super) fn bind_scalar_values(expression: &Atom, values: &BTreeMap<Symbol, Atom>) -> Atom {
    expression.replace_multiple(values.iter().map(|(symbol, value)| {
        Replacement::new(
            Pattern::Literal(Atom::var(*symbol)),
            Pattern::Literal(value.clone()),
        )
    }))
}

/// Rebuild only through native family/kinematics constructors. Keeping this
/// shared with GraphIntegral makes scalar points and auxiliary vectors obey the
/// same rules for both public input owners.
pub(super) fn specialize_family(
    family: &IntegralFamily,
    kinematics: &Kinematics,
    external_momenta: Vec<Atom>,
    values: &BTreeMap<Symbol, Atom>,
) -> Result<IntegralFamily> {
    let dimension = family.kinematics().dimension().to_symbolic();
    if kinematics.dimension().to_symbolic() != dimension {
        return Err(Error::ScalarBindings(
            "kinematics must retain the family's tensor dimension; use the integration dimension argument instead"
                .into(),
        ));
    }
    // Spenso's native scalar products encode even a symbol momentum k as the
    // function head k(mink(D)). A literal search for Atom::var(k) alone misses
    // that dependence. Use Symbolica's public traversal including function
    // symbols, without expanding or reconstructing the scalar products.
    let momentum_symbols: HashSet<_> = family
        .loop_momenta()
        .iter()
        .chain(&external_momenta)
        .flat_map(|momentum| momentum.get_all_symbols(true))
        .collect();
    for (key, value) in values {
        let momentum_key = momentum_symbols.contains(key);
        let key = Atom::var(*key);
        let value_symbols = value.get_all_symbols(true);
        if dimension.contains(key.as_view())
            || momentum_key
            || family
                .loop_momenta()
                .iter()
                .chain(&external_momenta)
                .any(|p| p.contains(key.as_view()))
            || value_symbols.contains(&feynkit_graph::symbols::loop_momentum())
            || momentum_symbols
                .iter()
                .any(|symbol| value_symbols.contains(symbol))
            || family
                .loop_momenta()
                .iter()
                .chain(&external_momenta)
                .any(|p| value.contains(p.as_view()))
            || values
                .keys()
                .any(|symbol| value.contains(Atom::var(*symbol).as_view()))
        {
            return Err(Error::ScalarBindings(
                "bindings must be closed scalar values and cannot replace the tensor dimension or formal momentum identities"
                    .into(),
            ));
        }
    }
    // Native family construction may already have substituted these products
    // into its stored denominators. Never pretend that a new value can undo
    // that substitution. Compare after the same closed scalar point so a
    // symbolic invariant and its explicit specialization remain compatible.
    for (i, p) in family.external_momenta().iter().enumerate() {
        for q in &family.external_momenta()[i..] {
            let old = family
                .kinematics()
                .scalar_product(p, q)
                .map_err(IntegralFamilyError::from)?;
            let new = kinematics
                .scalar_product(p, q)
                .map_err(IntegralFamilyError::from)?;
            if bind_scalar_values(&old, values) != bind_scalar_values(&new, values) {
                return Err(Error::ScalarBindings(
                    "kinematics cannot change existing family external products; specialize their symbolic invariants with scalar_values or construct a new native family"
                        .into(),
                ));
            }
        }
    }
    let mut kinematics = kinematics
        .clone()
        .with_momenta(
            family
                .loop_momenta()
                .iter()
                .chain(&external_momenta)
                .cloned(),
        )
        .map_err(IntegralFamilyError::from)?;
    for (i, p) in external_momenta.iter().enumerate() {
        for q in &external_momenta[i..] {
            let product = kinematics
                .scalar_product(p, q)
                .map_err(IntegralFamilyError::from)?;
            kinematics = kinematics
                .with_scalar_product(p, q, bind_scalar_values(&product, values))
                .map_err(IntegralFamilyError::from)?;
        }
    }
    Ok(IntegralFamily::new(
        family.loop_momenta().to_vec(),
        external_momenta,
        family
            .denominators()
            .iter()
            .map(|d| bind_scalar_values(d, values))
            .collect(),
        &kinematics,
    )?)
}

/// Specialize and project one explicitly weighted native family term.
///
/// `powers` supplies one signed power per ordered denominator, including any
/// auxiliary slots introduced by native family completion. Only positive
/// powers enter the projected family; a negative power multiplies the weighted
/// numerator by that denominator to its absolute power. Zero powers are omitted.
/// Native [`IntegralFamily::sector`] owns this projection. No loop shifts,
/// partial fractions, tensor contractions or scaleless-zero guesses are made.
///
/// `weighted_numerator` must already be a contracted scalar, including graph,
/// projector and extra measure factors exactly once. Scalar bindings are applied
/// consistently to this numerator, all denominators and external pair products.
/// Kinematics defaults to the family's own assumptions. An override retains its
/// tensor dimension and original external products after scalar binding: native
/// construction may already have substituted them into denominators. It may add
/// assumptions for extra numerator vectors, which must be explicitly listed in
/// `auxiliary_momenta`, with their pair products in the kinematics.
///
/// The returned native family, positive powers and numerator can be passed to
/// [`crate::parametric::ParametricIntegrand::from_family`]. Its dimension argument
/// sets the integration dimension (normally `4-2*epsilon`); symbolic tensor
/// dimensions are substituted there, while incompatible concrete dimensions
/// remain errors. This helper does not compile or evaluate the integral.
pub fn prepare_family_input(
    family: &IntegralFamily,
    powers: &[i32],
    weighted_numerator: Atom,
    kinematics: Option<&Kinematics>,
    scalar_values: &BTreeMap<Symbol, Atom>,
    auxiliary_momenta: &[Atom],
) -> Result<(IntegralFamily, Vec<u32>, Atom)> {
    if powers.len() != family.denominators().len() {
        return Err(IntegralFamilyError::InvalidPowers.into());
    }
    if !powers.iter().any(|power| *power > 0) {
        return Err(IntegralFamilyError::InvalidBasis(
            "family input requires at least one positive propagator power".into(),
        )
        .into());
    }
    // Reject overflow before attempting either symbolic multiplication or a
    // native family rebuild. i32::MIN has no representable positive counterpart.
    for power in powers.iter().filter(|p| **p < 0) {
        power
            .checked_neg()
            .ok_or(IntegralFamilyError::PowerOverflow)?;
    }
    let mut external_momenta = family.external_momenta().to_vec();
    external_momenta.extend_from_slice(auxiliary_momenta);
    let family = specialize_family(
        family,
        kinematics.unwrap_or_else(|| family.kinematics()),
        external_momenta,
        scalar_values,
    )?;
    let mut numerator = family
        .kinematics()
        .apply(&bind_scalar_values(&weighted_numerator, scalar_values));
    for (denominator, power) in family.denominators().iter().zip(powers) {
        if *power < 0 {
            numerator *= denominator.pow(Atom::num(-*power));
        }
    }
    let projected = family.sector(powers)?;
    let positive_powers = powers
        .iter()
        .filter(|power| **power > 0)
        .map(|power| *power as u32)
        .collect();
    Ok((projected, positive_powers, numerator))
}
