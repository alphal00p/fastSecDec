//! Scientific controls for the independent deferred-map/native-jet lane.
use fastsecdec::{
    generation::{
        CoefficientExpansionMethod, GeneratedIntegral, GenerationContext, GenerationError,
        GenerationMode, GenerationOptions, GenerationProgress, GenerationSession, GeometryJob,
        SubtractionStrategy, SymbolicJob, generate,
    },
    kernel::{
        CompilationSession, CompilationSettings, EvaluatorBackend, KernelSet, PrecisionPolicy,
        ReplayPolicy, StabilitySettings,
    },
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, rational::Rational},
    parse, symbol,
};

fn generated(
    input: &ParametricIntegrand,
    mode: GenerationMode,
    strategy: SubtractionStrategy,
    order: i32,
) -> GeneratedIntegral {
    let mut options = GenerationOptions {
        mode,
        subtraction: strategy,
        max_order: order,
        ..Default::default()
    };
    options.coefficient_expansion.method = CoefficientExpansionMethod::NativeNamed;
    generate(input, &options, |_| ControlFlow::Continue(())).unwrap()
}

fn compile(input: &GeneratedIntegral, runtime: &[Symbol], backend: EvaluatorBackend) -> KernelSet {
    let mut result = input
        .compile_with_settings_parameters_and_progress(
            PrecisionPolicy::default(),
            runtime,
            CompilationSettings {
                backend,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    without_weight_escalation(&mut result);
    result
}

fn without_weight_escalation(kernels: &mut KernelSet) {
    let mut stability = StabilitySettings::default();
    for level in &mut stability.levels {
        level.escalate_for_large_weight_threshold = None;
    }
    kernels.set_stability_settings(&stability).unwrap();
}

fn total(kernels: &mut KernelSet, point: &[f64]) -> Vec<f64> {
    let mut total = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut output = vec![0.0; total.len()];
        sector.evaluate(point, &mut output).unwrap();
        for (total, value) in total.iter_mut().zip(output) {
            *total += value;
        }
    }
    total
}

fn laurent_orders(kernels: &KernelSet) -> Vec<i32> {
    kernels
        .orders()
        .iter()
        .zip(kernels.components())
        .filter_map(|(order, component)| {
            (*component == CoefficientComponent::Real).then_some(*order)
        })
        .collect()
}

// A proved-real symbolic evaluator may omit imaginary outputs, whereas a
// deferred evaluator conservatively retains them. Compare the same physical
// Laurent vector without asserting an optional real-output optimization.
fn complex_values(kernels: &KernelSet, values: &[f64]) -> Vec<f64> {
    assert_eq!(values.len(), kernels.orders().len());
    let by_component = kernels
        .orders()
        .iter()
        .zip(kernels.components())
        .zip(values)
        .map(|((order, component), value)| ((*order, *component), *value))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(by_component.len(), values.len());
    laurent_orders(kernels)
        .into_iter()
        .flat_map(|order| {
            [
                by_component[&(order, CoefficientComponent::Real)],
                by_component
                    .get(&(order, CoefficientComponent::Imag))
                    .copied()
                    .unwrap_or(0.0),
            ]
        })
        .collect()
}

fn total_complex(kernels: &mut KernelSet, point: &[f64]) -> Vec<f64> {
    let values = total(kernels, point);
    complex_values(kernels, &values)
}

fn close(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            actual.is_finite() && (actual - expected).abs() <= tolerance * expected.abs().max(1.0),
            "complete vectors differ: {actual:?} vs {expected:?}"
        );
    }
}

#[test]
fn taylor_and_ibp_retain_analytic_complex_laurent_vector() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_science::x")],
        symbol!("dual_science::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(Complex::new(Rational::from(2), Rational::from(3))),
            vec![parse!("-3+dual_science::eps")],
            vec![PolynomialFactor::new(
                parse!("(1+dual_science::x)^2"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    // Exact analytic continuation: (2+3i)*(1/(eps-2)+2/(eps-1)+1/eps).
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let generated = generated(&input, GenerationMode::NumericalDual, strategy, 1);
        assert!(
            generated
                .sectors()
                .iter()
                .all(|sector| sector.generation_mode() == GenerationMode::NumericalDual)
        );
        for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
            let mut kernels = compile(&generated, &[], backend);
            assert_eq!(kernels.orders(), [-1, -1, 0, 0, 1, 1]);
            for point in [[0.23], [0.81], [1e-5]] {
                close(
                    &total(&mut kernels, &point),
                    &[2.0, 3.0, -5.0, -7.5, -4.5, -6.75],
                    2e-9,
                );
            }
        }
    }
}

#[test]
fn runtime_branch_hidden_in_a_source_factor_retains_complex_intermediates() {
    let parameter = symbol!("dual_hidden_branch::p"; Real);
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_hidden_branch::x")],
        symbol!("dual_hidden_branch::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-3+dual_hidden_branch::eps")],
            vec![PolynomialFactor::new(
                parse!("(1+dual_hidden_branch::x)^2*dual_hidden_branch::p^(1/2)"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    // The branch occurs inside the deferred source factor, not a literal
    // complex constant in the formal recipe. For real p of either sign,
    // int x^(-3+eps) (1+x)^2 sqrt(p) dx analytically continues to
    // sqrt(p) * (1/eps - 5/2 - 9*eps/4 + O(eps^2)).
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let generated = Arc::new(generated(
            &input,
            GenerationMode::NumericalDual,
            strategy,
            1,
        ));
        assert!(!generated.sectors().is_empty());
        assert!(
            generated
                .sectors()
                .iter()
                .all(|sector| { sector.generation_mode() == GenerationMode::NumericalDual })
        );
        for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
            let settings = CompilationSettings {
                backend,
                ..Default::default()
            };
            let direct = compile(&generated, &[parameter], backend);
            let mut session = CompilationSession::new(
                generated.clone(),
                vec![parameter],
                PrecisionPolicy::default(),
                settings,
            )
            .unwrap();
            while !session.is_complete() {
                session.step(1, |_| ControlFlow::Continue(())).unwrap();
            }
            let stepped = session.take_result().unwrap();
            let dispatched = generated
                .compile_with_settings_parameters_and_dispatch(
                    PrecisionPolicy::default(),
                    &[parameter],
                    settings,
                    &mut |jobs| {
                        let mut completed =
                            jobs.map(|job| job.run()).collect::<Result<Vec<_>, _>>()?;
                        completed.reverse();
                        Ok(completed)
                    },
                    |_| ControlFlow::Continue(()),
                )
                .unwrap();
            let template = direct.to_bytes().unwrap();
            for mut compiled in [direct, stepped, dispatched] {
                assert_eq!(compiled.to_bytes().unwrap(), template);
                let mut loaded = KernelSet::from_bytes(&template).unwrap();
                for kernels in [&mut compiled, &mut loaded] {
                    assert_eq!(kernels.orders(), [-1, -1, 0, 0, 1, 1]);
                    assert!(
                        kernels
                            .sectors()
                            .iter()
                            .all(|sector| { sector.statistics().arithmetic == "complex" })
                    );
                    for (value, expected) in [
                        (4.0, [2.0, 0.0, -5.0, 0.0, -4.5, 0.0]),
                        (-4.0, [0.0, 2.0, 0.0, -5.0, 0.0, -4.5]),
                    ] {
                        kernels
                            .bind_parameters(&BTreeMap::from([(parameter, value)]))
                            .unwrap();
                        assert_eq!(kernels.to_bytes().unwrap(), template);
                        for validated in [false, true] {
                            if validated {
                                kernels
                                    .set_stability_settings(&StabilitySettings::validated())
                                    .unwrap();
                            } else {
                                without_weight_escalation(kernels);
                            }
                            for point in [[0.19], [0.73]] {
                                close(&total(kernels, &point), &expected, 2e-11);
                            }
                            assert_branch_batch(kernels, &expected);
                        }
                    }
                }
            }
        }
    }
}

fn assert_branch_batch(kernels: &KernelSet, expected: &[f64]) {
    let points = [0.19, 0.73, 0.37, 0.61, 0.29, 0.89, 0.51];
    let weights = [1.0, 3.0, 0.5, 1.0, 2.0, 0.25, 1.0];
    let outputs = kernels.orders().len();
    let mut totals = weights
        .iter()
        .flat_map(|weight| {
            kernels
                .exact_coefficients()
                .iter()
                .map(move |value| weight * value)
        })
        .collect::<Vec<_>>();
    for sector in 0..kernels.sectors().len() {
        let mut scalar = kernels
            .evaluation_context(sector, ReplayPolicy::default())
            .unwrap();
        let mut batch = kernels
            .evaluation_context(sector, ReplayPolicy::default())
            .unwrap();
        let mut reference = vec![0.0; outputs * points.len()];
        for ((point, weight), row) in points
            .iter()
            .zip(weights)
            .zip(reference.chunks_exact_mut(outputs))
        {
            scalar.evaluate_weighted(&[*point], weight, row).unwrap();
        }
        let mut values = vec![0.0; reference.len()];
        let reports = batch
            .evaluate_weighted_batch(&points, &weights, &mut values)
            .unwrap();
        assert_eq!(reports.len(), points.len());
        close(&values, &reference, 2e-11);
        close(batch.state().maxima(), scalar.state().maxima(), 2e-11);
        assert_eq!(batch.state().verified(), scalar.state().verified());
        let timing = batch.evaluation_metrics().f64;
        if kernels.sectors()[sector].statistics().backend == "symjit_o2" {
            assert_eq!(timing.matrix_invocations, 1);
            assert_eq!(timing.matrix_points, points.len() as u64);
        } else {
            assert_eq!(timing.matrix_invocations, 0);
        }
        for (total, value) in totals.iter_mut().zip(values) {
            *total += value;
        }
    }
    for (row, weight) in totals.chunks_exact(outputs).zip(weights) {
        close(
            row,
            &expected
                .iter()
                .map(|value| weight * value)
                .collect::<Vec<_>>(),
            2e-11,
        );
    }
}

#[test]
fn shifted_factor_runtime_parameters_and_binary_replay_match_symbolic_lane() {
    let mass = symbol!("dual_shift::m");
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_shift::x")],
        symbol!("dual_shift::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("dual_shift::x^3*(dual_shift::m+dual_shift::x)"),
                parse!("-1-dual_shift::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let dual = generated(&input, GenerationMode::NumericalDual, strategy, 1);
        let symbolic = generated(&input, GenerationMode::Symbolic, strategy, 1);
        // Explicit inspection restores the actual coefficient, never the
        // internal request-placeholder recipe returned by a compact view.
        assert_eq!(dual.orders(), symbolic.orders());
        assert_eq!(dual.sectors().len(), symbolic.sectors().len());
        for (candidate, reference) in dual.sectors().iter().zip(symbolic.sectors()) {
            for (candidate, reference) in candidate
                .coefficients()
                .iter()
                .zip(reference.coefficients())
            {
                assert!((candidate - reference).together().expand().is_zero());
            }
        }
        assert!(
            dual.sectors()
                .iter()
                .all(|sector| sector.generation_mode() == GenerationMode::NumericalDual)
        );
        let mut dual = compile(&dual, &[mass], EvaluatorBackend::Eager);
        let mut reference = compile(&symbolic, &[mass], EvaluatorBackend::Eager);
        assert_eq!(laurent_orders(&dual), laurent_orders(&reference));
        let template = dual.to_bytes().unwrap();
        for value in [1.25, 3.5] {
            let point = BTreeMap::from([(mass, value)]);
            dual.bind_parameters(&point).unwrap();
            reference.bind_parameters(&point).unwrap();
            assert_eq!(dual.to_bytes().unwrap(), template);
            let mut restored = KernelSet::from_bytes(&template).unwrap();
            restored.bind_parameters(&point).unwrap();
            for coordinate in [[0.17], [0.74], [1e-5]] {
                let actual = total_complex(&mut dual, &coordinate);
                close(&actual, &total_complex(&mut reference, &coordinate), 2e-8);
                close(&actual, &total_complex(&mut restored, &coordinate), 2e-12);
            }
        }
    }
}

#[test]
fn unregulated_zero_faces_use_exact_admission_and_divergences_remain_errors() {
    let make = |regular| {
        ParametricIntegrand::new(
            vec![symbol!("dual_admission::x")],
            symbol!("dual_admission::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::num(-2)],
                vec![PolynomialFactor::new(
                    regular,
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            )],
        )
        .unwrap()
    };
    let mut options = GenerationOptions {
        mode: GenerationMode::NumericalDual,
        ..Default::default()
    };
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        options.subtraction = strategy;
        let convergent = generate(&make(parse!("dual_admission::x^2")), &options, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let mut kernels = compile(&convergent, &[], EvaluatorBackend::Eager);
        close(&total_complex(&mut kernels, &[0.4]), &[1.0, 0.0], 1e-12);
        assert!(matches!(
            generate(&make(Atom::one()), &options, |_| ControlFlow::Continue(())),
            Err(fastsecdec::generation::GenerationError::UnregulatedEndpoint { .. })
        ));
    }
}

#[test]
fn exact_zero_does_not_create_a_numerical_sector() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_zero::x")],
        symbol!("dual_zero::eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    let generated = generated(
        &input,
        GenerationMode::NumericalDual,
        SubtractionStrategy::Taylor,
        0,
    );
    let kernels = compile(&generated, &[], EvaluatorBackend::Eager);
    assert!(kernels.sectors().is_empty());
    assert_eq!(kernels.exact_coefficients(), [0.0]);
}

fn qmc(kernels: &mut KernelSet) -> fastsecdec::integration::VectorEstimate {
    use fastsecdec::{
        integration::{Periodization, QmcSession, QmcSettings, RuleSource},
        results::{KernelResultManifest, ResultScope},
    };
    let problem = KernelResultManifest::from_kernels(kernels)
        .integration_problem(&ResultScope::FullIntegral, kernels.content_id())
        .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 4096,
            shifts: 8,
            seed: 4981,
            package_points: 4096,
            periodization: Periodization::Korobov3,
            rule: RuleSource::Supplied(vec![1, 1583]),
        },
    )
    .unwrap();
    while let Some(task) = session.next_work().unwrap() {
        let sector = task.sector_id() as usize;
        let result = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[sector]
                    .evaluate(point, output)
                    .map(|_| ())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    session.estimate().unwrap()
}

#[test]
fn conservative_numerator_valuation_keeps_ibp_integral_without_pointwise_assumption() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_conservative::x")],
        symbol!("dual_conservative::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-2+dual_conservative::eps")],
            vec![
                PolynomialFactor::new(
                    parse!("dual_conservative::x"),
                    Atom::one(),
                    FactorRole::Polynomial,
                ),
                PolynomialFactor::new(
                    parse!("1+dual_conservative::x"),
                    Atom::num(-1),
                    FactorRole::Singularity,
                ),
            ],
        )],
    )
    .unwrap();
    // The original integral is int_0^1 x^(-1+eps)/(1+x) dx
    // = 1/eps - log(2) + O(eps). Conservative numerator valuation can leave
    // another IBP step, so this control compares integrals, not densities.
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let mut reference = compile(
            &generated(&input, GenerationMode::Symbolic, strategy, 0),
            &[],
            EvaluatorBackend::Eager,
        );
        let mut candidate = compile(
            &generated(&input, GenerationMode::NumericalDual, strategy, 0),
            &[],
            EvaluatorBackend::Eager,
        );
        assert_eq!(laurent_orders(&reference), laurent_orders(&candidate));
        for kernels in [&mut reference, &mut candidate] {
            let estimate = qmc(kernels);
            assert!(estimate.production_complete);
            close(
                &complex_values(kernels, &estimate.mean),
                &[1.0, 0.0, -std::f64::consts::LN_2, 0.0],
                2e-8,
            );
            assert_eq!(
                estimate.covariance_of_mean.len(),
                kernels.orders().len().pow(2)
            );
        }
    }
}

#[test]
fn epsilon_pole_in_a_regular_source_factor_uses_exact_symbolic_fallback() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_epsilon_pole::x")],
        symbol!("dual_epsilon_pole::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("dual_epsilon_pole::x/dual_epsilon_pole::eps"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let value = generated(&input, GenerationMode::NumericalDual, strategy, 1);
        assert!(
            value
                .sectors()
                .iter()
                .all(|sector| sector.generation_mode() == GenerationMode::Symbolic)
        );
        let mut kernels = compile(&value, &[], EvaluatorBackend::Eager);
        assert_eq!(kernels.orders(), [-1, 0, 1]);
        close(&qmc(&mut kernels).mean, &[0.5, 0.0, 0.0], 1e-9);
    }
}

#[test]
fn mixed_faces_and_regulator_prefactor_keep_the_complete_vector() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_mixed::x"), symbol!("dual_mixed::y")],
        symbol!("dual_mixed::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/dual_mixed::eps"),
            vec![parse!("-2+dual_mixed::eps"), parse!("-2+2*dual_mixed::eps")],
            vec![PolynomialFactor::new(
                parse!("(1+dual_mixed::x)*(1+dual_mixed::y)*(1+dual_mixed::eps)"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let mut reference = compile(
            &generated(&input, GenerationMode::Symbolic, strategy, 1),
            &[],
            EvaluatorBackend::Eager,
        );
        let value = generated(&input, GenerationMode::NumericalDual, strategy, 1);
        assert!(
            value
                .sectors()
                .iter()
                .all(|sector| sector.generation_mode() == GenerationMode::NumericalDual)
        );
        let mut candidate = compile(&value, &[], EvaluatorBackend::Eager);
        assert_eq!(laurent_orders(&candidate), laurent_orders(&reference));
        assert_eq!(candidate.orders().first(), Some(&-3));
        for point in [[0.17, 0.79], [0.91, 0.32]] {
            close(
                &total_complex(&mut candidate, &point),
                &total_complex(&mut reference, &point),
                1e-9,
            );
        }
    }
}

fn dispatch_input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("dual_dispatch::x"), symbol!("dual_dispatch::y")],
        symbol!("dual_dispatch::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("dual_dispatch::x+dual_dispatch::y"),
                parse!("-1-dual_dispatch::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap()
}

fn reverse_geometry(
    jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>,
) -> Result<Vec<fastsecdec::generation::GeometryCompletion>, fastsecdec_sectors::SectorError> {
    let mut results = jobs
        .map(|job| job.run(|_| ControlFlow::Continue(())))
        .collect::<Vec<_>>();
    results.reverse();
    Ok(results)
}

#[test]
fn retained_sessions_and_reversed_caller_dispatch_preserve_native_artifacts() {
    let input = dispatch_input();
    let options = GenerationOptions {
        mode: GenerationMode::NumericalDual,
        max_order: 1,
        ..Default::default()
    };
    let serial = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    assert!(serial.sectors().len() > 1);
    let serial = compile(&serial, &[], EvaluatorBackend::Eager)
        .to_bytes()
        .unwrap();
    let mut session = GenerationSession::new(input.clone(), options.clone());
    while !session.is_complete() {
        let before = session.completed_units();
        session.step(1, |_| ControlFlow::Break(())).unwrap();
        assert_eq!(session.completed_units(), before + 1);
        if !session.is_complete() {
            assert!(session.take_result().is_none());
        }
    }
    let resumed = session.take_result().unwrap();
    assert_eq!(
        serial,
        compile(&resumed, &[], EvaluatorBackend::Eager)
            .to_bytes()
            .unwrap()
    );
    let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = SymbolicJob>| {
        let mut completions = std::thread::scope(|scope| {
            let workers = jobs
                .map(|job| scope.spawn(move || job.run(|_| ControlFlow::Continue(()))))
                .collect::<Vec<_>>();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<Result<Vec<_>, _>>()
        })?;
        completions.reverse();
        Ok(completions)
    };
    let parallel = GenerationContext::new(0)
        .generate_with_all_dispatch(
            &input,
            &options,
            &mut reverse_geometry,
            &mut dispatch,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert_eq!(
        serial,
        compile(&parallel, &[], EvaluatorBackend::Eager)
            .to_bytes()
            .unwrap()
    );
}

#[test]
fn cancellation_and_invalid_completion_admission_never_return_a_partial_integral() {
    let input = dispatch_input();
    let options = GenerationOptions {
        mode: GenerationMode::NumericalDual,
        ..Default::default()
    };
    let cancelled = generate(&input, &options, |event| {
        if matches!(event, GenerationProgress::NumericalMapping { .. }) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    assert!(matches!(cancelled, Err(GenerationError::Cancelled)));
    let mut missing = |_: &mut dyn ExactSizeIterator<Item = SymbolicJob>| Ok(Vec::new());
    assert!(matches!(
        GenerationContext::new(0).generate_with_all_dispatch(
            &input,
            &options,
            &mut reverse_geometry,
            &mut missing,
            || false,
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Invariant(_))
    ));
    let mut foreign = None;
    let mut capture = |jobs: &mut dyn ExactSizeIterator<Item = SymbolicJob>| {
        foreign = Some(
            jobs.map(|job| job.run(|_| ControlFlow::Continue(())))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Err(GenerationError::Cancelled)
    };
    assert!(matches!(
        GenerationContext::new(0).generate_with_all_dispatch(
            &input,
            &options,
            &mut reverse_geometry,
            &mut capture,
            || false,
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Cancelled)
    ));
    let mut stale = |_: &mut dyn ExactSizeIterator<Item = SymbolicJob>| Ok(foreign.take().unwrap());
    assert!(matches!(
        GenerationContext::new(0).generate_with_all_dispatch(
            &input,
            &options,
            &mut reverse_geometry,
            &mut stale,
            || false,
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Invariant(_))
    ));
}

#[test]
fn zero_dimensional_projective_chart_remains_an_exact_contribution() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_zero_dim::x")],
        symbol!("dual_zero_dim::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![Atom::num(-1)],
            vec![],
        )],
    )
    .unwrap();
    let value = generated(
        &input,
        GenerationMode::NumericalDual,
        SubtractionStrategy::Taylor,
        0,
    );
    let mut kernels = compile(&value, &[], EvaluatorBackend::Eager);
    assert!(kernels.sectors().is_empty());
    assert_eq!(kernels.exact_coefficients(), [3.0]);
    assert_eq!(qmc(&mut kernels).mean, [3.0]);
}

#[test]
fn a_large_factored_regular_numerator_stays_compact_in_both_lanes() {
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_factored::x")],
        symbol!("dual_factored::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("(1+dual_factored::x)^10000"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let result = generated(&input, mode, SubtractionStrategy::Taylor, 0);
        let mut kernels = compile(&result, &[], EvaluatorBackend::Eager);
        close(
            &total_complex(&mut kernels, &[1e-6]),
            &[(1.0 + 1e-6_f64).powi(10000), 0.0],
            2e-11,
        );
    }
}

#[test]
fn direct_generated_bytes_lower_deferred_real_and_complex_vectors_without_jit() {
    let settings = CompilationSettings {
        backend: EvaluatorBackend::Eager,
        horner_iterations: 0,
        ..Default::default()
    };
    for complex in [false, true] {
        // Keep the imaginary constant inside an original source polynomial,
        // where inspecting only the formal recipe cannot detect the layout.
        let polynomial = if complex {
            parse!("1+(2+3i)*dual_direct_bytes::x+dual_direct_bytes::x^2")
        } else {
            parse!("1+2*dual_direct_bytes::x+dual_direct_bytes::x^2")
        };
        let input = ParametricIntegrand::new(
            vec![symbol!("dual_direct_bytes::x")],
            symbol!("dual_direct_bytes::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![parse!("-3+dual_direct_bytes::eps")],
                vec![PolynomialFactor::new(
                    polynomial,
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            )],
        )
        .unwrap();
        for strategy in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let value = generated(&input, GenerationMode::NumericalDual, strategy, 1);
            assert!(!value.sectors().is_empty());
            assert!(
                value
                    .sectors()
                    .iter()
                    .all(|sector| { sector.generation_mode() == GenerationMode::NumericalDual })
            );
            // This route builds only native exact IR. Eager policy also ensures
            // decoding and the independent compilation never construct JIT code.
            let bytes = value
                .to_kernel_bytes_with_settings(PrecisionPolicy::default(), settings)
                .unwrap();
            let mut restored = KernelSet::from_bytes(&bytes).unwrap();
            let mut compiled = value
                .compile_with_settings_parameters_and_progress(
                    PrecisionPolicy::default(),
                    &[],
                    settings,
                    |_| ControlFlow::Continue(()),
                )
                .unwrap();
            assert_eq!(restored.to_bytes().unwrap(), bytes);
            assert_eq!(compiled.to_bytes().unwrap(), bytes);
            assert_eq!(restored.components(), compiled.components());
            assert!(
                restored
                    .sectors()
                    .iter()
                    .all(|sector| { sector.statistics().backend == "symbolica_interpreter" })
            );
            let expected: &[f64] = if complex {
                &[1.0, 0.0, -2.5, -3.0, -2.25, -3.0]
            } else {
                &[1.0, 0.0, -2.5, 0.0, -2.25, 0.0]
            };
            assert_eq!(restored.orders(), [-1, -1, 0, 0, 1, 1]);
            for point in [[0.19], [0.73]] {
                close(&total(&mut restored, &point), expected, 2e-12);
                close(&total(&mut compiled, &point), expected, 2e-12);
            }
        }
    }
}
