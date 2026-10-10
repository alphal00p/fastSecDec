//! Public generation/compilation ownership of the saved determinant policy.
use fastsecdec::{
    Atom,
    contour::{
        ContourJacobian, ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
    },
    generation::{GenerationMode, GenerationOptions, generate},
    kernel::{CompilationSession, CompilationSettings, EvaluatorBackend, KernelSet, ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{domains::float::Complex, symbol};

fn input() -> ParametricIntegrand {
    let x = symbol!("jacobian_policy::x");
    ParametricIntegrand::new(
        vec![x],
        symbol!("jacobian_policy::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::one() + Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

fn settings() -> CompilationSettings {
    CompilationSettings {
        backend: EvaluatorBackend::Eager,
        horner_iterations: 0,
        ..Default::default()
    }
}

fn check_restored_density(kernels: &mut KernelSet) {
    let lambda = 0.125;
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: ContourMode::Fixed { lambda },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 1,
                },
            },
        )
        .unwrap();
    for chart in kernels.contour_validation_charts() {
        kernels
            .validate_contour_point(chart.chart_index, &[0.5], true)
            .unwrap();
    }
    kernels.finish_contour_pilot().unwrap();
    kernels
        .validate_integration_readiness(&fastsecdec::results::ResultScope::FullIntegral)
        .unwrap();
    assert_eq!(kernels.sectors().len(), 1);
    assert!(
        kernels
            .exact_coefficients()
            .iter()
            .all(|value| *value == 0.)
    );
    for x in [0.125, 0.5, 0.875] {
        // F=1+x gives z=x-i*lambda*x*(1-x) and dz/dx=1-i*lambda*(1-2*x).
        // This native complex closed form is independent of both saved programs.
        let expected = Complex::new(1., -lambda * (1. - 2. * x))
            / Complex::new(1. + x, -lambda * x * (1. - x));
        let mut value = vec![0.; kernels.orders().len()];
        kernels.sectors_mut()[0].evaluate(&[x], &mut value).unwrap();
        assert_eq!(value.len(), 2);
        for (actual, expected) in value.iter().zip([expected.re, expected.im]) {
            assert!(actual.is_finite() && (actual - expected).abs() < 2e-12);
        }
    }
}

#[test]
fn compilation_retains_generation_choice_across_direct_session_and_saved_owners() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for choice in [ContourJacobian::Symbolic, ContourJacobian::Dual] {
            let generated = Arc::new(
                generate(
                    &input(),
                    &GenerationOptions {
                        mode,
                        contour_jacobian: choice,
                        program_recipe: ProgramRecipe::FixedV1,
                        ..Default::default()
                    },
                    |_| ControlFlow::Continue(()),
                )
                .unwrap(),
            );
            assert_eq!(generated.contour_jacobian(), choice);
            let direct = generated.compile_with_settings(settings()).unwrap();
            assert_eq!(direct.compilation_settings().contour_jacobian, choice);
            let mut session =
                CompilationSession::new(generated.clone(), vec![], Default::default(), settings())
                    .unwrap();
            while !session.is_complete() {
                session.step(1, |_| ControlFlow::Continue(())).unwrap();
            }
            let stepped = session.take_result().unwrap();
            assert_eq!(stepped.content_id(), direct.content_id());
            assert_eq!(
                stepped.compilation_settings(),
                direct.compilation_settings()
            );
            for bytes in [
                direct.to_bytes().unwrap(),
                generated
                    .to_kernel_bytes_with_settings(Default::default(), settings())
                    .unwrap(),
            ] {
                let mut restored = KernelSet::from_bytes(&bytes).unwrap();
                assert_eq!(restored.compilation_settings().contour_jacobian, choice);
                assert_eq!(restored.content_id(), direct.content_id());
                check_restored_density(&mut restored);
            }
            if choice == ContourJacobian::Symbolic {
                let requested = CompilationSettings {
                    contour_jacobian: ContourJacobian::Dual,
                    ..settings()
                };
                assert!(generated.compile_with_settings(requested).is_err());
                assert!(
                    CompilationSession::new(
                        generated.clone(),
                        vec![],
                        Default::default(),
                        requested
                    )
                    .is_err()
                );
                assert!(
                    generated
                        .to_kernel_bytes_with_settings(Default::default(), requested)
                        .is_err()
                );
            }
        }
    }
}

#[test]
fn empty_and_exact_only_owners_retain_policy_and_value_without_a_chart() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for terms in [
            vec![],
            vec![ParametricTerm::new(
                Atom::num((3, 2)),
                vec![],
                vec![
                    PolynomialFactor::new(Atom::one(), Atom::num(-1), FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                ],
            )],
        ] {
            let expected_exact = if terms.is_empty() { 0. } else { 1.5 };
            let input = ParametricIntegrand::new(
                vec![],
                symbol!("jacobian_policy::exact_eps"),
                ParametricDomain::UnitCube,
                terms,
            )
            .unwrap();
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode,
                    contour_jacobian: ContourJacobian::Dual,
                    program_recipe: ProgramRecipe::FixedV1,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert!(generated.sectors().is_empty());
            assert_eq!(generated.contour_jacobian(), ContourJacobian::Dual);
            let bytes = generated
                .to_kernel_bytes_with_settings(Default::default(), settings())
                .unwrap();
            let mut restored = KernelSet::from_bytes(&bytes).unwrap();
            assert_eq!(
                restored.compilation_settings().contour_jacobian,
                ContourJacobian::Dual
            );
            restored
                .bind_parameters_with_contour(
                    &BTreeMap::new(),
                    &ContourSettings {
                        deformation: ContourMode::Fixed { lambda: 0.125 },
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(
                restored.exact_coefficients().first().copied().unwrap_or(0.),
                expected_exact
            );
            assert!(
                restored
                    .exact_coefficients()
                    .iter()
                    .skip(1)
                    .all(|v| *v == 0.)
            );
        }
    }
}

#[test]
fn symbolic_endpoints_keep_closed_form_subtracted_laurent_vectors() {
    use fastsecdec::{
        generation::SubtractionStrategy,
        status::CoefficientComponent::{Imag, Real},
    };
    use symbolica::domains::float::Real as _;
    let x = symbol!("jacobian_policy_endpoint::x");
    let eps = symbol!("jacobian_policy_endpoint::eps");
    for (power, subtraction) in [
        (1, SubtractionStrategy::Taylor),
        (2, SubtractionStrategy::IntegrateByParts),
    ] {
        let input = ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::var(eps) - Atom::num(power)],
                vec![
                    PolynomialFactor::new(
                        Atom::one() + Atom::var(x),
                        Atom::num(-1),
                        FactorRole::Singularity,
                    )
                    .with_semantics(FactorSemantics::Causal),
                ],
            )],
        )
        .unwrap();
        for choice in [ContourJacobian::Symbolic, ContourJacobian::Dual] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode: GenerationMode::Symbolic,
                    subtraction,
                    contour_jacobian: choice,
                    program_recipe: ProgramRecipe::FixedV1,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert!(
                generated
                    .sectors()
                    .iter()
                    .all(|s| s.generation_mode() == GenerationMode::Symbolic)
            );
            let compiled = generated.compile_with_settings(settings()).unwrap();
            let mut restored = KernelSet::from_bytes(&compiled.to_bytes().unwrap()).unwrap();
            assert_eq!(restored.compilation_settings().contour_jacobian, choice);
            // Compiler-only determinant placeholders cannot become public inputs.
            assert_eq!(
                restored.runtime_parameters(),
                [fastsecdec::contour::lambda_symbol()]
            );
            let lambda = 0.125;
            restored
                .bind_parameters_with_contour(
                    &BTreeMap::new(),
                    &ContourSettings {
                        deformation: ContourMode::Fixed { lambda },
                        validation: ContourValidationOptions {
                            policy: ContourValidation::Always,
                            pilot_points: 1,
                        },
                    },
                )
                .unwrap();
            for chart in restored.contour_validation_charts() {
                restored
                    .validate_contour_point(chart.chart_index, &[0.5], true)
                    .unwrap();
            }
            restored.finish_contour_pilot().unwrap();
            restored
                .validate_integration_readiness(&fastsecdec::results::ResultScope::FullIntegral)
                .unwrap();
            assert_eq!(restored.orders(), [-1, -1, 0, 0]);
            assert_eq!(restored.components(), [Real, Imag, Real, Imag]);
            let one = Complex::new(1., 0.);
            let a = Complex::new(1., -lambda);
            let b = Complex::new(0., lambda);
            let (pole, endpoint) = if power == 1 {
                (1., a.log())
            } else {
                (-1., -(one + b) / 2. - a.log() + b / (a * a) - one)
            };
            for (actual, expected) in restored.exact_coefficients().iter().zip([0., 0., 0., 0.]) {
                assert!(
                    (actual - expected).abs() < 2e-12,
                    "power={power}, jacobian={choice:?}, exact={:?}, actual={actual}, expected={expected}",
                    restored.exact_coefficients()
                );
            }
            assert_eq!(restored.sectors().len(), 1);
            // Native complex closed forms are independent of both compiled paths.
            // The singular source gives 1/eps−log2 for eps−1 and
            // −1/eps+log2−1 for eps−2 with IBP. Its analytic endpoint/boundary
            // pieces remain in the stochastic vector. Native assembly folds
            // only a whole coordinate-independent vector into the exact owner.
            for point in [0.125, 0.5, 0.875] {
                let ratio = a + b * point;
                let image = ratio * point;
                let derivative = a + b * (2. * point);
                let finite = if power == 1 {
                    (derivative / (ratio * (one + image)) - one) / point
                } else {
                    let smooth = derivative / (ratio * ratio * (one + image));
                    let smooth_derivative = smooth
                        * (b * 2. / derivative - b * 2. / ratio - derivative / (one + image));
                    (smooth_derivative + one) / point
                };
                let mut values = vec![0.; 4];
                restored.sectors_mut()[0]
                    .evaluate(&[point], &mut values)
                    .unwrap();
                let finite = endpoint + finite;
                for (actual, expected) in values.iter().zip([pole, 0., finite.re, finite.im]) {
                    assert!(
                        actual.is_finite() && (actual - expected).abs() < 2e-12,
                        "power={power}, jacobian={choice:?}, point={point}, values={values:?}, actual={actual}, expected={expected}"
                    );
                }
            }
        }
    }
}

#[test]
fn symbolic_ibp_dynamic_implicit_jacobians_restore_complete_vectors() {
    use fastsecdec::{
        contour::DynamicConstruction,
        generation::SubtractionStrategy,
        status::CoefficientComponent::{Imag, Real},
    };
    use symbolica::atom::AtomCore;
    let x = symbol!("public_symbolic_dynamic_jacobian::x");
    let eps = symbol!("public_symbolic_dynamic_jacobian::eps");
    let positive = Atom::one() + Atom::var(x).pow(2);
    let f = (Atom::num((1, 4)) - Atom::var(x)) * &positive;
    // The source's x^2 coefficient is 60(1+i), so x^(-3-eps)
    // has residue -60(1+i). This exact native source calculation is
    // independent of both contour determinant implementations.
    let second_coefficient = (Atom::one() / &f)
        .derivative(x)
        .derivative(x)
        .replace(Atom::var(x))
        .with(Atom::Zero)
        / Atom::num(2);
    assert_eq!(second_coefficient, Atom::num(60));
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() + Atom::i(),
            vec![Atom::num(-3) - Atom::var(eps)],
            vec![
                PolynomialFactor::new(f, Atom::num(-1), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(positive, Atom::Zero, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    for (recipe, construction) in [
        (
            ProgramRecipe::DynamicPolynomialV1,
            DynamicConstruction::Polynomial,
        ),
        (
            ProgramRecipe::DynamicSignAwareV1,
            DynamicConstruction::SignAware,
        ),
    ] {
        let mut reference: Option<Vec<Vec<f64>>> = None;
        for choice in [ContourJacobian::Symbolic, ContourJacobian::Dual] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode: GenerationMode::Symbolic,
                    subtraction: SubtractionStrategy::IntegrateByParts,
                    program_recipe: recipe,
                    contour_jacobian: choice,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.sectors().len(), 1);
            assert_eq!(
                generated.sectors()[0].generation_mode(),
                GenerationMode::Symbolic
            );
            let original = generated.compile_with_settings(settings()).unwrap();
            if choice == ContourJacobian::Dual {
                assert!(
                    original.sectors()[0]
                        .statistics()
                        .symbolic_endpoint_contour_partials
                        .unwrap()
                        > 0
                );
            }
            let content_id = original.content_id().to_owned();
            let bytes = original.to_bytes().unwrap();
            drop(original);
            let mut restored = KernelSet::from_bytes(&bytes).unwrap();
            assert_eq!(restored.content_id(), content_id);
            assert_eq!(restored.program_recipe(), recipe);
            assert_eq!(restored.compilation_settings().contour_jacobian, choice);
            assert_eq!(
                restored.sectors()[0]
                    .statistics()
                    .symbolic_endpoint_contour_partials,
                None
            );
            assert_eq!(
                restored.runtime_parameters(),
                recipe
                    .recipe_parameters()
                    .iter()
                    .map(|name| symbol!(*name))
                    .collect::<Vec<_>>()
            );
            restored
                .bind_parameters_with_contour(
                    &BTreeMap::new(),
                    &ContourSettings {
                        deformation: ContourMode::Dynamical {
                            safety_fraction: 0.8,
                            lambda_cap: 0.125,
                            displacement_cap: 1.,
                            construction,
                        },
                        validation: ContourValidationOptions {
                            policy: ContourValidation::Always,
                            pilot_points: 2,
                        },
                    },
                )
                .unwrap();
            for chart in restored.contour_validation_charts() {
                for point in [0.2, 0.5] {
                    restored
                        .validate_contour_point(chart.chart_index, &[point], true)
                        .unwrap();
                }
            }
            let report = restored.finish_contour_pilot().unwrap();
            assert!(report.pilot_complete && report.checked_arguments > 0);
            restored
                .validate_integration_readiness(&fastsecdec::results::ResultScope::FullIntegral)
                .unwrap();
            assert_eq!(restored.orders(), [-1, -1, 0, 0]);
            assert_eq!(restored.components(), [Real, Imag, Real, Imag]);
            let mut values = Vec::new();
            for point in [0.13, 0.37, 0.71] {
                let mut full = restored.exact_coefficients().to_vec();
                let mut residual = vec![0.; full.len()];
                restored.sectors_mut()[0]
                    .evaluate(&[point], &mut residual)
                    .unwrap();
                for (full, residual) in full.iter_mut().zip(residual) {
                    *full += residual;
                }
                assert!(full.iter().all(|v| v.is_finite()));
                for pole in &full[..2] {
                    assert!(
                        (pole + 60.).abs() < 2e-9,
                        "{recipe:?}/{choice:?}/{point}: {full:?}"
                    );
                }
                values.push(full);
            }
            if let Some(reference) = &reference {
                for (actual, expected) in values.iter().flatten().zip(reference.iter().flatten()) {
                    assert!(
                        (actual - expected).abs() < 2e-9 * (1. + expected.abs()),
                        "{recipe:?}: {actual} versus {expected}"
                    );
                }
            } else {
                reference = Some(values);
            }
        }
    }
}
