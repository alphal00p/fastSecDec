//! Threshold regularity is the caller's responsibility. Generation records the
//! chosen domain and factors without testing signs, faces or interior points.
#[cfg(test)]
mod tests;
use super::{
    BranchPolicy, DomainAssessment, FactorAssessment, FactorCertificate, GenerationError,
    subtraction::rational,
};
use crate::parametric::{FactorRole, ParametricIntegrand, PolynomialFactor};
use fastsecdec_sectors::ParametricDomain;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::atom::AtomField,
};

pub(super) fn is_singular(factor: &PolynomialFactor) -> bool {
    factor.role() == FactorRole::Singularity
        && !rational(factor.exponent()).is_some_and(|power| power.is_integer() && power >= 0)
}

/// Check only the algebraic invariant guaranteed by monomial extraction.
/// A nonzero symbolic constant is sufficient; no claim is made about its value
/// at an integration point or about zeros elsewhere on the closed cube.
pub(super) fn check_residual(
    residual: &Atom,
    parameters: &[Symbol],
) -> Result<(), GenerationError> {
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let constant = residual
        .to_polynomial_in_vars_with_field::<u32>(
            &variables,
            &AtomField {
                statistical_zero_test: false,
                ..AtomField::new()
            },
        )
        .coefficient(&vec![0; variables.len()]);
    if !constant.as_ref().is_some_and(|value| !value.is_zero()) {
        return Err(GenerationError::Invariant(
            "resolved singular polynomial has no nonzero constant term".into(),
        ));
    }
    Ok(())
}

pub(super) fn check(
    input: &ParametricIntegrand,
    asserted: bool,
) -> Result<DomainAssessment, GenerationError> {
    let factors = input
        .terms()
        .iter()
        .enumerate()
        .flat_map(|(term_index, term)| {
            term.factors()
                .iter()
                .enumerate()
                .map(move |(factor_index, factor)| (term_index, factor_index, factor.clone()))
        })
        .collect::<Vec<_>>();
    check_factors(input.parameters(), input.domain(), &factors, asserted)
}

pub(crate) fn check_factors(
    parameters: &[Symbol],
    domain: ParametricDomain,
    supplied: &[(usize, usize, PolynomialFactor)],
    asserted: bool,
) -> Result<DomainAssessment, GenerationError> {
    Ok(DomainAssessment {
        parameters: parameters.to_vec(),
        domain,
        branch: BranchPolicy::UserResponsible,
        caller_asserted: asserted,
        factors: supplied
            .iter()
            .filter(|(_, _, factor)| is_singular(factor))
            .map(|(term_index, factor_index, factor)| FactorAssessment {
                term_index: *term_index,
                factor_index: *factor_index,
                polynomial: factor.polynomial().clone(),
                exponent: factor.exponent().clone(),
                certificate: FactorCertificate::UncheckedUserResponsibility,
            })
            .collect(),
    })
}
