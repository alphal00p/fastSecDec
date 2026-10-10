use super::*;
use symbolica::{atom::Atom, symbol};
fn b() -> Budget {
    Budget::new(Limits {
        max_operations: 5_000_000,
        max_total_ideal_slots: 5_000_000,
        ..Limits::default()
    })
}
fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("recursive::x"),
                symbol!("recursive::y"),
                symbol!("recursive::z"),
            ],
            vec![],
        )
        .unwrap(),
    )
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "recursive", Default::default()).unwrap())
        .unwrap()
}
fn flat(r: &Arc<Ring>, axes: Vec<usize>, b: &mut Budget) -> Arc<EtaleFrame> {
    Arc::new(
        EtaleCertificate {
            source: LocalizedAlgebra::new(
                Ideal::new(r.clone(), vec![], b).unwrap(),
                axes.clone(),
                vec![],
                b,
            )
            .unwrap(),
            equations: vec![],
            dependent_axes: vec![],
            free_axes: axes,
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn input(r: &Arc<Ring>, ss: &[&str], mark: usize, b: &mut Budget) -> Arc<MarkedIdeal> {
    Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), ss.iter().map(|s| p(r, s)).collect(), b).unwrap(),
            mark,
            b,
        )
        .unwrap(),
    )
}
fn run(mut state: BoundaryFreeFirstCenter, b: &mut Budget) -> RecursiveOutcome {
    for _ in 0..32 {
        match state.advance(b).unwrap() {
            RecursiveAdvance::Progress => {}
            RecursiveAdvance::Complete => return state.outcome().clone(),
            RecursiveAdvance::Incomplete { reason } => panic!("incomplete: {reason}"),
        }
    }
    panic!("recursive control failed to terminate")
}
fn center(
    r: &Arc<Ring>,
    formulas: &[&str],
    mark: usize,
    axes: Vec<usize>,
    reverse: bool,
    ns: &str,
    b: &mut Budget,
) -> Arc<RecursiveCenter> {
    let state =
        BoundaryFreeFirstCenter::new(flat(r, axes, b), input(r, formulas, mark, b), ns, reverse)
            .unwrap();
    let RecursiveOutcome::Center(c) = run(state, b) else {
        panic!("missing recursive center")
    };
    c
}
fn assert_xy(c: &RecursiveCenter, b: &mut Budget) {
    let expected = Ideal::new(
        c.ideal.ring().clone(),
        vec![p(c.ideal.ring(), "x"), p(c.ideal.ring(), "y")],
        b,
    )
    .unwrap();
    for (a, z) in [(&c.ideal, &expected), (&expected, &c.ideal)] {
        for f in a.generators() {
            assert!(z.contains(f, &[], b).unwrap());
        }
    }
    assert_eq!(c.normals.len(), 2);
    assert_eq!(c.invariant[1], Rational::from(3) / Rational::from(2));
    let receipt = c.lift_receipt().unwrap();
    assert!(Arc::ptr_eq(c.source(), receipt.level().source()));
    let child = receipt.child();
    assert!(Arc::ptr_eq(
        child.frame(),
        receipt.level().contact().contact()
    ));
    assert!(child.lift_receipt().unwrap().child().normals().is_empty());
}
#[test]
fn bm_recursive_cusp_derived_center_and_all_pivots() {
    let r = ring();
    let mut b = b();
    for reverse in [false, true] {
        let c = center(
            &r,
            &["y^2-x^3"],
            2,
            vec![0, 1],
            reverse,
            if reverse {
                "recursive_reverse"
            } else {
                "recursive_forward"
            },
            &mut b,
        );
        assert_xy(&c, &mut b);
        assert_eq!(c.invariant[0], Rational::from(1));
        let FirstBlowup::MarkedResolved { charts, .. } = first_coordinate_blowup(
            c,
            if reverse {
                "blow_reverse"
            } else {
                "blow_forward"
            },
            &mut b,
        )
        .unwrap() else {
            panic!("marked cusp unresolved")
        };
        assert_eq!(charts.len(), 2);
        assert_eq!(
            charts
                .iter()
                .map(|c| c.pivot_source_axis())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        for chart in charts {
            assert!(chart.marked_cosupport_empty());
            let t = chart.transform();
            for (f, q) in t
                .adapted_source()
                .ideal()
                .generators()
                .iter()
                .zip(t.generator_quotients())
            {
                assert_eq!(
                    t.map().pull(f, &mut b).unwrap(),
                    b.mul(t.exceptional_factor(), q).unwrap()
                );
            }
        }
    }
}
#[test]
fn bm_recursive_suspended_nonprincipal_and_mark_one_retention() {
    let r = ring();
    let mut b = b();
    let c = center(
        &r,
        &["y^2-x^3", "y^3"],
        2,
        vec![0, 1, 2],
        false,
        "recursive_nonprincipal",
        &mut b,
    );
    assert_xy(&c, &mut b);
    let FirstBlowup::MarkedResolved { charts, .. } =
        first_coordinate_blowup(c, "blow_nonprincipal", &mut b).unwrap()
    else {
        panic!("nonprincipal gate")
    };
    assert_eq!(charts.len(), 2);
    assert!(
        charts
            .iter()
            .all(|c| c.transform().generator_quotients().len() == 2)
    );
    let c = center(
        &r,
        &["y^2-x^3"],
        1,
        vec![0, 1],
        false,
        "recursive_mark_one",
        &mut b,
    );
    assert_eq!(c.invariant[0], Rational::from(2));
    let FirstBlowup::FurtherCyclesRequired { charts, .. } =
        first_coordinate_blowup(c, "blow_mark_one", &mut b).unwrap()
    else {
        panic!("must retain unresolved mark-one charts")
    };
    assert_eq!(charts.len(), 2);
    assert!(charts.iter().any(|c| !c.marked_cosupport_empty()));
    // A zero coefficient on a positive-dimensional contact support lifts to
    // the embedded Cartier divisor, whose blowup is geometrically identity.
    let c = center(
        &r,
        &["y^2"],
        2,
        vec![0, 1],
        false,
        "recursive_cartier",
        &mut b,
    );
    let FirstBlowup::MarkedResolved { charts, .. } =
        first_coordinate_blowup(c, "blow_cartier", &mut b).unwrap()
    else {
        panic!("Cartier control");
    };
    assert_eq!(charts.len(), 1);
    assert_eq!(
        charts[0].transform().jacobian(),
        &charts[0].transform().jacobian().one()
    );
}
#[test]
fn bm_recursive_limits_retry_zero_and_multiple_open_outcomes() {
    let r = ring();
    let mut b = b();
    let mut state = BoundaryFreeFirstCenter::new(
        flat(&r, vec![0, 1], &mut b),
        input(&r, &["y^2-x^3"], 2, &mut b),
        "recursive_retry",
        false,
    )
    .unwrap();
    let mut capped = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        state.advance(&mut capped).unwrap(),
        RecursiveAdvance::Incomplete { .. }
    ));
    assert_eq!(state.accepted_steps(), 0);
    assert_eq!(state.depth(), 0);
    assert!(matches!(run(state, &mut b), RecursiveOutcome::Center(_)));
    let zero = BoundaryFreeFirstCenter::new(
        flat(&r, vec![0, 1], &mut b),
        input(&r, &[], 2, &mut b),
        "recursive_zero",
        false,
    )
    .unwrap();
    assert!(matches!(
        run(zero, &mut b),
        RecursiveOutcome::TopLevelZeroIdeal { .. }
    ));
    let multi = BoundaryFreeFirstCenter::new(
        flat(&r, vec![0], &mut b),
        input(&r, &["x^2*(x-1)^2"], 2, &mut b),
        "recursive_multi",
        false,
    )
    .unwrap();
    assert!(matches!(
        run(multi, &mut b),
        RecursiveOutcome::MultipleContactOpensRequired { .. }
    ));
    let empty = BoundaryFreeFirstCenter::new(
        flat(&r, vec![0, 1], &mut b),
        input(&r, &["y^2-x^3"], 3, &mut b),
        "recursive_empty",
        false,
    )
    .unwrap();
    assert!(matches!(
        run(empty, &mut b),
        RecursiveOutcome::EmptyMarkedCosupport { .. }
    ));
}
#[test]
fn bm_recursive_noncoordinate_and_parameter_refusals() {
    let r = ring();
    let mut b = b();
    let c = center(
        &r,
        &["(y-1)^2-x^3"],
        2,
        vec![0, 1],
        false,
        "recursive_shifted",
        &mut b,
    );
    assert!(matches!(
        first_coordinate_blowup(c, "shifted_general", &mut b).unwrap(),
        FirstBlowup::GeneralAdaptationRequired { .. }
    ));
    let rp = Arc::new(
        Ring::new(
            vec![
                symbol!("recursive::x"),
                symbol!("recursive::y"),
                symbol!("recursive::z"),
            ],
            vec![2],
        )
        .unwrap(),
    );
    let f = flat(&rp, vec![0, 1], &mut b);
    let i = input(&rp, &["y^2-x^3"], 2, &mut b);
    assert!(matches!(
        BoundaryFreeFirstCenter::new(f, i, "param_refusal", false),
        Err(Error::Invalid("first-center fixed-parameter source/frame"))
    ));
}
