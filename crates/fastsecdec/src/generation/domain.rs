//! Conservative, exact no-threshold checks. Numerical sampling is never used
//! as a positivity certificate. Rational witnesses can disprove the assertion.
use super::{
    BranchPolicy, DomainAssessment, FactorAssessment, FactorCertificate, GenerationError,
    subtraction::rational,
};
use crate::parametric::{FactorRole, ParametricIntegrand, PolynomialFactor};
use fastsecdec_sectors::ParametricDomain;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::rational::Rational,
    id::{Pattern, Replacement},
};

pub(super) fn is_singular(factor: &PolynomialFactor) -> bool {
    factor.role() == FactorRole::Singularity
        && !rational(factor.exponent()).is_some_and(|power| power.is_integer() && power >= 0)
}

/// Certify actual resolved residuals, including exceptional faces introduced by
/// the Newton map. Original-domain face checks cannot detect a vanishing leading
/// form such as (x-y)^2 at a finite ratio after blowing up the origin.
pub(super) fn check_residual(
    residual: &Atom,
    parameters: &[Symbol],
) -> Result<(), GenerationError> {
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let uniform_sign = |polynomial: &Atom| {
        if polynomial.is_zero() {
            return false;
        }
        let polynomial = polynomial.to_polynomial_in_vars::<u32>(&variables);
        let signs = (&polynomial)
            .into_iter()
            .map(|term| rational(term.coefficient))
            .collect::<Option<Vec<_>>>();
        signs.is_some_and(|values| {
            values.iter().all(|value| value > &Rational::from(0))
                || values.iter().all(|value| value < &Rational::from(0))
        })
    };
    let constant = residual
        .to_polynomial_in_vars::<u32>(&variables)
        .coefficient(&vec![0; variables.len()]);
    if !constant.as_ref().is_some_and(|value| !value.is_zero()) {
        return Err(GenerationError::Invariant(
            "resolved singular polynomial has no nonzero constant term".into(),
        ));
    }
    if uniform_sign(residual) {
        return Ok(());
    }
    let resolved = variables.iter().all(|variable| {
        [0, 1].into_iter().all(|endpoint| {
            let face = residual
                .replace(Pattern::Literal(variable.clone()))
                .with(Atom::num(endpoint))
                .expand();
            if !uniform_sign(&face) {
                return false;
            }
            // A nonzero constant plus coefficients of one sign bounds this face
            // away from zero on its entire closed cube, including intersections.
            let constant = face.replace_multiple(variables.iter().map(|variable| {
                Replacement::new(
                    Pattern::Literal(variable.clone()),
                    Pattern::Literal(Atom::Zero),
                )
            }));
            rational(&constant).is_some_and(|value| !value.is_zero())
        })
    });
    if resolved {
        Ok(())
    } else {
        Err(GenerationError::UnresolvedBoundary(residual.clone()))
    }
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
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let mut factors = Vec::new();
    for &(term_index, factor_index, ref factor) in supplied {
        if !is_singular(factor) {
            continue;
        }
        let exponent = rational(factor.exponent());
        let polynomial = factor.polynomial().to_polynomial_in_vars::<u32>(&variables);
        let signs = (&polynomial)
            .into_iter()
            .map(|term| rational(term.coefficient))
            .collect::<Option<Vec<_>>>();
        let positive = signs
            .as_ref()
            .is_some_and(|s| s.iter().all(|c| c > &Rational::from(0)));
        let negative = signs
            .as_ref()
            .is_some_and(|s| s.iter().all(|c| c < &Rational::from(0)));
        if positive {
            factors.push(FactorAssessment {
                term_index,
                factor_index,
                polynomial: factor.polynomial().clone(),
                exponent: factor.exponent().clone(),
                certificate: FactorCertificate::PositiveCoefficients,
            });
            continue;
        }
        if negative {
            if exponent.as_ref().is_some_and(Rational::is_integer) {
                factors.push(FactorAssessment {
                    term_index,
                    factor_index,
                    polynomial: factor.polynomial().clone(),
                    exponent: factor.exponent().clone(),
                    certificate: FactorCertificate::NegativeCoefficientsIntegerPower,
                });
                continue;
            }
            return Err(GenerationError::ComplexBranch(factor.polynomial().clone()));
        }
        // Mixed-sign cube factors can vanish at an upper corner, where a
        // Newton fan at the origin alone cannot resolve the singularity.
        // Be conservative until affine endpoint charts are constructed.
        if domain == ParametricDomain::UnitCube {
            let corners = 1usize
                .checked_shl(variables.len() as u32)
                .filter(|n| *n <= 65_536)
                .ok_or(GenerationError::ResourceLimit(
                    "mixed-sign cube boundary analysis",
                ))?;
            for corner in 1..corners {
                let value = factor
                    .polynomial()
                    .replace_multiple(variables.iter().enumerate().map(|(axis, variable)| {
                        Replacement::new(
                            Pattern::Literal(variable.clone()),
                            Pattern::Literal(Atom::num((corner >> axis) & 1)),
                        )
                    }))
                    .expand();
                if value.is_zero() {
                    return Err(GenerationError::UpperBoundary(factor.polynomial().clone()));
                }
            }
        }
        let values = if domain == ParametricDomain::UnitCube {
            vec![
                Rational::from((1, 4)),
                Rational::from((1, 2)),
                Rational::from((3, 4)),
            ]
        } else {
            vec![Rational::from((1, 4)), Rational::from(1), Rational::from(4)]
        };
        let mut points = values
            .iter()
            .map(|v| vec![v.clone(); variables.len()])
            .collect::<Vec<_>>();
        for axis in 0..variables.len() {
            for value in &values {
                let mut point = vec![values[1].clone(); variables.len()];
                point[axis] = value.clone();
                points.push(point);
            }
        }
        let (mut seen_positive, mut seen_negative) = (false, false);
        for point in points {
            let value = factor
                .polynomial()
                .replace_multiple(variables.iter().zip(point).map(|(variable, value)| {
                    Replacement::new(
                        Pattern::Literal(variable.clone()),
                        Pattern::Literal(Atom::num(value)),
                    )
                }))
                .expand();
            if let Some(value) = rational(&value) {
                seen_positive |= value > 0;
                seen_negative |= value < 0;
                if value.is_zero() || seen_positive && seen_negative {
                    return Err(GenerationError::Threshold(factor.polynomial().clone()));
                }
            }
        }
        if seen_negative && !exponent.as_ref().is_some_and(Rational::is_integer) {
            return Err(GenerationError::ComplexBranch(factor.polynomial().clone()));
        }
        if !asserted {
            return Err(GenerationError::UnknownDomain(factor.polynomial().clone()));
        }
        // An assertion concerns the open domain only. It cannot replace
        // resolution of zeros inside a boundary face. Certify every cube
        // face by the same exact coefficient-sign test. An identically
        // zero face is inconclusive here: after removing its coordinate
        // factor, the leading coefficient may still have an affine zero.
        // More general mixed-sign orthant/projective boundary geometry
        // requires affine charts and remains deliberately unsupported.
        let resolved = domain == ParametricDomain::UnitCube
            && variables.iter().all(|variable| {
                [0, 1].into_iter().all(|endpoint| {
                    let face = factor
                        .polynomial()
                        .replace(Pattern::Literal(variable.clone()))
                        .with(Atom::num(endpoint))
                        .expand();
                    if face.is_zero() {
                        return false;
                    }
                    let coefficients = face.to_polynomial_in_vars::<u32>(&variables);
                    let signs = (&coefficients)
                        .into_iter()
                        .map(|term| rational(term.coefficient))
                        .collect::<Option<Vec<_>>>();
                    signs.is_some_and(|values| {
                        values.iter().all(|v| v > &Rational::from(0))
                            || values.iter().all(|v| v < &Rational::from(0))
                    })
                })
            });
        if !resolved {
            return Err(GenerationError::UnresolvedBoundary(
                factor.polynomial().clone(),
            ));
        }
        factors.push(FactorAssessment {
            term_index,
            factor_index,
            polynomial: factor.polynomial().clone(),
            exponent: factor.exponent().clone(),
            certificate: FactorCertificate::ExplicitInteriorAssertion,
        });
    }
    Ok(DomainAssessment {
        parameters: parameters.to_vec(),
        domain,
        branch: BranchPolicy::NoThresholdReal,
        caller_asserted: asserted,
        factors,
    })
}
