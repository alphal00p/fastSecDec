//! Eliminate Gaussian sources as soon as their remaining derivative order is zero.

use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
};

/// The polynomial factor left after differentiating `U^a F^b` once for each
/// requested source occurrence. The caller retains the common U/F powers.
///
/// Evaluation at an unused source commutes with every remaining derivative.
/// A used source is eliminated only after its final derivative. All algebra and
/// differentiation remain native Atom operations; no numerator expansion is needed.
pub(super) fn source_moment(
    source_u: &Atom,
    source_f: &Atom,
    sources: &[Symbol],
    orders: &[u32],
    a: &Atom,
    b: &Atom,
) -> Atom {
    debug_assert_eq!(sources.len(), orders.len());
    let zero = |source| {
        Replacement::new(
            Pattern::Literal(Atom::var(source)),
            Pattern::Literal(Atom::Zero),
        )
    };
    let unused = sources
        .iter()
        .zip(orders)
        .filter(|(_, count)| **count == 0)
        .map(|(source, _)| zero(*source))
        .collect::<Vec<_>>();
    let mut u = source_u.replace_multiple(&unused);
    let mut f = source_f.replace_multiple(&unused);
    let mut differentiated = Atom::one();
    let mut completed = 0u64;
    for (&source, &count) in sources.iter().zip(orders) {
        if count == 0 {
            continue;
        }
        let du = u.derivative(source);
        let df = f.derivative(source);
        for _ in 0..count {
            // ∂[U^(a-r) F^(b-r) P] = U^(a-r-1) F^(b-r-1)
            // * [UF ∂P + ((a-r)F ∂U + (b-r)U ∂F)P].
            differentiated = &u * &f * differentiated.derivative(source)
                + ((a - Atom::num(completed)) * &f * &du + (b - Atom::num(completed)) * &u * &df)
                    * differentiated;
            completed += 1;
        }
        let replacement = [zero(source)];
        differentiated = differentiated.replace_multiple(&replacement);
        u = u.replace_multiple(&replacement);
        f = f.replace_multiple(&replacement);
    }
    differentiated
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{parse, symbol};

    #[test]
    fn eliminated_sources_match_native_mixed_derivatives() {
        let sources = [
            symbol!("moment_test::s"),
            symbol!("moment_test::t"),
            symbol!("moment_test::unused"),
        ];
        let [s, t, z] = sources.map(Atom::var);
        // Both U and F couple the retained sources to an unused source. Squared
        // and mixed orders exercise the required delay before eliminating s/t.
        let u = parse!("moment_test::x") + &s + &t + &s * &t + &z * (&s + &t);
        let f = parse!("moment_test::y") + &s * &s + &s * &t + &t * &t + &z * &s * &t;
        let a = Atom::num(7);
        let b = Atom::num(5);
        let zeroes = sources.map(|source| {
            Replacement::new(
                Pattern::Literal(Atom::var(source)),
                Pattern::Literal(Atom::Zero),
            )
        });
        let u0 = u.replace_multiple(&zeroes);
        let f0 = f.replace_multiple(&zeroes);
        for orders in [[0, 0, 0], [2, 0, 0], [1, 1, 0], [2, 2, 0]] {
            let degree = orders.iter().sum::<u32>();
            let mut expected = u.pow(a.clone()) * f.pow(b.clone());
            for (&source, &count) in sources.iter().zip(&orders) {
                for _ in 0..count {
                    expected = expected.derivative(source);
                }
            }
            expected = expected.replace_multiple(&zeroes);
            let actual = source_moment(&u, &f, &sources, &orders, &a, &b)
                * u0.pow(&a - Atom::num(degree))
                * f0.pow(&b - Atom::num(degree));
            assert!((actual - expected).expand().is_zero(), "orders={orders:?}");
        }
    }
}
