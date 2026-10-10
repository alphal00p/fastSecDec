use super::*;
use crate::generation::GeneratedIntegral;
use crate::kernel::{CompilationSettings, EvaluatorBackend};
use symbolica::domains::{
    float::{DoubleFloat, RealLike},
    rational::Rational,
};

pub(super) fn source() -> ParametricIntegrand {
    let x = symbol!("dynamic_cubic_gate::x");
    let eps = symbol!("dynamic_cubic_gate::eps");
    let positive = Atom::one() + Atom::var(x).pow(2);
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    (Atom::num((1, 4)) - Atom::var(x)) * &positive,
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                // The original density is unchanged, but the full-sector
                // radius must retain this declared positive-factor bound.
                PolynomialFactor::new(positive, Atom::Zero, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn cubic_reference_uses_native_exact_partial_fractions() {
    let input = source();
    let x = Atom::var(input.parameters()[0]);
    let f = input.terms()[0].factors()[0].polynomial();
    let original = Atom::one() / (&x * f);
    let decomposed = original.apart(input.parameters()[0]);
    let reference = Atom::num(4) / &x
        + Atom::num((64, 17)) / (Atom::num((1, 4)) - &x)
        + (Atom::num((-4, 17)) * &x + Atom::num((16, 17))) / (Atom::one() + x.pow(2));
    assert!((decomposed - &reference).together().cancel().is_zero());
    assert!((&original - reference).together().cancel().is_zero());
    let arctangent = ((Atom::one() + Atom::i() * &x).log() - (Atom::one() - Atom::i() * &x).log())
        / (Atom::num(2) * Atom::i());
    let primitive = -Atom::num((64, 17))
        * crate::contour::functions::causal_log(&(Atom::num((1, 4)) - &x))
        - Atom::num((2, 17)) * (Atom::one() + x.pow(2)).log()
        + Atom::num((16, 17)) * arctangent;
    assert!(
        (primitive.derivative(input.parameters()[0]) - (original - Atom::num(4) / &x))
            .together()
            .cancel()
            .is_zero()
    );
    let mut evaluator = primitive
        .evaluator(&[x])
        .build()
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let boundary = evaluator.evaluate_single(&[Complex::new(1., 0.)])
        - evaluator.evaluate_single(&[Complex::new(0., 0.)]);
    let pi = std::f64::consts::PI;
    let expected_real = -64. / 17. * 3f64.ln() - 2. / 17. * 2f64.ln() + 4. * pi / 17.;
    assert!((boundary.re - expected_real).abs() < 1e-12);
    assert!((boundary.im - 64. * pi / 17.).abs() < 1e-12);
}

/// Deterministic midpoint controls use identical nodes for all mathematical
/// recipes and one native vector evaluator. Deferred sectors execute native
/// Dualizer jets without materializing their coefficient expressions.
pub(super) fn integrate(generated: &GeneratedIntegral, recipe: ProgramRecipe) -> Vec<Complex<f64>> {
    let runtime = recipe
        .recipe_parameters()
        .iter()
        .map(|name| symbol!(*name))
        .collect::<Vec<_>>();
    let runtime_atoms = runtime.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
    let number = |value: f64| Complex::new(DoubleFloat::from(value), DoubleFloat::from(0.));
    let map =
        |c: &Complex<Rational>| Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im));
    let values = runtime
        .iter()
        .map(|s| {
            number(if *s == crate::contour::dynamic::safety_fraction_symbol() {
                0.8
            } else if *s == crate::contour::dynamic::displacement_cap_symbol() {
                1.
            } else {
                0.2
            })
        })
        .collect::<Vec<_>>();
    let mut result = vec![number(0.); generated.orders().len()];
    for (index, coefficient) in generated.exact_coefficients().iter().enumerate() {
        let mut evaluator = {
            let _preparing = generated.program_descriptor().map(|owner| owner.enter());
            coefficient
                .evaluator(&runtime_atoms)
                .build()
                .unwrap()
                .map_coeff(&map)
        };
        result[index] += evaluator.evaluate_single(&values);
    }
    for sector in generated.sectors() {
        assert_eq!(sector.parameters().len(), 1);
        let inputs = sector
            .parameters()
            .iter()
            .map(|s| Atom::var(*s))
            .chain(runtime_atoms.iter().cloned())
            .collect::<Vec<_>>();
        let mut point = std::iter::once(number(0.))
            .chain(values.iter().copied())
            .collect::<Vec<_>>();
        let mut evaluator = {
            let _preparing = generated.program_descriptor().map(|owner| owner.enter());
            let exact = if let Some(deferred) = &sector.deferred {
                crate::generation::numerical_dual::native::build(
                    deferred,
                    &runtime,
                    CompilationSettings {
                        backend: EvaluatorBackend::Eager,
                        ..Default::default()
                    },
                )
                .unwrap()
            } else {
                Atom::evaluator_multiple(sector.coefficients(), &inputs)
                    .build()
                    .unwrap()
            };
            exact.map_coeff(&map)
        };
        let points = 8192;
        let mut sum = vec![number(0.); result.len()];
        let mut sample = sum.clone();
        for i in 0..points {
            point[0].re = DoubleFloat::from((i as f64 + 0.5) / points as f64);
            evaluator.evaluate(&point, &mut sample);
            for (total, value) in sum.iter_mut().zip(&sample) {
                assert!(value.re.to_f64().is_finite() && value.im.to_f64().is_finite());
                *total += *value;
            }
        }
        for (total, sum) in result.iter_mut().zip(sum) {
            *total += sum / number(points as f64);
        }
        if sector.deferred.is_some() {
            assert!(
                sector.materialized.get().is_none(),
                "dual control materialized its coefficients"
            );
        }
    }
    result
        .into_iter()
        .map(|value| Complex::new(value.re.to_f64(), value.im.to_f64()))
        .collect()
}

#[test]
fn cubic_causal_and_positive_bounds_preserve_analytic_laurent_vector() {
    let input = source();
    let pi = std::f64::consts::PI;
    let expected_finite = Complex::new(
        -64. / 17. * 3f64.ln() - 2. / 17. * 2f64.ln() + 4. * pi / 17.,
        64. * pi / 17.,
    );
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let mut fixed: Option<Vec<Complex<f64>>> = None;
            for recipe in [
                ProgramRecipe::FixedV1,
                ProgramRecipe::DynamicPolynomialV1,
                ProgramRecipe::DynamicSignAwareV1,
            ] {
                let generated = generate(
                    &input,
                    &GenerationOptions {
                        program_recipe: recipe,
                        mode,
                        subtraction,
                        ..Default::default()
                    },
                    |_| ControlFlow::Continue(()),
                )
                .unwrap();
                assert_eq!(generated.orders(), &[-1, 0]);
                assert!(
                    generated
                        .sectors()
                        .iter()
                        .all(|sector| sector.generation_mode() == mode)
                );
                assert_eq!(generated.metadata().charts().len(), 1);
                let contour = generated.metadata().charts()[0].contour().unwrap();
                assert_eq!(contour.positive_polynomials().len(), 1);
                if recipe.is_dynamic() {
                    let descriptor = generated.program_descriptor().unwrap();
                    assert_eq!(descriptor.recipe(), recipe);
                    let chart = &descriptor.charts()[0];
                    assert_eq!(chart.dimension, 1);
                    assert_eq!(chart.causal_orders, vec![3]);
                    assert_eq!(chart.positive_orders, vec![vec![2]]);
                    assert_eq!(chart.coefficient_count, 2);
                    assert_eq!(generated.dynamic_check_sources().len(), 1);
                }
                if recipe == ProgramRecipe::DynamicSignAwareV1 {
                    assert!(
                        contour
                            .images()
                            .iter()
                            .chain(
                                contour
                                    .function_definitions()
                                    .entries()
                                    .iter()
                                    .map(|definition| definition.body())
                            )
                            .any(|coefficient| coefficient.contains_symbol(symbol!(
                                "fastsecdec::contour::smooth_positive_v1"
                            )))
                    );
                }
                let values = integrate(&generated, recipe);
                assert!((values[0].re + 4.).abs() < 1e-10);
                assert!(values[0].im.abs() < 1e-10);
                for (actual, expected) in [
                    (values[1].re, expected_finite.re),
                    (values[1].im, expected_finite.im),
                ] {
                    assert!(
                        (actual - expected).abs() < 2e-5,
                        "{mode:?}/{subtraction:?}/{recipe:?}: {values:?}"
                    );
                }
                if let Some(reference) = &fixed {
                    for (actual, expected) in values.iter().zip(reference) {
                        assert!((actual.re - expected.re).abs() < 3e-5);
                        assert!((actual.im - expected.im).abs() < 3e-5);
                    }
                } else {
                    fixed = Some(values);
                }
            }
        }
    }
}
