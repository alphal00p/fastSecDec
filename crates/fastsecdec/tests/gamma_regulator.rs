//! A valid underscore-suffixed regulator must remain a literal native symbol.
use std::ops::ControlFlow;

use fastsecdec::{
    Atom,
    generation::{GenerationOptions, generate},
    parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
    status::CoefficientComponent,
};
use symbolica::{symbol, transcendental::TranscendentalFunctions};

#[test]
fn gamma_and_endpoint_poles_keep_the_complete_underscored_regulator_vector() {
    let regulator = symbol!("gamma_literal::eps_");
    let epsilon = Atom::var(regulator);
    // Analytically Γ(eps) ∫_0^1 x^(eps-1) dx = Γ(eps)/eps.
    let input = ParametricIntegrand::new(
        vec![symbol!("gamma_literal::x_")],
        regulator,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            epsilon.clone().gamma(),
            vec![epsilon - 1],
            vec![],
        )],
    )
    .unwrap();
    let kernels = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap();
    assert_eq!(kernels.orders(), [-2, -1, 0, 1]);
    assert!(
        kernels
            .components()
            .iter()
            .all(|part| *part == CoefficientComponent::Real)
    );
    assert!(kernels.sectors().is_empty());
    // Independent analytic constants, including a positive epsilon order.
    let expected = [
        1.0,
        -0.577_215_664_901_532_9,
        0.989_055_995_327_972_6,
        -0.907_479_076_080_886_3,
    ];
    assert_eq!(kernels.exact_coefficients().len(), expected.len());
    for (actual, expected) in kernels.exact_coefficients().iter().zip(expected) {
        assert!(actual.is_finite() && (actual - expected).abs() < 2e-14);
    }
}
