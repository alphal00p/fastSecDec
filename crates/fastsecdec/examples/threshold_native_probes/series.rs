use super::ProbeResult;
use serde_json::{Value, json};
use symbolica::{
    atom::{Atom, AtomCore},
    parse,
    poly::series::SeriesDepth,
    symbol,
};

pub(super) fn run() -> ProbeResult<Value> {
    let x = symbol!("threshold_probe_series::x");
    let expression = parse!("threshold_probe_series::x^(1/3)*(1+threshold_probe_series::x)^(1/2)");
    let series = expression.series(x, 0, SeriesDepth::absolute((7, 3)))?;
    assert_eq!(series.get_trailing_exponent(), (1, 3));
    assert!(series.get_ramification().is_multiple_of(3));
    assert_eq!(series.coefficient((1, 3).into()), Some(Atom::num(1)));
    assert_eq!(series.coefficient((4, 3).into()), Some(Atom::num((1, 2))));
    assert_eq!(series.coefficient((7, 3).into()), Some(Atom::num((-1, 8))));
    assert_eq!(series.coefficient((1, 2).into()), Some(Atom::Zero));
    assert_eq!(series.coefficient(series.absolute_order()), None);

    // Native multiplication must recover the polynomial relation to the
    // certified truncation order; no guessed trailing coefficients are used.
    let square = (&series * &series).map_coeff(|coefficient| coefficient.expand());
    assert_eq!(square.coefficient((2, 3).into()), Some(Atom::num(1)));
    assert_eq!(square.coefficient((5, 3).into()), Some(Atom::num(1)));
    assert_eq!(square.coefficient((8, 3).into()), Some(Atom::Zero));

    let root = parse!("sqrt(threshold_probe_series::x+threshold_probe_series::x^2)").series(
        x,
        0,
        SeriesDepth::absolute((5, 2)),
    )?;
    assert_eq!(root.coefficient((1, 2).into()), Some(Atom::num(1)));
    assert_eq!(root.coefficient((3, 2).into()), Some(Atom::num((1, 2))));
    assert_eq!(root.coefficient((5, 2).into()), Some(Atom::num((-1, 8))));

    // This explicit positive branch admits x=u^2. Native differentiation of
    // its regular unit then has finite endpoint coefficients.
    let u = symbol!("threshold_probe_series::u");
    let regular = parse!("sqrt(1+threshold_probe_series::u^2)");
    let jets = regular
        .derivative(u)
        .derivative(u)
        .series(u, 0, SeriesDepth::absolute(0))?;
    assert_eq!(jets.coefficient(0.into()), Some(Atom::num(1)));

    Ok(json!({
        "third_root_ramification": series.get_ramification(),
        "absolute_remainder_order": series.absolute_order().to_string(),
        "known_fractional_coefficients": 3,
        "native_series_product_relation": true,
        "half_root_coefficients": 3,
        "ramified_unit_second_derivative_at_zero": "1",
        "branch_scope": "explicit positive real branch x>0",
        "implicit_newton_puiseux_solver_exercised": false
    }))
}
