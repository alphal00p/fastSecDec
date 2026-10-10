use super::super::{
    Budget, EtaleCertificate, EtaleFrame, Guard, Ideal, Limits, LocalizedAlgebra, OrderProduction,
    Poly, ProducedContactCover, Ring, produce_ordinary_contact_cover,
};
use super::{ContactProduction, ContactQuotient, construct_contact_quotient};
use std::sync::Arc;
use symbolica::{atom::Symbol, parse, symbol};
fn budget() -> Budget {
    Budget::new(Limits::default())
}
fn syms() -> [Symbol; 4] {
    [
        symbol!("contact_draft::p"),
        symbol!("contact_draft::x"),
        symbol!("contact_draft::y"),
        symbol!("contact_draft::q"),
    ]
}
fn fresh() -> [Symbol; 2] {
    [symbol!("contact_draft::z"), symbol!("contact_draft::v")]
}
fn ring() -> Arc<Ring> {
    Arc::new(Ring::new(syms().to_vec(), vec![0]).unwrap())
}
fn p(r: &Ring, s: symbolica::atom::Atom) -> Poly {
    r.atom(&s).unwrap()
}
fn affine(r: Arc<Ring>, guards: Vec<Guard>, b: &mut Budget) -> Arc<EtaleFrame> {
    let local =
        LocalizedAlgebra::new(Ideal::new(r, vec![], b).unwrap(), vec![1, 2], guards, b).unwrap();
    Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![1, 2],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn cover(frame: Arc<EtaleFrame>, f: Poly, b: &mut Budget) -> Arc<ProducedContactCover> {
    let source = Arc::new(Ideal::new(frame.local().ring().clone(), vec![f], b).unwrap());
    match produce_ordinary_contact_cover(frame, source, b).unwrap() {
        OrderProduction::ContactCover(c) => Arc::new(c),
        other => panic!("expected contact cover: {other:?}"),
    }
}
fn construct(c: Arc<ProducedContactCover>, index: usize, b: &mut Budget) -> ContactQuotient {
    match construct_contact_quotient(c, index, fresh(), b).unwrap() {
        ContactProduction::Constructed(c) => *c,
        other => panic!("expected contact quotient: {other:?}"),
    }
}
fn find(c: &ProducedContactCover, desired: &Poly) -> usize {
    c.candidates()
        .iter()
        .position(|v| &v.equation == desired && v.free_derivative_index == 0)
        .unwrap()
}
fn assert_coefficient(q: &ContactQuotient, expected: Poly, mark: usize, b: &mut Budget) {
    assert_eq!(q.differential_coefficient().mark(), mark);
    let local = q.contact().local();
    let mut relations = local.ideal().generators().to_vec();
    relations.extend_from_slice(local.unit_relations());
    let ideal = Ideal::new(local.ring().clone(), vec![expected.clone()], b).unwrap();
    assert!(
        q.differential_coefficient()
            .ideal()
            .contains(&expected, &relations, b)
            .unwrap()
    );
    for f in q.differential_coefficient().ideal().generators() {
        assert!(ideal.contains(f, &relations, b).unwrap());
    }
    assert!(
        !q.differential_coefficient()
            .ideal()
            .contains(&local.ring().one(), &relations, b)
            .unwrap()
    );
}
#[test]
fn contact_draft_plane_cusp_and_actual_recursive_descent() {
    let r = ring();
    let mut b = budget();
    let f = p(&r, parse!("contact_draft::x^2+contact_draft::y^3"));
    let c = cover(affine(r.clone(), vec![], &mut b), f, &mut b);
    let index = find(&c, &p(&r, parse!("2*contact_draft::x")));
    let q = construct(c, index, &mut b);
    let rr = q.contact().local().ring().clone();
    assert_coefficient(&q, p(&rr, parse!("contact_draft::y^3")), 2, &mut b);
    assert_eq!(q.contact().free_axes(), &[2]);
    assert_eq!(q.progress().completed_jets.len(), 2);
    let d2 = q
        .graph()
        .derivative(
            q.normal_derivative_index(),
            q.progress().completed_jets[1]
                .ambient
                .generators()
                .first()
                .unwrap(),
            &mut b,
        )
        .unwrap();
    // z=2x, so d²(x²)/dz²=1/2. The nonzero second derivative is
    // excluded from the order-two coefficient ideal.
    assert!(
        q.graph()
            .local()
            .zero(&(d2 - p(&rr, parse!("1/2"))), &mut b)
            .unwrap()
    );
    let next = produce_ordinary_contact_cover(
        q.contact().clone(),
        Arc::new(q.differential_coefficient().ideal().clone()),
        &mut b,
    )
    .unwrap();
    let OrderProduction::ContactCover(next) = next else {
        panic!("recursive order")
    };
    assert_eq!(next.algebraic_maximum_order(), 3);
    let idx = next
        .candidates()
        .iter()
        .position(|h| h.equation == p(&rr, parse!("6*contact_draft::y")))
        .unwrap();
    let raw_next = construct_contact_quotient(
        Arc::new(next),
        idx,
        [symbol!("contact_draft::zz"), symbol!("contact_draft::vv")],
        &mut b,
    )
    .unwrap();
    // Native quotient normalization now removes relation-zero/redundant terms
    // before marked powers, so the formerly bounded incomplete path completes.
    let ContactProduction::Constructed(raw_next) = raw_next else {
        panic!("normalized recursive contact did not complete: {raw_next:?}");
    };
    for f in raw_next.differential_coefficient().ideal().generators() {
        assert!(raw_next.contact().local().zero(f, &mut b).unwrap());
    }
    // assert_coefficient above independently proves equality on the contact
    // quotient with the compact native generating set <y^3>. Keep the original
    // marked weight 2 in q; this call is only the next ordinary local operation.
    let compact = Arc::new(
        Ideal::new(
            rr.clone(),
            vec![p(&rr, parse!("contact_draft::y^3"))],
            &mut b,
        )
        .unwrap(),
    );
    let OrderProduction::ContactCover(compact) =
        produce_ordinary_contact_cover(q.contact().clone(), compact, &mut b).unwrap()
    else {
        panic!("compact recursive order")
    };
    let idx = compact
        .candidates()
        .iter()
        .position(|h| h.equation == p(&rr, parse!("6*contact_draft::y")))
        .unwrap();
    let next = construct_contact_quotient(
        Arc::new(compact),
        idx,
        [symbol!("contact_draft::zz"), symbol!("contact_draft::vv")],
        &mut b,
    )
    .unwrap();
    let ContactProduction::Constructed(next) = next else {
        panic!("compact recursive quotient: {next:?}")
    };
    assert!(next.contact().free_axes().is_empty());
    for f in next.differential_coefficient().ideal().generators() {
        assert!(next.contact().local().zero(f, &mut b).unwrap());
    }
    assert!(matches!(
        produce_ordinary_contact_cover(
            next.contact().clone(),
            Arc::new(next.differential_coefficient().ideal().clone()),
            &mut b
        )
        .unwrap(),
        OrderProduction::ZeroIdeal { .. }
    ));
}
#[test]
fn contact_draft_nonlinear_contact_keeps_parameter_and_real_branch_unselected() {
    let r = ring();
    let mut b = budget();
    let x = r.coordinate(1).unwrap();
    let frame = affine(
        r.clone(),
        vec![Guard {
            factor: x.clone(),
            inverse_axis: 3,
        }],
        &mut b,
    );
    let h = p(
        &r,
        parse!("contact_draft::x^2+contact_draft::y^2-contact_draft::p"),
    );
    let f = &h.pow(2) + &r.coordinate(2).unwrap().pow(3);
    let c = cover(frame, f.clone(), &mut b);
    assert_eq!(c.algebraic_maximum_order(), 2);
    let index = find(&c, &f.derivative(1));
    let q = construct(c, index, &mut b);
    let rr = q.contact().local().ring().clone();
    assert_coefficient(&q, p(&rr, parse!("contact_draft::y^3")), 2, &mut b);
    assert!(
        q.contact()
            .local()
            .zero(&q.extension().pull(&h, &mut b).unwrap(), &mut b)
            .unwrap()
    );
    assert!(q.contact().coefficients(0).unwrap()[0].is_zero());
    assert_eq!(q.contact().dependent_axes(), &[1]);
    assert!(
        q.contact()
            .local()
            .guards()
            .iter()
            .any(|g| g.factor == rr.coordinate(1).unwrap())
    );
}
#[test]
fn contact_draft_rational_contact_clears_only_certified_units() {
    let r = ring();
    let mut b = budget();
    let g = p(&r, parse!("1+contact_draft::y^2"));
    let frame = affine(
        r.clone(),
        vec![Guard {
            factor: g.clone(),
            inverse_axis: 3,
        }],
        &mut b,
    );
    let h = p(
        &r,
        parse!("contact_draft::q*(contact_draft::x+contact_draft::p*contact_draft::y^2)"),
    );
    let f = &h.pow(2) + &r.coordinate(2).unwrap().pow(3);
    let derivative = frame.derivative(0, &f, &mut b).unwrap();
    let c = cover(frame, f, &mut b);
    let q = construct(c.clone(), find(&c, &derivative), &mut b);
    let rr = q.contact().local().ring().clone();
    assert_ne!(q.clearing().denominator, rr.one());
    assert_coefficient(&q, p(&rr, parse!("contact_draft::y^3")), 2, &mut b);
    assert!(
        q.contact()
            .local()
            .guards()
            .iter()
            .any(|v| v.factor == q.extension().pull(&g, &mut b).unwrap())
    );
    assert!(q.contact().coefficients(0).unwrap()[0].is_zero());
    let guard_derivative = q
        .graph()
        .derivative(0, &rr.coordinate(3).unwrap(), &mut b)
        .unwrap();
    assert!(
        q.graph()
            .local()
            .zero(
                &(guard_derivative - p(&rr, parse!("-2*contact_draft::y*contact_draft::q^2"))),
                &mut b,
            )
            .unwrap()
    );
    let bare = affine(r.clone(), vec![], &mut b);
    assert!(super::embedding::clear_units(bare.local(), &h, &mut b).is_err());
}
#[test]
fn contact_draft_existing_etale_system_and_terminal_contact() {
    let r = ring();
    let mut b = budget();
    let local = LocalizedAlgebra::new(
        Ideal::new(
            r.clone(),
            vec![p(
                &r,
                parse!("contact_draft::x^2+contact_draft::y^2-contact_draft::p"),
            )],
            &mut b,
        )
        .unwrap(),
        vec![1, 2],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![2],
            free_axes: vec![1],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let c = cover(frame, p(&r, parse!("contact_draft::x^2")), &mut b);
    let q = construct(
        c.clone(),
        find(&c, &p(&r, parse!("2*contact_draft::x"))),
        &mut b,
    );
    assert!(q.contact().free_axes().is_empty());
    assert_eq!(q.contact().dependent_axes(), &[2, 1]);
    assert!(
        q.contact()
            .local()
            .zero(
                &p(
                    q.contact().local().ring(),
                    parse!("contact_draft::y^2-contact_draft::p")
                ),
                &mut b
            )
            .unwrap()
    );
    assert!(q.differential_coefficient().ideal().generators().is_empty());
    // The exact normalization receipt still ties this canonical zero ideal to
    // every original restricted generator; no numerical zero inference.
    for jet in &q.progress().completed_differential_layers {
        for f in jet.restricted.ideal().generators() {
            assert!(q.contact().local().zero(f, &mut b).unwrap());
        }
    }
}
#[test]
fn contact_draft_empty_open_roles_and_resource_frontiers() {
    let r = ring();
    let mut b = budget();
    let f = p(&r, parse!("contact_draft::x^2"));
    let c = cover(affine(r.clone(), vec![], &mut b), f.clone(), &mut b);
    let i = find(&c, &f);
    assert!(matches!(
        construct_contact_quotient(c.clone(), i, fresh(), &mut b).unwrap(),
        ContactProduction::EmptyContactOpen { .. }
    ));
    let i = find(&c, &p(&r, parse!("2*contact_draft::x")));
    assert!(construct_contact_quotient(c.clone(), i, [syms()[0], fresh()[1]], &mut b).is_err());
    assert!(construct_contact_quotient(c.clone(), usize::MAX, fresh(), &mut b).is_err());
    let mut small = Budget::new(Limits {
        max_mark: 1,
        ..Limits::default()
    });
    let ContactProduction::Incomplete {
        source, progress, ..
    } = construct_contact_quotient(c.clone(), i, fresh(), &mut small).unwrap()
    else {
        panic!("budget")
    };
    assert!(Arc::ptr_eq(&source, &c));
    assert_eq!(progress.stage, "mark preflight");
    assert!(progress.completed_jets.is_empty());
    let mut small = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        construct_contact_quotient(c.clone(), i, fresh(), &mut small).unwrap(),
        ContactProduction::Incomplete { .. }
    ));
    let mut full = budget();
    let completed = construct(c.clone(), i, &mut full);
    let mut late = Budget::new(Limits {
        max_operations: completed.progress().operations - 1,
        ..Limits::default()
    });
    let ContactProduction::Incomplete { progress, .. } =
        construct_contact_quotient(c, i, fresh(), &mut late).unwrap()
    else {
        panic!("expected late resource frontier")
    };
    assert!(progress.verified_graph.is_some());
    assert!(progress.verified_contact.is_some());
    assert!(!progress.completed_jets.is_empty());
}

#[test]
fn contact_draft_full_differential_is_not_the_normal_jet_ideal() {
    let mut b = budget();
    let r = Arc::new(
        Ring::new(
            vec![syms()[0], syms()[1], syms()[2], symbol!("contact_draft::w")],
            vec![0],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut b).unwrap(),
        vec![1, 2, 3],
        vec![],
        &mut b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![1, 2, 3],
            determinant_inverse_axis: None,
        }
        .verify(&mut b)
        .unwrap(),
    );
    let c = cover(
        frame,
        p(
            &r,
            parse!("contact_draft::x^2+contact_draft::y^2+contact_draft::p*contact_draft::w^3"),
        ),
        &mut b,
    );
    let q = construct(
        c.clone(),
        find(&c, &p(&r, parse!("2*contact_draft::x"))),
        &mut b,
    );
    let rr = q.contact().local().ring();
    let mut relations = q.contact().local().ideal().generators().to_vec();
    relations.extend_from_slice(q.contact().local().unit_relations());
    let y2 = p(rr, parse!("contact_draft::y^2"));
    let pw3 = p(rr, parse!("contact_draft::p*contact_draft::w^3"));
    let mixed = p(
        rr,
        parse!("contact_draft::p*contact_draft::y*contact_draft::w^2"),
    );
    let expected = Ideal::new(
        rr.clone(),
        vec![y2.clone(), pw3.clone(), mixed.clone()],
        &mut b,
    )
    .unwrap();
    assert_eq!(q.normal_coefficient().mark(), 2);
    assert_eq!(q.differential_coefficient().mark(), 2);
    assert!(
        !q.normal_coefficient()
            .ideal()
            .contains(&y2, &relations, &mut b)
            .unwrap()
    );
    for f in [&y2, &pw3, &mixed] {
        assert!(
            q.differential_coefficient()
                .ideal()
                .contains(f, &relations, &mut b)
                .unwrap()
        );
    }
    for f in q.differential_coefficient().ideal().generators() {
        assert!(expected.contains(f, &relations, &mut b).unwrap());
    }
    for i in 0..q.graph().free_axes().len() {
        assert!(q.graph().coefficients(i).unwrap()[0].is_zero());
    }
    assert_eq!(q.progress().completed_differential_layers.len(), 2);
    let layer = &q.progress().completed_differential_layers[1].ambient;
    let first = &q.progress().completed_differential_layers[0].ambient;
    for f in first.generators() {
        assert!(layer.generators().contains(f));
    }
    assert!(
        layer
            .generators()
            .contains(&p(rr, parse!("2*contact_draft::y")))
    );
    assert!(
        layer
            .generators()
            .contains(&p(rr, parse!("3*contact_draft::p*contact_draft::w^2")))
    );
}
