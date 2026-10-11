use super::*;
use symbolica::symbol;
#[test]
fn terminal_disconnected_contact_retains_both_components_and_incomplete_owner() {
    let mut b = Budget::new(Limits::default());
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("terminal_component::t"),
                symbol!("terminal_component::inverse"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let t = r.coordinate(0).unwrap();
    let relation = t.pow(2) - r.one();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![relation.clone()], &mut b).unwrap(),
        vec![0],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![0],
            free_axes: vec![],
            determinant_inverse_axis: Some(1),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![t - r.one()], &mut b).unwrap(),
            1,
            &b,
        )
        .unwrap(),
    );
    let mut driver = BoundaryFreeFirstCenter::new(
        frame.clone(),
        source.clone(),
        "terminal_component_source",
        false,
    )
    .unwrap();
    assert!(matches!(
        driver.advance(&mut b).unwrap(),
        RecursiveAdvance::Incomplete { .. }
    ));
    let RecursiveOutcome::ComponentOpensRequired(cover) = driver.outcome() else {
        panic!()
    };
    assert!(Arc::ptr_eq(cover.frame(), &frame) && Arc::ptr_eq(cover.source(), &source));
    assert!(Arc::ptr_eq(cover.normalization().source(), &source));
    assert_eq!(cover.split().pattern(), ComponentPattern::Mixed);
    assert!(cover.levels().is_empty());
    let SncProduction::Verified(ledger) =
        verify_initial_relative_snc(frame, vec![], &mut b).unwrap()
    else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    for side in 0..2 {
        let LocalizationProduction::Complete(open) = localize_component_open(
            history.clone(),
            cover.split().clone(),
            side,
            &format!("terminal_component_side{side}"),
            &mut b,
        )
        .unwrap() else {
            panic!()
        };
        let local = open.history().ledger().frame().local();
        let i = open
            .open()
            .extension()
            .ideal(source.ideal(), &mut b)
            .unwrap();
        let zero = i
            .generators()
            .iter()
            .all(|f| local.zero(f, &mut b).unwrap());
        let unit = local
            .ideal()
            .sum(&i, &mut b)
            .unwrap()
            .contains(&local.ring().one(), local.unit_relations(), &mut b)
            .unwrap();
        assert_eq!(
            (zero, unit),
            if side == 0 {
                (true, false)
            } else {
                (false, true)
            }
        );
    }
    let steps = driver.accepted_steps();
    let mut stopped = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        driver.advance(&mut stopped).unwrap(),
        RecursiveAdvance::Incomplete { .. }
    ));
    assert_eq!(steps, driver.accepted_steps());
    assert_eq!(stopped.operations(), 0);
    assert!(matches!(
        driver.outcome(),
        RecursiveOutcome::ComponentOpensRequired(_)
    ));
}
