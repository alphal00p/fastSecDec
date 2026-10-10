use super::*;
use crate::contour::{ContourDiagnosticsMode, functions::dynamic::diagnostics::Configuration};

#[test]
fn aggregate_retains_discarded_matrix_and_native_rescue_work() {
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
        let (helper, x, exact) = fixture();
        let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
        let configuration = Configuration {
            mode: ContourDiagnosticsMode::Aggregate,
            displacement_cap: 1.,
        };
        let _mode = configuration.enter();
        let mut kernels = KernelSet::from_programs_for_load(
            vec![-1, 0],
            vec![SectorProgram {
                symbolic_endpoint_contour_partials: None,
                parameters: vec![x],
                runtime_parameters: vec![],
                exact,
                cancellation: Cancellation::new(0, None, 1).unwrap(),
                exact_zero: vec![false; 2],
                real_coefficients: vec![true; 2],
            }],
            vec![Atom::Zero; 2],
            PrecisionPolicy::default(),
            None,
            false,
            vec![],
            CompilationSettings {
                backend,
                ..Default::default()
            },
        )
        .unwrap();
        kernels
            .set_stability_settings(&StabilitySettings::validated())
            .unwrap();
        let mut context = kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap();
        let mut output = vec![0.; 6];
        let rescued = context
            .evaluate_weighted_batch(&[0.1, 1., 0.2], &[1.; 3], &mut output)
            .unwrap();
        assert_eq!(output, [1., 2., 1., 2., 1., 2.]);
        assert!(rescued.iter().all(|report| report.precision.rescued));
        let report = context.take_contour_runtime_report().unwrap().unwrap();
        assert!(
            report.evaluation.callback_calls > 3,
            "discarded provisional rows and complete retries must both be observed"
        );
        assert!(report.evaluation.callback_failures >= 1);
        assert!(report.evaluation.maximum_bits > 53);
        assert!(context.contour_runtime_report().unwrap().is_none());
        let healthy = context
            .evaluate_weighted_batch(&[0.1, 0.2, 0.3], &[1.; 3], &mut output)
            .unwrap();
        assert!(healthy.iter().all(|report| !report.precision.rescued));
        assert_eq!(output, [1., 2., 1., 2., 1., 2.]);
        let report = context.take_contour_runtime_report().unwrap().unwrap();
        assert_eq!(report.evaluation.callback_calls, 3);
        assert_eq!(report.evaluation.callback_failures, 0);
        assert!(dynamic::take_failure().is_none());
    }
}
