//! Public dynamic-contour scientific controls, also shared with the portable test crate.
//! No private generation/evaluator admission or numerical replacement is used here.
use fastsecdec::{
    Atom, AtomCore,
    contour::{
        ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction,
    },
    generation::{GenerationMode, GenerationOptions, SubtractionStrategy, generate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet, ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::ResultScope,
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{parse, symbol};

fn endpoint(a: i64) -> ParametricIntegrand {
    let x = symbol!("public_dynamic::x");
    let eps = symbol!("public_dynamic::eps");
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1+𝑖"),
            vec![-Atom::num(a) - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    Atom::one() - 2 * Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

fn bubble() -> ParametricIntegrand {
    let x = symbol!("public_dynamic::bubble_x");
    let eps = symbol!("public_dynamic::bubble_eps");
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() / Atom::var(eps),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::one() - 5 * Atom::var(x) * (Atom::one() - Atom::var(x)),
                    -Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

fn recipe(construction: DynamicConstruction) -> ProgramRecipe {
    match construction {
        DynamicConstruction::Polynomial => ProgramRecipe::DynamicPolynomialV1,
        DynamicConstruction::SignAware => ProgramRecipe::DynamicSignAwareV1,
    }
}

fn restored(
    input: &ParametricIntegrand,
    construction: DynamicConstruction,
    mode: GenerationMode,
    subtraction: SubtractionStrategy,
) -> KernelSet {
    let generated = generate(
        input,
        &GenerationOptions {
            program_recipe: recipe(construction),
            mode,
            subtraction,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let original = generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            horner_iterations: 0,
            ..Default::default()
        })
        .unwrap();
    let bytes = original.to_bytes().unwrap();
    let mut restored = KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(restored.program_recipe(), recipe(construction));
    assert_eq!(restored.content_id(), original.content_id());
    assert_eq!(restored.to_bytes().unwrap(), bytes);
    restored
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: ContourMode::Dynamical {
                    safety_fraction: 0.8,
                    lambda_cap: 1.,
                    displacement_cap: 1.,
                    construction,
                },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 2,
                },
            },
        )
        .unwrap();
    assert!(
        restored
            .validate_integration_readiness(&ResultScope::FullIntegral)
            .is_err()
    );
    for chart in restored.contour_validation_charts() {
        // x=1/2 explicitly reaches the bubble's stationary negative-real F.
        // Native pilots also restrict all retained subtraction-face requests.
        for x in [0.2, 0.5] {
            restored
                .validate_contour_point(chart.chart_index, &vec![x; chart.dimension], true)
                .unwrap();
        }
    }
    let report = restored.finish_contour_pilot().unwrap();
    assert!(report.pilot_complete);
    assert!(report.checked_arguments > 0);
    assert!(report.maximum_bits >= 96);
    restored
        .validate_integration_readiness(&ResultScope::FullIntegral)
        .unwrap();
    restored
}

fn set_policy(kernels: &mut KernelSet, policy: ContourValidation) {
    let identity = kernels.content_id().to_owned();
    kernels
        .set_contour_validation(ContourValidationOptions {
            policy,
            pilot_points: 2,
        })
        .unwrap();
    assert_eq!(kernels.content_id(), identity);
    assert!(kernels.contour_validation_report().unwrap().pilot_complete);
    kernels
        .validate_integration_readiness(&ResultScope::FullIntegral)
        .unwrap();
}

fn integrate_midpoint(kernels: &mut KernelSet, points: usize) -> Vec<f64> {
    let mut sum = kernels.exact_coefficients().to_vec();
    let mut output = vec![0.; sum.len()];
    for (sector_index, sector) in kernels.sectors_mut().iter_mut().enumerate() {
        assert_eq!(sector.dimension(), 1);
        for index in 0..points {
            let point = [(index as f64 + 0.5) / points as f64];
            sector
                .evaluate(&point, &mut output)
                .unwrap_or_else(|error| {
                    panic!(
                        "sector {sector_index}, sample {index}/{points}, point {point:?}: {error}"
                    )
                });
            for (sum, value) in sum.iter_mut().zip(&output) {
                *sum += value / points as f64;
            }
        }
    }
    sum
}

fn assert_vector(
    kernels: &KernelSet,
    values: &[f64],
    expected: impl Fn(i32, CoefficientComponent) -> f64,
    tolerance: f64,
) {
    assert_eq!(values.len(), kernels.orders().len());
    for order in [-1, 0] {
        for part in [CoefficientComponent::Real, CoefficientComponent::Imag] {
            assert!(
                kernels
                    .orders()
                    .iter()
                    .zip(kernels.components())
                    .any(|(o, p)| *o == order && *p == part),
                "missing eps^{order} {part:?}"
            );
        }
    }
    for ((&order, &part), &value) in kernels
        .orders()
        .iter()
        .zip(kernels.components())
        .zip(values)
    {
        let reference = expected(order, part);
        assert!(
            value.is_finite() && (value - reference).abs() < tolerance,
            "eps^{order} {part:?}: {value}, expected {reference}"
        );
    }
}

#[test]
fn native_symbolic_identity_checks_the_endpoint_reference_recurrence() {
    let variable = symbol!("public_dynamic::reference_x");
    let x = Atom::var(variable);
    let f: Atom = Atom::one() - 2 * &x;
    for a in [2_i64, 3] {
        let left: Atom = Atom::one() / (x.pow(a) * &f);
        let right: Atom = Atom::one() / x.pow(a) + 2 / (x.pow(a - 1) * &f);
        assert!((left - right).together().cancel().is_zero());
    }
    let regular: Atom = Atom::one() / (&x * &f) - Atom::one() / &x;
    // The derivative is branch independent; the lower-lip endpoint value of
    // -log(1-2*x-i0) is +i*pi, fixing the causal finite part of J_1.
    let primitive = -f.log();
    assert!(
        (primitive.derivative(variable) - regular)
            .together()
            .cancel()
            .is_zero()
    );
}

#[test]
fn endpoint_poles_complex_numerator_and_higher_jets_through_public_admission() {
    // Pairwise public coverage complements the full internal generation matrix.
    // J_a=1/(1-a-eps)+2 J_(a-1), with J_1=-1/eps+i*pi+O(eps).
    // Thus J_3=-4/eps-5/2+4*i*pi+O(eps), before multiplying by 1+i.
    for (construction, mode, subtraction) in [
        (
            DynamicConstruction::Polynomial,
            GenerationMode::Symbolic,
            SubtractionStrategy::Taylor,
        ),
        (
            DynamicConstruction::Polynomial,
            GenerationMode::NumericalDual,
            SubtractionStrategy::IntegrateByParts,
        ),
        (
            DynamicConstruction::SignAware,
            GenerationMode::Symbolic,
            SubtractionStrategy::IntegrateByParts,
        ),
        (
            DynamicConstruction::SignAware,
            GenerationMode::NumericalDual,
            SubtractionStrategy::Taylor,
        ),
    ] {
        for a in [1, 3] {
            eprintln!("endpoint control: {construction:?}, {mode:?}, {subtraction:?}, a={a}");
            let mut kernels = restored(&endpoint(a), construction, mode, subtraction);
            set_policy(&mut kernels, ContourValidation::Off);
            let values = integrate_midpoint(&mut kernels, 8192);
            let pole = if a == 1 { -1. } else { -4. };
            let finite = if a == 1 { 0. } else { -2.5 };
            let imaginary = if a == 1 {
                std::f64::consts::PI
            } else {
                4. * std::f64::consts::PI
            };
            assert_vector(
                &kernels,
                &values,
                |order, part| match (order, part) {
                    (-1, _) => pole,
                    (0, CoefficientComponent::Real) => finite - imaginary,
                    (0, CoefficientComponent::Imag) => finite + imaginary,
                    _ => 0.,
                },
                if a == 1 { 2e-5 } else { 1e-4 },
            );
        }
    }
}

#[test]
fn threshold_bubble_policies_preserve_values_identity_and_stationary_branch() {
    for (construction, mode) in [
        (DynamicConstruction::Polynomial, GenerationMode::Symbolic),
        (
            DynamicConstruction::SignAware,
            GenerationMode::NumericalDual,
        ),
    ] {
        let mut kernels = restored(&bubble(), construction, mode, SubtractionStrategy::Taylor);
        let mut expected = None;
        for policy in [
            ContourValidation::Always,
            ContourValidation::Pilot,
            ContourValidation::Off,
        ] {
            set_policy(&mut kernels, policy);
            let before = kernels
                .contour_validation_report()
                .unwrap()
                .production_checked_arguments;
            let width = kernels.orders().len();
            let mut values = Vec::new();
            for sector in kernels.sectors_mut() {
                let mut output = vec![0.; width];
                assert_eq!(sector.dimension(), 1);
                sector.evaluate(&[0.5], &mut output).unwrap();
                values.extend(output);
            }
            if let Some(reference) = &expected {
                assert_eq!(&values, reference);
            } else {
                expected = Some(values);
            }
            let report = kernels.contour_validation_report().unwrap();
            if policy == ContourValidation::Always {
                assert!(report.production_checked_arguments > before);
                assert!(report.production_maximum_bits >= 96);
            } else {
                assert_eq!(
                    report.production_checked_arguments, before,
                    "unchecked production retained causal-check work"
                );
            }
        }
        let values = integrate_midpoint(&mut kernels, 8192);
        let beta = 0.2_f64.sqrt();
        let real = 2. - beta * ((1. + beta) / (1. - beta)).ln();
        assert_vector(
            &kernels,
            &values,
            |order, part| match (order, part) {
                (-1, CoefficientComponent::Real) => 1.,
                (0, CoefficientComponent::Real) => real,
                (0, CoefficientComponent::Imag) => std::f64::consts::PI * beta,
                _ => 0.,
            },
            2e-6,
        );
    }
}
