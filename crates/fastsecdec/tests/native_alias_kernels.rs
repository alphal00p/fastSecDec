//! Independent public-pipeline controls for retained native coefficient images.
use fastsecdec::{
    generation::{
        CoefficientExpansionMethod, CoefficientExpansionOptions, GenerationOptions, generate,
    },
    kernel::{KernelSet, ReplayPolicy},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, AtomCore, AtomView},
    domains::{
        float::{Complex, Float, RealLike},
        rational::Rational,
    },
    symbol,
    transcendental::TranscendentalFunctions,
};

#[test]
fn an_unused_reserved_looking_regulator_is_not_a_coordinate_image() {
    let x = symbol!("alias_reserved::x");
    let input = ParametricIntegrand::new(
        vec![x],
        symbol!("fastsecdec::laurent_template::c0"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::one(), vec![Atom::num(2)], vec![])],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(generated.orders(), &[0]);
    assert_eq!(generated.sectors().len(), 1);
    let mut kernels = generated.compile().unwrap();
    let mut output = [0.0];
    kernels.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert_eq!(output, [0.0625]);
}

#[test]
fn hidden_complex_coordinate_images_preserve_gamma_vector_and_weighted_reload() {
    let x = symbol!("alias_public::x_");
    let eps = symbol!("alias_public::eps_");
    let variable = Atom::var(x);
    let regulator = Atom::var(eps);
    let polynomial = (1..=24).fold(
        Atom::num(Complex::new(Rational::from(2), Rational::from(3))),
        |sum, degree| sum + variable.pow(degree) / Atom::num(degree),
    );
    let prefactor = regulator.gamma();
    let density = &prefactor * variable.pow(&regulator) * &polynomial;
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            prefactor,
            vec![regulator],
            vec![PolynomialFactor::new(
                polynomial,
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    for (method, maximum) in [
        CoefficientExpansionMethod::Physical,
        CoefficientExpansionMethod::NativeNamed,
    ]
    .into_iter()
    .flat_map(|method| [-1, 2].map(|maximum| (method, maximum)))
    {
        let generated = generate(
            &input,
            &GenerationOptions {
                max_order: maximum,
                coefficient_expansion: CoefficientExpansionOptions {
                    method,
                    ..Default::default()
                },
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(generated.orders(), &(-1..=maximum).collect::<Vec<_>>());
        assert_eq!(
            generated.sectors().len(),
            1,
            "hidden coordinate dependence must remain numerical"
        );
        let sector = &generated.sectors()[0];
        assert!(
            sector.aliased_coefficients().iter().any(|coefficient| {
                coefficient.get_aliases().iter().any(|(handle, body)| {
                    let mut complex = false;
                    body.visitor(&mut |term| {
                        if let AtomView::Num(number) = term {
                            complex |= !number.get_coeff_view().is_real();
                        }
                        !complex
                    });
                    coefficient.get_root().contains(handle.as_view()) && complex
                })
            }),
            "this control must exercise a reachable complex image"
        );

        // The reference uses native absolute series on the original admitted
        // density, without the production template or alias builder.
        let direct = density.series(eps, 0, i64::from(maximum)).unwrap();
        let expected_coefficients = direct
            .terms()
            .map(|(_, coefficient)| coefficient.clone())
            .collect::<Vec<_>>();
        assert_eq!(expected_coefficients.len(), generated.orders().len());
        let exact =
            Atom::evaluator_multiple(&expected_coefficients, std::slice::from_ref(&variable))
                .build()
                .unwrap();
        let mut reference = exact.map_coeff_with_prec(
            &|value| {
                Complex::new(
                    value.re.to_multi_prec_float(512),
                    value.im.to_multi_prec_float(512),
                )
            },
            512,
        );

        let mut kernels = generated.compile().unwrap();
        kernels
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        let bytes = kernels.to_bytes().unwrap();
        let mut restored = KernelSet::from_bytes(&bytes).unwrap();
        restored
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        assert_eq!(restored.content_id(), kernels.content_id());
        assert_eq!(restored.orders().len(), 2 * expected_coefficients.len());
        for kernels in [&kernels, &restored] {
            for minimum_bits in [128, 256] {
                let mut context = kernels
                    .evaluation_context(
                        0,
                        ReplayPolicy {
                            minimum_bits,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                let mut worker = kernels.sectors()[0].try_clone().unwrap();
                for point in [0.2, 0.6, 1e-80] {
                    let mut expected =
                        vec![
                            Complex::new(Float::with_val(512, 0), Float::with_val(512, 0));
                            expected_coefficients.len()
                        ];
                    reference.evaluate(
                        &[Complex::new(
                            Float::with_val(512, point),
                            Float::with_val(512, 0),
                        )],
                        &mut expected,
                    );
                    let mut cloned_values = vec![0.0; kernels.orders().len()];
                    worker.evaluate(&[point], &mut cloned_values).unwrap();
                    for (pair, expected) in cloned_values.as_chunks::<2>().0.iter().zip(&expected) {
                        for (actual, expected) in pair.iter().zip([&expected.re, &expected.im]) {
                            let expected = expected.to_f64();
                            assert!((actual - expected).abs() <= 2e-11 * expected.abs().max(1.0));
                        }
                    }
                    let weight = 1e40;
                    let mut values = vec![0.0; kernels.orders().len()];
                    let report = context
                        .evaluate_weighted(&[point], weight, &mut values)
                        .unwrap();
                    assert!(values.iter().all(|value| value.is_finite()));
                    if point == 0.2 {
                        assert!(report.replayed);
                        assert!(report.precision.bits >= minimum_bits * 2);
                    }
                    for (pair, expected) in values.as_chunks::<2>().0.iter().zip(expected) {
                        for (actual, expected) in pair.iter().zip([expected.re, expected.im]) {
                            let expected = (expected * Float::with_val(512, weight)).to_f64();
                            assert!(
                                (actual - expected).abs() <= 2e-11 * expected.abs().max(1.0),
                                "order maximum {maximum}, point {point}: {actual} vs {expected}"
                            );
                        }
                    }
                }
            }
        }
    }
}
