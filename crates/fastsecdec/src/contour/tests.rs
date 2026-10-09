use super::*;
use crate::{
    generation::{self, GenerationMode, GenerationOptions, SubtractionStrategy},
    kernel::{CompilationSettings, EvaluatorBackend},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{
    atom::{Atom, AtomCore},
    domains::float::Complex,
    parse, symbol,
};

fn input(power: Atom, prefactor: Atom, f: Atom, exponent: Atom) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("contour_tests::x")],
        symbol!("contour_tests::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            prefactor,
            vec![power],
            vec![
                PolynomialFactor::new(f, exponent, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn fixed_map_native_derivatives_preserve_faces_and_endpoint_ratios() {
    let x = symbol!("contour_map::x");
    let y = symbol!("contour_map::y");
    let f = parse!("1-3*contour_map::x-contour_map::y^2+2*contour_map::x*contour_map::y");
    let map = FixedContourMap::new(&[x, y], f).unwrap();
    for (axis, parameter) in [x, y].iter().enumerate() {
        assert!(
            map.metadata().images()[axis]
                .replace(*parameter)
                .with(0)
                .is_zero()
        );
        assert!(
            (map.metadata().images()[axis].replace(*parameter).with(1) - 1)
                .expand()
                .is_zero()
        );
        assert_eq!(
            &map.metadata().images()[axis],
            &(Atom::var(*parameter) * &map.metadata().ratios()[axis])
        );
        assert!(
            !map.metadata().ratios()[axis]
                .replace(*parameter)
                .with(0)
                .to_string()
                .contains("Undefined")
        );
    }
    let images = map.metadata().images();
    let manual = images[0].derivative(x) * images[1].derivative(y)
        - images[0].derivative(y) * images[1].derivative(x);
    assert!((map.metadata().jacobian() - manual).expand().is_zero());
    assert_eq!(
        map.metadata().jacobian().replace(lambda_symbol()).with(0),
        Atom::one()
    );
}

#[test]
fn four_dimensional_native_bareiss_determinant_removes_pivot_poles() {
    let parameters = [
        symbol!("contour_4d::a"),
        symbol!("contour_4d::b"),
        symbol!("contour_4d::c"),
        symbol!("contour_4d::d"),
    ];
    let f = parse!(
        "contour_4d::a^2+contour_4d::a*contour_4d::b+contour_4d::b^2+contour_4d::c^2+contour_4d::c*contour_4d::d+contour_4d::d^2"
    );
    let map = FixedContourMap::new(&parameters, f).unwrap();
    // At x=0, all off-diagonal entries vanish, including along a vanishing
    // first Bareiss pivot. The exact determinant must remain a polynomial.
    let origin = map
        .metadata()
        .jacobian()
        .replace_multiple(parameters.iter().map(|p| {
            symbolica::id::Replacement::new(
                symbolica::id::Pattern::Literal(Atom::var(*p)),
                symbolica::id::Pattern::Literal(Atom::Zero),
            )
        }));
    assert_eq!(origin, Atom::one());
    assert!(
        map.metadata()
            .jacobian()
            .is_polynomial(true, false)
            .is_some()
    );
    let pivot_zero = map
        .metadata()
        .jacobian()
        .replace_multiple(parameters.iter().map(|p| {
            symbolica::id::Replacement::new(
                symbolica::id::Pattern::Literal(Atom::var(*p)),
                symbolica::id::Pattern::Literal(parse!("1/2")),
            )
        }))
        .replace(lambda_symbol())
        .with(-2 * Atom::i());
    assert_eq!(pivot_zero, parse!("1/16"));
}

fn integrate(
    generated: &generation::GeneratedIntegral,
    lambda: f64,
) -> BTreeMap<(i32, CoefficientComponent), f64> {
    let mut kernels = generated
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[lambda_symbol()],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: ContourMode::Fixed { lambda },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Off,
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let mut result = kernels
        .orders()
        .iter()
        .copied()
        .zip(kernels.components().iter().copied())
        .zip(kernels.exact_coefficients().iter().copied())
        .collect::<BTreeMap<_, _>>();
    let layout = kernels
        .orders()
        .iter()
        .copied()
        .zip(kernels.components().iter().copied())
        .collect::<Vec<_>>();
    for sector in kernels.sectors_mut() {
        assert_eq!(sector.dimension(), 1);
        let n = 8192;
        let mut sum = vec![0.0; layout.len()];
        let mut sample = vec![0.0; layout.len()];
        for index in 0..n {
            sector
                .evaluate(&[(index as f64 + 0.5) / n as f64], &mut sample)
                .unwrap();
            for (total, value) in sum.iter_mut().zip(&sample) {
                *total += value / n as f64;
            }
        }
        for (key, value) in layout.iter().zip(sum) {
            *result.entry(*key).or_default() += value;
        }
    }
    result
}

#[test]
fn above_threshold_log_bubble_has_causal_lower_lip_and_strength_invariance() {
    let input = input(
        Atom::Zero,
        parse!("1/contour_tests::eps"),
        parse!("1-5*contour_tests::x*(1-contour_tests::x)"),
        parse!("-contour_tests::eps"),
    );
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let options = GenerationOptions {
            contour: true,
            mode,
            ..Default::default()
        };
        let generated =
            generation::generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
        assert!(
            generated
                .metadata()
                .charts()
                .iter()
                .all(|chart| chart.contour().is_some())
        );
        let beta = 0.2_f64.sqrt();
        let finite = Complex::new(
            2.0 - beta * ((1.0 + beta) / (1.0 - beta)).ln(),
            std::f64::consts::PI * beta,
        );
        for lambda in [0.05, 0.2, 0.5] {
            let values = integrate(&generated, lambda);
            assert!(
                (values[&(0, CoefficientComponent::Real)] - finite.re).abs() < 2e-6,
                "{values:?}"
            );
            assert!(
                (values[&(0, CoefficientComponent::Imag)] - finite.im).abs() < 2e-6,
                "{values:?}"
            );
            assert!((values[&(-1, CoefficientComponent::Real)] - 1.0).abs() < 2e-6);
        }
    }
}

#[test]
fn deformation_precedes_taylor_and_ibp_endpoint_subtraction() {
    for degree in [1, 2] {
        let input = input(
            Atom::num(-degree) - Atom::var(symbol!("contour_tests::eps")),
            Atom::one(),
            parse!("1/4-contour_tests::x"),
            Atom::num(-1),
        );
        for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
            for subtraction in [
                SubtractionStrategy::Taylor,
                SubtractionStrategy::IntegrateByParts,
            ] {
                let options = GenerationOptions {
                    contour: true,
                    subtraction,
                    mode,
                    ..Default::default()
                };
                let generated =
                    generation::generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
                let faces = generated.metadata().charts()[0]
                    .contour()
                    .unwrap()
                    .validation_faces();
                assert!(faces.iter().any(|face| face.contains(&(0, 0))));
                assert_eq!(
                    faces.iter().any(|face| face.contains(&(0, 1))),
                    degree == 2 && subtraction == SubtractionStrategy::IntegrateByParts
                );
                let values = integrate(&generated, 0.2);
                let residue = 4.0_f64.powi(degree);
                let extra = if degree == 2 { -4.0 } else { 0.0 };
                assert!(
                    (values[&(-1, CoefficientComponent::Real)] + residue).abs() < 2e-6,
                    "{values:?}"
                );
                assert!(
                    (values[&(0, CoefficientComponent::Real)] + residue * 3.0_f64.ln() - extra)
                        .abs()
                        < 2e-5,
                    "{values:?}"
                );
                assert!(
                    (values[&(0, CoefficientComponent::Imag)] - residue * std::f64::consts::PI)
                        .abs()
                        < 2e-5,
                    "{values:?}"
                );
            }
        }
    }
}

#[test]
fn generation_never_guesses_f_and_runtime_strength_is_strictly_positive() {
    let input = ParametricIntegrand::new(
        vec![symbol!("contour_tests::x")],
        symbol!("contour_tests::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1-contour_tests::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let options = GenerationOptions {
        contour: true,
        mode: GenerationMode::NumericalDual,
        ..Default::default()
    };
    assert!(
        generation::generate(&input, &options, |_| ControlFlow::Continue(()))
            .unwrap_err()
            .to_string()
            .contains("explicitly designated")
    );
    for lambda in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            ContourSettings {
                deformation: ContourMode::Fixed { lambda },
                ..Default::default()
            }
            .validate()
            .is_err()
        );
    }
}

#[test]
fn complex_causal_coefficients_are_rejected_even_without_runtime_checks() {
    let input = input(
        Atom::Zero,
        Atom::one(),
        Atom::one() + Atom::i() * Atom::var(symbol!("contour_tests::x")),
        Atom::num(-1),
    );
    let options = GenerationOptions {
        contour: true,
        ..Default::default()
    };
    let error = generation::generate(&input, &options, |_| ControlFlow::Continue(()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("must be real"), "{error}");
}

#[test]
fn native_dual_keeps_meromorphic_term_prefactors_outside_smooth_jets() {
    let eps = symbol!("contour_tests::eps");
    let f = parse!("1-5*contour_tests::x*(1-contour_tests::x)");
    let input = ParametricIntegrand::new(
        vec![symbol!("contour_tests::x")],
        eps,
        ParametricDomain::UnitCube,
        [1, 2]
            .into_iter()
            .map(|multiple| {
                ParametricTerm::new(
                    Atom::num(if multiple == 1 { 1 } else { -1 }) / Atom::var(eps).pow(2),
                    vec![Atom::Zero],
                    vec![
                        PolynomialFactor::new(
                            f.clone(),
                            -Atom::num(multiple) * Atom::var(eps),
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Causal),
                    ],
                )
            })
            .collect(),
    )
    .unwrap();
    let mut results = Vec::new();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let generated = generation::generate(
            &input,
            &GenerationOptions {
                contour: true,
                mode,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        results.push(integrate(&generated, 0.2));
    }
    for component in results[0].keys().chain(results[1].keys()) {
        let value = results[0].get(component).copied().unwrap_or(0.0);
        let other = results[1].get(component).copied().unwrap_or(0.0);
        assert!(
            (value - other).abs() < 1e-9,
            "{component:?}: {value} != {other}"
        );
    }
}

#[test]
fn zero_dimensional_causal_offsets_retain_contour_capability() {
    let eps = symbol!("contour_zero::eps");
    let input = ParametricIntegrand::new(
        vec![],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::var(eps).pow(-1),
            vec![],
            vec![
                PolynomialFactor::new(Atom::num(-1), -Atom::var(eps), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let generated = generation::generate(
        &input,
        &GenerationOptions {
            contour: true,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert!(generated.sectors().is_empty());
    assert!(
        generated
            .metadata()
            .charts()
            .iter()
            .any(|chart| chart.contour().is_some())
    );
    let values = integrate(&generated, 0.2);
    assert!((values[&(0, CoefficientComponent::Imag)] - std::f64::consts::PI).abs() < 1e-13);
}

#[cfg(feature = "native")]
#[test]
fn physical_contour_native_scalar_and_batch_match_eager_at_fixed_points() {
    use crate::kernel::ReplayPolicy;
    let input = input(
        Atom::Zero,
        parse!("1/contour_tests::eps"),
        parse!("1-5*contour_tests::x*(1-contour_tests::x)"),
        parse!("-contour_tests::eps"),
    );
    let generated = generation::generate(
        &input,
        &GenerationOptions {
            contour: true,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let points = [0.01, 0.15, 0.275, 0.499, 0.5, 0.501, 0.725, 0.85, 0.99];
    let settings = ContourSettings {
        deformation: ContourMode::Fixed { lambda: 0.2 },
        validation: ContourValidationOptions {
            policy: ContourValidation::Off,
            ..Default::default()
        },
    };
    let mut references = None::<Vec<f64>>;
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        let mut kernels = generated
            .compile_with_settings_parameters_and_progress(
                Default::default(),
                &[],
                CompilationSettings {
                    backend,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        kernels
            .bind_parameters_with_contour(&BTreeMap::new(), &settings)
            .unwrap();
        let count = kernels.orders().len();
        assert_eq!(kernels.sectors().len(), 1);
        let mut values = vec![0.0; points.len() * count];
        for (point, output) in points.iter().zip(values.chunks_mut(count)) {
            kernels.sectors_mut()[0]
                .evaluate(&[*point], output)
                .unwrap();
        }
        let mut batch = vec![0.0; values.len()];
        kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap()
            .evaluate_weighted_batch(&points, &[1.0; 9], &mut batch)
            .unwrap();
        for (scalar, batch) in values.iter().zip(batch) {
            assert!(
                (scalar - batch).abs() < 1e-12,
                "{backend:?}: scalar={scalar} batch={batch}"
            );
        }
        if let Some(reference) = &references {
            for (actual, expected) in values.iter().zip(reference) {
                assert!(
                    (actual - expected).abs() < 1e-12,
                    "{backend:?}: {actual} vs eager={expected}"
                );
            }
        } else {
            references = Some(values);
        }
    }
}
