//! Native branch-role admission. Undeformed generation retains caller-owned
//! threshold responsibility; contour generation requires real, designated F/U.
#[cfg(test)]
mod tests;
use super::{
    BranchPolicy, DomainAssessment, FactorAssessment, FactorCertificate, GenerationError,
    subtraction::rational,
};
use crate::parametric::{FactorRole, FactorSemantics, ParametricIntegrand, PolynomialFactor};
use fastsecdec_sectors::ParametricDomain;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::atom::AtomField,
};

pub(super) fn is_singular(factor: &PolynomialFactor) -> bool {
    factor.role() == FactorRole::Singularity
        && !rational(factor.exponent()).is_some_and(|power| power.is_integer() && power >= 0)
}

pub(super) fn is_geometry_factor(factor: &PolynomialFactor, contour: bool) -> bool {
    is_singular(factor) || (contour && factor.semantics() != FactorSemantics::Generic)
}

pub(super) fn check_options(
    input: &ParametricIntegrand,
    options: &super::GenerationOptions,
) -> Result<DomainAssessment, GenerationError> {
    let mut assessment = check(input, options.assume_no_threshold)?;
    if !options.contour_enabled() {
        return Ok(assessment);
    }
    if options.program_recipe.is_dynamic() && input.terms().is_empty() {
        return Err(GenerationError::Contour(
            "empty dynamic input requires an explicit empty-recipe descriptor; this admission is pending".into(),
        ));
    }
    let reserved = [
        crate::contour::lambda_symbol(),
        crate::contour::dynamic::safety_fraction_symbol(),
        crate::contour::dynamic::lambda_cap_symbol(),
        crate::contour::dynamic::displacement_cap_symbol(),
        crate::contour::dynamic::radius_fraction_symbol(),
    ];
    let source_expressions = input.terms().iter().flat_map(|term| {
        std::iter::once(term.prefactor())
            .chain(term.monomial_powers())
            .chain(
                term.factors()
                    .iter()
                    .flat_map(|factor| [factor.polynomial(), factor.exponent()]),
            )
    });
    if input
        .parameters()
        .iter()
        .any(|symbol| reserved.contains(symbol))
        || reserved.contains(&input.regulator())
        || source_expressions.into_iter().any(|expression| {
            reserved
                .iter()
                .any(|symbol| expression.contains_symbol(*symbol))
        })
    {
        return Err(GenerationError::Contour(
            "input collides with a reserved contour mathematical symbol".into(),
        ));
    }
    let mut source_f = None;
    for term in input.terms() {
        let causal = term
            .factors()
            .iter()
            .filter(|factor| factor.semantics() == FactorSemantics::Causal)
            .collect::<Vec<_>>();
        if causal.len() != 1 {
            return Err(GenerationError::Contour(
                "each term requires exactly one explicitly designated F polynomial".into(),
            ));
        }
        if let Some(previous) = source_f
            && previous != causal[0].polynomial()
        {
            return Err(GenerationError::Contour(
                "terms must share the same designated F polynomial".into(),
            ));
        }
        source_f = Some(causal[0].polynomial());
        for factor in term.factors() {
            if factor.semantics() != FactorSemantics::Generic
                && factor
                    .polynomial()
                    .contains(Atom::var(input.regulator()).as_view())
            {
                return Err(GenerationError::Contour(
                    "causal and positive polynomials must be regulator independent".into(),
                ));
            }
            if factor.semantics() != FactorSemantics::Generic
                && !crate::kernel::is_real_expression(
                    factor.polynomial(),
                    &factor
                        .polynomial()
                        .get_all_symbols(true)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
            {
                return Err(GenerationError::Contour("causal and positive polynomials must be real for real inputs; complex or unproven coefficient expressions are unsupported".into()));
            }
            if factor.semantics() == FactorSemantics::Generic && is_singular(factor) {
                return Err(GenerationError::Contour(
                    "every singular factor needs explicit causal or positive branch semantics"
                        .into(),
                ));
            }
        }
    }
    assessment.branch = BranchPolicy::CausalContour;
    Ok(assessment)
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
