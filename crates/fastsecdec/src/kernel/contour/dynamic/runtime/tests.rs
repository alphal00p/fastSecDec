use super::*;
use crate::{
    contour::{ContourMode, ContourValidation, ContourValidationOptions, DynamicConstruction},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::ResultScope,
};
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, Symbol},
    symbol,
};

fn fixture(backend: EvaluatorBackend) -> (KernelSet, Symbol, ContourSettings) {
    let x = symbol!("dynamic_runtime_lifecycle::x");
    let p = symbol!("dynamic_runtime_lifecycle::p");
    let eps = symbol!("dynamic_runtime_lifecycle::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() / Atom::var(p),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    Atom::num((1, 4)) - Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut original = generated
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[p],
            CompilationSettings {
                backend,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    // Add an independent exact contribution to the same valid native record.
    // Endpoint terms of this generated map stay in its stochastic vector, so
    // they alone would not exercise a failing exact-offset rebind.
    original.exact_expressions[0] += Atom::one() / Atom::var(p);
    original.initialize_artifact().unwrap();
    let restored = KernelSet::from_bytes(&original.to_bytes().unwrap()).unwrap();
    let settings = ContourSettings {
        deformation: ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 1.,
            displacement_cap: 1.,
            construction: DynamicConstruction::Polynomial,
        },
        validation: ContourValidationOptions {
            policy: ContourValidation::Always,
            pilot_points: 2,
        },
    };
    (restored, p, settings)
}
fn point(p: Symbol, value: f64) -> BTreeMap<Symbol, f64> {
    use crate::contour::dynamic::{
        displacement_cap_symbol, lambda_cap_symbol, safety_fraction_symbol,
    };
    BTreeMap::from([
        (p, value),
        (safety_fraction_symbol(), 0.8),
        (lambda_cap_symbol(), 1.),
        (displacement_cap_symbol(), 1.),
    ])
}
fn sample(kernels: &mut KernelSet) -> Vec<f64> {
    let mut output = vec![0.; kernels.orders.len()];
    kernels.sectors[0].evaluate(&[0.37], &mut output).unwrap();
    output
}

#[test]
fn restored_dynamic_binding_pilot_policy_factories_and_atomic_rebind() {
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        let (mut kernels, p, settings) = fixture(backend);
        let bytes = kernels.artifact_bytes().unwrap().to_vec();
        kernels.bind_dynamic(&point(p, 1.), &settings).unwrap();
        let identity = kernels.content_id().to_owned();
        assert!(
            kernels
                .validate_integration_readiness(&ResultScope::FullIntegral)
                .is_err()
        );
        let chart = kernels.contour_validation_charts().remove(0);
        for x in [0.2, 0.6] {
            kernels
                .validate_contour_point(chart.chart_index, &[x], true)
                .unwrap();
            assert_eq!(kernels.dynamic_pilot.sectors.len(), 1);
            assert_eq!(
                kernels
                    .contour_validation_report()
                    .unwrap()
                    .production_checked_arguments,
                0
            );
            assert!(
                kernels
                    .try_clone()
                    .unwrap()
                    .dynamic_pilot
                    .sectors
                    .is_empty()
            );
        }
        assert!(kernels.finish_contour_pilot().unwrap().pilot_complete);
        assert!(kernels.dynamic_pilot.sectors.is_empty());
        kernels
            .validate_integration_readiness(&ResultScope::FullIntegral)
            .unwrap();
        let expected = sample(&mut kernels);
        let first_count = kernels
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments;
        assert!(first_count > 0);

        // The saved exact pole in p fails after replacement numeric
        // owners have been prepared. The previously bound problem stays usable.
        assert!(kernels.bind_dynamic(&point(p, 0.), &settings).is_err());
        assert_eq!(kernels.content_id(), identity);
        kernels
            .validate_integration_readiness(&ResultScope::FullIntegral)
            .unwrap();
        assert_eq!(sample(&mut kernels), expected);
        let mut previous = kernels
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments;
        for policy in [
            ContourValidation::Pilot,
            ContourValidation::Off,
            ContourValidation::Always,
        ] {
            kernels
                .set_contour_validation(ContourValidationOptions {
                    policy,
                    pilot_points: 2,
                })
                .unwrap();
            assert_eq!(kernels.content_id(), identity);
            assert!(kernels.contour_validation_report().unwrap().pilot_complete);
            let actual = sample(&mut kernels);
            for (a, b) in actual.iter().zip(&expected) {
                assert!((a - b).abs() < 1e-11 * (1. + b.abs()));
            }
            let now = kernels
                .contour_validation_report()
                .unwrap()
                .production_checked_arguments;
            if policy == ContourValidation::Always {
                assert!(now > previous);
            } else {
                assert_eq!(now, previous);
            }
            previous = now;
        }
        assert_eq!(kernels.artifact_bytes().unwrap(), bytes);
        let report = kernels.sectors[0].take_contour_validation_report().unwrap();
        assert_eq!(report.checked_arguments, previous);
        assert!(report.maximum_bits >= 96);
        assert_eq!(
            kernels
                .contour_validation_report()
                .unwrap()
                .production_checked_arguments,
            0
        );
        kernels.bind_dynamic(&point(p, 2.), &settings).unwrap();
        assert_ne!(kernels.content_id(), identity);
        assert!(
            kernels
                .validate_integration_readiness(&ResultScope::FullIntegral)
                .is_err()
        );
        assert!(!kernels.contour_validation_report().unwrap().pilot_complete);
    }
}
