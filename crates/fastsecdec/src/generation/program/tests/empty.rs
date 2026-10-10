//! Explicit empty recipes and genuinely zero-dimensional exact inputs.
use super::*;
use crate::{
    contour::{ContourMode, ContourSettings, DynamicConstruction},
    generation::{GeneratedIntegral, GenerationSession, streaming},
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet},
    results::ResultScope,
};
use std::collections::BTreeMap;

fn empty() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("empty_dynamic_generation::x")],
        symbol!("empty_dynamic_generation::eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap()
}

fn settings(recipe: ProgramRecipe) -> ContourSettings {
    ContourSettings {
        deformation: ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 1.,
            displacement_cap: 1.,
            construction: match recipe {
                ProgramRecipe::DynamicPolynomialV1 => DynamicConstruction::Polynomial,
                ProgramRecipe::DynamicSignAwareV1 => DynamicConstruction::SignAware,
                _ => unreachable!(),
            },
        },
        ..Default::default()
    }
}

fn check_exact(generated: GeneratedIntegral, recipe: ProgramRecipe, expected: f64) {
    assert!(generated.sectors().is_empty());
    assert_eq!(generated.program_descriptor().unwrap().recipe(), recipe);
    let compiled = generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(compiled.program_recipe(), recipe);
    let bytes = compiled.to_bytes().unwrap();
    let mut restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.program_recipe(), recipe);
    assert_eq!(restored.to_bytes().unwrap(), bytes);
    restored
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(recipe))
        .unwrap();
    assert!(restored.sectors().is_empty());
    assert_eq!(restored.orders(), &[0]);
    assert_eq!(restored.exact_coefficients(), &[expected]);
    assert!(restored.contour_validation_charts().is_empty());
    let report = restored.finish_contour_pilot().unwrap();
    assert!(report.pilot_complete);
    assert_eq!(report.checked_arguments, 0);
    restored
        .validate_integration_readiness(&ResultScope::FullIntegral)
        .unwrap();
}

#[test]
fn empty_dynamic_recipes_survive_direct_cooperative_and_streamed_generation() {
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
            let input = empty();
            let options = GenerationOptions {
                mode,
                program_recipe: recipe,
                ..Default::default()
            };
            let direct = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
            assert!(direct.metadata().charts().is_empty());
            assert!(direct.dynamic_check_sources().is_empty());
            check_exact(direct, recipe, 0.);

            let mut session = GenerationSession::new(input.clone(), options.clone());
            while !session.is_complete() {
                session.step(1, |_| ControlFlow::Continue(())).unwrap();
            }
            check_exact(session.take_result().unwrap(), recipe, 0.);

            let directory = tempfile::tempdir().unwrap();
            let prepared = streaming::prepare(&input, &options, directory.path(), |_| {
                ControlFlow::Continue(())
            })
            .unwrap();
            assert!(prepared.charts.is_empty());
            let plan = streaming::finish_preparation(&prepared, vec![], vec![], vec![]).unwrap();
            assert_eq!(plan.sectors.len(), 1);
            let unit = streaming::generate_sector(directory.path(), &plan.sectors[0], |_| {
                ControlFlow::Continue(())
            })
            .unwrap();
            assert!(unit.source_indices.is_empty());
            check_exact(unit.generated, recipe, 0.);
        }
    }
}

#[test]
fn zero_dimensional_dynamic_density_retains_its_nonzero_exact_value() {
    let input = ParametricIntegrand::new(
        vec![],
        symbol!("zero_dimensional_dynamic_generation::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![],
            vec![
                PolynomialFactor::new(Atom::num(2), Atom::num(-1), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode,
                    program_recipe: recipe,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.metadata().charts().len(), 1);
            check_exact(generated, recipe, 1.5);
        }
    }
}

#[test]
fn explicit_zero_prefactor_does_not_admit_an_identically_singular_denominator() {
    let input = |prefactor, causal| {
        ParametricIntegrand::new(
            vec![],
            symbol!("zero_dynamic_admission::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                prefactor,
                vec![],
                vec![
                    PolynomialFactor::new(causal, Atom::num(-1), FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                ],
            )],
        )
    };
    assert!(input(Atom::one(), Atom::Zero).is_err());
    assert!(input(Atom::Zero, Atom::Zero).is_err());
    let zero = input(Atom::Zero, Atom::num(2)).unwrap();
    assert!(zero.terms().is_empty());
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        let generated = generate(
            &zero,
            &GenerationOptions {
                program_recipe: recipe,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        check_exact(generated, recipe, 0.);
    }
}

#[test]
fn symbolic_endpoint_cancelled_exact_offsets_keep_both_jacobian_choices() {
    let input = ParametricIntegrand::new(
        vec![],
        symbol!("cancelled_exact_contour::eps"),
        ParametricDomain::UnitCube,
        [(3, -1), (-6, -2)]
            .into_iter()
            .map(|(prefactor, power)| {
                ParametricTerm::new(
                    Atom::num(prefactor),
                    vec![],
                    vec![
                        PolynomialFactor::new(
                            Atom::num(2),
                            Atom::num(power),
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Causal),
                    ],
                )
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(input.terms().len(), 2);
    for recipe in [
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        for contour_jacobian in [
            crate::contour::ContourJacobian::Symbolic,
            crate::contour::ContourJacobian::Dual,
        ] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    mode: GenerationMode::Symbolic,
                    program_recipe: recipe,
                    contour_jacobian,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert!(generated.sectors().is_empty());
            check_exact(generated, recipe, 0.);
        }
    }
}
