//! Exercise the public contour API with the portable numeric/evaluator owners.
use fastsecdec::{
    Atom,
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{GenerationMode, GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{parse, symbol};

fn bubble() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("portable_contour::x")],
        symbol!("portable_contour::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/portable_contour::eps"),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    parse!("1-5*portable_contour::x*(1-portable_contour::x)"),
                    parse!("-portable_contour::eps"),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn threshold_bubble_preserves_complex_coefficients_and_certified_pilot_after_restore() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let generated = generate(
            &bubble(),
            &GenerationOptions {
                contour: true,
                mode,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let original = generated
            .compile_with_settings_parameters_and_progress(
                Default::default(),
                &[],
                CompilationSettings {
                    backend: EvaluatorBackend::Eager,
                    horner_iterations: 0,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        let bytes = original.to_bytes().unwrap();
        let mut restored = KernelSet::from_bytes(&bytes).unwrap();
        assert!(restored.contour_capable());
        assert_eq!(original.content_id(), restored.content_id());
        assert_eq!(restored.to_bytes().unwrap(), bytes);
        restored
            .bind_parameters_with_contour(
                &BTreeMap::new(),
                &ContourSettings {
                    deformation: ContourMode::Fixed { lambda: 0.2 },
                    validation: ContourValidationOptions {
                        policy: ContourValidation::Always,
                        pilot_points: 2,
                    },
                },
            )
            .unwrap();
        // Pilot points are explicit caller work. In particular x=1/2 tests
        // the stationary negative F and its causal lower-lip logarithm.
        for chart in restored.contour_validation_charts() {
            for x in [0.2, 0.5] {
                let check = restored
                    .validate_contour_point(chart.chart_index, &[x], true)
                    .unwrap();
                assert!(check.maximum_bits >= 96);
                assert!(check.checked_arguments > 0);
            }
        }
        assert!(restored.finish_contour_pilot().unwrap().pilot_complete);
        let width = restored.orders().len();
        let mut checked = vec![0.0; width];
        restored.sectors_mut()[0]
            .evaluate(&[0.5], &mut checked)
            .unwrap();
        let report = restored.sectors()[0].contour_validation_report().unwrap();
        assert!(report.checked_arguments > 0);
        assert!(report.maximum_bits >= 96);
        let identity = restored.content_id().to_owned();
        restored
            .set_contour_validation(ContourValidationOptions {
                policy: ContourValidation::Off,
                pilot_points: 2,
            })
            .unwrap();
        assert_eq!(restored.content_id(), identity);
        assert!(restored.contour_validation_report().unwrap().pilot_complete);
        let mut unchecked = vec![0.0; width];
        restored.sectors_mut()[0]
            .evaluate(&[0.5], &mut unchecked)
            .unwrap();
        assert_eq!(checked, unchecked);

        // Deterministic quadrature tests the whole complex Laurent vector,
        // including the exact UV residue, without adding a sampling engine.
        let layout = restored
            .orders()
            .iter()
            .copied()
            .zip(restored.components().iter().copied())
            .collect::<Vec<_>>();
        let mut coefficients = restored.exact_coefficients().to_vec();
        let points = 8192;
        let mut output = vec![0.0; width];
        for sector in restored.sectors_mut() {
            assert_eq!(sector.dimension(), 1);
            for index in 0..points {
                sector
                    .evaluate(&[(index as f64 + 0.5) / points as f64], &mut output)
                    .unwrap();
                for (sum, value) in coefficients.iter_mut().zip(&output) {
                    *sum += value / points as f64;
                }
            }
        }
        let coefficient = |order, component| {
            coefficients[layout
                .iter()
                .position(|item| *item == (order, component))
                .unwrap()]
        };
        let beta = 0.2_f64.sqrt();
        let expected_real = 2.0 - beta * ((1.0 + beta) / (1.0 - beta)).ln();
        assert!((coefficient(-1, CoefficientComponent::Real) - 1.0).abs() < 2e-6);
        assert!(coefficient(-1, CoefficientComponent::Imag).abs() < 2e-6);
        assert!((coefficient(0, CoefficientComponent::Real) - expected_real).abs() < 2e-6);
        assert!(
            (coefficient(0, CoefficientComponent::Imag) - std::f64::consts::PI * beta).abs() < 2e-6
        );
    }
}
