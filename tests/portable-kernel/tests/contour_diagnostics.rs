//! Optional observations use the portable native arithmetic owners unchanged.
use fastsecdec::{
    Atom,
    contour::{
        ContourDiagnosticsMode, ContourMode, ContourSettings, ContourValidation,
        ContourValidationOptions, DynamicConstruction,
    },
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::symbol;

#[test]
fn portable_diagnostics_preserve_cubic_vectors_and_native_solver_work() {
    let [x, y, z, eps] = [
        symbol!("portable_diagnostics::x"),
        symbol!("portable_diagnostics::y"),
        symbol!("portable_diagnostics::z"),
        symbol!("portable_diagnostics::eps"),
    ];
    let input = ParametricIntegrand::new(
        vec![x, y, z],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() / Atom::var(eps),
            vec![Atom::Zero; 3],
            vec![
                PolynomialFactor::new(
                    (1 - 2 * Atom::var(x)) * (1 + Atom::var(y)) * (1 + Atom::var(z)),
                    -Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    for construction in [
        DynamicConstruction::Polynomial,
        DynamicConstruction::SignAware,
    ] {
        let settings = ContourSettings {
            deformation: ContourMode::Dynamical {
                safety_fraction: 0.8,
                lambda_cap: 1.,
                displacement_cap: 2.,
                construction,
            },
            validation: ContourValidationOptions {
                policy: ContourValidation::Pilot,
                pilot_points: 1,
            },
        };
        let generated = generate(
            &input,
            &GenerationOptions {
                program_recipe: settings.deformation.program_recipe(),
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut kernels = generated
            .compile_with_settings(CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            })
            .unwrap();
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &settings)
            .unwrap();
        for chart in kernels.contour_validation_charts() {
            kernels
                .validate_contour_point(chart.chart_index, &[0.25, 0.4, 0.6], true)
                .unwrap();
        }
        assert!(kernels.finish_contour_pilot().unwrap().pilot_complete);
        kernels
            .set_contour_validation(ContourValidationOptions {
                policy: ContourValidation::Off,
                pilot_points: 1,
            })
            .unwrap();
        let identity = kernels.content_id().to_owned();
        let bytes = kernels.artifact_bytes().unwrap().to_vec();
        let mut plain = kernels.evaluation_context(0, Default::default()).unwrap();
        let point = [0.23, 0.37, 0.61];
        let mut expected = vec![0.; plain.output_count()];
        plain
            .evaluate_weighted_batch(&point, &[1.], &mut expected)
            .unwrap();
        assert!(plain.contour_runtime_report().unwrap().is_none());
        kernels
            .set_contour_diagnostics(ContourDiagnosticsMode::Aggregate)
            .unwrap();
        let mut observed = kernels.evaluation_context(0, Default::default()).unwrap();
        let mut actual = vec![0.; observed.output_count()];
        observed
            .evaluate_weighted_batch(&point, &[1.], &mut actual)
            .unwrap();
        assert_eq!(actual, expected);
        let report = observed.take_contour_runtime_report().unwrap().unwrap();
        assert!(report.evaluation.callback_calls > 0);
        assert_eq!(
            report.evaluation.solver_calls,
            report.evaluation.callback_calls
        );
        assert_eq!(report.evaluation.solver_failures, 0);
        assert!(report.evaluation.solver_iterations > 0);
        assert!(report.evaluation.solver_evaluations >= report.evaluation.solver_iterations);
        assert_eq!(
            report.evaluation.strength.count,
            report.evaluation.callback_calls
        );
        assert!(report.evaluation.normalized_displacement.minimum.unwrap() > 0.);
        assert_eq!(
            report.evaluation.physical_displacement.minimum.unwrap(),
            2. * report.evaluation.normalized_displacement.minimum.unwrap()
        );
        assert!(observed.contour_runtime_report().unwrap().is_none());
        assert_eq!(kernels.content_id(), identity);
        assert_eq!(kernels.artifact_bytes().unwrap(), bytes);
        let clone = kernels.try_clone().unwrap();
        assert_eq!(
            clone.contour_diagnostics_mode(),
            ContourDiagnosticsMode::Aggregate
        );
        assert!(clone.contour_runtime_report().unwrap().is_none());
    }
}
