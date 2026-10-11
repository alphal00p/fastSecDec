use super::super::quasiordinary::{self, InputPolynomial, Preparation, QuasiOrdinaryPolynomial};
use super::*;
use symbolica::symbol;
// Explicit fixture budget accommodates existing conservative full-Jacobian
// bounds across two nested factor frames; production defaults are unchanged.
fn budget() -> Budget {
    Budget::new(Limits {
        max_operations: 10_000_000,
        max_total_ideal_slots: 10_000_000,
        max_terms: 1_000_000_000_000,
        ..Limits::default()
    })
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "aj_control", Default::default()).unwrap())
        .unwrap()
}
fn fixture(
    formula: &str,
    guard: Option<&str>,
    b: &mut Budget,
) -> (Arc<QuasiOrdinaryPolynomial>, Arc<GermPoint>) {
    let x = symbol!("aj_control::x");
    let u = symbol!("aj_control::u");
    let z = symbol!("aj_control::z");
    let r = Arc::new(Ring::new(vec![x, u], vec![]).unwrap());
    let guards = guard
        .map(|h| {
            vec![Guard {
                factor: p(&r, h),
                inverse_axis: 1,
            }]
        })
        .unwrap_or_default();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], b).unwrap(),
        vec![0],
        guards,
        b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    );
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(
        frame.clone(),
        vec![InitialDivisor {
            id: BoundaryId(0),
            equation: r.coordinate(0).unwrap(),
        }],
        b,
    )
    .unwrap() else {
        panic!("SNC")
    };
    let a = Atom::parse(formula, "aj_control", Default::default()).unwrap();
    let raw = a
        .as_view()
        .to_polynomial::<_, u16>(&Q, Some(Arc::new(vec![z.into(), x.into(), u.into()])))
        .to_univariate(0);
    let coeff = raw
        .coefficients()
        .iter()
        .map(|p| r.atom(&p.to_expression()).unwrap())
        .collect();
    let input = InputPolynomial::new(frame.clone(), z, coeff, b).unwrap();
    let Preparation::Admitted(q) =
        quasiordinary::prepare(input, ledger, 64, "aj_initial", b).unwrap()
    else {
        panic!("quasiordinary")
    };
    let inv = guard
        .map(|h| {
            let v = p(&r, h).evaluate_with_coeff_map(
                Clone::clone,
                &[Rational::zero(), Rational::zero()],
                &Q,
            );
            Atom::num(Rational::one() / v)
        })
        .unwrap_or_else(|| Atom::num(0));
    let point = GermPoint::verify(frame, vec![Atom::num(0), inv], b).unwrap();
    (q, point)
}
fn run(mut state: AjRecursion, b: &mut Budget) -> AjRecursion {
    for _ in 0..64 {
        match state.advance(b).unwrap() {
            Advance::Complete => return state,
            Advance::Progress => {}
            Advance::ResourceIncomplete { reason } => panic!("resource: {reason}"),
            Advance::CoordinatePreparationRequired => panic!("coordinates"),
            Advance::MonomialGeneratorUnresolved => panic!("valuation"),
            Advance::DiscriminantUnitUnresolved => panic!("discriminant"),
        }
    }
    panic!("degree descent did not terminate")
}
#[test]
fn aj_constructs_fifth_ramification_and_keeps_nonbinomial_bulk() {
    for (i, f) in ["z^5-x", "z^5-x*(1+x)", "(z-(1+x))^5-x*(1+x)"]
        .into_iter()
        .enumerate()
    {
        let mut b = budget();
        let (q, point) = fixture(f, Some("1+x"), &mut b);
        let state = run(
            AjRecursion::new(q, point, &format!("aj_auto5_{i}")).unwrap(),
            &mut b,
        );
        assert_eq!(state.roots().len(), 1);
        let root = &state.roots()[0];
        assert_eq!(root.steps().len(), 1);
        assert_eq!(root.steps()[0].change().exponents(), &[5]);
        assert_eq!(root.steps()[0].source_degree(), 5);
        assert_eq!(root.steps()[0].child_degree(), 1);
        assert_eq!(root.steps()[0].clusters().nonreal_multiplicity(), 4);
        assert_eq!(root.steps()[0].factor().factors().1.len(), 5);
        if i > 0 {
            assert!(
                root.point()
                    .frame()
                    .local()
                    .ideal()
                    .generators()
                    .iter()
                    .any(|p| p.nterms() > 2)
            );
        }
        assert!(state.pending_count() == 0);
    }
}
#[test]
fn aj_signed_square_retains_both_real_lifts_and_negative_source() {
    let mut b = budget();
    let (q, point) = fixture("z^2-x", None, &mut b);
    let state = run(AjRecursion::new(q, point, "aj_square").unwrap(), &mut b);
    assert_eq!(state.roots().len(), 2);
    for r in state.roots() {
        assert_eq!(r.steps()[0].change().exponents(), &[2]);
        assert_eq!(r.steps()[0].change().signs(), &[1]);
    }
    assert_eq!(state.nonreal_fibers().len(), 1);
    assert_eq!(state.nonreal_fibers()[0].change.signs(), &[-1]);
    assert_eq!(state.nonreal_fibers()[0].clusters.nonreal_multiplicity(), 2);
}
#[test]
fn aj_actual_repeated_cluster_recurses_and_resource_keeps_frontier() {
    let mut b = budget();
    let (q, point) = fixture("(z^2-x)*(z-1)", Some("1-x"), &mut b);
    let mut state = AjRecursion::new(q, point, "aj_two_levels").unwrap();
    let mut empty = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        state.advance(&mut empty).unwrap(),
        Advance::ResourceIncomplete { .. }
    ));
    assert_eq!(state.pending_count(), 1);
    assert_eq!(state.accepted_steps(), 0);
    let state = run(state, &mut b);
    assert_eq!(state.roots().len(), 3);
    assert_eq!(
        state
            .roots()
            .iter()
            .filter(|r| r.steps().len() == 2)
            .count(),
        2
    );
    for r in state.roots().iter().filter(|r| r.steps().len() == 2) {
        assert!(matches!(
            r.steps()[0].factor().as_ref(),
            Factorization::Coprime(_)
        ));
        assert!(matches!(
            r.steps()[1].factor().as_ref(),
            Factorization::Simple(_)
        ));
        assert_eq!(
            (r.steps()[0].source_degree(), r.steps()[0].child_degree()),
            (3, 2)
        );
        assert_eq!(
            (r.steps()[1].source_degree(), r.steps()[1].child_degree()),
            (2, 1)
        );
    }
}
#[test]
fn aj_guarded_native_factor_uses_cleared_rows() {
    let mut b = budget();
    let (q, point) = fixture("z^2-x", Some("1+x"), &mut b);
    let r = point.frame().local().ring();
    let source = MonicPolynomial::from_regular_coefficients(
        point.frame().clone(),
        symbol!("aj_control::z"),
        vec![-r.coordinate(1).unwrap(), r.one().zero(), r.one()],
        &mut b,
    )
    .unwrap();
    let clusters = discover_clusters(source, point.clone(), &mut b).unwrap();
    assert_eq!(clusters.real().len(), 2);
    for (i, c) in clusters.real().iter().enumerate() {
        let seed = c.seed().unwrap().clone();
        let fresh = (0..3)
            .map(|j| symbol!(format!("aj_guarded_{i}::s{j}")))
            .collect::<Vec<_>>();
        let branch = FactorBranch::prepare(seed, &fresh, &mut b).unwrap();
        assert!(
            branch
                .coefficient_clearings()
                .iter()
                .any(|c| c.denominator != r.one())
        );
        assert!(
            branch
                .frame()
                .local()
                .zero(
                    &(branch.frame().determinant()
                        - &b.mul(branch.row_scale(), branch.product_minor()).unwrap()),
                    &mut b
                )
                .unwrap()
        );
        let _ = GermPoint::after_factor(&branch, &mut b).unwrap();
    }
    assert_eq!(q.prepared().monic().degree(), 2);
}

#[test]
fn aj_native_sparse_jacobian_preflight_and_dense_refusal() {
    fn frame(
        formulas: &[&str],
        dependent: usize,
        limit: usize,
    ) -> std::result::Result<EtaleFrame, Error> {
        let names = (0..dependent + 2)
            .map(|i| symbol!(format!("aj_sparse::v{i}")))
            .collect::<Vec<_>>();
        let ring = Arc::new(Ring::new(names, vec![]).unwrap());
        let mut b = Budget::new(Limits {
            max_terms: limit,
            ..Limits::default()
        });
        let equations = formulas
            .iter()
            .map(|s| {
                ring.atom(&Atom::parse(s, "aj_sparse", Default::default()).unwrap())
                    .unwrap()
            })
            .collect();
        let local = LocalizedAlgebra::new(
            Ideal::new(ring, equations, &mut b)?,
            (0..=dependent).collect(),
            vec![],
            &mut b,
        )?;
        EtaleCertificate {
            source: local,
            equations: (0..dependent).collect(),
            dependent_axes: (1..=dependent).collect(),
            free_axes: vec![0],
            determinant_inverse_axis: Some(dependent + 1),
        }
        .verify(&mut b)
    }
    // Old dense bound: 5!*2^5=3840 >128. Native Jacobian is I, and
    // each Cramer numerator is bounded by the new 2^5=32 row count.
    let sparse = frame(
        &["v1-v0", "v2-v0^2", "v3-v0^3", "v4-v0^4", "v5-v0^5"],
        5,
        128,
    )
    .unwrap();
    assert_eq!(sparse.determinant(), &sparse.local().ring().one());
    let dense = ["v1+v2+v3-v0", "v1+2*v2+3*v3-v0", "v1+3*v2+2*v3-v0"];
    assert!(matches!(
        frame(&dense, 3, 63),
        Err(Error::ResourceIncomplete("Jacobian determinant bound"))
    ));
    let dense = frame(&dense, 3, 64).unwrap();
    assert!(!dense.determinant().is_zero());
    assert!(matches!(
        frame(&["v0"], 1, 64),
        Err(Error::Invalid("singular selected relative minor"))
    ));
}

#[test]
fn aj_regular_quintic_retains_complete_native_bulk() {
    let mut b = budget();
    let (q, point) = fixture("z^5-z-x", Some("3125*x^4-256"), &mut b);
    let state = run(AjRecursion::new(q, point, "aj_regular5").unwrap(), &mut b);
    assert_eq!(state.roots().len(), 3);
    assert!(
        state
            .roots()
            .iter()
            .all(|r| r.steps().len() == 1 && r.steps()[0].change().exponents() == [1])
    );
    assert!(
        state
            .roots()
            .iter()
            .all(|r| r.original_coefficients().len() == 6)
    );
}
#[test]
fn aj_nested_algebraic_constants_preserve_parameter_roles() {
    let mut b = budget();
    let (x, y, v, p, z) = symbol!(
        "aj_nested::x",
        "aj_nested::y",
        "aj_nested::v",
        "aj_nested::p",
        "aj_nested::z"
    );
    let ring = Arc::new(Ring::new(vec![x, y, v, p], vec![3]).unwrap());
    let relation = ring
        .atom(&Atom::parse("y^2-2", "aj_nested", Default::default()).unwrap())
        .unwrap();
    let local = LocalizedAlgebra::new(
        Ideal::new(ring.clone(), vec![relation], &mut b).unwrap(),
        vec![0, 1],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![1],
            free_axes: vec![0],
            determinant_inverse_axis: Some(2),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(
        frame.clone(),
        vec![InitialDivisor {
            id: BoundaryId(0),
            equation: ring.coordinate(0).unwrap(),
        }],
        &mut b,
    )
    .unwrap() else {
        panic!()
    };
    let constant = ring
        .atom(&Atom::parse("-(1+y)*x", "aj_nested", Default::default()).unwrap())
        .unwrap();
    let mut coefficients = vec![constant];
    coefficients.extend((0..4).map(|_| ring.one().zero()));
    coefficients.push(ring.one());
    let input = InputPolynomial::new(frame.clone(), z, coefficients, &mut b).unwrap();
    let Preparation::Admitted(q) =
        quasiordinary::prepare(input, ledger, 64, "aj_nested_initial", &mut b).unwrap()
    else {
        panic!()
    };
    let sqrt = Atom::num(2).pow((1, 2));
    let point = GermPoint::verify(
        frame,
        vec![
            Atom::num(0),
            sqrt.clone(),
            Atom::num((1, 2)) / &sqrt,
            Atom::num(7),
        ],
        &mut b,
    )
    .unwrap();
    let state = run(AjRecursion::new(q, point, "aj_nested5").unwrap(), &mut b);
    assert_eq!(state.roots().len(), 1);
    let r = &state.roots()[0];
    assert_eq!(r.steps()[0].change().exponents(), &[5]);
    assert_eq!(r.point().context().field().poly().degree(0), 10);
    assert!(r.point().frame().local().ring().is_parameter(3));
    let parameter = r.point().frame().local().ring().coordinate(3).unwrap();
    for i in 0..r.point().frame().free_axes().len() {
        assert!(
            r.point()
                .frame()
                .derivative(i, &parameter, &mut b)
                .unwrap()
                .is_zero()
        );
    }
}

#[test]
fn aj_simple_root_guarded_derivative_uses_original_base_only() {
    let mut b = budget();
    let (_, point) = fixture("z^2-x", Some("1+x"), &mut b);
    let ring = point.frame().local().ring();
    let source = MonicPolynomial::from_regular_coefficients(
        point.frame().clone(),
        symbol!("aj_control::z"),
        vec![-ring.one(), ring.coordinate(1).unwrap(), ring.one()],
        &mut b,
    )
    .unwrap();
    let clusters = discover_clusters(source, point.clone(), &mut b).unwrap();
    for (i, c) in clusters.real().iter().enumerate() {
        let f = SimpleRootBranch::prepare(
            c.seed().unwrap().clone(),
            &[
                symbol!(format!("aj_simple_guard{i}::a")),
                symbol!(format!("aj_simple_guard{i}::v")),
            ],
            &mut b,
        )
        .unwrap();
        let axis = ring.len();
        let rhs = b.mul(f.row_scale(), f.derivative()).unwrap();
        let diff = &f.equation().derivative(axis) - &rhs;
        assert!(
            !diff.is_zero(),
            "the guard relation must be needed by this control"
        );
        let ideal = f
            .extension()
            .ideal(point.frame().local().ideal(), &mut b)
            .unwrap();
        let units = point
            .frame()
            .local()
            .unit_relations()
            .iter()
            .map(|p| f.extension().pull(p, &mut b).unwrap())
            .collect::<Vec<_>>();
        assert!(ideal.contains(&diff, &units, &mut b).unwrap());
        assert!(
            !ideal
                .contains(&(&diff + &diff.one()), &units, &mut b)
                .unwrap()
        );
        assert!(
            f.point()
                .frame()
                .local()
                .zero(f.equation(), &mut b)
                .unwrap()
        );
    }
}
