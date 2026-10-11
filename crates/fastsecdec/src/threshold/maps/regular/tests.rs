use super::*;
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
use symbolica::{
    atom::{Atom, AtomCore},
    domains::rational::Rational,
    symbol,
};
fn maps(degree: i64) -> Vec<Arc<CellMap>> {
    let (x, y, e, s, t) = symbol!(
        &format!("regular_owner_{degree}::x"),
        &format!("regular_owner_{degree}::y"),
        &format!("regular_owner_{degree}::eps"),
        &format!("regular_owner_{degree}::s"),
        &format!("regular_owner_{degree}::t")
    );
    let input = ParametricIntegrand::new(
        vec![x, y],
        e,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::zero(); 2],
            vec![
                PolynomialFactor::new(
                    Atom::var(y).pow(degree) + Atom::var(y) - Atom::var(x),
                    -Atom::one() - Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let mut l = GcadRequest::default_limits();
    l.wall_time_secs = 30.;
    l.memory_mib = 1024;
    let owner = Arc::new(
        GcadRequest::unit_cube(
            &input,
            GcadKinematics::default(),
            SolverOptions {
                order: vec!["v0".into(), "v1".into()],
                ..Default::default()
            },
            l,
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    (0..owner.cells().len())
        .map(|i| CellMap::new(owner.clone(), i, vec![s, t]).unwrap())
        .collect()
}
fn proposal() -> BracketProposal {
    BracketProposal {
        lower: Rational::zero(),
        upper: Rational::from((4, 5)),
        derivative_margin: Rational::from((1, 2)),
    }
}
#[test]
fn actual_native_sections_generic_degree_and_closed_margin() {
    for degree in [3, 5] {
        let maps = maps(degree);
        assert_eq!(maps.len(), 2);
        let mut count = 0;
        for map in maps {
            for side in [SectionSide::Lower, SectionSide::Upper] {
                let raw = map.decomposition().native_result();
                let bound = match side {
                    SectionSide::Lower => map.native_axes()[1].lower.as_ref(),
                    SectionSide::Upper => map.native_axes()[1].upper.as_ref(),
                }
                .unwrap();
                let a = symgcad::algebra::Algebra::new(&raw.order).unwrap();
                if a.parse(&raw.polynomials[bound.polynomial])
                    .unwrap()
                    .degree(1)
                    <= 1
                {
                    continue;
                }
                let section = RegularSection::prepare(
                    map.clone(),
                    1,
                    side,
                    proposal(),
                    &mut Budget::new(Limits::default()),
                )
                .unwrap();
                assert_eq!(section.polynomial().degree(1), degree as u32);
                assert_eq!(section.coefficients().len(), degree as usize + 1);
                assert!(Arc::ptr_eq(section.source(), &map));
                assert_eq!(section.axis(), 1);
                assert_eq!(section.side(), side);
                assert_eq!(section.bound(), bound);
                assert!(!section.equation().is_zero());
                assert_eq!(section.orientation(), -1);
                assert_eq!(section.derivative_certificate().sign, 1);
                assert!(section.endpoint_certificates().iter().all(Option::is_some));
                assert!(section.anchor().interval.0 >= section.bracket().lower);
                assert_eq!(section.original_domain().variables.len(), 2);
                assert_eq!(section.cylinder().bounds[1].upper.as_deref(), Some("4/5"));
                symgcad::domain::verify_certificate(
                    &a,
                    &a.parse(&section.derivative_certificate().polynomial)
                        .unwrap(),
                    section.cylinder_constraints(),
                    &[],
                    section.derivative_certificate(),
                )
                .unwrap();
                assert_eq!(section.original_constraints().len(), 4);
                count += 1;
            }
        }
        assert_eq!(count, 2);
    }
}
#[test]
fn proposal_domain_resource_and_cancellation_refusals() {
    let map = maps(5).remove(0);
    let side = SectionSide::Upper;
    let mut p = proposal();
    p.upper = Rational::from((1, 2));
    assert!(matches!(
        RegularSection::prepare(map.clone(), 1, side, p, &mut Budget::new(Limits::default())),
        Err(Error::Unsupported(_))
    ));
    let mut p = proposal();
    p.derivative_margin = Rational::from(2);
    assert!(
        RegularSection::prepare(map.clone(), 1, side, p, &mut Budget::new(Limits::default()))
            .is_err()
    );
    let l = Limits {
        max_degree: 4,
        ..Default::default()
    };
    assert!(matches!(
        RegularSection::prepare(map.clone(), 1, side, proposal(), &mut Budget::new(l)),
        Err(Error::ResourceIncomplete(_))
    ));
    let cancel = Arc::new(AtomicBool::new(true));
    assert!(matches!(
        RegularSection::prepare(
            map,
            1,
            side,
            proposal(),
            &mut Budget::new(Limits::default()).with_cancellation(cancel)
        ),
        Err(Error::Cancelled)
    ));
}
#[test]
fn composed_native_maps_derivatives_precisions_and_owned_clones() {
    use callback::{RootProgram, Scope, attempt};
    use program::CompiledMap;
    use symbolica::{
        domains::float::{Complex, DoubleFloat, Float, RealLike},
        evaluate::OptimizationSettings,
    };
    let maps = maps(5);
    let regular = maps[0].clone();
    let global = BracketProposal {
        lower: Rational::from(-1),
        upper: Rational::from(2),
        derivative_margin: Rational::from((1, 2)),
    };
    for map in maps {
        let raw = map.decomposition().native_result();
        let algebra = symgcad::algebra::Algebra::new(&raw.order).unwrap();
        let proposals = map
            .native_axes()
            .iter()
            .enumerate()
            .map(|(axis, a)| {
                [a.lower.as_ref().unwrap(), a.upper.as_ref().unwrap()].map(|r| {
                    if algebra
                        .parse(&raw.polynomials[r.polynomial])
                        .unwrap()
                        .degree(axis)
                        > 1
                    {
                        proposal()
                    } else {
                        global.clone()
                    }
                })
            })
            .collect::<Vec<_>>();
        let exact = CompiledMap::prepare(
            map.clone(),
            &proposals,
            OptimizationSettings::default().cores(1),
            &mut Budget::new(Limits::default()),
        )
        .unwrap();
        assert!(Arc::ptr_eq(exact.source(), &map));
        assert_eq!(exact.sections().len(), 4);
        assert_eq!(exact.inputs().len(), 2);
        assert_eq!(exact.exact().get_output_len(), 7);
        let mut f = exact.map::<f64>(53);
        assert_eq!(f.sections().len(), 4);
        let mut restored = exact.native_roundtrip().unwrap().map::<f64>(53);
        assert_eq!(
            f.evaluate(&[0.37, 0.22]).unwrap(),
            restored.evaluate(&[0.37, 0.22]).unwrap()
        );
        let mut complex = exact.map_complex::<f64>(53);
        let mut complex_out = vec![Complex::new(0., 0.); 7];
        attempt(|| {
            complex.evaluate(
                &[Complex::new(0.37, 0.), Complex::new(0.22, 0.)],
                &mut complex_out,
            )
        })
        .unwrap();
        for (a, z) in f.evaluate(&[0.37, 0.22]).unwrap().iter().zip(complex_out) {
            assert!((*a - z.re).abs() < 1e-14);
            assert_eq!(z.im, 0.);
        }
        let mut dd = exact.map::<DoubleFloat>(106);
        let mut high = exact.map::<Float>(192);
        for x in [0., 0.25, 0.5, 1.] {
            let a = f.evaluate(&[x, 0.4]).unwrap();
            let d = dd
                .evaluate(&[DoubleFloat::from(x), DoubleFloat::from(0.4)])
                .unwrap();
            let h = high
                .evaluate(&[Float::with_val(192, x), Float::with_val(192, 0.4)])
                .unwrap();
            for i in 0..5 {
                assert!((a[i] - d[i].to_f64()).abs() < 2e-13);
                assert!((a[i] - h[i].to_f64()).abs() < 2e-13);
            }
            assert!((a[0] - x).abs() < 1e-14);
            assert!(a[1] >= 0. && a[1] <= 1.);
            assert_eq!(a[0], a[2]);
            assert_eq!(a[1], a[3]);
        }
        let expected = f.evaluate(&[0.37, 0.22]).unwrap();
        let mut clone = f.clone();
        let thread = std::thread::spawn(move || clone.evaluate(&[0.37, 0.22]).unwrap());
        drop(exact);
        assert_eq!(thread.join().unwrap(), expected);
        assert!(f.evaluate(&[f64::NAN, 0.4]).is_err());
        assert!(f.evaluate(&[1.01, 0.4]).is_err());
    }
    // Native symbolic differentiation differentiates THROUGH the root/minor.
    let section = RegularSection::prepare(
        regular.clone(),
        1,
        SectionSide::Upper,
        proposal(),
        &mut Budget::new(Limits::default()),
    )
    .unwrap();
    let helper = RootProgram::prepare(&section, OptimizationSettings::default().cores(1)).unwrap();
    let tag = symbol!("regular_derivative_owner");
    let call = helper.call(tag, section.coefficients()).unwrap();
    let x = regular.axes()[0].symbol();
    let first = call.derivative(x);
    let second = first.derivative(x);
    let exact = Atom::evaluator_multiple(&[call, first, second], &[Atom::var(x)])
        .build()
        .unwrap();
    let mut scope = Scope::default();
    scope.insert(tag, helper).unwrap();
    let mut evaluator = scope.enter(53, || exact.clone().map_coeff(&|c| c.re.to_f64()));
    let mut out = [0.; 3];
    attempt(|| evaluator.evaluate(&[0.4], &mut out)).unwrap();
    let r = out[0];
    let d = 1. + 5. * r.powi(4);
    assert!((out[1] - 1. / d).abs() < 1e-13);
    assert!((out[2] + 20. * r.powi(3) / d.powi(3)).abs() < 1e-12);
    assert!(attempt(|| evaluator.evaluate(&[2.], &mut out)).is_err());
    // Missing owner is a numerical failure, never a shape-only proof recovery.
    let mut missing = exact.map_coeff(&|c| c.re.to_f64());
    assert!(attempt(|| missing.evaluate(&[0.4], &mut out)).is_err());
}
#[test]
fn nested_generic_sections_retain_every_native_cell() {
    use program::CompiledMap;
    use symbolica::evaluate::OptimizationSettings;
    let (x, y, z, e, s, t, u) = symbol!(
        "regular_nested::x",
        "regular_nested::y",
        "regular_nested::z",
        "regular_nested::eps",
        "regular_nested::s",
        "regular_nested::t",
        "regular_nested::u"
    );
    let input = ParametricIntegrand::new(
        vec![x, y, z],
        e,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::zero(); 3],
            vec![
                PolynomialFactor::new(
                    Atom::var(y).pow(3) + Atom::var(y) - Atom::var(x),
                    -Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::var(z).pow(3) + Atom::var(z) - Atom::var(y),
                    -Atom::var(e),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let mut l = GcadRequest::default_limits();
    l.wall_time_secs = 30.;
    l.memory_mib = 1024;
    let owner = Arc::new(
        GcadRequest::unit_cube(
            &input,
            GcadKinematics::default(),
            SolverOptions {
                order: vec!["v0".into(), "v1".into(), "v2".into()],
                ..Default::default()
            },
            l,
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    let raw = owner.native_result();
    let algebra = symgcad::algebra::Algebra::new(&raw.order).unwrap();
    let global = BracketProposal {
        lower: Rational::from(-1),
        upper: Rational::from(2),
        derivative_margin: Rational::from((1, 2)),
    };
    let mut accepted = 0;
    for (i, cell) in raw.cells.iter().enumerate() {
        let map = CellMap::new(owner.clone(), i, vec![s, t, u]).unwrap();
        let proposals = map
            .native_axes()
            .iter()
            .enumerate()
            .map(|(axis, a)| {
                [a.lower.as_ref().unwrap(), a.upper.as_ref().unwrap()].map(|r| {
                    if algebra
                        .parse(&raw.polynomials[r.polynomial])
                        .unwrap()
                        .degree(axis)
                        > 1
                    {
                        proposal()
                    } else {
                        global.clone()
                    }
                })
            })
            .collect::<Vec<_>>();
        let exact = CompiledMap::prepare(
            map,
            &proposals,
            OptimizationSettings::default().cores(1),
            &mut Budget::new(Limits::default()),
        )
        .unwrap();
        let out = exact.map::<f64>(53).evaluate(&[0.31, 0.43, 0.57]).unwrap();
        let signs = [
            out[1].powi(3) + out[1] - out[0],
            out[2].powi(3) + out[2] - out[1],
        ];
        for (value, sign) in signs.iter().zip(&cell.signs) {
            assert!(*value * f64::from(*sign) > 0.);
        }
        assert!(out[6] > 0.);
        accepted += 1;
    }
    assert_eq!(accepted, raw.cells.len());
    assert!(accepted >= 4);
    println!("all {accepted} native nested cells retained");
}

#[test]
fn identically_zero_endpoint_requires_real_uniform_regular_root() {
    let map = maps(5).remove(0);
    let mut b = Budget::new(Limits::default());
    let section = RegularSection::prepare(
        map.clone(),
        0,
        SectionSide::Lower,
        BracketProposal {
            lower: Rational::zero(),
            upper: Rational::one(),
            derivative_margin: Rational::from((1, 2)),
        },
        &mut b,
    )
    .unwrap();
    assert!(section.endpoint_certificates()[0].is_none());
    assert!(section.anchor().interval.0.is_zero());
    let limits = Limits {
        max_rational_bits: 2,
        ..Default::default()
    };
    assert!(matches!(
        RegularSection::prepare(
            map,
            1,
            SectionSide::Upper,
            proposal(),
            &mut Budget::new(limits)
        ),
        Err(Error::ResourceIncomplete(_))
    ));
}

#[test]
fn duplicate_scope_failure_preserves_original_and_native_substitution_semantics() {
    use callback::{RootProgram, Scope, attempt};
    use symbolica::evaluate::OptimizationSettings;
    let map = maps(5).remove(0);
    let section = RegularSection::prepare(
        map.clone(),
        1,
        SectionSide::Upper,
        proposal(),
        &mut Budget::new(Limits::default()),
    )
    .unwrap();
    let helper = RootProgram::prepare(&section, OptimizationSettings::default().cores(1)).unwrap();
    let other = RegularSection::prepare(
        map.clone(),
        0,
        SectionSide::Lower,
        BracketProposal {
            lower: Rational::from(-1),
            upper: Rational::one(),
            derivative_margin: Rational::from((1, 2)),
        },
        &mut Budget::new(Limits::default()),
    )
    .unwrap();
    let tag = symbol!("regular_duplicate_scope");
    let mut scope = Scope::default();
    scope.insert(tag, helper.clone()).unwrap();
    assert!(
        scope
            .insert(
                tag,
                RootProgram::prepare(&other, OptimizationSettings::default().cores(1)).unwrap()
            )
            .is_err()
    );
    let call = helper.call(tag, section.coefficients()).unwrap();
    let exact = call
        .evaluator(&[Atom::var(map.axes()[0].symbol())])
        .build()
        .unwrap();
    let mut eval = scope.enter(53, || exact.map_coeff(&|c| c.re.to_f64()));
    assert!(
        attempt(|| eval.evaluate_single(&[0.4]))
            .unwrap()
            .is_finite()
    );
    // Native source documents replace(n,value) as coefficient-field evaluation;
    // this executable control also retains the exact original variable map.
    let algebra = symgcad::algebra::Algebra::new(&["x".into(), "y".into()]).unwrap();
    let p = algebra.parse("3*y^5+2*x*y+7*x").unwrap();
    let replaced = p.replace(1, &Rational::from((2, 3)));
    assert_eq!(replaced, algebra.parse("32/81+25*x/3").unwrap());
    assert_eq!(replaced.variables(), p.variables());
}
