use super::*;
use crate::generation::laurent::{self, TemplateCache};
use symbolica::{parse, symbol};

fn term(prefactor: Atom, regular: Atom, powers: Vec<Atom>) -> MappedTerm {
    MappedTerm {
        prefactor,
        regular,
        powers,
    }
}

fn compare(
    terms: Vec<MappedTerm>,
    parameters: &[Symbol],
    regulator: Symbol,
    options: GenerationOptions,
) -> (BTreeMap<i32, Atom>, Vec<Attempt>) {
    let (actual, attempts) = expand(&terms, parameters, regulator, &options).unwrap();
    let (density, _, _) =
        crate::generation::subtraction::subtract(terms, parameters, regulator, &options).unwrap();
    let expected = laurent::expand(
        &density,
        parameters,
        regulator,
        options.max_order,
        &mut TemplateCache::default(),
    )
    .unwrap();
    let orders = actual
        .keys()
        .chain(expected.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for order in orders {
        let difference = actual.get(&order).cloned().unwrap_or(Atom::Zero)
            - expected.get(&order).cloned().unwrap_or(Atom::Zero);
        assert!(
            difference.as_view().get_byte_size() < 128_000,
            "small-control comparison must not normalize a large expression"
        );
        assert!(
            difference.together().expand().is_zero(),
            "order {order}: {difference}"
        );
    }
    assert!(!attempts.is_empty());
    assert!(
        attempts
            .iter()
            .all(|attempt| attempt.width > 0 && attempt.seconds >= 0.0)
    );
    (actual, attempts)
}

#[test]
fn native_series_first_complete_vectors_match_both_subtraction_strategies() {
    let x = symbol!("series_first_control::x");
    let y = symbol!("series_first_control::y");
    let eps = symbol!("series_first_control::eps");
    let e = Atom::var(eps);
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        for (prefactor, regular, powers, maximum) in [
            (
                Atom::one(),
                parse!("(1+series_first_control::x)^(-1-series_first_control::eps)"),
                vec![-Atom::one() + &e, Atom::Zero],
                1,
            ),
            (
                parse!("gamma(2*series_first_control::eps)"),
                parse!("1/(1+series_first_control::x+series_first_control::y)"),
                vec![Atom::num(-2) + &e, Atom::num(-1) + 2 * &e],
                0,
            ),
            (
                parse!("series_first_control::eps*gamma(series_first_control::eps)"),
                parse!("1/(1+series_first_control::x)^2"),
                vec![Atom::num(-1) + &e, Atom::Zero],
                1,
            ),
            (
                parse!("gamma(series_first_control::eps)/series_first_control::eps^3"),
                parse!("exp(series_first_control::eps)-1-series_first_control::eps"),
                vec![Atom::num(-1) + &e, Atom::Zero],
                -2,
            ),
            (
                Atom::one(),
                parse!("(1+series_first_control::x)^(-1-series_first_control::eps)"),
                vec![Atom::num((-3, 2)) + &e, Atom::Zero],
                0,
            ),
            (
                parse!("2+3*𝑖"),
                parse!(
                    "1+series_first_control::eps*series_first_control::x+series_first_control::eps^2*series_first_control::y"
                ),
                vec![Atom::num(-1) + &e, Atom::num(-1) + 2 * &e],
                0,
            ),
        ] {
            compare(
                vec![term(prefactor, regular, powers)],
                &[x, y],
                eps,
                GenerationOptions {
                    max_order: maximum,
                    subtraction: strategy,
                    ..GenerationOptions::default()
                },
            );
        }
    }
}

#[test]
fn native_series_first_exact_poles_cancellation_and_unknown_zero_bounds() {
    let x = symbol!("series_first_analytic::x");
    let y = symbol!("series_first_analytic::y");
    let eps = symbol!("series_first_analytic::eps");
    let e = Atom::var(eps);
    let (coefficients, attempts) = compare(
        vec![term(Atom::one(), Atom::one(), vec![&e - 1, 2 * &e - 1])],
        &[x, y],
        eps,
        GenerationOptions::default(),
    );
    assert_eq!(coefficients, BTreeMap::from([(-2, Atom::num((1, 2)))]));
    assert!(
        attempts.len() > 1,
        "initial width must not cover two endpoint poles"
    );
    assert_eq!(attempts.last().unwrap().absolute_bound, "1");
    assert!(
        attempts.last().unwrap().pieces >= 4,
        "unknown-zero branches are retained"
    );
    let prefactor = parse!("gamma(series_first_analytic::eps)/series_first_analytic::eps^3");
    let (zero, _) = compare(
        vec![
            term(prefactor.clone(), Atom::one(), vec![&e - 1]),
            term(prefactor, -Atom::one(), vec![&e - 1]),
        ],
        &[x],
        eps,
        GenerationOptions::default(),
    );
    assert!(zero.is_empty());
    let (positive, _) = compare(
        vec![term(e.pow(Atom::num(6)), Atom::one(), vec![Atom::Zero])],
        &[x],
        eps,
        GenerationOptions {
            max_order: -2,
            ..GenerationOptions::default()
        },
    );
    assert!(positive.is_empty());
    assert!(
        expand(&[], &[x], eps, &GenerationOptions::default())
            .unwrap()
            .0
            .is_empty()
    );
}

#[test]
fn native_series_first_preserves_fractional_essential_and_unregulated_errors() {
    let x = symbol!("series_first_error::x");
    let eps = symbol!("series_first_error::eps");
    let options = GenerationOptions::default();
    assert!(matches!(
        expand(
            &[term(
                parse!("series_first_error::eps^(1/2)"),
                Atom::one(),
                vec![Atom::Zero]
            )],
            &[x],
            eps,
            &GenerationOptions {
                max_order: 1,
                ..options.clone()
            }
        ),
        Err(GenerationError::FractionalLaurent(_))
    ));
    assert!(matches!(
        expand(
            &[term(
                Atom::one(),
                parse!("exp(1/series_first_error::eps)"),
                vec![Atom::Zero]
            )],
            &[x],
            eps,
            &options
        ),
        Err(GenerationError::Series(_))
    ));
    assert!(matches!(
        expand(
            &[term(Atom::one(), Atom::one(), vec![Atom::num(-1)])],
            &[x],
            eps,
            &options
        ),
        Err(GenerationError::UnregulatedEndpoint { .. })
    ));
    // Vanishing exact boundary coefficients still allow an unregulated power.
    let (regular, _) = compare(
        vec![term(
            Atom::one(),
            Atom::var(x).pow(Atom::num(2)),
            vec![Atom::num(-2)],
        )],
        &[x],
        eps,
        options,
    );
    assert_eq!(regular, BTreeMap::from([(0, Atom::one())]));
}

#[test]
fn native_series_first_keeps_large_regular_power_compact() {
    let x = symbol!("series_first_compact::x");
    let eps = symbol!("series_first_compact::eps");
    let x_atom = Atom::var(x);
    let regular = (Atom::one() + &x_atom).pow(Atom::num(10_000));
    let (coefficients, _) = expand(
        &[term(
            Atom::one(),
            regular.clone(),
            vec![Atom::num(-2) + Atom::var(eps)],
        )],
        &[x],
        eps,
        &GenerationOptions::default(),
    )
    .unwrap();
    assert_eq!(coefficients.keys().copied().collect::<Vec<_>>(), [-1, 0]);
    assert_eq!(coefficients[&-1], Atom::num(10_000));
    let expected = Atom::num(-1)
        + (&regular - Atom::one() - Atom::num(10_000) * &x_atom) / x_atom.pow(Atom::num(2));
    assert!(coefficients[&0].as_view().get_byte_size() < 2048);
    // Compare the independently known finite coefficient without requiring
    // identical grouping of -(1+10000*x). Keep the shared degree-10000 body
    // opaque during this small rational/distributive identity check.
    let difference = (&coefficients[&0] - expected)
        .replace(Pattern::Literal(regular))
        .with(Atom::var(symbol!("series_first_compact::opaque")));
    assert!(difference.together().expand().is_zero());
}

#[test]
fn native_series_first_pruned_polynomial_before_unregulated_axis_uses_exact_fallback() {
    let x = symbol!("series_first_pruned::x");
    let y = symbol!("series_first_pruned::y");
    let eps = symbol!("series_first_pruned::eps");
    let regular = (Atom::one() + Atom::var(x)).pow(Atom::num(2)) * Atom::var(y).pow(Atom::num(2));
    let (coefficients, attempts) = compare(
        vec![term(
            Atom::one(),
            regular,
            vec![Atom::num(-3) + Atom::var(eps), Atom::num(-2)],
        )],
        &[x, y],
        eps,
        GenerationOptions::default(),
    );
    assert!(
        attempts
            .iter()
            .all(|attempt| attempt.route == "exact_unregulated_fallback")
    );
    assert_eq!(coefficients[&-1], Atom::one());
    assert_eq!(coefficients[&0], Atom::num((-5, 2)));
}
