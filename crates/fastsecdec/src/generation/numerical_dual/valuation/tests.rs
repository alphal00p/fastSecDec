use super::*;
use symbolica::{parse, symbol};

#[test]
fn factored_high_powers_require_only_a_leading_native_series() {
    let cache = ValuationCache::new(&[
        symbol!("valuation_test::x"),
        symbol!("valuation_test::y"),
        symbol!("valuation_test::z"),
    ]);
    let polynomial = parse!(
        "(valuation_test::x+valuation_test::y)^1000*(valuation_test::x+valuation_test::z)^1000"
    );
    assert_eq!(
        cache
            .bound(&polynomial, &[1.into(), 2.into(), 3.into()])
            .unwrap(),
        2000
    );
}

#[test]
fn hidden_cancellation_can_only_underestimate_a_regular_valuation() {
    let cache = ValuationCache::new(&[symbol!("valuation_test::x"), symbol!("valuation_test::y")]);
    let polynomial = parse!(
        "(valuation_test::x+valuation_test::y)^2-valuation_test::x^2-2*valuation_test::x*valuation_test::y-valuation_test::y^2+valuation_test::x^3"
    );
    assert_eq!(polynomial.expand(), parse!("valuation_test::x^3"));
    let bound = cache.bound(&polynomial, &[1.into(), 1.into()]).unwrap();
    assert!((0..=3).contains(&bound));
    assert!(
        (polynomial / parse!("valuation_test::x").pow(Atom::num(bound)))
            .expand()
            .is_polynomial(true, false)
            .is_some()
    );
}

#[test]
fn runtime_zeroes_and_reserved_scale_symbols_do_not_establish_a_power() {
    let cache = ValuationCache::new(&[symbol!("valuation_test::x")]);
    assert_eq!(
        cache
            .bound(
                &parse!("valuation_test::a*valuation_test::x+valuation_test::x^2"),
                &[1.into()]
            )
            .unwrap(),
        1
    );
    assert_eq!(
        cache
            .bound(
                &parse!("fastsecdec::numerical_dual::valuation_scale*valuation_test::x^8"),
                &[1.into()]
            )
            .unwrap(),
        0
    );
    assert_eq!(
        cache
            .bound(&parse!("1/(1+valuation_test::x)"), &[1.into()])
            .unwrap(),
        0
    );
}
