use super::*;
use crate::contour::{
    ContourDiagnosticsMode,
    functions::dynamic::{
        self, ProgramScope, RootProgram,
        diagnostics::{Accumulator, Configuration, Phase},
    },
};
use symbolica::{atom::AtomCore, function};

#[test]
fn aggregate_mode_is_operational_atomic_and_clone_local() {
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        let (mut kernels, p, mut settings) = fixture(backend);
        settings.validation.policy = ContourValidation::Off;
        kernels.bind_dynamic(&point(p, 1.), &settings).unwrap();
        let identity = kernels.content_id().to_owned();
        let bytes = kernels.artifact_bytes().unwrap().to_vec();
        let baseline = sample(&mut kernels);
        assert_eq!(
            kernels.contour_diagnostics_mode(),
            ContourDiagnosticsMode::Disabled
        );
        assert!(kernels.contour_runtime_report().unwrap().is_none());
        kernels
            .set_contour_diagnostics(ContourDiagnosticsMode::Aggregate)
            .unwrap();
        assert_eq!(sample(&mut kernels), baseline);
        let first = kernels.contour_runtime_report().unwrap().unwrap();
        assert!(first.evaluation.callback_calls > 0);
        assert_eq!(
            first.evaluation.closed_form_calls,
            first.evaluation.callback_calls
        );
        assert_eq!(
            first.evaluation.correction_evaluations,
            first.evaluation.callback_calls
        );
        assert_eq!(first.evaluation.solver_calls, 0);
        assert_eq!(first.evaluation.callback_failures, 0);
        let mut cloned = kernels.try_clone().unwrap();
        assert_eq!(
            cloned.contour_diagnostics_mode(),
            ContourDiagnosticsMode::Aggregate
        );
        assert!(cloned.contour_runtime_report().unwrap().is_none());
        assert_eq!(sample(&mut cloned), baseline);
        assert_eq!(kernels.contour_runtime_report().unwrap().unwrap(), first);
        kernels
            .set_contour_diagnostics(ContourDiagnosticsMode::Disabled)
            .unwrap();
        let disabled = kernels.contour_runtime_report().unwrap().unwrap();
        assert_eq!(sample(&mut kernels), baseline);
        assert_eq!(kernels.contour_runtime_report().unwrap().unwrap(), disabled);
        kernels
            .set_contour_diagnostics(ContourDiagnosticsMode::Aggregate)
            .unwrap();
        // The independent exact 1/p offset fails binding after factory setup.
        // Operational history survives, while the old mathematical owner stays usable.
        assert!(kernels.bind_dynamic(&point(p, 0.), &settings).is_err());
        assert_eq!(kernels.content_id(), identity);
        assert_eq!(sample(&mut kernels), baseline);
        assert_eq!(kernels.artifact_bytes().unwrap(), bytes);
        let before = kernels.contour_runtime_report().unwrap().unwrap();
        assert!(before.evaluation.callback_calls > first.evaluation.callback_calls);
        assert_eq!(kernels.take_contour_runtime_report().unwrap(), Some(before));
        assert!(kernels.contour_runtime_report().unwrap().is_none());
        assert_eq!(sample(&mut kernels), baseline);
        assert!(
            kernels
                .contour_runtime_report()
                .unwrap()
                .unwrap()
                .evaluation
                .callback_calls
                > 0
        );
    }
}

#[test]
fn actual_pilot_and_policy_remaps_keep_work_out_of_production() {
    let (mut kernels, p, settings) = fixture(EvaluatorBackend::Eager);
    kernels
        .set_contour_diagnostics(ContourDiagnosticsMode::Aggregate)
        .unwrap();
    kernels.bind_dynamic(&point(p, 1.), &settings).unwrap();
    let chart = kernels.contour_validation_charts().remove(0);
    for x in [0.2, 0.6] {
        kernels
            .validate_contour_point(chart.chart_index, &[x], true)
            .unwrap();
        let report = kernels.contour_runtime_report().unwrap().unwrap();
        assert_eq!(report.evaluation.callback_calls, 0);
        assert!(report.pilot.callback_calls > 0);
    }
    let pilot = kernels.contour_runtime_report().unwrap().unwrap().pilot;
    kernels.finish_contour_pilot().unwrap();
    assert_eq!(
        kernels.contour_runtime_report().unwrap().unwrap().pilot,
        pilot
    );
    let baseline = sample(&mut kernels);
    let identity = kernels.content_id().to_owned();
    let mut count = kernels
        .contour_runtime_report()
        .unwrap()
        .unwrap()
        .evaluation
        .callback_calls;
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
        let actual = sample(&mut kernels);
        for (a, b) in actual.iter().zip(&baseline) {
            assert!((a - b).abs() < 1e-11 * (1. + b.abs()));
        }
        let report = kernels.contour_runtime_report().unwrap().unwrap();
        assert!(report.evaluation.callback_calls > count);
        assert_eq!(report.pilot, pilot);
        count = report.evaluation.callback_calls;
        assert_eq!(kernels.content_id(), identity);
        assert!(kernels.contour_validation_report().unwrap().pilot_complete);
    }
    let mut worker = kernels.evaluation_context(0, Default::default()).unwrap();
    assert!(worker.contour_runtime_report().unwrap().is_none());
    let mut output = vec![0.; worker.output_count()];
    worker
        .evaluate_weighted_batch(&[0.37], &[1.], &mut output)
        .unwrap();
    assert!(
        worker
            .take_contour_runtime_report()
            .unwrap()
            .unwrap()
            .evaluation
            .callback_calls
            > 0
    );
    assert!(worker.contour_runtime_report().unwrap().is_none());
}

#[test]
fn exact_aggregate_counts_native_executed_cache_only_and_all_retries() {
    let helper = RootProgram::build(1).unwrap();
    let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let q = symbolica::symbol!("exact_diagnostics::q");
    let root =
        dynamic::strength(&helper, &[Atom::var(q)], &Atom::num((4, 5)), &Atom::one()).unwrap();
    let invalid =
        dynamic::strength(&helper, &[Atom::num(-1)], &Atom::num((4, 5)), &Atom::one()).unwrap();
    let condition = symbolica::symbol!("exact_diagnostics::condition");
    let expressions = [
        function!(Symbol::IF, Atom::var(condition), invalid, root.clone()),
        &root * 2,
    ];
    let config = Configuration {
        mode: ContourDiagnosticsMode::Aggregate,
        displacement_cap: 1.,
    };
    let _mode = config.enter();
    let mut history = Accumulator::default();
    let actual = history
        .measure(config, Phase::Exact, || {
            crate::kernel::exact::evaluate(
                &expressions,
                &BTreeMap::from([(q, 4.), (condition, 0.)]),
                false,
            )
        })
        .unwrap();
    assert_eq!(actual, [0.4, 0.8]);
    let report = history.take().unwrap().unwrap();
    assert_eq!(report.exact.callback_calls, 1);
    assert_eq!(report.exact.callback_failures, 0);
    // Coefficient exp(800) overflows in f64/DD but remains finite natively in MP.
    let rescue = dynamic::strength(
        &helper,
        &[(Atom::var(q) * 800).exp()],
        &Atom::num((4, 5)),
        &Atom::one(),
    )
    .unwrap();
    let value = history
        .measure(config, Phase::Exact, || {
            crate::kernel::exact::evaluate(&[rescue], &BTreeMap::from([(q, 1.)]), false)
        })
        .unwrap();
    assert!(value[0].is_finite() && value[0] > 0.);
    let report = history.take().unwrap().unwrap();
    assert!(report.exact.callback_failures >= 1);
    assert!(report.exact.callback_calls > report.exact.callback_failures);
    assert!(report.exact.maximum_bits > 53);
    assert_eq!(report.evaluation.callback_calls, 0);
    assert!(dynamic::take_failure().is_none());
}
