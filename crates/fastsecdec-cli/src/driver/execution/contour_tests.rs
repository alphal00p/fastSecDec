use super::*;
use fastsecdec::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, ReplayPolicy},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    status::IntegrationStage,
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::atomic::AtomicBool};
use symbolica::{atom::Atom, symbol};

#[test]
fn worker_contour_deltas_do_not_duplicate_resident_checks_or_count_adaptation_as_production() {
    let x = symbol!("contour_worker_counter_test::x");
    let eps = symbol!("contour_worker_counter_test::eps");
    let variable = Atom::var(x);
    let source = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::one() - 5 * &variable * (Atom::one() - &variable),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let template = generate(
        &source,
        &GenerationOptions {
            contour: true,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile_with_settings(CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    })
    .unwrap();
    let mut expected = None;
    for policy in [
        ContourValidation::Always,
        ContourValidation::Pilot,
        ContourValidation::Off,
    ] {
        let mut kernels = template.try_clone().unwrap();
        kernels
            .bind_parameters_with_contour(
                &BTreeMap::new(),
                &ContourSettings {
                    deformation: ContourMode::Fixed { lambda: 0.2 },
                    validation: ContourValidationOptions {
                        policy,
                        pilot_points: 1,
                    },
                },
            )
            .unwrap();
        if policy != ContourValidation::Off {
            kernels.validate_contour_point(0, &[0.5], true).unwrap();
            kernels.finish_contour_pilot().unwrap();
        }
        let mut context = kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap();
        let mut diagnostics = EvaluationDiagnostics::default();
        let meter = observations::WorkerMeter::default();
        let stop = AtomicBool::new(false);
        let mut values = Vec::new();
        for (stage, points) in [
            (IntegrationStage::Pilot, vec![0.2, 0.3]),
            (IntegrationStage::Production, vec![0.4, 0.6, 0.7]),
        ] {
            let mut output = vec![f64::NAN; points.len() * context.output_count()];
            let mut aborted = false;
            evaluate_batch_observed(
                &mut context,
                0,
                stage,
                &points,
                &vec![1.; points.len()],
                &mut output,
                &mut diagnostics,
                &meter,
                &stop,
                &mut aborted,
            )
            .unwrap();
            assert!(!aborted);
            values.extend(output);
        }
        if let Some(expected) = &expected {
            assert_eq!(&values, expected);
        } else {
            expected = Some(values);
        }
        match policy {
            ContourValidation::Always => {
                let counters = diagnostics.contour.unwrap();
                assert_eq!(counters.adaptation.checked_arguments, 2);
                assert_eq!(counters.production.checked_arguments, 3);
                assert_eq!(counters.production.maximum_bits, 96);
                assert_eq!(
                    context
                        .take_contour_validation_report()
                        .unwrap()
                        .checked_arguments,
                    0
                );
            }
            ContourValidation::Pilot => {
                assert_eq!(diagnostics.contour.unwrap(), Default::default())
            }
            ContourValidation::Off => assert!(diagnostics.contour.is_none()),
        }
    }
}
