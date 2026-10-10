//! Complete source-witness admission through compilation and saved checks.
use super::*;
use crate::{
    contour::{
        ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet},
};
use std::collections::BTreeMap;

fn point(kernels: &mut KernelSet, x: f64) -> Vec<f64> {
    let mut result = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut value = vec![0.; result.len()];
        sector.evaluate(&[x], &mut value).unwrap();
        for (sum, value) in result.iter_mut().zip(value) {
            *sum += value;
        }
    }
    result
}

#[test]
fn source_symmetric_compact_charts_keep_complete_checked_face_associations() {
    let x = symbol!("compact_symmetry::x");
    let y = symbol!("compact_symmetry::y");
    let eps = symbol!("compact_symmetry::eps");
    let power = -Atom::one() - Atom::var(eps);
    let f = (Atom::var(x) + Atom::var(y)).pow(Atom::num(2)) - 5 * Atom::var(x) * Atom::var(y);
    let factor = |f| {
        PolynomialFactor::new(f, Atom::var(eps), FactorRole::Singularity)
            .with_semantics(FactorSemantics::Causal)
    };
    let input = ParametricIntegrand::new(
        vec![x, y],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![power.clone(), power.clone()],
            vec![factor(f)],
        )],
    )
    .unwrap();
    // Each native primary chart is the same unit interval. This independent
    // native input supplies the exact multiplicity without symmetry admission.
    let reference = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(2),
            vec![power],
            vec![factor(
                Atom::one() - 3 * Atom::var(x) + Atom::var(x).pow(Atom::num(2)),
            )],
        )],
    )
    .unwrap();
    for (recipe, deformation) in [
        (ProgramRecipe::FixedV1, ContourMode::Fixed { lambda: 0.05 }),
        (
            ProgramRecipe::DynamicPolynomialV1,
            ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: 0.05,
                displacement_cap: 1.,
                construction: DynamicConstruction::Polynomial,
            },
        ),
        (
            ProgramRecipe::DynamicSignAwareV1,
            ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: 0.05,
                displacement_cap: 1.,
                construction: DynamicConstruction::SignAware,
            },
        ),
    ] {
        let options = GenerationOptions {
            program_recipe: recipe,
            max_order: 1,
            ..Default::default()
        };
        let generated = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
        assert_eq!(generated.metadata().charts().len(), 2);
        assert_eq!(generated.metadata().charts()[1].representative(), 0);
        assert_eq!(generated.sectors().len(), 1);
        let contour = generated.metadata().charts()[0].contour().unwrap();
        assert!(!contour.function_definitions().is_empty());
        assert!(contour.validation_faces().contains(&vec![(0, 0)]));
        let reference = generate(&reference, &options, |_| ControlFlow::Continue(())).unwrap();
        let compile = |generated: &crate::generation::GeneratedIntegral| {
            let original = generated
                .compile_with_settings(CompilationSettings {
                    backend: EvaluatorBackend::Eager,
                    ..Default::default()
                })
                .unwrap();
            let mut restored = KernelSet::from_bytes(&original.to_bytes().unwrap()).unwrap();
            restored
                .bind_parameters_with_contour(
                    &BTreeMap::new(),
                    &ContourSettings {
                        deformation,
                        validation: ContourValidationOptions {
                            policy: ContourValidation::Always,
                            pilot_points: 1,
                        },
                    },
                )
                .unwrap();
            for chart in restored.contour_validation_charts() {
                restored
                    .validate_contour_point(chart.chart_index, &[0.31], true)
                    .unwrap();
            }
            restored.finish_contour_pilot().unwrap();
            restored
        };
        let mut actual = compile(&generated);
        let mut expected = compile(&reference);
        for x in [0.07, 0.31, 0.63, 0.91] {
            let a = point(&mut actual, x);
            let b = point(&mut expected, x);
            assert_eq!(a.len(), b.len());
            for (a, b) in a.into_iter().zip(b) {
                assert!(
                    (a - b).abs() < 2e-10 * (1. + b.abs()),
                    "{recipe:?}: {a} != {b}"
                );
            }
        }
    }
}

#[test]
fn one_projective_input_preserves_distinct_chart_signatures_after_restore() {
    let x = symbol!("compact_signatures::x");
    let y = symbol!("compact_signatures::y");
    let r = symbol!("compact_signatures::r");
    let t = symbol!("compact_signatures::t");
    let eps = symbol!("compact_signatures::eps");
    // The two primary charts have different native gradient dependencies:
    // r + t*x + t*x² uses only t; r*x² + t*x + t uses both r and t.
    let f = Atom::var(r) * Atom::var(x).pow(Atom::num(2))
        + Atom::var(t) * Atom::var(x) * Atom::var(y)
        + Atom::var(t) * Atom::var(y).pow(Atom::num(2));
    let input = ParametricIntegrand::new(
        vec![x, y],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(f, Atom::num(-1), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let bindings = BTreeMap::from([(r, 0.7), (t, 0.4)]);
    let settings = ContourSettings {
        deformation: ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 0.05,
            displacement_cap: 1.,
            construction: DynamicConstruction::Polynomial,
        },
        validation: ContourValidationOptions {
            policy: ContourValidation::Always,
            pilot_points: 1,
        },
    };
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let generated = generate(
            &input,
            &GenerationOptions {
                program_recipe: ProgramRecipe::DynamicPolynomialV1,
                mode,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(generated.sectors().len(), 2);
        let signatures = generated
            .sectors()
            .iter()
            .map(|sector| {
                let definitions = sector.contour_definitions();
                assert!(!definitions.is_empty());
                let arity = definitions.entries()[0].parameters().len();
                assert!(
                    definitions
                        .entries()
                        .iter()
                        .all(|entry| entry.parameters().len() == arity)
                );
                arity
            })
            .collect::<Vec<_>>();
        assert_ne!(signatures[0], signatures[1]);
        let original = generated
            .compile_with_settings_parameters_and_progress(
                Default::default(),
                &[r, t],
                CompilationSettings {
                    backend: EvaluatorBackend::Eager,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        let restored = KernelSet::from_bytes(&original.to_bytes().unwrap()).unwrap();
        let mut bound = [original, restored];
        for kernels in &mut bound {
            kernels
                .bind_parameters_with_contour(&bindings, &settings)
                .unwrap();
            for chart in kernels.contour_validation_charts() {
                kernels
                    .validate_contour_point(chart.chart_index, &[0.31], true)
                    .unwrap();
            }
            kernels.finish_contour_pilot().unwrap();
        }
        for coordinate in [0.13, 0.43, 0.81] {
            let [original, restored] = &mut bound;
            let a = point(original, coordinate);
            let b = point(restored, coordinate);
            assert_eq!(a.len(), b.len());
            for (a, b) in a.into_iter().zip(b) {
                assert!((a - b).abs() < 1e-12 * (1. + b.abs()));
            }
        }
    }
}
