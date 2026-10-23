//! A valid underscore-suffixed regulator must remain a literal native symbol.
use std::{collections::BTreeMap, ops::ControlFlow};

use fastsecdec::{
    Atom,
    generation::{
        CoefficientExpansionMethod, CoefficientExpansionOptions, GenerationOptions, generate,
    },
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
    for method in [
        CoefficientExpansionMethod::Physical,
        CoefficientExpansionMethod::NativeNamed,
    ] {
        let kernels = generate(
            &input,
            &GenerationOptions {
                max_order: 1,
                coefficient_expansion: CoefficientExpansionOptions {
                    method,
                    ..Default::default()
                },
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
        .compile()
        .unwrap();
        assert!(kernels.sectors().is_empty());
        assert_eq!(kernels.orders().len(), kernels.components().len());
        assert_eq!(kernels.orders().len(), kernels.exact_coefficients().len());
        // Native reality inference can conservatively retain complex output.
        // Check every component without assuming a real-only storage layout.
        let mut real = BTreeMap::new();
        let mut imaginary = BTreeMap::new();
        for ((order, component), value) in kernels
            .orders()
            .iter()
            .zip(kernels.components())
            .zip(kernels.exact_coefficients())
        {
            let values = match component {
                CoefficientComponent::Real => &mut real,
                CoefficientComponent::Imag => &mut imaginary,
            };
            assert!(values.insert(*order, *value).is_none());
        }
        assert_eq!(real.keys().copied().collect::<Vec<_>>(), [-2, -1, 0, 1]);
        if !imaginary.is_empty() {
            assert_eq!(
                imaginary.keys().copied().collect::<Vec<_>>(),
                [-2, -1, 0, 1]
            );
            assert!(imaginary.values().all(|value| *value == 0.0));
        }
        // Independent analytic constants, including a positive epsilon order.
        let expected = [
            1.0,
            -std::f64::consts::EULER_GAMMA,
            0.989_055_995_327_972_6,
            -0.907_479_076_080_886_3,
        ];
        for (actual, expected) in real.values().zip(expected) {
            assert!(actual.is_finite() && (actual - expected).abs() < 2e-14);
        }
    }
}
