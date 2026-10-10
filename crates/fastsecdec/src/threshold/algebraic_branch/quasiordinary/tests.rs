use super::*;
use crate::threshold::algebraic_branch::{FactorBranch, FactorSeed, MonicPolynomial};
use crate::threshold::resolution::*;
use symbolica::{
    atom::{Atom, Symbol},
    domains::rational::Rational,
    parse, symbol,
};
fn b() -> Budget {
    Budget::new(Limits::default())
}
fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("qo_owner::p"),
                symbol!("qo_owner::x"),
                symbol!("qo_owner::y"),
                symbol!("qo_owner::u"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn affine(r: Arc<Ring>, guards: Vec<Guard>, budget: &mut Budget) -> Arc<EtaleFrame> {
    let local = LocalizedAlgebra::new(
        Ideal::new(r, vec![], budget).unwrap(),
        vec![1, 2],
        guards,
        budget,
    )
    .unwrap();
    Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![1, 2],
            determinant_inverse_axis: None,
        }
        .verify(budget)
        .unwrap(),
    )
}
fn snc(
    frame: Arc<EtaleFrame>,
    factors: Vec<Poly>,
    budget: &mut Budget,
) -> Arc<VerifiedRelativeSnc> {
    match verify_initial_relative_snc(
        frame,
        factors
            .into_iter()
            .enumerate()
            .map(|(i, equation)| InitialDivisor {
                id: BoundaryId(i as u64),
                equation,
            })
            .collect(),
        budget,
    )
    .unwrap()
    {
        SncProduction::Verified(s) => s,
        other => panic!("{other:?}"),
    }
}
fn source(
    frame: Arc<EtaleFrame>,
    coefficients: Vec<Poly>,
    budget: &mut Budget,
) -> Arc<InputPolynomial> {
    InputPolynomial::new(frame, symbol!("qo_owner::z"), coefficients, budget).unwrap()
}
fn accepted(p: Preparation) -> Arc<QuasiOrdinaryPolynomial> {
    match p {
        Preparation::Admitted(a) => a,
        other => panic!("{other:?}"),
    }
}
#[test]
fn quintic_and_crossing_divisor_certificates_retain_exact_coefficients() {
    let mut budget = b();
    let r = ring();
    let frame = affine(r.clone(), vec![], &mut budget);
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let mut c = vec![-x.clone()];
    c.extend((1..5).map(|_| r.one().zero()));
    c.push(r.one());
    let input = source(frame.clone(), c, &mut budget);
    let ledger = snc(frame.clone(), vec![x.clone()], &mut budget);
    let q = accepted(
        prepare(
            input.clone(),
            ledger.clone(),
            8,
            "qo_owner::q5",
            &mut budget,
        )
        .unwrap(),
    );
    assert!(Arc::ptr_eq(q.prepared().cleared().source(), &input));
    assert!(Arc::ptr_eq(q.ledger(), &ledger));
    assert_eq!(
        q.discriminant(),
        &r.atom(&parse!("3125*qo_owner::x^4")).unwrap()
    );
    assert_eq!(q.factorization().progress().powers, vec![4]);
    assert_eq!(q.prepared().monic().coefficients(), input.coefficients());
    let input = source(
        frame.clone(),
        vec![-(&x * &y), r.one().zero(), r.one()],
        &mut budget,
    );
    let ledger = snc(frame, vec![x, y], &mut budget);
    let q = accepted(prepare(input, ledger, 4, "qo_owner::q2", &mut budget).unwrap());
    assert_eq!(q.factorization().progress().powers, vec![1, 1]);
    assert_eq!(
        q.discriminant(),
        &r.atom(&parse!("4*qo_owner::x*qo_owner::y")).unwrap()
    );
}
#[test]
fn original_guard_required_for_residual_and_leading_units() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let h = &x + &r.one();
    for guarded in [false, true] {
        let guards = if guarded {
            vec![Guard {
                factor: h.clone(),
                inverse_axis: 3,
            }]
        } else {
            vec![]
        };
        let f = affine(r.clone(), guards, &mut budget);
        let s = snc(f.clone(), vec![x.clone()], &mut budget);
        let p = source(
            f.clone(),
            vec![-(&x * &h), r.one().zero(), r.one()],
            &mut budget,
        );
        let outcome = prepare(p, s.clone(), 4, "qo_owner::unit", &mut budget).unwrap();
        assert_eq!(matches!(outcome, Preparation::Admitted(_)), guarded);
        if !guarded {
            assert!(matches!(
                outcome,
                Preparation::ResidualNeedsRefinement { .. }
            ));
        }
        let p = source(f, vec![-x.clone(), r.one().zero(), h.clone()], &mut budget);
        let outcome = prepare(p, s, 4, "qo_owner::leading", &mut budget).unwrap();
        assert_eq!(matches!(outcome, Preparation::Admitted(_)), guarded);
        if !guarded {
            assert!(matches!(outcome, Preparation::LeadingStratum { .. }));
        }
    }
}
#[test]
fn guarded_coefficient_clearing_keeps_original_inverse_role() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let h = &x + &r.one();
    let u = r.coordinate(3).unwrap();
    let frame = affine(
        r.clone(),
        vec![Guard {
            factor: h.clone(),
            inverse_axis: 3,
        }],
        &mut budget,
    );
    let s = snc(frame.clone(), vec![x.clone()], &mut budget);
    let p = source(
        frame.clone(),
        vec![-(&x * &u), r.one().zero(), h.clone()],
        &mut budget,
    );
    let q = accepted(prepare(p.clone(), s, 8, "qo_owner::clearing", &mut budget).unwrap());
    assert_eq!(
        q.prepared().cleared().clearings()[0].original,
        p.coefficients()[0]
    );
    assert_eq!(q.prepared().cleared().common_denominator(), &h);
    assert_eq!(q.prepared().root_scale(), &h.pow(2));
    assert_eq!(q.prepared().monic().coefficients()[0], -(&x * &h.pow(2)));
    assert_eq!(q.factorization().progress().powers, vec![1]);
    assert_eq!(q.prepared().monic().frame().free_axes(), frame.free_axes());
    assert!(q.prepared().monic().frame().local().ring().is_parameter(0));
    // All original source coefficients survive, including the guard slot.
    assert!(p.coefficients()[0].degree(3) > 0);
    assert!(
        q.prepared()
            .monic()
            .coefficients()
            .iter()
            .all(|p| p.degree(3) == 0)
    );
}
#[test]
fn nonlinear_base_repetition_is_checked_modulo_original_relations() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let relation = &y.pow(2) - &x;
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![relation], &mut budget).unwrap(),
        vec![1, 2],
        vec![],
        &mut budget,
    )
    .unwrap();
    let f = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![2],
            free_axes: vec![1],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut budget)
        .unwrap(),
    );
    let s = snc(f.clone(), vec![], &mut budget);
    let input = source(
        f,
        vec![x, -(&y * &r.atom(&Atom::num(2)).unwrap()), r.one()],
        &mut budget,
    );
    match prepare(input, s, 4, "qo_owner::repeated", &mut budget).unwrap() {
        Preparation::Repeated {
            prepared,
            discriminant,
            ..
        } => {
            assert!(!discriminant.is_zero());
            assert!(
                prepared
                    .monic()
                    .frame()
                    .local()
                    .zero(&discriminant, &mut budget)
                    .unwrap()
            );
        }
        other => panic!("{other:?}"),
    }
}
#[test]
fn disconnected_multiplicities_do_not_create_a_global_power_vector() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let e = r.coordinate(2).unwrap();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![&e.pow(2) - &e], &mut budget).unwrap(),
        vec![1, 2],
        vec![],
        &mut budget,
    )
    .unwrap();
    let f = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![2],
            free_axes: vec![1],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut budget)
        .unwrap(),
    );
    let s = snc(f.clone(), vec![x.clone()], &mut budget);
    let c = -(&(&e * &x.pow(2)) + &(&(&r.one() - &e) * &x));
    let p = source(f, vec![c, r.one().zero(), r.one()], &mut budget);
    match prepare(p, s, 4, "qo_owner::components", &mut budget).unwrap() {
        Preparation::ResidualNeedsRefinement { factors, .. } => {
            assert_eq!(factors.progress().powers, vec![1])
        }
        other => panic!("{other:?}"),
    }
}
#[test]
fn owner_and_resource_refusals_preserve_partial_evidence() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let f = affine(r.clone(), vec![], &mut budget);
    let other = affine(r.clone(), vec![], &mut budget);
    let s = snc(f.clone(), vec![x.clone()], &mut budget);
    let wrong = snc(other, vec![x.clone()], &mut budget);
    let mut c = vec![-x];
    c.extend((1..5).map(|_| r.one().zero()));
    c.push(r.one());
    let p = source(f, c, &mut budget);
    assert!(matches!(
        prepare(p.clone(), wrong, 8, "qo_owner::wrong", &mut budget),
        Err(Error::Invalid(_))
    ));
    let mut tight = Budget::new(Limits {
        max_terms: 2,
        ..Default::default()
    });
    match prepare(p.clone(), s.clone(), 8, "qo_owner::budget", &mut tight).unwrap() {
        Preparation::Incomplete {
            source, progress, ..
        } => {
            assert!(Arc::ptr_eq(&source, &p));
            assert!(progress.monicization.is_some());
            assert!(progress.discriminant.is_none());
        }
        other => panic!("{other:?}"),
    }
    match prepare(p, s, 2, "qo_owner::power", &mut budget).unwrap() {
        Preparation::Incomplete { progress, .. } => {
            assert!(progress.discriminant.is_some());
            assert!(progress.pending_factors.is_some());
        }
        other => panic!("{other:?}"),
    }
}
#[test]
fn nonbinomial_quintic_guarded_family_is_admitted() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let h = &x + &r.one();
    let f = affine(
        r.clone(),
        vec![Guard {
            factor: h.clone(),
            inverse_axis: 3,
        }],
        &mut budget,
    );
    let s = snc(f.clone(), vec![x.clone()], &mut budget);
    let mut c = vec![-(&x * &h)];
    c.extend((1..5).map(|_| r.one().zero()));
    c.push(r.one());
    let q = accepted(
        prepare(
            source(f, c, &mut budget),
            s,
            8,
            "qo_owner::nonbinomial",
            &mut budget,
        )
        .unwrap(),
    );
    assert_eq!(q.factorization().progress().powers, vec![4]);
    assert_eq!(
        q.discriminant(),
        &r.atom(&parse!("3125*qo_owner::x^4*(1+qo_owner::x)^4"))
            .unwrap()
    );
}
#[test]
fn supplied_ramification_then_native_coprime_germ_keeps_exact_bulk() {
    // NEXT-stage feasibility only: this test supplies x=t^5 explicitly.
    // It does not claim automatic ramification selection or a closed real atlas.
    let mut budget = b();
    let t = symbol!("qo_ram::t");
    let z = symbol!("qo_ram::z");
    let r = Arc::new(Ring::new(vec![t], vec![]).unwrap());
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut budget).unwrap(),
        vec![0],
        vec![],
        &mut budget,
    )
    .unwrap();
    let f = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0],
            determinant_inverse_axis: None,
        }
        .verify(&mut budget)
        .unwrap(),
    );
    let tp = r.coordinate(0).unwrap();
    let mut c = vec![-(&r.one() + &tp.pow(5))];
    c.extend((1..5).map(|_| r.one().zero()));
    c.push(r.one());
    let p = MonicPolynomial::new(f, z, c, &mut budget).unwrap();
    let seed = FactorSeed::verify(
        p,
        vec![Atom::Zero],
        vec![Atom::num(-1), Atom::one()],
        vec![Atom::one(); 5],
        &mut budget,
    )
    .unwrap();
    let fresh = (0..6)
        .map(|i| symbol!(format!("qo_ram::a{i}")))
        .collect::<Vec<Symbol>>();
    let branch = FactorBranch::prepare(seed, &fresh, &mut budget).unwrap();
    let local = branch.frame().local();
    let t = branch.extension().pull(&tp, &mut budget).unwrap();
    let y = -branch.factors().0[0].clone();
    let z = &t * &y;
    assert!(
        local
            .zero(
                &(&z.pow(5) - &(&t.pow(5) * &(&local.ring().one() + &t.pow(5)))),
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(
        branch.jet(false, 0, &[0], &mut budget).unwrap().at_fiber(),
        &branch.seed().context().field().constant(Rational::from(0))
    );
    assert_eq!(
        branch.seed().resultant(),
        &branch.seed().context().field().constant(Rational::from(5))
    );
}
#[test]
fn genuine_nonbinomial_quintic_and_native_selected_germ() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let disc = r.atom(&parse!("3125*qo_owner::x^4-256")).unwrap();
    let f = affine(
        r.clone(),
        vec![Guard {
            factor: disc.clone(),
            inverse_axis: 3,
        }],
        &mut budget,
    );
    let s = snc(f.clone(), vec![], &mut budget);
    let p = source(
        f,
        vec![
            -x.clone(),
            -r.one(),
            r.one().zero(),
            r.one().zero(),
            r.one().zero(),
            r.one(),
        ],
        &mut budget,
    );
    let q = accepted(prepare(p, s, 8, "qo_owner::genuine", &mut budget).unwrap());
    assert_eq!(q.discriminant(), &disc);
    assert!(q.factorization().progress().powers.is_empty());
    let seed = FactorSeed::verify(
        q.prepared().monic().clone(),
        vec![
            Atom::Zero,
            Atom::Zero,
            Atom::Zero,
            Atom::num(Rational::from((-1, 256))),
        ],
        vec![Atom::num(-1), Atom::one()],
        vec![
            Atom::Zero,
            Atom::one(),
            Atom::one(),
            Atom::one(),
            Atom::one(),
        ],
        &mut budget,
    )
    .unwrap();
    let fresh = (0..6)
        .map(|i| symbol!(format!("qo_nonbin::a{i}")))
        .collect::<Vec<_>>();
    let germ = FactorBranch::prepare(seed, &fresh, &mut budget).unwrap();
    let local = germ.frame().local();
    let x = germ.extension().pull(&x, &mut budget).unwrap();
    let root = -germ.factors().0[0].clone();
    assert!(
        local
            .zero(&(&(&root.pow(5) - &root) - &x), &mut budget)
            .unwrap()
    );
    assert_eq!(
        germ.jet(false, 0, &[0], &mut budget).unwrap().at_fiber(),
        &germ
            .seed()
            .context()
            .field()
            .constant(Rational::from((-1, 4)))
    );
}
#[test]
fn guard_conversion_preflight_refuses_before_native_expansion() {
    let mut budget = b();
    let r = ring();
    let x = r.coordinate(1).unwrap();
    let y = r.coordinate(2).unwrap();
    let u = r.coordinate(3).unwrap();
    let h = &(&r.one() + &x) + &y;
    let f = affine(
        r.clone(),
        vec![Guard {
            factor: h,
            inverse_axis: 3,
        }],
        &mut budget,
    );
    let s = snc(f.clone(), vec![], &mut budget);
    let p = source(f, vec![u.pow(256), r.one().zero(), r.one()], &mut budget);
    match prepare(p, s, 4, "qo_owner::guard_bound", &mut budget).unwrap() {
        Preparation::Incomplete {
            progress, reason, ..
        } => {
            assert!(reason.contains("guard clearing"));
            assert!(progress.clearings.is_empty());
            assert!(progress.cleared.is_none());
        }
        other => panic!("{other:?}"),
    }
}
