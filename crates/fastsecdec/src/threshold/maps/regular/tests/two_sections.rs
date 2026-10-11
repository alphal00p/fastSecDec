use super::super::{self as regular, BracketProposal, Budget, Limits, RegularSection, SectionSide};
use crate::{
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    threshold::{
        gcad::{GcadKinematics, GcadRequest, SolverOptions},
        maps::CellMap,
    },
};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::rational::Rational,
    evaluate::OptimizationSettings,
    symbol,
};
use symgcad::{
    algebra::Algebra,
    domain::{CertificateOptions, certify_nonzero, derive_domain, verify_certificate},
};
#[test]
fn actual_degree_six_cell_has_two_regular_sections_and_closed_unit() {
    let (x, y, e, s, t, ta, tb) = symbol!(
        "two_section::x",
        "two_section::y",
        "two_section::eps",
        "two_section::s",
        "two_section::t",
        "two_section::lower",
        "two_section::upper"
    );
    let p = Atom::var(y).pow(6) - Atom::var(y) + (Atom::one() + Atom::var(x)) / Atom::num(16);
    let input = ParametricIntegrand::new(
        vec![x, y],
        e,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::zero(); 2],
            vec![
                PolynomialFactor::new(p, -Atom::one() - Atom::var(e), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let mut limits = GcadRequest::default_limits();
    limits.wall_time_secs = 30.;
    limits.memory_mib = 1024;
    let owner = Arc::new(
        GcadRequest::unit_cube(
            &input,
            GcadKinematics::default(),
            SolverOptions {
                order: vec!["v0".into(), "v1".into()],
                ..Default::default()
            },
            limits,
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    assert_eq!(owner.cells().len(), 3);
    let maps = (0..3)
        .map(|i| CellMap::new(owner.clone(), i, vec![s, t]).unwrap())
        .collect::<Vec<_>>();
    let raw = owner.native_result();
    let a = Algebra::new(&raw.order).unwrap();
    let middle = maps
        .iter()
        .find(|m| {
            let ax = &m.native_axes()[1];
            [ax.lower.as_ref().unwrap(), ax.upper.as_ref().unwrap()]
                .iter()
                .all(|r| a.parse(&raw.polynomials[r.polynomial]).unwrap().degree(1) == 6)
        })
        .unwrap();
    let mut budget = Budget::new(Limits::default());
    let lower = RegularSection::prepare(
        middle.clone(),
        1,
        SectionSide::Lower,
        BracketProposal {
            lower: Rational::from((1, 32)),
            upper: Rational::from((1, 4)),
            derivative_margin: Rational::from((1, 4)),
        },
        &mut budget,
    )
    .unwrap();
    let upper = RegularSection::prepare(
        middle.clone(),
        1,
        SectionSide::Upper,
        BracketProposal {
            lower: Rational::from((3, 4)),
            upper: Rational::one(),
            derivative_margin: Rational::from((1, 4)),
        },
        &mut budget,
    )
    .unwrap();
    assert_eq!(lower.polynomial(), upper.polynomial());
    assert_ne!(lower.bound(), upper.bound());
    // Stronger than necessary: a rational box contains both selected sections
    // for every prefix, their whole cell, and the positive root gap.
    let alg = Algebra::new(&["x".into(), "y".into(), "a".into(), "b".into()]).unwrap();
    let parse = |v: &str| alg.parse(v).unwrap();
    let py = parse("y^6-y+(1+x)/16");
    let ya = parse("y-a");
    let yb = parse("y-b");
    let (q1, r1) = (&py - &py.replace_with_poly(1, &parse("a"))).quot_rem(&ya, false);
    assert!(r1.is_zero());
    let (q2, r2) = (&q1 - &q1.replace_with_poly(1, &parse("b"))).quot_rem(&yb, false);
    assert!(r2.is_zero());
    let constraints = ["x", "1-x", "y", "1-y", "a-1/32", "1/4-a", "b-3/4", "1-b"]
        .iter()
        .map(|p| parse(p))
        .collect::<Vec<_>>();
    let domain = derive_domain(&alg, &constraints, &[]).unwrap();
    for margin in [&q2 - &parse("1/4"), parse("b-a-1/4")] {
        let cert = certify_nonzero(&alg, &margin, &domain, &CertificateOptions::default())
            .unwrap()
            .expect("closed positive unit certificate");
        verify_certificate(&alg, &margin, &constraints, &[], &cert).unwrap();
        assert_eq!(cert.sign, 1);
    }
    let pa =
        regular::callback::RootProgram::prepare(&lower, OptimizationSettings::default().cores(1))
            .unwrap();
    let pb =
        regular::callback::RootProgram::prepare(&upper, OptimizationSettings::default().cores(1))
            .unwrap();
    let ca = pa.call(ta, lower.coefficients()).unwrap();
    let cb = pb.call(tb, upper.coefficients()).unwrap();
    let axis = middle.axes()[0].symbol();
    let exprs = [
        ca.clone(),
        cb.clone(),
        ca.derivative(axis),
        cb.derivative(axis),
    ];
    let exact = Atom::evaluator_multiple(&exprs, &[Atom::var(axis)])
        .build()
        .unwrap();
    let mut scope = regular::callback::Scope::default();
    scope.insert(ta, pa).unwrap();
    scope.insert(tb, pb).unwrap();
    let mut eval = scope.enter(53, || exact.map_coeff(&|c| c.re.to_f64()));
    for ix in [0, 1, 4, 7, 8] {
        let x = ix as f64 / 8.;
        let mut out = [0.; 4];
        regular::callback::attempt(|| eval.evaluate(&[x], &mut out)).unwrap();
        for j in 0..2 {
            let r = out[j];
            assert!((r.powi(6) - r + (1. + x) / 16.).abs() < 2e-14);
            assert!((out[j + 2] + 1. / (16. * (6. * r.powi(5) - 1.))).abs() < 2e-13);
        }
        assert!(out[1] - out[0] > 0.5);
        let fiber = a
            .specialize_univariate(lower.polynomial(), 1, &[Rational::from((ix, 8))])
            .unwrap();
        let mut roots = symgcad::roots::isolate_union(vec![(0, fiber)]).unwrap();
        assert_eq!(roots.len(), 2);
        for (root, value) in roots.iter_mut().zip(&out[..2]) {
            for _ in 0..64 {
                symgcad::roots::refine_once(root);
            }
            assert!(*value >= root.interval.0.to_f64() - 2e-14);
            assert!(*value <= root.interval.1.to_f64() + 2e-14);
        }
    }
}
