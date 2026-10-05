//! Native one-coordinate collection for an already admitted regular polynomial.
//! Other coordinates remain factored Atom coefficients instead of full support.

use super::{Atom, AtomCore, Integer, Pattern, factored_residual};
use symbolica::domains::atom::AtomField;

pub(super) fn common_monomial(mapped: &Atom, variables: &[Atom]) -> Option<(Vec<Integer>, Atom)> {
    if mapped.is_zero() || variables.is_empty() {
        return None;
    }
    let field = AtomField {
        statistical_zero_test: false,
        ..AtomField::new()
    };
    let mut minima = Vec::with_capacity(variables.len());
    let mut shifts = Vec::with_capacity(variables.len());
    for variable in variables {
        // A structurally nonzero face may still cancel identically. Keeping
        // power zero is conservative and avoids collecting a large regular
        // power in a coordinate that needs no monomial removed.
        let minimum = if !mapped
            .replace(Pattern::Literal(variable.clone()))
            .with(Atom::Zero)
            .is_zero()
        {
            0
        } else {
            // Match the original source-support exponent type. A factored
            // product can exceed the signed residual range even when each
            // factor fits; check both bounds before proposing a signed shift.
            let polynomial = mapped
                .to_polynomial_in_vars_with_field::<u32>(std::slice::from_ref(variable), &field);
            if polynomial.is_zero()
                || (&polynomial)
                    .into_iter()
                    .any(|term| term.coefficient.contains(variable.as_view()))
            {
                return None;
            }
            let (minimum, maximum) = polynomial.degree_bounds(0);
            i32::try_from(maximum).ok()?;
            i32::try_from(minimum).ok()?
        };
        minima.push(Integer::from(minimum));
        shifts.push(minimum.checked_neg()?);
    }
    // Literal coefficient-zero testing can retain a disguised zero coefficient,
    // underestimating a valuation. We only propose a common monomial: native
    // collection and polynomial recognition must admit its complete quotient.
    // This is neither a maximal-valuation nor a nonzero certificate.
    let residual = factored_residual(mapped, variables, &shifts)?;
    Some((minima, residual))
}
