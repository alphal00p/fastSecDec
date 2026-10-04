//! Independent integral identity with a shared polynomial in different roles.

use fastsecdec::{
    generation::{GenerationOptions, generate},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{atom::Atom, parse, symbol};

#[test]
fn repeated_source_polynomial_keeps_each_occurrence_exponent_and_role() {
    let polynomial = parse!("support_reuse::x+support_reuse::y");
    let terms = [(1, 2), (2, 1)]
        .into_iter()
        .map(|(weight, numerator_power)| {
            ParametricTerm::new(
                Atom::num(weight),
                vec![Atom::Zero; 2],
                vec![
                    PolynomialFactor::new(
                        polynomial.clone(),
                        parse!("support_reuse::eps") - Atom::num(numerator_power),
                        FactorRole::Singularity,
                    ),
                    PolynomialFactor::new(
                        polynomial.clone(),
                        Atom::num(numerator_power),
                        FactorRole::Polynomial,
                    ),
                ],
            )
        })
        .collect();
    let input = ParametricIntegrand::new(
        vec![symbol!("support_reuse::x"), symbol!("support_reuse::y")],
        symbol!("support_reuse::eps"),
        ParametricDomain::UnitCube,
        terms,
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(generated.orders(), [0, 1]);
    let mut kernels = generated.compile().unwrap();
    let mut total = kernels.exact_coefficients().to_vec();
    // The unreduced density is exactly 3*(x+y)^eps. Independently integrating
    // its first two coefficients gives [3, 3*(2 log(2)-3/2)]. The midpoint
    // quadrature is confined to this small two-dimensional identity.
    let points = 256;
    for sector in kernels.sectors_mut() {
        assert_eq!(sector.dimension(), 2);
        for i in 0..points {
            for j in 0..points {
                let point = [
                    (f64::from(i) + 0.5) / f64::from(points),
                    (f64::from(j) + 0.5) / f64::from(points),
                ];
                let mut values = [0.0; 2];
                sector.evaluate(&point, &mut values).unwrap();
                for (target, value) in total.iter_mut().zip(values) {
                    *target += value / f64::from(points * points);
                }
            }
        }
    }
    assert!((total[0] - 3.0).abs() < 1e-10, "{total:?}");
    assert!(
        (total[1] - 3.0 * (2.0 * 2.0f64.ln() - 1.5)).abs() < 5e-5,
        "{total:?}"
    );
}
