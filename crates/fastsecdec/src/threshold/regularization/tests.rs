use crate::{generation, parametric::*, threshold};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::Complex,
    evaluate::{ExpressionEvaluator, FunctionMap, OptimizationSettings},
    poly::series::SeriesDepth,
    prelude::Rational,
    symbol,
};
use threshold::{
    gcad::{GcadKinematics, GcadRequest, SolverOptions, VerifiedDecomposition},
    regularization::*,
};
fn source(x: Symbol, eps: Symbol, f: Atom, q: Atom, numerator: Atom) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(f, q, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(numerator, Atom::one(), FactorRole::Polynomial),
            ],
        )],
    )
    .unwrap()
}
fn solve(input: &ParametricIntegrand, kinematics: GcadKinematics) -> Arc<VerifiedDecomposition> {
    Arc::new(
        GcadRequest::unit_cube(
            input,
            kinematics,
            SolverOptions::default(),
            GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    )
}
fn restored(
    outputs: &[Atom],
    inputs: &[Atom],
    functions: FunctionMap,
) -> ExpressionEvaluator<Complex<f64>> {
    let program = Atom::evaluator_multiple(outputs, inputs)
        .function_map(functions)
        .optimization_settings(OptimizationSettings::default().cores(1))
        .build()
        .unwrap();
    let bytes = bincode::encode_to_vec(&program, bincode::config::standard()).unwrap();
    let (restored, used): (ExpressionEvaluator<Complex<Rational>>, usize) =
        bincode::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(program.get_input_len(), restored.get_input_len());
    restored.map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
}
#[test]
fn moving_fiber_units_coverage_continuation_and_rejections() {
    let (x, t, a, eps) = symbol!(
        "regular_draft::x",
        "regular_draft::t",
        "regular_draft::a",
        "regular_draft::eps"
    );
    let input = source(
        x,
        eps,
        Atom::var(x) - Atom::var(a),
        -Atom::one() - Atom::var(eps),
        Atom::one() + Atom::i() * Atom::var(a) * Atom::var(x),
    );
    let owner = solve(
        &input,
        GcadKinematics {
            runtime_parameters: vec![a],
            strict_positive: vec![Atom::var(a), Atom::one() - Atom::var(a)],
            ..Default::default()
        },
    );
    for value in [Rational::from((1, 4)), Rational::from((2, 3))] {
        let fiber = RegularizedFiber::admit(
            owner.clone(),
            BTreeMap::from([(a, value.clone())]),
            t,
            Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(fiber.charts().len(), 4);
        for incompatible in [
            generation::GenerationOptions {
                mode: generation::GenerationMode::NumericalDual,
                ..Default::default()
            },
            generation::GenerationOptions {
                program_recipe: crate::kernel::indexed::ProgramRecipe::FixedV1,
                ..Default::default()
            },
            generation::GenerationOptions {
                contour_jacobian: crate::contour::ContourJacobian::Dual,
                ..Default::default()
            },
            generation::GenerationOptions {
                source_sectors: Some(vec![0]),
                ..Default::default()
            },
        ] {
            assert!(
                fiber
                    .continue_symbolically(&incompatible, |_| ControlFlow::Continue(()))
                    .is_err()
            );
        }
        assert_eq!(fiber.convergence_strip().upper(), Some(&Rational::zero()));
        let mut total = Rational::zero();
        for chart in fiber.charts() {
            let (lower, upper) = chart.interval();
            assert!(lower < upper);
            total += upper - lower;
            assert!(matches!(chart.orientation(), -1 | 1));
            for unit in chart.units(0).unwrap() {
                let (l, u) = unit.endpoint_values();
                assert!(l > &Rational::zero() && u > &Rational::zero());
            }
        }
        assert_eq!(total, Rational::one());
        for subtraction in [
            generation::SubtractionStrategy::Taylor,
            generation::SubtractionStrategy::IntegrateByParts,
        ] {
            let continued = fiber
                .continue_symbolically(
                    &generation::GenerationOptions {
                        max_subtractions_per_axis: 3,
                        subtraction,
                        ..Default::default()
                    },
                    |_| ControlFlow::Continue(()),
                )
                .unwrap();
            assert_eq!(continued.profiles().len(), 4);
            let coefficients = continued
                .expression()
                .series(eps, 0, SeriesDepth::absolute(1))
                .unwrap();
            let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
            assert_eq!(bound.bindings(), &[(a, value.clone())]);
            assert_eq!(bound.coordinates(), [t]);
            assert_eq!(bound.regulators(), [eps]);
            assert!(!bound.expression().contains_symbol(a));
            let bound_series = bound
                .expression()
                .series(eps, 0, SeriesDepth::absolute(1))
                .unwrap();
            let mut bound_evaluator = restored(
                &[
                    bound_series.coefficient((-1).into()).unwrap_or(Atom::Zero),
                    bound_series.coefficient(0.into()).unwrap(),
                ],
                &[Atom::var(t)],
                bound.functions().clone(),
            );
            let pole = coefficients.coefficient((-1).into()).unwrap_or(Atom::Zero);
            let finite = coefficients.coefficient(0.into()).unwrap();
            let mut evaluator = restored(
                &[pole, finite],
                &[Atom::var(a), Atom::var(t)],
                continued.functions().clone(),
            );
            for point in [0.17, 0.43, 0.79] {
                let mut out = vec![Complex::new(0., 0.); 2];
                evaluator.evaluate(
                    &[Complex::new(value.to_f64(), 0.), Complex::new(point, 0.)],
                    &mut out,
                );
                assert!(
                    out[0].re.abs() < 1e-12 && out[0].im.abs() < 1e-12,
                    "uncancelled cell pole {out:?}"
                );
                assert!(out[1].re.is_finite() && out[1].im.is_finite());
                let mut bound_values = [Complex::new(0., 0.); 2];
                bound_evaluator.evaluate(&[Complex::new(point, 0.)], &mut bound_values);
                for i in 0..2 {
                    assert!(
                        (out[i].re - bound_values[i].re).abs() < 1e-12
                            && (out[i].im - bound_values[i].im).abs() < 1e-12
                    );
                }
            }
        }
    }
    for power in [-1, -2] {
        let upper = source(
            x,
            eps,
            Atom::one() - Atom::var(x),
            Atom::num(power) - Atom::var(eps),
            Atom::one(),
        );
        let fiber = RegularizedFiber::admit(
            solve(&upper, Default::default()),
            BTreeMap::new(),
            t,
            Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(fiber.charts().len(), 2);
        assert_eq!(
            fiber.convergence_strip().upper(),
            Some(&Rational::from(power + 1))
        );
        assert!(
            fiber
                .charts()
                .iter()
                .any(|c| c.orientation() == -1 && c.units(0).unwrap()[0].multiplicity() == 1)
        );
    }
    assert!(
        RegularizedFiber::admit(
            owner.clone(),
            BTreeMap::from([(a, Rational::one())]),
            t,
            Limits::default(),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
    assert!(matches!(
        RegularizedFiber::admit(
            owner.clone(),
            BTreeMap::from([(a, Rational::from((1, 3)))]),
            t,
            Limits {
                max_charts: 3,
                ..Default::default()
            },
            |_| ControlFlow::Continue(())
        ),
        Err(Error::ResourceIncomplete(_))
    ));
    assert!(matches!(
        RegularizedFiber::admit(
            owner,
            BTreeMap::from([(a, Rational::from((1, 3)))]),
            t,
            Limits::default(),
            |_| ControlFlow::Break(())
        ),
        Err(Error::Cancelled)
    ));
    let incompatible = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::var(x),
                    -Atom::one() - Atom::var(eps),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    Atom::one() - Atom::var(x),
                    -Atom::one() + Atom::var(eps),
                    FactorRole::Singularity,
                ),
            ],
        )],
    )
    .unwrap();
    assert!(
        RegularizedFiber::admit(
            solve(&incompatible, Default::default()),
            BTreeMap::new(),
            t,
            Limits::default(),
            |_| ControlFlow::Continue(())
        )
        .is_err()
    );
    for (prefactor, numerator) in [
        (Atom::one(), Atom::num(1.25)),
        (Atom::num(1.25), Atom::one()),
    ] {
        let inexact = ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                prefactor,
                vec![Atom::Zero],
                vec![
                    PolynomialFactor::new(Atom::var(x), -Atom::var(eps), FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                    PolynomialFactor::new(numerator, Atom::one(), FactorRole::Polynomial),
                ],
            )],
        )
        .unwrap();
        assert!(matches!(
            RegularizedFiber::admit(
                solve(&inexact, Default::default()),
                BTreeMap::new(),
                t,
                Limits::default(),
                |_| ControlFlow::Continue(())
            ),
            Err(Error::Unsupported(_))
        ));
    }
    let zero_power = source(x, eps, Atom::var(x), Atom::Zero, Atom::one());
    let zero_fiber = RegularizedFiber::admit(
        solve(&zero_power, Default::default()),
        BTreeMap::new(),
        t,
        Limits::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert!(
        zero_fiber
            .charts()
            .iter()
            .all(|c| c.units(0).unwrap().is_empty())
    );
    assert!(
        ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::Zero],
                vec![PolynomialFactor::new(
                    Atom::Zero,
                    Atom::Zero,
                    FactorRole::Singularity
                )]
            )]
        )
        .is_err()
    );
    println!(
        "PASS moving rational fiber/closed units/coverage/phases/native continuation/upper faces/bounds"
    );
}
#[test]
fn complete_native_causal_phase_and_projective_measure() {
    let (x, y, t, eps) = symbol!(
        "regular_draft_causal::x",
        "regular_draft_causal::y",
        "regular_draft_causal::t",
        "regular_draft_causal::eps"
    );
    let cube = source(
        x,
        eps,
        Atom::var(x) - Atom::num((1, 2)),
        -Atom::one() - Atom::var(eps),
        Atom::one(),
    );
    let projective = ParametricIntegrand::new(
        vec![x, y],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero, Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::var(x) - Atom::var(y),
                    -Atom::one() - Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::var(x) + Atom::var(y),
                    -Atom::one() + Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    let prepared =
        threshold::projective::AffineProjectivePreparation::last_coordinate(&projective).unwrap();
    let projective_owner = Arc::new(
        GcadRequest::projective(
            prepared,
            Default::default(),
            Default::default(),
            GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    for (owner, normalization) in [
        (solve(&cube, Default::default()), 1.),
        (projective_owner, 0.5),
    ] {
        let fiber = RegularizedFiber::admit(owner, BTreeMap::new(), t, Limits::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let continued = fiber
            .continue_symbolically(
                &generation::GenerationOptions {
                    max_subtractions_per_axis: 2,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        let series = continued
            .expression()
            .series(eps, 0, SeriesDepth::absolute(1))
            .unwrap();
        let mut evaluator = restored(
            &[
                series.coefficient((-1).into()).unwrap_or(Atom::Zero),
                series.coefficient(0.into()).unwrap(),
            ],
            &[Atom::var(t)],
            continued.functions().clone(),
        );
        for point in [0.125, 0.5, 0.875] {
            let mut out = vec![Complex::new(0., 0.); 2];
            evaluator.evaluate(&[Complex::new(point, 0.)], &mut out);
            assert!(
                out[0].re.abs() < 1e-13 && out[0].im.abs() < 1e-13,
                "pole {out:?}"
            );
            assert!(
                out[1].re.abs() < 1e-12
                    && (out[1].im - normalization * std::f64::consts::PI).abs() < 1e-12,
                "finite {out:?}"
            );
        }
    }
    println!("PASS complete native bridge finite i*pi and projective i*pi/2, normalization once");
}
#[test]
fn bubble_keeps_single_signed_factor_and_exact_cells() {
    let (x, t, eps) = symbol!(
        "regular_bubble::x",
        "regular_bubble::t",
        "regular_bubble::eps"
    );
    let f = Atom::num(3) - Atom::num(16) * Atom::var(x) * (Atom::one() - Atom::var(x));
    let owner = solve(
        &source(x, eps, f.clone(), -Atom::var(eps), Atom::one()),
        Default::default(),
    );
    assert_eq!(
        owner.request().signed_factors().len(),
        1,
        "do not turn two geometric roots into two causal factors"
    );
    assert_eq!(owner.cells().len(), 3);
    let native = owner.native_result();
    let algebra = symgcad::algebra::Algebra::new(&native.order).unwrap();
    for (index, cell) in native.cells.iter().enumerate() {
        for bound in [&cell.axes[0].lower, &cell.axes[0].upper]
            .into_iter()
            .flatten()
        {
            let polynomial = algebra
                .parse(&native.polynomials[bound.polynomial])
                .unwrap();
            assert_eq!(
                polynomial.degree(0),
                1,
                "native bubble root section remained nonlinear"
            );
            println!(
                "bubble cell {index} bound: {} selector {:?}/{}",
                native.polynomials[bound.polynomial], bound.index_domain, bound.index
            );
        }
    }
    let fiber = RegularizedFiber::admit(owner, BTreeMap::new(), t, Limits::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(fiber.charts().len(), 6);
    assert_eq!(fiber.convergence_strip().upper(), Some(&Rational::one()));
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_subtractions_per_axis: 2,
                max_order: 1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let series = continued
        .expression()
        .series(eps, 0, SeriesDepth::absolute(2))
        .unwrap();
    let mut evaluator = restored(
        &[
            series.coefficient(0.into()).unwrap(),
            series.coefficient(1.into()).unwrap(),
        ],
        &[Atom::var(t)],
        continued.functions().clone(),
    );
    for point in [0.125, 0.5, 0.875] {
        let mut values = vec![Complex::new(0., 0.); 2];
        evaluator.evaluate(&[Complex::new(point, 0.)], &mut values);
        assert!((values[0].re - 1.).abs() < 1e-13 && values[0].im.abs() < 1e-13);
        assert!(
            (values[1].im - std::f64::consts::PI / 2.).abs() < 1e-12,
            "single causal F phase lost: {values:?}"
        );
    }
    println!(
        "PASS equal-mass bubble rational native bounds and ONE-F phase coefficient; physical Gamma prefactor not yet admitted"
    );
}

mod epsilon_scales;
mod prefactor_reference;
mod prefactors;
mod reference;
