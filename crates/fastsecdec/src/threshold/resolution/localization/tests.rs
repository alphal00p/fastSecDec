use super::super::*;
use std::sync::Arc;
use symbolica::{atom::Atom, symbol};
fn ring() -> Arc<Ring> {
    Arc::new(
        Ring::new(
            vec![
                symbol!("localization_test::p"),
                symbol!("localization_test::x"),
                symbol!("localization_test::y"),
                symbol!("localization_test::u"),
                symbol!("localization_test::v"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn b() -> Budget {
    Budget::new(Limits {
        max_operations: 1_000_000,
        max_total_ideal_slots: 1_000_000,
        ..Limits::default()
    })
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "localization_test", Default::default()).unwrap())
        .unwrap()
}
fn local(
    r: &Arc<Ring>,
    f: Vec<Poly>,
    axes: Vec<usize>,
    guards: Vec<Guard>,
    b: &mut Budget,
) -> Arc<LocalizedAlgebra> {
    LocalizedAlgebra::new(Ideal::new(r.clone(), f, b).unwrap(), axes, guards, b).unwrap()
}
fn flat(local: Arc<LocalizedAlgebra>, b: &mut Budget) -> Arc<EtaleFrame> {
    Arc::new(
        EtaleCertificate {
            free_axes: local.axes().to_vec(),
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn history(frame: Arc<EtaleFrame>, h: Poly, b: &mut Budget) -> Arc<ResolutionHistory> {
    let SncProduction::Verified(l) = verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(1),
            equation: h,
        }],
        b,
    )
    .unwrap() else {
        panic!("unchecked boundary")
    };
    ResolutionHistory::initial(l).unwrap()
}
fn split(
    history: &Arc<ResolutionHistory>,
    f: Vec<Poly>,
    b: &mut Budget,
) -> Arc<VerifiedComponentSplit> {
    let BoundaryAlgebra::Regular(owner) =
        RegularAlgebra::boundary(history.ledger().clone(), 0, b).unwrap()
    else {
        panic!()
    };
    let source = Arc::new(Ideal::new(owner.local().ring().clone(), f, b).unwrap());
    let ComponentProduction::Complete(s) =
        produce_component_split(owner, source, "localized_component", b).unwrap()
    else {
        panic!()
    };
    Arc::new(*s)
}
fn localized(
    history: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    namespace: &str,
    b: &mut Budget,
) -> Box<LocalizedHistory> {
    let LocalizationProduction::Complete(v) =
        localize_component_open(history, split, side, namespace, b).unwrap()
    else {
        panic!("missing open")
    };
    v
}
#[test]
fn localization_producer_disconnected_sides_stable_and_exact() {
    let r = ring();
    let mut b = b();
    let frame = flat(local(&r, vec![], vec![1, 2], vec![], &mut b), &mut b);
    let h = history(
        frame,
        p(&r, "localization_test::x*(localization_test::x-1)"),
        &mut b,
    );
    let s = split(&h, vec![p(&r, "localization_test::x^2")], &mut b);
    // Reordered visits retain semantic side identities and the same history root.
    let right = localized(h.clone(), s.clone(), 1, "localization_right", &mut b);
    let left = localized(h.clone(), s.clone(), 0, "localization_left", &mut b);
    assert_ne!(left.history().chart_path(), right.history().chart_path());
    for (side, open) in [(0, &left), (1, &right)] {
        assert_eq!(open.open().side(), side);
        assert!(open.history().same_root(&h));
        assert_eq!(open.history().stage(), h.stage());
        assert_eq!(open.history().births(), h.births());
        assert_eq!(open.history().old_snapshot(), h.old_snapshot());
        assert!(
            matches!(&open.history().chart_path()[0],HistoryStep::PrincipalOpen{semantic_side,..}if *semantic_side==side)
        );
        assert_eq!(
            open.open().coordinate_jacobian(),
            open.history().ledger().frame().local().ring().one()
        );
        assert_eq!(
            open.history().ledger().frame().free_axes(),
            h.ledger().frame().free_axes()
        );
        assert!(
            open.history()
                .ledger()
                .frame()
                .local()
                .ring()
                .is_parameter(0)
        );
        assert!(open.progress().old_coordinate_derivatives_checked > 0);
        for (old, new) in h
            .ledger()
            .divisors()
            .iter()
            .zip(open.history().ledger().divisors())
        {
            let f = open.open().extension().pull(&old.equation, &mut b).unwrap();
            assert_eq!(old.id, new.id);
            assert!(
                open.history()
                    .ledger()
                    .frame()
                    .local()
                    .zero(&(&f - &new.equation), &mut b)
                    .unwrap()
            );
        }
    }
    let again = localized(h.clone(), s, 0, "localization_left", &mut b);
    assert_eq!(again.history().chart_path(), left.history().chart_path());
}
#[test]
fn localization_producer_relative_guarded_frame_and_wrong_owner() {
    let r = ring();
    let mut b = b();
    let source = local(
        &r,
        vec![p(&r, "localization_test::y-localization_test::x^2")],
        vec![1, 2],
        vec![Guard {
            factor: p(&r, "localization_test::p+1"),
            inverse_axis: 3,
        }],
        &mut b,
    );
    let frame = Arc::new(
        EtaleCertificate {
            source,
            free_axes: vec![1],
            dependent_axes: vec![2],
            equations: vec![0],
            determinant_inverse_axis: Some(4),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let h = history(
        frame.clone(),
        p(&r, "localization_test::x*(localization_test::x-1)"),
        &mut b,
    );
    let s = split(&h, vec![p(&r, "localization_test::y^2")], &mut b);
    let open = localized(h.clone(), s.clone(), 0, "localization_relative", &mut b);
    assert_eq!(open.history().ledger().frame().dependent_axes(), &[2]);
    assert_eq!(
        open.history().ledger().frame().local().guards().len(),
        frame.local().guards().len() + 2
    );
    let y = open
        .open()
        .extension()
        .pull(&r.coordinate(2).unwrap(), &mut b)
        .unwrap();
    let dy = open
        .history()
        .ledger()
        .frame()
        .derivative(0, &y, &mut b)
        .unwrap();
    let two_x = open
        .open()
        .extension()
        .pull(&p(&r, "2*localization_test::x"), &mut b)
        .unwrap();
    assert!(
        open.history()
            .ledger()
            .frame()
            .local()
            .zero(&(&dy - &two_x), &mut b)
            .unwrap()
    );
    let foreign = history(
        frame,
        p(&r, "localization_test::x*(localization_test::x-1)"),
        &mut b,
    );
    assert!(matches!(
        localize_component_open(foreign, s.clone(), 0, "wrong_owner", &mut b),
        Err(Error::Invalid("boundary split ledger owner"))
    ));
    assert!(matches!(
        localize_component_open(h, s, 2, "wrong_side", &mut b),
        Err(Error::Invalid("component open side"))
    ));
}
#[test]
fn localization_producer_empty_unit_and_resource_receipts() {
    let r = ring();
    let mut b = b();
    let frame = flat(local(&r, vec![], vec![1], vec![], &mut b), &mut b);
    let h = history(frame, r.coordinate(1).unwrap(), &mut b);
    let s = split(&h, vec![r.one()], &mut b);
    assert!(matches!(
        localize_component_open(h.clone(), s.clone(), 0, "empty_open", &mut b).unwrap(),
        LocalizationProduction::Empty { side: 0, .. }
    ));
    let v = localized(h.clone(), s.clone(), 1, "unit_open", &mut b);
    assert_eq!(v.history().stage(), 0);
    let mut cap = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        localize_component_open(h.clone(), s.clone(), 1, "capped_open", &mut cap).unwrap(),
        LocalizationProduction::Incomplete { .. }
    ));
    let mut retained = false;
    for n in (10..400).step_by(5) {
        let mut cap = Budget::new(Limits {
            max_operations: n,
            ..Limits::default()
        });
        if let LocalizationProduction::Incomplete { progress, .. } =
            localize_component_open(h.clone(), s.clone(), 1, "partly_capped_open", &mut cap)
                .unwrap()
            && progress.frame.is_some()
        {
            retained = true;
            break;
        }
    }
    assert!(retained);
}
fn blowup_once(
    history: Arc<ResolutionHistory>,
    namespace: &str,
    b: &mut Budget,
) -> Arc<MonomialBlowupChart> {
    let ledger = history.ledger();
    assert_eq!(ledger.divisors().len(), 1);
    let r = ledger.frame().local().ring();
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![ledger.divisors()[0].equation.clone()], b).unwrap(),
            1,
            b,
        )
        .unwrap(),
    );
    let witness = MonomialWitness {
        source,
        powers: vec![1],
        quotients: vec![r.one()],
    };
    let MonomialProduction::Center(center) =
        produce_monomial_center(ledger.clone(), witness, b).unwrap()
    else {
        panic!("missing Cartier center")
    };
    let center = Arc::new(*center);
    let AdaptedProduction::Complete(cover) =
        produce_adapted_cover(ledger.clone(), &format!("{namespace}_adapt"), b)
            .unwrap_or_else(|error| panic!("{namespace}: {error:?}"))
    else {
        panic!("missing cover")
    };
    let BlowupProduction::Complete(out) =
        produce_monomial_blowup(center, history, cover, namespace, b).unwrap()
    else {
        panic!("missing blowup")
    };
    out.charts()
        .iter()
        .find(|c| c.pivot() == Some(0))
        .unwrap()
        .clone()
}
#[test]
fn localization_producer_births_include_open_ancestry_and_keep_old_snapshot() {
    let r = ring();
    let mut b = b();
    let frame = flat(local(&r, vec![], vec![1], vec![], &mut b), &mut b);
    let h = history(
        frame,
        p(&r, "localization_test::x*(localization_test::x-1)"),
        &mut b,
    );
    let s = split(&h, vec![p(&r, "localization_test::x^2")], &mut b);
    let left = localized(h.clone(), s.clone(), 0, "birth_left_open", &mut b);
    let right = localized(h, s, 1, "birth_right_open", &mut b);
    let left = blowup_once(left.history().clone(), "birth_left", &mut b);
    let right = blowup_once(right.history().clone(), "birth_right", &mut b);
    assert_eq!(left.history().stage(), 1);
    assert_eq!(right.history().stage(), 1);
    assert_eq!(left.history().births(), right.history().births());
    assert_ne!(
        left.history().birth_context(BoundaryId(2)),
        right.history().birth_context(BoundaryId(2))
    );
    assert_eq!(left.history().old_snapshot(), &[BoundaryId(1)]);
    assert!(!left.history().old_snapshot().contains(&BoundaryId(2)));
    let source = Arc::new(
        Ideal::new(
            left.frame().local().ring().clone(),
            vec![left.frame().local().ring().one()],
            &mut b,
        )
        .unwrap(),
    );
    let ComponentProduction::Complete(s) = produce_component_split(
        RegularAlgebra::ambient(left.frame().clone()),
        source,
        "born_ambient_split",
        &mut b,
    )
    .unwrap() else {
        panic!()
    };
    let localized = localized(
        left.history().clone(),
        Arc::new(*s),
        1,
        "born_unit_open",
        &mut b,
    );
    assert!(localized.history().same_root(left.history()));
    assert_eq!(localized.history().births(), left.history().births());
    assert_eq!(
        localized.history().birth_context(BoundaryId(2)),
        left.history().birth_context(BoundaryId(2))
    );
    assert_eq!(
        localized.history().old_snapshot(),
        left.history().old_snapshot()
    );
    assert_eq!(localized.history().stage(), 1);
    let second = blowup_once(localized.history().clone(), "born_after_open", &mut b);
    assert_eq!(second.history().stage(), 2);
    assert_eq!(
        second
            .history()
            .birth_context(BoundaryId(3))
            .unwrap()
            .parent_chart_path,
        localized.history().chart_path()
    );
    assert_eq!(second.history().old_snapshot(), &[BoundaryId(1)]);
}
