use super::super::*;
use super::*;
use std::sync::Arc;
use symbolica::{atom::Atom, symbol};
fn b() -> Budget {
    Budget::new(Limits {
        max_operations: 5_000_000,
        max_total_ideal_slots: 5_000_000,
        ..Limits::default()
    })
}
fn order(formula: &str, mark: usize, b: &mut Budget) -> Arc<ComponentResidualOrder> {
    let r = Arc::new(
        Ring::new(
            vec![symbol!("companion_open::x"), symbol!("companion_open::y")],
            vec![],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], b).unwrap(),
        vec![0, 1],
        vec![],
        b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0, 1],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    );
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(1),
            equation: r.coordinate(0).unwrap(),
        }],
        b,
    )
    .unwrap() else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(
                r.clone(),
                vec![
                    r.atom(&Atom::parse(formula, "companion_open", Default::default()).unwrap())
                        .unwrap(),
                ],
                b,
            )
            .unwrap(),
            mark,
            b,
        )
        .unwrap(),
    );
    let mut front =
        ComponentFactorFrontier::new(history, source, "companion_open_fixture".into(), b).unwrap();
    for _ in 0..80 {
        let Some(path) = front.pending().next().cloned() else {
            break;
        };
        assert!(matches!(
            front
                .advance(&path, &ComponentFactorLimits::default(), b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = front.try_complete().unwrap() else {
        panic!()
    };
    let leaf = done.nodes().values().find_map(|n| n.leaf()).unwrap();
    let ComponentResidualProduction::Order(order) =
        produce_component_residual_order(leaf.clone(), b).unwrap()
    else {
        panic!()
    };
    Arc::new(*order)
}
#[test]
fn companion_open_positive_contact_old_boundary_and_actual_owner() {
    let mut b = b();
    let order = order("companion_open::x^2*companion_open::y", 3, &mut b);
    assert_eq!(order.algebraic_maximum_on_cosupport(), 1);
    let mut contacts = 0;
    let mut below = 0;
    let mut coefficients = 0;
    for index in 0..order.upper_order_cover().opens().len() {
        match produce_companion_open(
            order.clone(),
            index,
            &format!("companion_stage_positive{index}"),
            &mut b,
        )
        .unwrap()
        {
            CompanionOpenProduction::Contact(open) => {
                contacts += 1;
                let open = Arc::new(*open);
                assert!(Arc::ptr_eq(
                    open.contact().frame(),
                    open.restriction().history().ledger().frame()
                ));
                assert_eq!(
                    open.contact().algebraic_maximum_order(),
                    open.companion().mark()
                );
                assert_eq!(
                    open.restriction().history().old_snapshot(),
                    &[BoundaryId(1)]
                );
                for i in 0..open.contact().candidates().len() {
                    let fresh = [
                        symbol!(format!("companion_stage_coeff{index}_{i}")),
                        symbol!(format!("companion_stage_inv{index}_{i}")),
                    ];
                    match construct_companion_coefficient(open.clone(), i, fresh, &mut b).unwrap() {
                        CompanionCoefficientProduction::Coefficient(c) => {
                            coefficients += 1;
                            assert_eq!(c.coefficient().algebraic_maximum_old_count(), 1);
                            assert!(Arc::ptr_eq(
                                c.coefficient().history(),
                                open.restriction().history()
                            ));
                        }
                        CompanionCoefficientProduction::ContactPending {
                            evidence: ContactProduction::EmptyContactOpen { .. },
                            ..
                        } => {}
                        _ => panic!("coefficient unfinished"),
                    }
                }
            }
            CompanionOpenProduction::BelowMaximum { .. } => below += 1,
            CompanionOpenProduction::EmptyOpen { .. } => {}
            _ => panic!("unexpected stage"),
        }
    }
    assert!(contacts > 0 && below > 0 && coefficients > 0);
}
#[test]
fn companion_open_residual_unit_monomial_lane_is_checked() {
    let mut b = b();
    let order = order("companion_open::x^2", 2, &mut b);
    assert_eq!(order.algebraic_maximum_on_cosupport(), 0);
    let mut centers = 0;
    for index in 0..order.upper_order_cover().opens().len() {
        let CompanionOpenProduction::Monomial {
            restriction,
            witness,
            checked,
            ..
        } = produce_companion_open(
            order.clone(),
            index,
            &format!("companion_stage_mono{index}"),
            &mut b,
        )
        .unwrap()
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(
            witness.source.ideal().ring(),
            restriction.history().ledger().frame().local().ring()
        ));
        assert_eq!(witness.powers, vec![2]);
        if let MonomialProduction::Center(center) = checked {
            centers += 1;
            assert!(center.progress().residual_unit);
        }
    }
    assert!(centers > 0);
}
#[test]
fn companion_open_resource_receipts_and_wrong_cover_owner() {
    let mut b = b();
    let order = order("companion_open::x^2*companion_open::y", 3, &mut b);
    let mut tiny = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        produce_companion_open(order.clone(), 0, "companion_stage_capped", &mut tiny).unwrap(),
        CompanionOpenProduction::Incomplete { .. }
    ));
    let history = order.factor().data().history();
    let local = history.ledger().frame().local();
    let other = LocalizedAlgebra::new(
        local.ideal().clone(),
        local.axes().to_vec(),
        local.guards().to_vec(),
        &mut b,
    )
    .unwrap();
    let cover = Arc::new(
        OpenCoverCertificate {
            algebra: other,
            support: order.cosupport().as_ref().clone(),
            opens: order.upper_order_cover().opens().to_vec(),
        }
        .verify(&mut b)
        .unwrap(),
    );
    assert!(
        localize_verified_cover_open(history.clone(), cover, 0, "wrong_cover_owner", &mut b)
            .is_err()
    );
    assert!(
        localize_verified_cover_open(
            history.clone(),
            Arc::new(order.upper_order_cover().clone()),
            usize::MAX,
            "bad_cover_index",
            &mut b
        )
        .is_err()
    );
}
#[test]
fn companion_open_clears_guard_units_and_retains_empty_open_receipts() {
    let mut b = b();
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("companion_guard::x"),
                symbol!("companion_guard::y"),
                symbol!("companion_guard::inverse"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut b).unwrap(),
        vec![0, 1],
        vec![Guard {
            factor: r.coordinate(0).unwrap(),
            inverse_axis: 2,
        }],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local.clone(),
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0, 1],
            determinant_inverse_axis: None,
        }
        .verify(&mut b)
        .unwrap(),
    );
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(9),
            equation: r.coordinate(1).unwrap(),
        }],
        &mut b,
    )
    .unwrap() else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let cover = Arc::new(
        OpenCoverCertificate {
            algebra: local,
            support: Ideal::new(r.clone(), vec![], &mut b).unwrap(),
            opens: vec![r.one().zero(), r.coordinate(2).unwrap()],
        }
        .verify(&mut b)
        .unwrap(),
    );
    assert!(matches!(
        localize_verified_cover_open(
            history.clone(),
            cover.clone(),
            0,
            "companion_empty_open",
            &mut b
        )
        .unwrap(),
        CoverLocalizationProduction::Empty { index: 0, .. }
    ));
    let CoverLocalizationProduction::Complete(open) =
        localize_verified_cover_open(history.clone(), cover, 1, "companion_unit_clearing", &mut b)
            .unwrap()
    else {
        panic!()
    };
    assert_eq!(open.open().clearing().numerator, r.one());
    assert_eq!(open.open().clearing().denominator, r.coordinate(0).unwrap());
    assert!(open.history().same_root(&history));
    assert_eq!(open.history().old_snapshot(), history.old_snapshot());
    assert_eq!(
        open.open().coordinate_jacobian(),
        open.history().ledger().frame().local().ring().one()
    );
}
