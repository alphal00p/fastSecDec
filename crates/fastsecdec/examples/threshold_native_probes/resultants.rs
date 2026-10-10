use super::ProbeResult;
use serde_json::{Value, json};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::{algebraic::AlgebraicExtension, rational::Q},
    poly::{PolyVariable, polynomial::MultivariatePolynomial},
    symbol,
};

pub(super) fn run() -> ProbeResult<Value> {
    let (a, z) = symbol!("threshold_probe_norm::a", "threshold_probe_norm::z");
    let variables = Arc::new(vec![PolyVariable::from(a), PolyVariable::from(z)]);
    let poly = |atom: Atom| -> MultivariatePolynomial<_, u16> {
        atom.to_polynomial(&Q, Some(variables.clone()))
    };
    let a = Atom::var(a);
    let z = Atom::var(z);
    let minimal = poly(a.pow(2) - 2);
    let g = poly(z.pow(2) - 2);
    let elimination = minimal.to_univariate(0).resultant(&g.to_univariate(0));
    assert_eq!(elimination, poly((z.pow(2) - 2).pow(2)));

    let field = AlgebraicExtension::new(minimal.clone());
    let shifted = g.to_number_field(&field).norm();
    // Public norm searches f(z-s*a) until the resultant is square-free.
    // Here s=0 and s=1 fail that criterion; s=2 yields this different divisor.
    assert_eq!(shifted, poly(z.pow(4) - 20 * z.pow(2) + 36));
    assert_ne!(shifted, elimination);
    let linear = poly(&z - &a);
    let linear_elimination = minimal.to_univariate(0).resultant(&linear.to_univariate(0));
    assert_eq!(linear_elimination, poly(z.pow(2) - 2));
    assert_eq!(linear.to_number_field(&field).norm(), linear_elimination);
    Ok(json!({
        "field": "Q[a]/(a^2-2)",
        "input_polynomial": "z^2-2",
        "unshifted_resultant": "(z^2-2)^2",
        "public_norm_result": "z^4-20*z^2+36",
        "shifted_norm_is_not_divisor_norm": true,
        "linear_control_agrees": true,
        "resolver_recommendation": "use native resultant with explicit elimination variable and leading-coefficient accounting"
    }))
}
