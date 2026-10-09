use super::*;
use crate::{
    generation::{
        CoefficientExpansionMethod, GenerationMode, GenerationOptions, SubtractionStrategy,
        generate,
    },
    kernel::{
        CompilationSettings, EvaluatorBackend, KernelSet, PrecisionPolicy, RuntimeMassConstraint,
    },
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, ops::ControlFlow, path::Path};
use symbolica::{
    atom::{Atom, Symbol},
    domains::{float::Complex, rational::Rational},
    parse, symbol,
};
mod program;

fn keep(_: &crate::generation::GenerationProgress) -> ControlFlow<()> {
    ControlFlow::Continue(())
}

fn prepared(
    root: &Path,
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    runtime: &[Symbol],
    constraints: &[RuntimeMassConstraint],
) -> PreparedGeneration {
    let preparation =
        prepare_with_runtime(input, options, runtime, constraints, root, keep).unwrap();
    let charts = preparation
        .charts
        .iter()
        .map(|job| discover(root, &preparation, job, keep).unwrap())
        .collect::<Vec<_>>();
    let mut representatives = Vec::new();
    let mut assignments = Vec::new();
    let mut formulas = BTreeMap::new();
    for chart in &charts {
        let candidates = if options.mode == GenerationMode::Symbolic {
            representatives
                .iter()
                .filter(|c: &&DiscoveredSector| c.symmetry_key == chart.symmetry_key)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let assignment = compare_symmetry(root, &preparation, chart, &candidates, keep).unwrap();
        if assignment.representative == chart.index {
            representatives.push(chart.clone());
        }
        assignments.push(assignment);
        if let Some(key) = &chart.formula_key {
            formulas
                .entry(key.clone())
                .or_insert_with(|| build_formula(root, &preparation, chart, keep).unwrap());
        }
    }
    // Reordered completed jobs must not change source/representative ordering.
    let mut charts = charts;
    charts.reverse();
    assignments.reverse();
    finish_preparation(
        &preparation,
        charts,
        assignments,
        formulas.into_values().collect(),
    )
    .unwrap()
}

type Vector = BTreeMap<(i32, CoefficientComponent), f64>;
fn sample(mut kernels: KernelSet, point: &[f64], runtime: &[f64]) -> Vector {
    if !runtime.is_empty() {
        let mut values = kernels
            .runtime_parameters()
            .iter()
            .copied()
            .zip(runtime.iter().copied())
            .collect::<BTreeMap<_, _>>();
        if let Some(lambda) = values.remove(&crate::contour::lambda_symbol()) {
            kernels
                .bind_parameters_with_contour(
                    &values,
                    &crate::contour::ContourSettings {
                        deformation: crate::contour::ContourMode::Fixed { lambda },
                        validation: crate::contour::ContourValidationOptions {
                            policy: crate::contour::ContourValidation::Off,
                            ..Default::default()
                        },
                    },
                )
                .unwrap();
        } else {
            kernels.bind_parameters(&values).unwrap();
        }
    }
    let mut values = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut output = vec![0.; values.len()];
        sector.evaluate(point, &mut output).unwrap();
        for (a, b) in values.iter_mut().zip(output) {
            *a += b;
        }
    }
    kernels
        .orders()
        .iter()
        .copied()
        .zip(kernels.components().iter().copied())
        .zip(values)
        .collect()
}
fn compile(generated: &crate::generation::GeneratedIntegral, runtime: &[Symbol]) -> KernelSet {
    generated
        .compile_with_settings_parameters_and_progress(
            PrecisionPolicy::default(),
            runtime,
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
}
fn close(actual: &Vector, expected: &Vector) {
    for key in actual.keys().chain(expected.keys()) {
        let a = actual.get(key).copied().unwrap_or(0.);
        let b = expected.get(key).copied().unwrap_or(0.);
        assert!(
            a.is_finite() && b.is_finite() && (a - b).abs() < 2e-10 * b.abs().max(1.),
            "{key:?}: {a} vs {b}"
        );
    }
}
fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("streaming_x"), symbol!("streaming_y")],
        symbol!("streaming_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(Complex::new(Rational::from(2), Rational::from(3))),
            vec![parse!("-1+streaming_eps"), parse!("streaming_eps")],
            vec![
                PolynomialFactor::new(
                    parse!("streaming_x+streaming_y"),
                    parse!("-1-streaming_eps"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("1+streaming_mass*streaming_x"),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn contour_source_semantics_and_maps_survive_streamed_generation() {
    use crate::parametric::FactorSemantics;
    let input = ParametricIntegrand::new(
        vec![symbol!("contour_stream::x")],
        symbol!("contour_stream::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/contour_stream::eps"),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("1+contour_stream::x"),
                    parse!("contour_stream::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Positive),
                PolynomialFactor::new(
                    parse!("1-5*contour_stream::x*(1-contour_stream::x)"),
                    parse!("-contour_stream::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let options = GenerationOptions {
            program_recipe: crate::kernel::indexed::ProgramRecipe::FixedV1,
            mode,
            ..Default::default()
        };
        let runtime = [crate::contour::lambda_symbol()];
        let ordinary = generate(&input, &options, keep).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let preparation = prepared(directory.path(), &input, &options, &runtime, &[]);
        assert_eq!(preparation.sectors.len(), 1);
        let unit = generate_sector(directory.path(), &preparation.sectors[0], keep).unwrap();
        let chart = &unit.generated.metadata().charts()[0];
        assert!(chart.contour().is_some());
        assert_eq!(chart.contour().unwrap().positive_polynomials().len(), 1);
        for point in [[0.1], [0.5], [0.9]] {
            close(
                &sample(compile(&ordinary, &runtime), &point, &[0.2]),
                &sample(compile(&unit.generated, &runtime), &point, &[0.2]),
            );
        }
    }
}

#[test]
fn streamed_native_sectors_preserve_complete_complex_vectors_and_runtime_context() {
    let input = input();
    let runtime = [symbol!("streaming_mass")];
    let constraints = [RuntimeMassConstraint {
        name: "mass".into(),
        expression: Atom::var(runtime[0]),
    }];
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let options = GenerationOptions {
                mode,
                subtraction,
                max_order: 0,
                ..Default::default()
            };
            let ordinary = generate(&input, &options, keep).unwrap();
            let expected = sample(compile(&ordinary, &runtime), &[0.37, 0.61], &[1.7]);
            let dir = tempfile::tempdir().unwrap();
            let plan = prepared(dir.path(), &input, &options, &runtime, &constraints);
            assert!(plan.source_charts > 1);
            // Context import must work even after unrelated native symbols exist.
            let _ = symbol!("streaming_unrelated_before_restore");
            let mut actual = Vector::new();
            for job in &plan.sectors {
                let unit = generate_sector(dir.path(), job, keep).unwrap();
                assert!(unit.generated.sectors().len() <= 1);
                assert_eq!(unit.runtime_parameters, runtime);
                assert_eq!(
                    unit.runtime_mass_constraints[0].expression,
                    constraints[0].expression
                );
                for (key, value) in sample(
                    compile(&unit.generated, &unit.runtime_parameters),
                    &[0.37, 0.61],
                    &[1.7],
                ) {
                    *actual.entry(key).or_default() += value;
                }
            }
            close(&actual, &expected);
        }
    }
}

#[test]
fn symmetry_multiplicity_and_all_original_chart_metadata_survive_spooling() {
    let input = ParametricIntegrand::new(
        vec![symbol!("stream_sym_x"), symbol!("stream_sym_y")],
        symbol!("stream_sym_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("stream_sym_x+stream_sym_y"),
                parse!("-2+stream_sym_eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let mut options = GenerationOptions::default();
    options.coefficient_expansion.method = CoefficientExpansionMethod::NativeNamed;
    let ordinary = generate(&input, &options, keep).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let plan = prepared(dir.path(), &input, &options, &[], &[]);
    assert!(plan.source_charts > plan.sectors.len());
    assert_eq!(plan.sectors.len(), ordinary.sectors().len());
    let mut actual = Vector::new();
    let mut covered = vec![];
    for job in &plan.sectors {
        let unit = generate_sector(dir.path(), job, keep).unwrap();
        assert_eq!(
            unit.source_indices.len(),
            unit.generated.metadata().charts().len()
        );
        covered.extend(unit.source_indices);
        for (k, v) in sample(compile(&unit.generated, &[]), &[0.29, 0.67], &[]) {
            *actual.entry(k).or_default() += v;
        }
    }
    covered.sort();
    assert_eq!(covered, (0..plan.source_charts).collect::<Vec<_>>());
    close(
        &actual,
        &sample(compile(&ordinary, &[]), &[0.29, 0.67], &[]),
    );
}

#[test]
fn exact_and_empty_inputs_still_deliver_a_complete_zero_sector_unit() {
    for terms in [
        vec![],
        vec![ParametricTerm::new(Atom::num(7), vec![], vec![])],
    ] {
        let input = ParametricIntegrand::new(
            vec![],
            symbol!("stream_exact_eps"),
            ParametricDomain::UnitCube,
            terms,
        )
        .unwrap();
        for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
            let options = GenerationOptions {
                mode,
                max_order: 1,
                ..Default::default()
            };
            let dir = tempfile::tempdir().unwrap();
            let plan = prepared(dir.path(), &input, &options, &[], &[]);
            assert_eq!(plan.sectors.len(), 1);
            let unit = generate_sector(dir.path(), &plan.sectors[0], keep).unwrap();
            assert!(unit.generated.sectors().is_empty());
            let ordinary = generate(&input, &options, keep).unwrap();
            assert_eq!(unit.generated.orders(), ordinary.orders());
            assert_eq!(
                unit.generated.exact_coefficients(),
                ordinary.exact_coefficients()
            );
        }
    }
}

#[test]
fn corrupted_records_and_incomplete_or_foreign_receipts_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let options = GenerationOptions::default();
    let input = input();
    let prep = prepare(&input, &options, dir.path(), keep).unwrap();
    let first = discover(dir.path(), &prep, &prep.charts[0], keep).unwrap();
    assert!(finish_preparation(&prep, vec![first.clone()], vec![], vec![]).is_err());
    let mut foreign = first.clone();
    foreign.source_id = "wrong source".into();
    assert!(compare_symmetry(dir.path(), &prep, &foreign, &[], keep).is_err());
    let file = first.record.resolve(dir.path()).unwrap();
    let mut bytes = std::fs::read(&file).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(file, bytes).unwrap();
    assert!(
        compare_symmetry(dir.path(), &prep, &first, &[], keep)
            .unwrap_err()
            .to_string()
            .contains("digest mismatch")
    );
    // An immutable source is not affected by failure in a chart's staging file.
    prep.source.verify(dir.path()).unwrap();
}

#[test]
fn cancellation_does_not_publish_a_completed_chart() {
    let dir = tempfile::tempdir().unwrap();
    let options = GenerationOptions::default();
    let input = input();
    let prep = prepare(&input, &options, dir.path(), keep).unwrap();
    let failure = discover(dir.path(), &prep, &prep.charts[0], |_| {
        ControlFlow::Break(())
    })
    .unwrap_err();
    assert!(matches!(
        failure,
        StreamingError::Generation(crate::generation::GenerationError::Cancelled)
    ));
    assert!(!std::fs::read_dir(dir.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("chart-")
    }));
}

#[test]
fn reusable_formula_records_and_exact_fallback_both_match_native_generation() {
    let shared = ParametricIntegrand::new(
        vec![symbol!("stream_formula_x"), symbol!("stream_formula_y")],
        symbol!("stream_formula_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("stream_formula_x+stream_formula_y"),
                    parse!("-1-stream_formula_eps"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("1+2*stream_formula_x+3*stream_formula_y"),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
            ],
        )],
    )
    .unwrap();
    let fallback = ParametricIntegrand::new(
        vec![symbol!("stream_fallback_x")],
        symbol!("stream_fallback_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("stream_fallback_x/stream_fallback_eps"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    for (input, point, shared_formulas) in [
        (&shared, vec![0.3, 0.7], true),
        (&fallback, vec![0.4], false),
    ] {
        let options = GenerationOptions {
            mode: GenerationMode::NumericalDual,
            max_order: 1,
            ..Default::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let plan = prepared(dir.path(), input, &options, &[], &[]);
        if shared_formulas {
            assert_eq!(plan.unique_formulas, 1);
            assert!(plan.sectors.len() > 1);
            assert!(
                plan.sectors
                    .windows(2)
                    .all(|pair| pair[0].formula.as_ref().unwrap().record
                        == pair[1].formula.as_ref().unwrap().record)
            );
        } else {
            assert_eq!(plan.unique_formulas, 0);
        }
        let mut actual = Vector::new();
        for job in &plan.sectors {
            let unit = generate_sector(dir.path(), job, keep).unwrap();
            if !shared_formulas {
                assert!(
                    unit.generated
                        .sectors()
                        .iter()
                        .all(|s| s.generation_mode() == GenerationMode::Symbolic)
                );
            }
            for (k, v) in sample(compile(&unit.generated, &[]), &point, &[]) {
                *actual.entry(k).or_default() += v;
            }
        }
        let ordinary = generate(input, &options, keep).unwrap();
        close(&actual, &sample(compile(&ordinary, &[]), &point, &[]));
    }
}

#[test]
fn an_unregulated_divergence_is_never_a_completed_zero_sector() {
    let input = ParametricIntegrand::new(
        vec![symbol!("stream_unregulated_x")],
        symbol!("stream_unregulated_eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::num(-2)],
            vec![],
        )],
    )
    .unwrap();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let options = GenerationOptions {
            mode,
            ..Default::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let plan = prepared(dir.path(), &input, &options, &[], &[]);
        let failure = match generate_sector(dir.path(), &plan.sectors[0], keep) {
            Ok(_) => panic!("unregulated divergence became a successful unit"),
            Err(error) => error,
        };
        assert!(matches!(
            failure,
            StreamingError::Generation(
                crate::generation::GenerationError::UnregulatedEndpoint { .. }
            )
        ));
    }
}
