//! Native one-coordinate collection for an already admitted regular polynomial.
//! Other coordinates remain factored Atom coefficients instead of full support.

use super::{Atom, AtomCore, Integer, Pattern, factored_residual};
use symbolica::{atom::AtomView, domains::atom::AtomField};

pub(super) fn common_monomial(mapped: &Atom, variables: &[Atom]) -> Option<(Vec<Integer>, Atom)> {
    if mapped.is_zero() || variables.is_empty() {
        return None;
    }
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
            match structural_valuation(mapped, variable) {
                Some(minimum) => minimum,
                None => collected_valuation(mapped, variable)?,
            }
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

/// The minimum exponent of `variable` in the native one-variable collection of
/// `mapped` over Atom coefficients with literal zero testing. Collection
/// accumulates every coefficient by repeated normalized addition, which is
/// quadratic in the coefficient size for a large factored expression.
fn collected_valuation(mapped: &Atom, variable: &Atom) -> Option<i32> {
    let field = AtomField {
        statistical_zero_test: false,
        ..AtomField::new()
    };
    // Match the original source-support exponent type. A factored
    // product can exceed the signed residual range even when each
    // factor fits; check both bounds before proposing a signed shift.
    let polynomial =
        mapped.to_polynomial_in_vars_with_field::<u32>(std::slice::from_ref(variable), &field);
    if polynomial.is_zero()
        || (&polynomial)
            .into_iter()
            .any(|term| term.coefficient.contains(variable.as_view()))
    {
        return None;
    }
    let (minimum, maximum) = polynomial.degree_bounds(0);
    i32::try_from(maximum).ok()?;
    i32::try_from(minimum).ok()
}

/// The same minimum as [`collected_valuation`], found in one structural pass.
///
/// Sums take the minimum of their terms, products add and positive integer
/// powers multiply the valuations of their factors. The coefficient of the
/// proposed lowest power is the normalized sum, over the terms attaining it,
/// of the products of the lowest coefficients of their factors: the same
/// normalized Atom that native collection assigns to that power. Only when it
/// is literally zero, so that collection drops it and reports a larger
/// minimum, or when the expression is not a polynomial in `variable` within
/// the collected exponent range, does this return `None` and defer to
/// collection, which then also decides rejection.
fn structural_valuation(mapped: &Atom, variable: &Atom) -> Option<i32> {
    let lowest = Lowest::of(mapped.as_view(), variable.as_view())?;
    if lowest.coefficient.is_zero()
        || lowest.maximum > i64::from(i32::MAX)
        || lowest.minimum > lowest.maximum
    {
        return None;
    }
    i32::try_from(lowest.minimum).ok()
}

/// Exponent bounds of one variable without cancellation between terms, and
/// the literal coefficient of the lower bound.
struct Lowest {
    minimum: i64,
    maximum: i64,
    coefficient: Atom,
}

impl Lowest {
    fn of(view: AtomView<'_>, variable: AtomView<'_>) -> Option<Self> {
        if !view.contains(variable) {
            return Some(Self {
                minimum: 0,
                maximum: 0,
                coefficient: view.to_owned(),
            });
        }
        match view {
            AtomView::Var(_) => (view == variable).then(|| Self {
                minimum: 1,
                maximum: 1,
                coefficient: Atom::num(1),
            }),
            AtomView::Pow(pow) => {
                let (base, exponent) = pow.get_base_exp();
                let power = i64::try_from(exponent).ok().filter(|power| *power > 0)?;
                let base = Self::of(base, variable)?;
                Some(Self {
                    minimum: base.minimum.checked_mul(power)?,
                    maximum: base.maximum.checked_mul(power)?,
                    coefficient: base.coefficient.pow(Atom::num(power)),
                })
            }
            AtomView::Mul(mul) => {
                let (mut minimum, mut maximum) = (0i64, 0i64);
                let mut coefficients = Vec::with_capacity(mul.get_nargs());
                for factor in mul.iter() {
                    let factor = Self::of(factor, variable)?;
                    minimum = minimum.checked_add(factor.minimum)?;
                    maximum = maximum.checked_add(factor.maximum)?;
                    coefficients.push(factor.coefficient);
                }
                Some(Self {
                    minimum,
                    maximum,
                    coefficient: Atom::mul_many(coefficients.iter()),
                })
            }
            AtomView::Add(add) => {
                let terms = add
                    .iter()
                    .map(|term| Self::of(term, variable))
                    .collect::<Option<Vec<_>>>()?;
                let minimum = terms.iter().map(|term| term.minimum).min()?;
                let maximum = terms.iter().map(|term| term.maximum).max()?;
                Some(Self {
                    minimum,
                    maximum,
                    coefficient: Atom::add_many(
                        terms
                            .iter()
                            .filter(|term| term.minimum == minimum)
                            .map(|term| &term.coefficient),
                    ),
                })
            }
            AtomView::Num(_) | AtomView::Fun(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::parse;

    /// The structural pass agrees with native collection whenever it decides,
    /// and defers exactly when the lowest literal coefficient cancels or the
    /// expression is not polynomial in the coordinate.
    #[test]
    fn structural_valuation_matches_native_collection() {
        let t = parse!("regular_test::t");
        let mut decided = 0;
        let mut deferred = 0;
        for (text, collected_minimum) in [
            ("t^3*(1+t*y)^4+t^5*y", Some(3)),
            ("(t*y+t^2)^3*(y+t)^2-t^4", Some(3)),
            ("t^2*(y-1)^2*t+t^3*y^7", Some(3)),
            ("(1+t)*(2+y)+t^9", Some(0)),
            // Lowest literal coefficients cancel; collection finds t^3.
            ("t^2*(a+t)-t^2*a", Some(3)),
            ("2*(t*y+t^2)^2-2*(t*y+t^2)^2*(1+t)", Some(3)),
            // Not polynomial in t: both reject.
            ("t*f(t)", None),
            ("t^-1+y", None),
        ] {
            let mapped = parse!(text, default_namespace = "regular_test");
            let collected = collected_valuation(&mapped, &t);
            assert_eq!(collected, collected_minimum, "{text}");
            match structural_valuation(&mapped, &t) {
                Some(minimum) => {
                    decided += 1;
                    assert_eq!(Some(minimum), collected, "{text}");
                }
                None => {
                    deferred += 1;
                    let lowest = Lowest::of(mapped.as_view(), t.as_view());
                    assert!(
                        lowest.is_none_or(|lowest| lowest.coefficient.is_zero()),
                        "{text}"
                    );
                }
            }
        }
        assert_eq!((decided, deferred), (4, 4));
    }

    /// Regression for the factored numerator: a sum of products of powers of
    /// unexpanded polynomials with symbolic, regulator-dependent coefficients
    /// (the shape of a source moment in its jets) is decided by the structural
    /// pass alone, with the minimum and residual of native collection. Before,
    /// collection over Atom coefficients made one such chart of the five-loop
    /// P1 numerator take 427-514 s and 5.4-9.0 GB.
    #[test]
    fn structural_valuation_decides_factored_jet_sums() {
        let t = parse!("regular_test::t");
        let y = parse!("regular_test::y");
        let jet = |shift: i64| {
            parse!("regular_test::t*regular_test::y+regular_test::t^2")
                + Atom::num(shift) * parse!("regular_test::t^3*regular_test::z")
        };
        let mapped = (1..=24)
            .map(|i| {
                (parse!("regular_test::a") + Atom::num(i) * parse!("regular_test::eps"))
                    * jet(i).pow(Atom::num(3 + i % 4))
                    * (&y + &t).pow(Atom::num(i % 3))
            })
            .sum::<Atom>();
        assert_eq!(structural_valuation(&mapped, &t), Some(3));
        assert_eq!(
            structural_valuation(&mapped, &t),
            collected_valuation(&mapped, &t)
        );
        let variables = [t.clone(), y.clone()];
        let (minima, residual) = common_monomial(&mapped, &variables).unwrap();
        assert_eq!(minima, vec![Integer::from(3), Integer::from(0)]);
        let restored = residual * t.pow(Atom::num(3));
        assert!((restored - &mapped).expand().is_zero());
    }
}
