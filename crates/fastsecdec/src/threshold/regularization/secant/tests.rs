use super as normalized;
use crate::{generation, threshold::maps::regular};
use crate::{
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    threshold::gcad::{GcadKinematics, GcadRequest, SolverOptions, VerifiedDecomposition},
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::OptimizationSettings,
    symbol,
};
fn solved(input: ParametricIntegrand) -> Arc<VerifiedDecomposition> {
    let mut limits = GcadRequest::default_limits();
    limits.wall_time_secs = 30.;
    limits.memory_mib = 1024;
    let order = (0..input.parameters().len())
        .map(|i| format!("v{i}"))
        .collect();
    Arc::new(
        GcadRequest::unit_cube(
            &input,
            GcadKinematics::default(),
            SolverOptions {
                order,
                ..Default::default()
            },
            limits,
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    )
}
fn input(
    coords: Vec<Symbol>,
    eps: Symbol,
    p: Atom,
    q: Atom,
    extras: Vec<PolynomialFactor>,
    powers: Option<Vec<Atom>>,
) -> ParametricIntegrand {
    let mut factors = vec![
        PolynomialFactor::new(p, q, FactorRole::Singularity)
            .with_semantics(FactorSemantics::Causal),
    ];
    factors.extend(extras);
    let n = coords.len();
    ParametricIntegrand::new(
        coords,
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            powers.unwrap_or_else(|| vec![Atom::zero(); n]),
            factors,
        )],
    )
    .unwrap()
}
fn options() -> generation::GenerationOptions {
    generation::GenerationOptions {
        mode: generation::GenerationMode::Symbolic,
        ..Default::default()
    }
}
fn oracle(
    family: &normalized::NormalizedFamily,
    points: &[Vec<f64>],
    numerator: impl Fn(f64) -> f64,
    pz: impl Fn(f64, f64) -> f64,
) {
    let continued = family.continue_symbolically(&options()).unwrap();
    let vector = continued.expand_vector(1).unwrap();
    if let Some(pole) = vector.get(&-1) {
        assert!(pole.clone().into_inner().together().is_zero());
    }
    assert_eq!(continued.charts().len(), 2);
    let poles = continued
        .charts()
        .iter()
        .map(|chart| {
            generation::threshold_expand_vector(
                chart,
                family.coordinates(),
                family.regulators()[0],
                0,
            )
            .unwrap()[&-1]
                .clone()
                .into_inner()
        })
        .collect::<Vec<_>>();
    assert!(!poles[0].is_zero());
    assert!(!poles[1].is_zero());
    assert!((&poles[0] + &poles[1]).together().is_zero());
    let expressions = [
        vector[&0].clone().into_inner(),
        vector[&1].clone().into_inner(),
        family.root().clone(),
    ];
    let params = family
        .coordinates()
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let exact = Atom::evaluator_multiple(&expressions, &params)
        .function_map(continued.functions().clone())
        .optimization_settings(OptimizationSettings::default().cores(1))
        .build()
        .unwrap();
    let mut evaluator = family.callback_scope().enter(53, || {
        exact.map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
    });
    let mut values = vec![Complex::new(0., 0.); expressions.len()];
    let mut worst = 0.0_f64;
    for point in points {
        let inputs = point
            .iter()
            .map(|x| Complex::new(*x, 0.))
            .collect::<Vec<_>>();
        regular::callback::attempt(|| evaluator.evaluate(&inputs, &mut values)).unwrap();
        assert!(values.iter().all(|v| v.re.is_finite() && v.im.is_finite()));
        let a = values[2].re;
        let error = (values[0].im
            - std::f64::consts::PI * numerator(a) / pz(point[point.len() - 2], a))
        .abs();
        worst = worst.max(error);
        assert!(error < 2e-10, "{point:?}: {values:?}, error={error}");
    }
    println!(
        "oracle dim{} maxerror={worst:.5e}",
        family.coordinates().len()
    );
}
fn sample_vector(
    family: &normalized::NormalizedFamily,
    points: &[Vec<f64>],
) -> Vec<Vec<Complex<f64>>> {
    let continued = family.continue_symbolically(&options()).unwrap();
    let vector = continued.expand_vector(1).unwrap();
    let expressions = [
        vector[&0].clone().into_inner(),
        vector[&1].clone().into_inner(),
    ];
    let parameters = family
        .coordinates()
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let exact = Atom::evaluator_multiple(&expressions, &parameters)
        .function_map(continued.functions().clone())
        .optimization_settings(OptimizationSettings::default().cores(1))
        .build()
        .unwrap();
    let mut eval = family.callback_scope().enter(53, || {
        exact.map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
    });
    points
        .iter()
        .map(|point| {
            let inputs = point
                .iter()
                .map(|x| Complex::new(*x, 0.))
                .collect::<Vec<_>>();
            let mut out = vec![Complex::new(0., 0.); 2];
            regular::callback::attempt(|| eval.evaluate(&inputs, &mut out)).unwrap();
            out
        })
        .collect()
}
#[test]
fn actual_closed_secant_family_native_continuation_controls() {
    let started = Instant::now();
    let (x, y, eps, s, u, w, v) = symbol!(
        "normalized_owner::x",
        "normalized_owner::y",
        "normalized_owner::eps",
        "normalized_owner::s",
        "normalized_owner::u",
        "normalized_owner::w",
        "normalized_owner::v"
    );
    let q = -Atom::one() - Atom::var(eps);
    let p = Atom::var(y).pow(5) + Atom::var(y) - Atom::var(x);
    let owner = solved(input(vec![x, y], eps, p.clone(), q.clone(), vec![], None));
    let family = normalized::NormalizedFamily::admit(
        owner.clone(),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    assert_eq!(family.strip().upper(), Some(&Rational::zero()));
    let points = [0.01, 0.2, 0.5, 0.9, 1.]
        .into_iter()
        .flat_map(|x| [0.001, 0.13, 0.5, 0.9, 1.].map(move |u| vec![x, u]))
        .collect::<Vec<_>>();
    oracle(&family, &points, |_| 1., |_, a| 1. + 5. * a.powi(4));
    // Complete complex coherent sum preserves both original terms and every order.
    let terms = [Atom::one(), Atom::i()]
        .into_iter()
        .map(|prefactor| {
            ParametricTerm::new(
                prefactor,
                vec![Atom::zero(); 2],
                vec![
                    PolynomialFactor::new(p.clone(), q.clone(), FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                ],
            )
        })
        .collect();
    let complex = normalized::NormalizedFamily::admit(
        solved(
            ParametricIntegrand::new(vec![x, y], eps, ParametricDomain::UnitCube, terms).unwrap(),
        ),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    assert!(complex.charts().iter().all(|c| c.terms().len() == 2));
    let baseline = sample_vector(&family, &points);
    let complex_values = sample_vector(&complex, &points);
    for (base, actual) in baseline.iter().zip(complex_values) {
        for (a, b) in base.iter().zip(actual) {
            let expected = Complex::new(a.re - a.im, a.re + a.im);
            assert!((b.re - expected.re).abs() < 2e-9 * (1. + expected.re.abs()));
            assert!((b.im - expected.im).abs() < 2e-9 * (1. + expected.im.abs()));
        }
    }
    let mut taylor = options();
    taylor.subtraction = generation::SubtractionStrategy::Taylor;
    assert!(
        (family.continue_symbolically(&taylor).unwrap().expression()
            - family
                .continue_symbolically(&options())
                .unwrap()
                .expression())
        .together()
        .is_zero()
    );
    // Different arbitrary degree; leading coefficient vanishes on x=0 while P_y>=1.
    let dropped = Atom::var(x) * Atom::var(y).pow(7) + Atom::var(y) - Atom::var(x);
    let d = normalized::NormalizedFamily::admit(
        solved(input(vec![x, y], eps, dropped, q.clone(), vec![], None)),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    oracle(&d, &points, |_| 1., |x, a| 1. + 7. * x * a.powi(6));
    // Epsilon coefficient bodies remain outside their coordinate function maps.
    let numerator = PolynomialFactor::new(
        Atom::one() + 2 * Atom::var(y) + Atom::var(eps) * Atom::var(y).pow(2),
        Atom::one(),
        FactorRole::Polynomial,
    );
    let n = normalized::NormalizedFamily::admit(
        solved(input(
            vec![x, y],
            eps,
            p.clone(),
            q.clone(),
            vec![numerator],
            None,
        )),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    oracle(&n, &points, |a| 1. + 2. * a, |_, a| 1. + 5. * a.powi(4));
    // Additional denominator remains a certified closed positive unit, not a lost factor.
    let extra = PolynomialFactor::new(
        Atom::num(2) + Atom::var(y),
        -Atom::one(),
        FactorRole::Singularity,
    )
    .with_semantics(FactorSemantics::Positive);
    let m = normalized::NormalizedFamily::admit(
        solved(input(
            vec![x, y],
            eps,
            p.clone(),
            q.clone(),
            vec![extra],
            None,
        )),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    assert_eq!(m.charts()[0].terms()[0].ledger().len(), 2);
    oracle(&m, &points, |a| 1. / (2. + a), |_, a| 1. + 5. * a.powi(4));
    // Genuine 3D storage and subtraction, with one inert cube direction.
    let three = normalized::NormalizedFamily::admit(
        solved(input(
            vec![w, x, y],
            eps,
            p.clone(),
            q.clone(),
            vec![],
            None,
        )),
        vec![v, s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    oracle(
        &three,
        &[vec![0.17, 0.3, 0.4], vec![0.8, 0.7, 0.9]],
        |_| 1.,
        |_, a| 1. + 5. * a.powi(4),
    );
    // Repeated pole: corner pole is retained; do not require complete pole cancellation.
    let repeated = normalized::NormalizedFamily::admit(
        solved(input(
            vec![x, y],
            eps,
            p.clone(),
            -Atom::num(2) - Atom::var(eps),
            vec![PolynomialFactor::new(
                Atom::one() + Atom::var(y).pow(2),
                Atom::one(),
                FactorRole::Polynomial,
            )],
            None,
        )),
        vec![s, u],
        normalized::Limits::default(),
    )
    .unwrap();
    let rc = repeated.continue_symbolically(&options()).unwrap();
    let rv = rc.expand_vector(0).unwrap();
    assert!(
        rv.iter()
            .any(|(order, a)| *order < 0 && !a.clone().into_inner().together().is_zero())
    );
    let rexpressions = rv
        .values()
        .map(|a| a.clone().into_inner())
        .collect::<Vec<_>>();
    let rexact = Atom::evaluator_multiple(&rexpressions, &[Atom::var(s), Atom::var(u)])
        .function_map(rc.functions().clone())
        .optimization_settings(OptimizationSettings::default().cores(1))
        .build()
        .unwrap();
    let mut reval = repeated.callback_scope().enter(53, || {
        rexact.map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()))
    });
    let mut out = vec![Complex::new(0., 0.); rexpressions.len()];
    regular::callback::attempt(|| {
        reval.evaluate(&[Complex::new(0.3, 0.), Complex::new(0.7, 0.)], &mut out)
    })
    .unwrap();
    assert!(out.iter().all(|a| a.re.is_finite() && a.im.is_finite()));
    // No source can authorize inconsistent regulator neighborhoods or unsafe routing.
    let empty = solved(input(
        vec![x, y],
        eps,
        p.clone(),
        q.clone(),
        vec![],
        Some(vec![-Atom::one() + Atom::var(eps), Atom::zero()]),
    ));
    assert!(matches!(
        normalized::NormalizedFamily::admit(empty, vec![s, u], normalized::Limits::default()),
        Err(normalized::Error::Unsupported(_))
    ));
    let cancel = normalized::Limits {
        cancellation: Some(Arc::new(AtomicBool::new(true))),
        ..Default::default()
    };
    assert!(matches!(
        normalized::NormalizedFamily::admit(owner.clone(), vec![s, u], cancel),
        Err(normalized::Error::Cancelled)
    ));
    let mut bounded = normalized::Limits::default();
    bounded.geometry.max_degree = 2;
    assert!(matches!(
        normalized::NormalizedFamily::admit(owner, vec![s, u], bounded),
        Err(normalized::Error::ResourceIncomplete(_))
    ));
    let mut bad = options();
    bad.mode = generation::GenerationMode::NumericalDual;
    assert!(matches!(
        family.continue_symbolically(&bad),
        Err(normalized::Error::Unsupported(_))
    ));
    println!(
        "PASS normalized owner baseline/degree-drop/numerator/unit/3D/repeated-pole/refusal groups, elapsed={:.6}",
        started.elapsed().as_secs_f64()
    );
}
