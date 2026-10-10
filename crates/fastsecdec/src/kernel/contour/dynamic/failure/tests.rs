use crate::{
    contour::{
        ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet, ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{
    atom::{Atom, AtomCore},
    symbol,
};

fn fixture(canceled: bool) -> KernelSet {
    let x = symbol!("dynamic_failure_context::x");
    let eps = symbol!("dynamic_failure_context::eps");
    let f = (Atom::var(x) - Atom::num((1, 2))).pow(2);
    let mut factors = vec![
        PolynomialFactor::new(f.clone(), Atom::num(-1), FactorRole::Singularity)
            .with_semantics(FactorSemantics::Causal),
    ];
    if canceled {
        factors.push(PolynomialFactor::new(
            f,
            Atom::one(),
            FactorRole::Polynomial,
        ));
    }
    let source = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::one(), vec![Atom::Zero], factors)],
    )
    .unwrap();
    let generated = generate(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let original = generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap();
    let mut kernels = KernelSet::from_bytes(&original.to_bytes().unwrap()).unwrap();
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: ContourMode::Dynamical {
                    safety_fraction: 0.8,
                    lambda_cap: 1.,
                    displacement_cap: 1.,
                    construction: DynamicConstruction::Polynomial,
                },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Off,
                    pilot_points: 2,
                },
            },
        )
        .unwrap();
    kernels
}

#[test]
fn terminal_stationary_zero_has_context_but_a_canceled_density_stays_finite() {
    let mut singular = fixture(false);
    let mut output = vec![0.; singular.orders().len()];
    let error = singular.sectors_mut()[0]
        .evaluate(&[0.5], &mut output)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unresolved deformation"), "{error}");
    assert!(
        error.contains("certifies F=0 and weighted-gradient magnitude=0"),
        "{error}"
    );
    assert!(
        error.contains("does not identify the unique failing density branch"),
        "{error}"
    );
    singular.sectors_mut()[0]
        .evaluate(&[0.2], &mut output)
        .unwrap();
    assert!(output.iter().all(|value| value.is_finite()));
    assert_eq!(
        singular
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments,
        0
    );

    let mut canceled = fixture(true);
    let mut output = vec![0.; canceled.orders().len()];
    canceled.sectors_mut()[0]
        .evaluate(&[0.5], &mut output)
        .unwrap();
    assert!(output.iter().all(|value| value.is_finite()));
    assert!(output.iter().any(|value| *value != 0.));
    assert_eq!(
        canceled
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments,
        0
    );
}

#[test]
fn root_free_exact_failure_keeps_the_native_failure_without_stationary_claim() {
    let p = symbol!("dynamic_exact_failure_context::p");
    let eps = symbol!("dynamic_exact_failure_context::eps");
    let source = ParametricIntegrand::new(
        vec![],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![],
            vec![
                PolynomialFactor::new(Atom::var(p), Atom::num(-1), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut kernels = generated
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[p],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let settings = ContourSettings {
        deformation: ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 1.,
            displacement_cap: 1.,
            construction: DynamicConstruction::Polynomial,
        },
        validation: ContourValidationOptions {
            policy: ContourValidation::Off,
            pilot_points: 2,
        },
    };
    kernels
        .bind_parameters_with_contour(&BTreeMap::from([(p, 1.)]), &settings)
        .unwrap();
    let previous = kernels.exact_coefficients().to_vec();
    let error = kernels
        .bind_parameters_with_contour(&BTreeMap::from([(p, 0.)]), &settings)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unresolved deformation in exact contribution"),
        "{error}"
    );
    assert!(error.contains("non-finite"), "{error}");
    assert!(!error.contains("certifies F=0"), "{error}");
    assert_eq!(kernels.exact_coefficients(), previous);
}
