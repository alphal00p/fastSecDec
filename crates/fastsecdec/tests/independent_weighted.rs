//! Independent range checks of separately accumulated complex components.
use std::ops::ControlFlow;

use fastsecdec::{
    generation::{GenerationOptions, generate},
    kernel::ReplayPolicy,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{atom::Atom, parse, symbol};

#[test]
fn amplification_recovers_a_single_underflowed_complex_component_after_large_maxima() {
    for (prefactor, swap) in [(parse!("1+𝑖*10^-200"), false), (parse!("10^-200+𝑖"), true)] {
        let input = ParametricIntegrand::new(
            vec![symbol!("independent_weighted::x")],
            symbol!("independent_weighted::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                prefactor,
                vec![Atom::Zero],
                vec![PolynomialFactor::new(
                    parse!("independent_weighted::x^2+independent_weighted::eps*independent_weighted::x^3"),
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            )],
        )
        .unwrap();
        let mut kernels = generate(
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
        // The optional validated policy retains the component-underflow test.
        kernels
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        let mut context = kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap();
        let mut output = [0.0; 4];
        context
            .evaluate_weighted(&[0.5], 1e300, &mut output)
            .unwrap();
        let report = context
            .evaluate_weighted(&[1e-100], 1e300, &mut output)
            .unwrap();
        assert!(
            !report.weighted_check,
            "all components are below prior maxima"
        );
        assert!(report.precision.rescued);
        let mut expected = [1e100, 1e-100, 1.0, 1e-200];
        if swap {
            expected.swap(0, 1);
            expected.swap(2, 3);
        }
        for (actual, expected) in output.into_iter().zip(expected) {
            assert!(
                (actual / expected - 1.0).abs() < 1e-12,
                "{actual} != {expected}"
            );
        }
    }
}
