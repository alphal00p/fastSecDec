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
    for choice in [ContourJacobian::Symbolic, ContourJacobian::Dual] {
        let generated = Arc::new(
            generate(
                &input(),
                &GenerationOptions {
                    mode: GenerationMode::NumericalDual,
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
                CompilationSession::new(generated.clone(), vec![], Default::default(), requested)
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

#[test]
fn unsupported_symbolic_generation_is_rejected_before_observer_work() {
    let mut events = 0;
    let error = generate(
        &input(),
        &GenerationOptions {
            contour_jacobian: ContourJacobian::Dual,
            program_recipe: ProgramRecipe::FixedV1,
            ..Default::default()
        },
        |_| {
            events += 1;
            ControlFlow::Continue(())
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("numerical_dual"));
    assert_eq!(events, 0);
}

#[test]
fn empty_and_exact_only_owners_retain_dual_policy_without_a_chart() {
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
                mode: GenerationMode::NumericalDual,
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
        let restored = KernelSet::from_bytes(&bytes).unwrap();
        assert_eq!(
            restored.compilation_settings().contour_jacobian,
            ContourJacobian::Dual
        );
    }
}
