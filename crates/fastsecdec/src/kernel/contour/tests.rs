use super::*;
use crate::contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions};
use crate::kernel::{
    CompilationSettings, EvaluatorBackend, PrecisionPolicy, cancellation::Cancellation,
    evaluator::MappingRequirements, program,
};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore},
    domains::float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, RealLike},
    symbol,
};

fn template(backend: EvaluatorBackend) -> KernelSet {
    let x = symbol!("contour_runtime_test::x");
    let argument = Atom::var(x) - 2;
    let coefficient = crate::contour::functions::causal_log(&argument);
    let program = program::build_with_settings(
        vec![x],
        &[],
        &[AliasedAtom::from(coefficient)],
        Cancellation::new(0, None, 1).unwrap(),
        CompilationSettings {
            backend,
            ..Default::default()
        },
    )
    .unwrap();
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![program],
        vec![Atom::Zero],
        PrecisionPolicy::default(),
        None,
        true,
        vec![],
        CompilationSettings {
            backend,
            ..Default::default()
        },
    )
    .unwrap();
    kernels.initialize_artifact().unwrap();
    kernels
}

#[test]
fn stationary_negative_f_has_lower_lip_in_eager_and_native_kernels() {
    let backends = [EvaluatorBackend::Eager, EvaluatorBackend::Auto];
    for backend in backends {
        let mut kernel = template(backend);
        let mut output = [0.0; 2];
        kernel.sectors_mut()[0]
            .evaluate(&[0.5], &mut output)
            .unwrap();
        assert!((output[0] - 1.5_f64.ln()).abs() < 1e-14);
        assert!((output[1] + std::f64::consts::PI).abs() < 1e-14);
    }
}

#[test]
fn causal_log_supports_native_precision_and_retains_tracked_errors() {
    let x = symbol!("contour_runtime_test::precision_x");
    let exact = crate::contour::functions::causal_log(&Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    let requirements = MappingRequirements::new(&exact).unwrap();
    let mut dd = requirements
        .map(
            &exact,
            |c| Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im)),
            106,
        )
        .unwrap();
    let mut dd_out = [Complex::new(DoubleFloat::from(0.), DoubleFloat::from(0.))];
    dd.evaluate(
        &[Complex::new(DoubleFloat::from(-2.), DoubleFloat::from(0.))],
        &mut dd_out,
    );
    assert!((dd_out[0].im.to_f64() + std::f64::consts::PI).abs() < 1e-15);
    let mut mp = requirements
        .map(
            &exact,
            |c| Complex::new(c.re.to_multi_prec_float(256), c.im.to_multi_prec_float(256)),
            256,
        )
        .unwrap();
    let mut mp_out = [Complex::new(
        Float::with_val(256, 0),
        Float::with_val(256, 0),
    )];
    mp.evaluate(
        &[Complex::new(
            Float::with_val(256, -2),
            Float::with_val(256, 0),
        )],
        &mut mp_out,
    );
    assert!((mp_out[0].im.to_f64() + std::f64::consts::PI).abs() < 1e-15);
    let mut tracked = requirements
        .map(
            &exact,
            |c| {
                Complex::new(
                    ErrorPropagatingFloat::new(c.re.to_f64(), 15.),
                    ErrorPropagatingFloat::new(c.im.to_f64(), 15.),
                )
            },
            53,
        )
        .unwrap();
    let mut tracked_out = [Complex::new(
        ErrorPropagatingFloat::new(0., 15.),
        ErrorPropagatingFloat::new(0., 15.),
    )];
    tracked.evaluate(
        &[Complex::new(
            ErrorPropagatingFloat::new(-2., 12.),
            ErrorPropagatingFloat::new_with_accuracy(0., f64::INFINITY),
        )],
        &mut tracked_out,
    );
    assert!(tracked_out[0].re.get_absolute_error() > 0.);
    assert!(tracked_out[0].im.get_absolute_error() > 0.);
    assert!((tracked_out[0].im.to_f64() + std::f64::consts::PI).abs() < 1e-15);
}

#[test]
fn causal_log_saved_program_registers_in_a_fresh_process() {
    const PATH_ENV: &str = "FASTSECDEC_CAUSAL_LOG_TEST_ARTIFACT";
    if let Some(path) = std::env::var_os(PATH_ENV) {
        // Deliberately do not touch the callback symbol before decoding.
        let mut loaded = KernelSet::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        let mut output = [0.0; 2];
        loaded.sectors_mut()[0]
            .evaluate(&[0.5], &mut output)
            .unwrap();
        assert!((output[1] + std::f64::consts::PI).abs() < 1e-14);
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("causal.fsd");
    std::fs::write(
        &path,
        template(EvaluatorBackend::Auto).artifact_bytes().unwrap(),
    )
    .unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "kernel::contour::tests::causal_log_saved_program_registers_in_a_fresh_process",
            "--nocapture",
        ])
        .env(PATH_ENV, path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn deformed(f: Atom, positive: Option<Atom>) -> KernelSet {
    use crate::{
        generation::{GenerationOptions, generate},
        parametric::{
            FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
            PolynomialFactor,
        },
    };
    let mut factors = vec![
        PolynomialFactor::new(f, Atom::num(-1), FactorRole::Singularity)
            .with_semantics(FactorSemantics::Causal),
    ];
    if let Some(u) = positive {
        factors.push(
            PolynomialFactor::new(u, Atom::one(), FactorRole::Singularity)
                .with_semantics(FactorSemantics::Positive),
        );
    }
    let source = ParametricIntegrand::new(
        vec![symbol!("contour_validation_test::x")],
        symbol!("contour_validation_test::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(Atom::one(), vec![Atom::Zero], factors)],
    )
    .unwrap();
    generate(
        &source,
        &GenerationOptions {
            program_recipe: crate::kernel::ProgramRecipe::FixedV1,
            ..Default::default()
        },
        |_| std::ops::ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    })
    .unwrap()
}
fn settings(lambda: f64, policy: ContourValidation) -> ContourSettings {
    ContourSettings {
        deformation: ContourMode::Fixed { lambda },
        validation: ContourValidationOptions {
            policy,
            pilot_points: 2,
        },
    }
}

#[test]
fn certified_checks_admit_stationary_negative_f_and_reject_causal_crossing() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let mut stationary = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    stationary
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Always))
        .unwrap();
    let check = stationary.validate_contour_point(0, &[0.5], true).unwrap();
    assert_eq!(check.checked_arguments, 4);
    stationary.validate_contour_point(0, &[0.2], true).unwrap();
    stationary.finish_contour_pilot().unwrap();
    let mut output = vec![0.; stationary.orders().len()];
    stationary.sectors_mut()[0]
        .evaluate(&[0.5], &mut output)
        .unwrap();
    assert!(output.iter().all(|value| value.is_finite()));

    let mut crossing = deformed(Atom::one() - &x - x.pow(3), None);
    crossing
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(4., ContourValidation::Always))
        .unwrap();
    let error = crossing
        .validate_contour_point(0, &[0.5], true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("positive imaginary"), "{error}");
    let evidence = crossing.contour_validation_report().unwrap();
    assert_eq!(evidence.accepted_pilot_points, 0);
    assert_eq!(evidence.checked_arguments, 3);
    assert!(
        crossing.sectors_mut()[0]
            .evaluate(&[0.5], &mut output)
            .is_err()
    );
}

#[test]
fn positive_factor_and_unresolved_stationary_zero_are_never_silently_accepted() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let mut positive = deformed(Atom::one() - &x, Some(Atom::one() + x.pow(2)));
    positive
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(8., ContourValidation::Always))
        .unwrap();
    let error = positive
        .validate_contour_point(0, &[0.5], false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("U[0]"), "{error}");
    // This U becomes positive at the deformed endpoint despite starting on
    // the wrong real branch; validating only z would miss that crossing.
    let mut wrong_origin = deformed(Atom::one() - &x, Some(-Atom::one() - x.pow(2)));
    wrong_origin
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(8., ContourValidation::Always))
        .unwrap();
    let error = wrong_origin
        .validate_contour_point(0, &[0.5], false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("homotopy fraction=0"), "{error}");
    let mut pinch = deformed((&x - Atom::num((1, 2))).pow(2), None);
    pinch
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Always))
        .unwrap();
    let error = pinch
        .validate_contour_point(0, &[0.5], true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unresolved"), "{error}");
}

#[test]
fn homotopy_pilot_rejects_an_intermediate_crossing_even_when_the_endpoint_passes() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let h = &x - Atom::num((1, 2));
    let mut kernels = deformed(
        Atom::one() + Atom::num(4) * &h + Atom::num(40) * h.pow(3) + Atom::num(50) * h.pow(5),
        None,
    );
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(1., ContourValidation::Always))
        .unwrap();
    for _ in 0..2 {
        kernels.validate_contour_point(0, &[0.5], false).unwrap();
    }
    assert_eq!(
        kernels
            .contour_validation_report()
            .unwrap()
            .accepted_pilot_points,
        0
    );
    assert!(kernels.finish_contour_pilot().is_err());
    let error = kernels
        .validate_contour_point(0, &[0.5], true)
        .unwrap_err()
        .to_string();
    assert!(error.contains("positive imaginary"), "{error}");
    assert!(error.contains("homotopy fraction=0.5"), "{error}");
    assert_eq!(
        kernels
            .contour_validation_report()
            .unwrap()
            .accepted_pilot_points,
        0
    );
}

#[test]
fn optional_certified_checks_do_not_prohibit_supported_unchecked_algebraic_coefficients() {
    let mut kernels = deformed(
        symbolica::parse!("1-2^(1/2)*contour_validation_test::x"),
        None,
    );
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Always))
        .unwrap();
    assert!(kernels.validate_contour_point(0, &[0.25], true).is_err());
    assert!(kernels.finish_contour_pilot().is_err());
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Off))
        .unwrap();
    let mut values = vec![0.; kernels.orders().len()];
    kernels.sectors_mut()[0]
        .evaluate(&[0.25], &mut values)
        .unwrap();
    assert!(values.iter().all(|x| x.is_finite()));
    assert!(kernels.contour_validation_charts().is_empty());
}

#[test]
fn pilot_unlocks_production_and_policy_changes_preserve_numerical_identity() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let mut kernels = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Pilot))
        .unwrap();
    let id = kernels.content_id().to_owned();
    let mut output = vec![0.; kernels.orders().len()];
    assert!(
        kernels.sectors_mut()[0]
            .evaluate(&[0.4], &mut output)
            .is_err()
    );
    assert!(kernels.finish_contour_pilot().is_err());
    for point in [0.2, 0.5] {
        kernels.validate_contour_point(0, &[point], true).unwrap();
    }
    let report = kernels.finish_contour_pilot().unwrap();
    assert!(report.pilot_complete);
    kernels.sectors_mut()[0]
        .evaluate(&[0.4], &mut output)
        .unwrap();
    let value = output.clone();
    kernels
        .set_contour_validation(ContourValidationOptions {
            policy: ContourValidation::Off,
            pilot_points: 2,
        })
        .unwrap();
    assert_eq!(kernels.content_id(), id);
    assert!(kernels.contour_validation_report().unwrap().pilot_complete);
    kernels.sectors_mut()[0]
        .evaluate(&[0.4], &mut output)
        .unwrap();
    assert_eq!(output, value);
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.3, ContourValidation::Pilot))
        .unwrap();
    assert_ne!(kernels.content_id(), id);
    assert!(!kernels.contour_validation_report().unwrap().pilot_complete);
}

#[test]
fn saved_validation_programs_survive_native_and_indexed_reloads() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let kernels = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    let (indexed, _) = crate::kernel::indexed::to_bytes(&kernels).unwrap();
    for bytes in [kernels.to_bytes().unwrap(), indexed] {
        let mut loaded = KernelSet::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.contour_checks.len(), 1);
        assert_eq!(
            loaded.contour_checks[0].program,
            kernels.contour_checks[0].program
        );
        loaded
            .bind_parameters_with_contour(
                &BTreeMap::new(),
                &settings(0.2, ContourValidation::Always),
            )
            .unwrap();
        loaded.validate_contour_point(0, &[0.5], true).unwrap();
    }
}

#[test]
fn validation_counters_belong_to_the_evaluator_that_performs_the_work() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let mut kernels = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Always))
        .unwrap();
    kernels.validate_contour_point(0, &[0.2], true).unwrap();
    kernels.validate_contour_point(0, &[0.5], true).unwrap();
    kernels.finish_contour_pilot().unwrap();
    let mut output = vec![0.; kernels.orders().len()];
    kernels.sectors_mut()[0]
        .evaluate(&[0.3], &mut output)
        .unwrap();
    assert_eq!(
        kernels
            .contour_validation_report()
            .unwrap()
            .checked_arguments,
        8
    );
    assert_eq!(
        kernels
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments,
        1
    );
    let mut worker = kernels.sectors()[0].try_clone().unwrap();
    assert_eq!(
        worker
            .contour_validation_report()
            .unwrap()
            .checked_arguments,
        0
    );
    for point in [0.25, 0.4] {
        worker.evaluate(&[point], &mut output).unwrap();
    }
    assert_eq!(
        worker
            .contour_validation_report()
            .unwrap()
            .checked_arguments,
        2
    );
    assert_eq!(
        worker
            .take_contour_validation_report()
            .unwrap()
            .checked_arguments,
        2
    );
    assert_eq!(
        worker
            .contour_validation_report()
            .unwrap()
            .checked_arguments,
        0
    );
    worker.evaluate(&[0.4], &mut output).unwrap();
    assert_eq!(
        worker
            .take_contour_validation_report()
            .unwrap()
            .checked_arguments,
        1
    );
    assert_eq!(
        kernels
            .contour_validation_report()
            .unwrap()
            .production_checked_arguments,
        1
    );
    let context = kernels
        .evaluation_context(0, crate::kernel::ReplayPolicy::default())
        .unwrap();
    assert_eq!(
        context
            .contour_validation_report()
            .unwrap()
            .checked_arguments,
        0
    );
}

#[test]
fn completing_a_cloned_pilot_does_not_unlock_another_bound_owner() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let mut kernels = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, ContourValidation::Pilot))
        .unwrap();
    let mut cloned = kernels.try_clone().unwrap();
    for point in [0.2, 0.5] {
        cloned.validate_contour_point(0, &[point], true).unwrap();
    }
    cloned.finish_contour_pilot().unwrap();
    let mut output = vec![0.; kernels.orders().len()];
    cloned.sectors_mut()[0]
        .evaluate(&[0.4], &mut output)
        .unwrap();
    assert!(
        kernels.sectors_mut()[0]
            .evaluate(&[0.4], &mut output)
            .is_err()
    );
    assert!(!kernels.contour_validation_report().unwrap().pilot_complete);
    kernels
        .set_contour_validation(ContourValidationOptions {
            policy: ContourValidation::Off,
            pilot_points: 2,
        })
        .unwrap();
    assert!(kernels.finish_contour_pilot().is_err());
    assert!(!kernels.contour_validation_report().unwrap().pilot_complete);
    assert!(kernels.sectors()[0].contour_validation_report().is_none());
}

#[test]
fn indexed_literal_zero_exact_record_needs_no_contour_chart_evidence() {
    let x = Atom::var(symbol!("contour_validation_test::x"));
    let kernels = deformed(Atom::one() - 5 * &x * (Atom::one() - &x), None);
    let (bytes, catalogue) = crate::kernel::indexed::to_bytes(&kernels).unwrap();
    let record = catalogue
        .records
        .iter()
        .position(|record| record.sector.is_none())
        .unwrap();
    let mut reader = crate::kernel::indexed::IndexedReader::from_reader(
        std::io::Cursor::new(bytes),
        crate::kernel::KernelLoadOptions::default(),
    )
    .unwrap();
    for policy in [ContourValidation::Always, ContourValidation::Pilot] {
        let mut exact = reader.load_record(record).unwrap();
        assert!(exact.sectors().is_empty());
        assert!(exact.exact_expressions.iter().all(|value| value.is_zero()));
        exact
            .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, policy))
            .unwrap();
        assert!(exact.contour_validation_charts().is_empty());
        let report = exact.finish_contour_pilot().unwrap();
        assert!(report.pilot_complete);
        assert_eq!(report.checked_arguments, 0);
        // Even an apparently tiny nonzero expression requires metadata. This
        // must not become a tolerance-based exemption for missing provenance.
        exact.exact_expressions[0] = Atom::num((1, 1_000_000));
        exact.metadata = None;
        assert!(
            exact
                .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, policy))
                .is_err()
        );
    }
}

#[test]
fn exact_contour_problem_requires_scoped_pilot_after_binding_and_reload() {
    use crate::results::{ExactContributionPolicy, KernelResultManifest, ResultScope};
    let template = deformed(Atom::num(2), None);
    assert!(template.sectors().is_empty());
    let bytes = template.to_bytes().unwrap();
    let problem = |kernels: &KernelSet, scope: &ResultScope| {
        KernelResultManifest::integration_problem_from_kernels(kernels, scope, kernels.content_id())
    };
    let exclude = ResultScope::SelectedSectors {
        sector_ids: Vec::new(),
        exact_policy: ExactContributionPolicy::ExcludeAll,
    };
    let include = ResultScope::SelectedSectors {
        sector_ids: Vec::new(),
        exact_policy: ExactContributionPolicy::IncludeAll,
    };
    for policy in [ContourValidation::Always, ContourValidation::Pilot] {
        let mut kernels = KernelSet::from_bytes(&bytes).unwrap();
        assert!(problem(&kernels, &ResultScope::FullIntegral).is_err());
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.2, policy))
            .unwrap();
        // Inspection does not claim validated integration readiness.
        assert!(kernels.exact_coefficients().iter().any(|v| *v != 0.));
        assert!(problem(&kernels, &ResultScope::FullIntegral).is_err());
        assert!(problem(&kernels, &include).is_err());
        assert!(
            problem(&kernels, &exclude)
                .unwrap()
                .exact_coefficients
                .iter()
                .all(|v| *v == 0.)
        );
        for chart in kernels.contour_validation_charts() {
            for coordinate in [0.3, 0.6] {
                kernels
                    .validate_contour_point(
                        chart.chart_index,
                        &vec![coordinate; chart.dimension],
                        true,
                    )
                    .unwrap();
            }
        }
        kernels.finish_contour_pilot().unwrap();
        let accepted = problem(&kernels, &ResultScope::FullIntegral).unwrap();
        assert_eq!(accepted.exact_coefficients, kernels.exact_coefficients());
        assert!(problem(&kernels, &include).is_ok());
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.3, policy))
            .unwrap();
        assert!(problem(&kernels, &ResultScope::FullIntegral).is_err());
        kernels
            .set_contour_validation(ContourValidationOptions {
                policy: ContourValidation::Off,
                pilot_points: 2,
            })
            .unwrap();
        assert!(problem(&kernels, &ResultScope::FullIntegral).is_ok());
    }
}

#[test]
fn selected_pilot_scope_does_not_check_or_unlock_an_excluded_sector() {
    use crate::{
        generation::{GenerationOptions, generate},
        parametric::{
            FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
            PolynomialFactor,
        },
    };
    let x = symbol!("contour_scope_test::x");
    let y = symbol!("contour_scope_test::y");
    let source = ParametricIntegrand::new(
        vec![x, y],
        symbol!("contour_scope_test::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(
                    (Atom::var(x) - Atom::num(2) * Atom::var(y)).pow(2),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &source,
        &GenerationOptions {
            program_recipe: crate::kernel::ProgramRecipe::FixedV1,
            ..Default::default()
        },
        |_| std::ops::ControlFlow::Continue(()),
    )
    .unwrap();
    let template = generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(template.sectors().len(), 2);
    for policy in [ContourValidation::Always, ContourValidation::Pilot] {
        let mut kernels = template.try_clone().unwrap();
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &settings(0.05, policy))
            .unwrap();
        let mut selected = None;
        let mut excluded = None;
        for chart in kernels.contour_validation_charts() {
            if kernels
                .validate_contour_point(chart.chart_index, &[0.5], true)
                .is_ok()
            {
                selected = Some(chart);
            } else {
                excluded = Some(chart);
            }
        }
        let selected = selected.expect("one chart has no stationary zero at the pilot point");
        let excluded = excluded.expect("the other chart has an unresolved stationary zero");
        kernels
            .validate_contour_point(selected.chart_index, &[0.25], true)
            .unwrap();
        let report = kernels
            .finish_contour_pilot_for_charts(&[selected.chart_index])
            .unwrap();
        assert!(report.pilot_complete);
        assert_eq!(report.required_charts, vec![selected.chart_index]);
        assert_eq!(report.validated_charts, vec![selected.chart_index]);
        assert!(!kernels.contour_validation_report().unwrap().pilot_complete);
        assert!(kernels.finish_contour_pilot().is_err());
        let scope = crate::results::ResultScope::SelectedSectors {
            sector_ids: vec![selected.kernel_sector.unwrap() as u64],
            exact_policy: crate::results::ExactContributionPolicy::ExcludeAll,
        };
        assert!(
            crate::results::KernelResultManifest::integration_problem_from_kernels(
                &kernels,
                &scope,
                kernels.content_id(),
            )
            .is_ok()
        );
        assert!(
            crate::results::KernelResultManifest::integration_problem_from_kernels(
                &kernels,
                &crate::results::ResultScope::FullIntegral,
                kernels.content_id(),
            )
            .is_err()
        );
        let mut output = vec![0.; kernels.orders().len()];
        kernels.sectors_mut()[selected.kernel_sector.unwrap()]
            .evaluate(&[0.3], &mut output)
            .unwrap();
        assert!(
            kernels.sectors_mut()[excluded.kernel_sector.unwrap()]
                .evaluate(&[0.3], &mut output)
                .is_err()
        );
    }
}
