use super::*;
use symbolica::symbol;
fn actual_terminal(b: &mut Budget) -> Arc<RecursiveComponentCover> {
    // A smooth disconnected relative curve: one component has source (x),
    // the other a unit. The actual contact x=0 retains both components.
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("component_local::t"),
                symbol!("component_local::x"),
                symbol!("component_local::inverse"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let t = r.coordinate(0).unwrap();
    let x = r.coordinate(1).unwrap();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![t.pow(2) - r.one()], b).unwrap(),
        vec![0, 1],
        vec![],
        b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![0],
            free_axes: vec![1],
            determinant_inverse_axis: Some(2),
        }
        .verify(b)
        .unwrap(),
    );
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![x, t - r.one()], b).unwrap(),
            1,
            b,
        )
        .unwrap(),
    );
    let mut driver = BoundaryFreeFirstCenter::new(
        frame.clone(),
        source.clone(),
        "component_local_driver",
        false,
    )
    .unwrap();
    for _ in 0..20 {
        if !matches!(driver.advance(b).unwrap(), RecursiveAdvance::Progress) {
            break;
        }
    }
    let RecursiveOutcome::ComponentOpensRequired(cover) = driver.outcome() else {
        panic!("expected disconnected source")
    };
    assert!(Arc::ptr_eq(cover.origin().frame(), &frame));
    assert!(Arc::ptr_eq(cover.origin().source(), &source));
    assert_eq!(cover.levels().len(), 1);
    cover.clone()
}
#[test]
fn actual_original_contact_localizes_then_blows_up_all_standard_charts() {
    let mut b = Budget::new(Limits::default());
    let terminal = actual_terminal(&mut b);
    let owner = ComponentLocalization::prepare(terminal, &mut b).unwrap();
    let mut f =
        ComponentLocalizationFrontier::new(owner.clone(), "component_local_test".into(), &mut b)
            .unwrap();
    let mut stopped = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        f.advance(0, &mut stopped).unwrap(),
        ComponentLocalizationAdvance::Incomplete { .. }
    ));
    assert!(matches!(f.states()[0], ComponentLocalizationState::Pending));
    assert!(!f.complete_inventory());
    for i in 0..f.states().len() {
        assert!(matches!(
            f.advance(i, &mut b).unwrap(),
            ComponentLocalizationAdvance::Progress
        ));
    }
    assert!(f.complete_inventory());
    let mut centers = 0;
    let mut complements = 0;
    for s in f.states() {
        match s {
            ComponentLocalizationState::Center(c) => {
                centers += 1;
                assert!(Arc::ptr_eq(c.sources().owner(), &owner));
                assert!(c.history().same_root(owner.origin().history()));
                assert_eq!(c.history().stage(), 0);
                let checked = CheckedRecursiveCenter::new(
                    RecursiveCenterOrigin::LocalizedComponent(c.clone()),
                    &mut b,
                )
                .unwrap();
                let adapted =
                    adapt_recursive_center(checked, "component_test_adapt", &mut b).unwrap();
                let expected: usize = adapted
                    .opens()
                    .iter()
                    .map(|o| {
                        if o.identity_side().is_some() {
                            1
                        } else {
                            o.center_axes().len()
                        }
                    })
                    .sum();
                let result =
                    blowup_recursive_center(adapted, "component_test_transform", &mut b).unwrap();
                assert_eq!(result.charts().len(), expected);
                assert!(expected > 0);
                for chart in result.charts() {
                    assert!(chart.history().same_root(owner.origin().history()));
                    assert!(Arc::ptr_eq(chart.parent().source(), c.source()));
                    let local = chart.geometry().frame().local();
                    let factor = chart
                        .geometry()
                        .exceptional()
                        .map_or_else(|| local.ring().one(), |e| e.pow(c.source().mark()));
                    for g in chart.parent().generators() {
                        assert!(
                            local
                                .zero(&(&g.total - &(&factor * &g.quotient)), &mut b)
                                .unwrap()
                        );
                    }
                    assert_eq!(
                        chart.history().stage(),
                        usize::from(chart.geometry().born().is_some()) as u64
                    );
                }
            }
            ComponentLocalizationState::UnitChild(s) => {
                complements += 1;
                assert!(Arc::ptr_eq(s.owner(), &owner));
            }
            ComponentLocalizationState::EmptyContact { receipt, .. } => {
                complements += 1;
                assert!(matches!(
                    receipt,
                    PhysicalSupportRestriction::EmptySupport { .. }
                ));
            }
            ComponentLocalizationState::EmptyPrincipal { receipt } => {
                complements += 1;
                assert!(matches!(receipt, CoverLocalizationProduction::Empty { .. }));
            }
            _ => panic!("incomplete actual source"),
        }
    }

    let mut reversed =
        ComponentLocalizationFrontier::new(owner.clone(), "component_reordered".into(), &mut b)
            .unwrap();
    for i in (0..reversed.states().len()).rev() {
        assert!(matches!(
            reversed.advance(i, &mut b).unwrap(),
            ComponentLocalizationAdvance::Progress
        ));
    }
    assert!(reversed.complete_inventory());
    for (a, z) in f.states().iter().zip(reversed.states()) {
        assert_eq!(std::mem::discriminant(a), std::mem::discriminant(z));
    }
    assert_eq!(centers, 1);
    assert!(complements > 0);
    let mut stopped = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    for i in 0..f.states().len() {
        assert!(matches!(
            f.advance(i, &mut stopped).unwrap(),
            ComponentLocalizationAdvance::Accepted
        ));
    }
    assert_eq!(stopped.operations(), 0);
}
#[test]
fn foreign_original_source_and_cover_are_refused() {
    let mut b = Budget::new(Limits::default());
    let terminal = actual_terminal(&mut b);
    let owner = ComponentLocalization::prepare(terminal.clone(), &mut b).unwrap();
    let other = actual_terminal(&mut b);
    let wrong = terminal.test_replaced_origin(other.origin().clone());
    assert!(matches!(
        ComponentLocalization::prepare(wrong, &mut b),
        Err(Error::Invalid("component localization original hierarchy"))
    ));
    let other_owner = ComponentLocalization::prepare(other, &mut b).unwrap();
    assert!(matches!(
        owner.cover().localize(
            other_owner.origin().history().clone(),
            0,
            "wrong_original_history",
            &mut b
        ),
        Err(Error::Invalid(_))
    ));
    let CoverLocalizationProduction::Complete(open) = owner
        .cover()
        .localize(owner.origin().history().clone(), 0, "right_open", &mut b)
        .unwrap()
    else {
        panic!()
    };
    assert!(matches!(
        other_owner
            .cover()
            .restrict_support(Arc::new(*open), "foreign_component_cover", &mut b),
        Err(Error::Invalid(_))
    ));
}
