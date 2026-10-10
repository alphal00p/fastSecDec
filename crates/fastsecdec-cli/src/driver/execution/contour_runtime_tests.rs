use super::*;
use fastsecdec::{
    Atom, AtomCore,
    contour::{ContourDiagnosticsMode, ContourMode, ContourSettings, ContourValidation},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, ProgramRecipe, ReplayPolicy},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    status::IntegrationStage,
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::atomic::AtomicBool};
use symbolica::symbol;

fn template(backend: EvaluatorBackend) -> KernelSet {
    let x = symbol!("cli_runtime_work::x");
    let source = ParametricIntegrand::new(
        vec![x],
        symbol!("cli_runtime_work::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    (Atom::var(x) - Atom::num((1, 2))).pow(2),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    generate(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicSignAwareV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend,
        ..Default::default()
    })
    .unwrap()
}

#[test]
fn runtime_deltas_preserve_failed_matrix_work_and_never_recount_a_drain() {
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
        let template = template(backend);
        let mut expected = None;
        for mode in [
            ContourDiagnosticsMode::Disabled,
            ContourDiagnosticsMode::Aggregate,
        ] {
            let mut kernels = template.try_clone().unwrap();
            kernels.set_contour_diagnostics(mode).unwrap();
            kernels
                .bind_parameters_with_contour(
                    &BTreeMap::new(),
                    &ContourSettings {
                        deformation: ContourMode::dynamical(0.8),
                        validation: fastsecdec::contour::ContourValidationOptions {
                            policy: ContourValidation::Off,
                            pilot_points: 2,
                        },
                    },
                )
                .unwrap();
            let mut context = kernels
                .evaluation_context(0, ReplayPolicy::default())
                .unwrap();
            let operations =
                observations::Operations::new(1, kernels.orders(), std::time::Duration::ZERO);
            let meter = operations.worker(0);
            let mut diagnostics = EvaluationDiagnostics::default();
            let stop = AtomicBool::new(false);
            let mut aborted = false;
            let mut output = vec![0.; 2 * context.output_count()];
            evaluate_batch_observed(
                &mut context,
                0,
                IntegrationStage::Pilot,
                &[0.2, 0.3],
                &[1.; 2],
                &mut output,
                &mut diagnostics,
                &meter,
                &stop,
                &mut aborted,
            )
            .unwrap();
            if let Some(expected) = &expected {
                assert_eq!(&output, expected);
            } else {
                expected = Some(output.clone());
            }
            let adaptation = diagnostics
                .contour_runtime
                .as_ref()
                .map(|r| r.adaptation.clone());
            output.resize(3 * context.output_count(), 0.);
            assert!(
                evaluate_batch_observed(
                    &mut context,
                    0,
                    IntegrationStage::Production,
                    &[0.2, 0.5, 0.3],
                    &[1.; 3],
                    &mut output,
                    &mut diagnostics,
                    &meter,
                    &stop,
                    &mut aborted
                )
                .is_err()
            );
            assert!(!aborted);
            assert_eq!(diagnostics.failures, 1);
            assert!(context.take_contour_runtime_report().unwrap().is_none());
            if mode == ContourDiagnosticsMode::Aggregate {
                let report = diagnostics.contour_runtime.as_ref().unwrap();
                assert_eq!(Some(&report.adaptation), adaptation.as_ref());
                assert!(report.adaptation.evaluation.callback_calls > 0);
                assert!(report.production.evaluation.callback_calls >= 2);
                assert_eq!(report.production.pilot.callback_calls, 0);
                assert_eq!(
                    operations.snapshot().unwrap().diagnostics.contour_runtime,
                    diagnostics.contour_runtime
                );
            } else {
                assert!(diagnostics.contour_runtime.is_none());
            }
        }
    }
}
