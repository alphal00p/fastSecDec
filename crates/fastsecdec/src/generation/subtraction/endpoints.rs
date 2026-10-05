//! Shared exact endpoint admission. Native Symbolica owns affine recognition,
//! rational arithmetic and the floor determining the subtraction degree.

use super::GenerationError;
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::Coefficient,
    domains::rational::Rational,
    id::Pattern,
};

#[cfg(test)]
mod tests;

pub(in crate::generation) struct EndpointAdmission {
    pub constant: Rational,
    pub slope: Rational,
    pub subtractions: usize,
}

pub(in crate::generation) fn rational(expression: &Atom) -> Option<Rational> {
    if expression.is_zero() {
        return Some(Rational::from(0));
    }
    let AtomView::Num(number) = expression.as_view() else {
        return None;
    };
    match number.get_coeff_view().to_owned() {
        Coefficient::Complex(value) if value.im.is_zero() => Some(value.re),
        _ => None,
    }
}

pub(in crate::generation) fn endpoint_power(
    expression: &Atom,
    regulator: Symbol,
) -> Result<(Rational, Rational), GenerationError> {
    let epsilon = Atom::var(regulator);
    let constant = expression
        .replace(Pattern::Literal(epsilon.clone()))
        .with(Atom::Zero)
        .expand();
    let slope = expression.derivative(regulator).expand();
    if rational(&slope).is_none()
        || !(expression - &constant - &slope * epsilon)
            .expand()
            .is_zero()
    {
        return Err(GenerationError::EndpointExponent(expression.clone()));
    }
    rational(&constant)
        .map(|a| (a, rational(&slope).unwrap()))
        .ok_or_else(|| GenerationError::EndpointExponent(expression.clone()))
}

/// Admit the exponent before checking its degree, preserving error precedence.
/// A zero slope is allowed here: only actual nonzero boundary coefficients can
/// establish an unregulated divergence in endpoint subtraction.
pub(in crate::generation) fn admit(
    expression: &Atom,
    regulator: Symbol,
    max_subtractions: usize,
) -> Result<EndpointAdmission, GenerationError> {
    let (constant, slope) = endpoint_power(expression, regulator)?;
    let subtractions = subtraction_count(&constant, max_subtractions)?;
    Ok(EndpointAdmission {
        constant,
        slope,
        subtractions,
    })
}

fn subtraction_count(constant: &Rational, maximum: usize) -> Result<usize, GenerationError> {
    if *constant > -1 {
        return Ok(0);
    }
    // Retain the existing native integer conversion boundary as well as the
    // caller's limit; this extraction does not enlarge the admitted degrees.
    let count = (-constant.clone())
        .floor()
        .to_i64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(GenerationError::ResourceLimit("Taylor subtraction degree"))?;
    if count > maximum {
        return Err(GenerationError::ResourceLimit("Taylor subtraction degree"));
    }
    Ok(count)
}

/// Checked total degree of the retained physical endpoint profiles.
pub(in crate::generation) fn checked_cancellation_degree(
    rows: &[Vec<usize>],
) -> Result<usize, GenerationError> {
    rows.iter().try_fold(0, |maximum, row| {
        let degree = row.iter().try_fold(0usize, |sum, power| {
            sum.checked_add(*power)
                .ok_or(GenerationError::ResourceLimit(
                    "endpoint cancellation degree",
                ))
        })?;
        Ok(maximum.max(degree))
    })
}
