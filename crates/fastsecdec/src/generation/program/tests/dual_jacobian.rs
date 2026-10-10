//! Real endpoint recipes exercise the composed Jacobian before native outer jets.
use super::*;
use crate::{
    contour::{
        ContourJacobian, ContourMode, ContourSettings, ContourValidation, ContourValidationOptions,
        DynamicConstruction, functions::dynamic,
    },
    generation::{GeneratedIntegral, numerical_dual::native},
    kernel::{CompilationSettings, EvaluatorBackend, KernelSet},
};
use std::collections::BTreeMap;
use symbolica::domains::{
    float::{Float, RealLike},
    rational::Rational,
};

fn source() -> ParametricIntegrand {
    let input = super::cubic::source();
    ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![ParametricTerm::new(
            Atom::one() + Atom::i(),
            vec![Atom::num(-3) - Atom::var(input.regulator())],
            input.terms()[0].factors().to_vec(),
        )],
    )
    .unwrap()
}

fn values(generated: &GeneratedIntegral, recipe: ProgramRecipe) -> Vec<Vec<f64>> {
    let runtime = recipe
        .recipe_parameters()
        .iter()
        .map(|name| symbol!(*name))
        .collect::<Vec<_>>();
    let point = |x| {
        std::iter::once(x)
            .chain(runtime.iter().map(|s| {
                if *s == crate::contour::dynamic::safety_fraction_symbol() {
                    0.8
                } else if *s == crate::contour::dynamic::displacement_cap_symbol() {
                    1.
                } else {
                    0.125
                }
            }))
            .map(|v| Complex::new(Float::with_val(192, v), Float::with_val(192, 0)))
            .collect::<Vec<_>>()
    };
    assert_eq!(generated.sectors().len(), 1);
    assert!(
        generated
            .exact_coefficients()
            .iter()
            .all(|value| value.is_zero())
    );
    let sector = &generated.sectors()[0];
    let exact = {
        let _owner = generated.program_descriptor().map(|owner| owner.enter());
        native::build(
            sector.deferred.as_ref().unwrap(),
            &runtime,
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
        )
        .unwrap()
    };
    let mut evaluator = {
        let _owner = generated.program_descriptor().map(|owner| owner.enter());
        dynamic::with_precision(192, || {
            exact.map_coeff_with_prec(
                &|c: &Complex<Rational>| {
                    Complex::new(c.re.to_multi_prec_float(192), c.im.to_multi_prec_float(192))
                },
                192,
            )
        })
    };
    [0.13, 0.37, 0.71]
        .into_iter()
        .map(|x| {
            let mut output = vec![
                Complex::new(Float::with_val(192, 0), Float::with_val(192, 0));
                generated.orders().len()
            ];
            let (_, failure) =
                dynamic::isolated_attempt(|| evaluator.evaluate(&point(x), &mut output));
            assert!(failure.is_none(), "{recipe:?} {failure:?}");
            output
                .into_iter()
                .flat_map(|v| [v.re.to_f64(), v.im.to_f64()])
                .collect()
        })
        .collect()
}

fn restore_and_bind(bytes: &[u8], recipe: ProgramRecipe) -> KernelSet {
    let mut kernels = KernelSet::from_bytes(bytes).unwrap();
    let deformation = match recipe {
        ProgramRecipe::FixedV1 => ContourMode::Fixed { lambda: 0.125 },
        _ => ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 0.125,
            displacement_cap: 1.,
            construction: if recipe == ProgramRecipe::DynamicPolynomialV1 {
                DynamicConstruction::Polynomial
            } else {
                DynamicConstruction::SignAware
            },
        },
    };
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation,
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 1,
                },
            },
        )
        .unwrap();
    for chart in kernels.contour_validation_charts() {
        kernels
            .validate_contour_point(chart.chart_index, &[0.37], true)
            .unwrap();
    }
    kernels.finish_contour_pilot().unwrap();
    kernels
        .validate_integration_readiness(&crate::results::ResultScope::FullIntegral)
        .unwrap();
    kernels
}

#[test]
fn dual_jacobian_complete_higher_jet_vectors_restore_with_actual_checked_owners() {
    for recipe in [
        ProgramRecipe::FixedV1,
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let options = GenerationOptions {
                program_recipe: recipe,
                mode: GenerationMode::NumericalDual,
                subtraction,
                ..Default::default()
            };
            let reference = generate(&source(), &options, |_| ControlFlow::Continue(())).unwrap();
            let expected = values(&reference, recipe);
            let generated = generate(
                &source(),
                &GenerationOptions {
                    contour_jacobian: ContourJacobian::Dual,
                    ..options
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.contour_jacobian(), ContourJacobian::Dual);
            assert_eq!(generated.orders(), &[-1, 0]);
            assert!(
                generated.sectors()[0]
                    .deferred
                    .as_ref()
                    .unwrap()
                    .recipe
                    .requests
                    .iter()
                    .any(|request| request.derivatives[0] >= 2)
            );
            let actual = values(&generated, recipe);
            for (a, b) in actual.iter().flatten().zip(expected.iter().flatten()) {
                assert!(
                    a.is_finite() && b.is_finite() && (a - b).abs() < 2e-12 * (1. + b.abs()),
                    "{recipe:?}/{subtraction:?}: {a} vs {b}"
                );
            }
            let mut backends = vec![EvaluatorBackend::Eager];
            if cfg!(feature = "native") {
                backends.push(EvaluatorBackend::Symjit);
            }
            for backend in backends {
                let original = generated
                    .compile_with_settings(CompilationSettings {
                        backend,
                        ..Default::default()
                    })
                    .unwrap();
                let bytes = original.to_bytes().unwrap();
                drop(original);
                let mut restored = restore_and_bind(&bytes, recipe);
                assert_eq!(restored.orders(), &[-1, -1, 0, 0]);
                for (x, expected) in [0.13, 0.37, 0.71].into_iter().zip(&expected) {
                    let mut value = vec![0.; expected.len()];
                    restored.sectors_mut()[0]
                        .evaluate(&[x], &mut value)
                        .unwrap();
                    assert_eq!(value.len(), expected.len());
                    for (a, b) in value.iter().zip(expected) {
                        assert!(
                            (a - b).abs() < 2e-9 * (1. + b.abs()),
                            "{recipe:?}/{subtraction:?}/{backend:?}: {a} vs {b}"
                        );
                    }
                }
            }
            assert!(generated.sectors()[0].materialized.get().is_none());
        }
    }
}
