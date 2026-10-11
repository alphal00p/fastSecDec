use super::*;
#[test]
fn relative_transition_relative_nonlinear_graph_both_pivots_jacobian_and_saturation() {
    let r = ring();
    let mut b = budget();
    let source = frame(
        &r,
        &["z^2-1-p*x"],
        vec![2, 3, 4],
        vec![4],
        vec![2, 3],
        &mut b,
    );
    let graph = frame(
        &r,
        &["z^2-1-p*x", "y-z+1-w"],
        vec![2, 3, 4, 5],
        vec![4, 3],
        vec![2, 5],
        &mut b,
    );
    let center = ideal(&r, &["x", "y-z+1"], &mut b);
    let graph_center = ideal(&r, &["x", "w"], &mut b);
    assert!(equal(
        &graph.local().ideal().sum(&center, &mut b).unwrap(),
        &graph.local().ideal().sum(&graph_center, &mut b).unwrap(),
        graph.local().unit_relations(),
        &mut b
    ));
    assert!(source.derivative(0, &p(&r, "p"), &mut b).unwrap().is_zero());
    assert!(graph.derivative(1, &p(&r, "p"), &mut b).unwrap().is_zero());
    for (eq, axes, free, e, wanted, empty) in [
        (
            ["z^2-1-p*x", "y-z+1-x*r"],
            vec![2, 3, 4, 6],
            vec![2, 6],
            "x",
            "r",
            false,
        ),
        (
            ["z^2-1-p*w*r", "y-z+1-w"],
            vec![3, 4, 5, 6],
            vec![6, 5],
            "w",
            "1",
            true,
        ),
    ] {
        let f = frame(&r, &eq, axes, vec![4, 3], free, &mut b);
        let ep = p(&r, e);
        let input = ideal(&r, &["y-z+1"], &mut b);
        let strict = saturation(f.local(), &input, &ep, &mut b);
        let expected = f
            .local()
            .ideal()
            .sum(&ideal(&r, &[wanted], &mut b), &mut b)
            .unwrap();
        assert!(equal(
            &strict,
            &expected,
            f.local().unit_relations(),
            &mut b
        ));
        assert_eq!(
            strict
                .contains(&r.one(), f.local().unit_relations(), &mut b)
                .unwrap(),
            empty
        );
        assert!(strict.generators().iter().all(|g| g.degree(0) == 0));
        assert!(!f.local().guards().iter().any(|g| g.factor == ep));
        let exceptional = snc(f.clone(), ep.clone(), &mut b);
        let CartierDivision::Quotient(q) = divide_cartier(
            exceptional,
            0,
            p(&r, "y-z+1"),
            &format!("next_api::{e}_divide"),
            &mut b,
        )
        .unwrap() else {
            panic!("division")
        };
        assert!(
            f.local()
                .zero(&(q.quotient() - &p(&r, wanted)), &mut b)
                .unwrap()
        );
        let source_x = if e == "x" { p(&r, "x") } else { p(&r, "w*r") };
        let source_y = p(&r, "y");
        let entries = [source_x, source_y]
            .iter()
            .flat_map(|v| {
                (0..2)
                    .map(|j| f.derivative(j, v, &mut b).unwrap().to_expression())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let field = AtomField {
            statistical_zero_test: false,
            cancel_check_on_division: true,
            ..AtomField::new()
        };
        let m = Matrix::from_linear(entries, 2, 2, field).unwrap();
        let jac = r.atom(&m.det().unwrap().together().expand()).unwrap();
        assert!(f.local().zero(&(&jac - &ep), &mut b).unwrap());
        assert!((0..2).all(|j| f.derivative(j, &p(&r, "p"), &mut b).unwrap().is_zero()));
    }
}

#[test]
fn relative_transition_general_saturation_not_single_visible_division_or_parameter_inversion() {
    let r = ring();
    let mut b = budget();
    let local = LocalizedAlgebra::new(ideal(&r, &[], &mut b), vec![2, 3], vec![], &mut b).unwrap();
    let input = ideal(&r, &["x*y", "x*p+y"], &mut b);
    let strict = saturation(&local, &input, &p(&r, "x"), &mut b);
    assert!(equal(&strict, &ideal(&r, &["p", "y"], &mut b), &[], &mut b));
    assert!(!equal(
        &strict,
        &ideal(&r, &["y", "x*p+y"], &mut b),
        &[],
        &mut b
    ));
    assert!(!strict.contains(&r.one(), &[], &mut b).unwrap());
    assert!(r.is_parameter(1));
    assert!(local.guards().is_empty());
    // Saturation retained the exceptional parameter stratum. It must not be
    // accepted as a relative Etale support over the unrestricted parameter p.
    let equations = ideal(&r, &["p", "y"], &mut b);
    let y = equations
        .generators()
        .iter()
        .position(|f| f == &p(&r, "y"))
        .unwrap();
    let stratified = LocalizedAlgebra::new(equations, vec![2, 3], vec![], &mut b).unwrap();
    assert!(
        EtaleCertificate {
            source: stratified,
            equations: vec![y],
            dependent_axes: vec![3],
            free_axes: vec![2],
            determinant_inverse_axis: Some(8)
        }
        .verify(&mut b)
        .is_err()
    );
}

#[test]
fn relative_transition_center_boundary_incidence_requires_joint_rank() {
    let r = ring();
    let mut b = budget();
    // A smooth diagonal center and SNC axes are individually smooth, but the
    // center is not normal-crossing with their union at the origin.
    let local = LocalizedAlgebra::new(ideal(&r, &[], &mut b), vec![2, 3], vec![], &mut b).unwrap();
    let center = ideal(&r, &["x-y"], &mut b);
    assert!(!center.contains(&p(&r, "x"), &[], &mut b).unwrap());
    assert!(!center.contains(&p(&r, "y"), &[], &mut b).unwrap());
    let origin = center.sum(&ideal(&r, &["x", "y"], &mut b), &mut b).unwrap();
    let uncovered = OpenCoverCertificate {
        algebra: local,
        support: origin,
        opens: vec![],
    }
    .verify(&mut b);
    assert!(matches!(uncovered, Err(Error::Invalid(_))));
}

#[test]
fn relative_transition_owned_noncoordinate_recursive_center_all_pivots() {
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("next_api::t"),
                symbol!("next_api::p"),
                symbol!("next_api::z"),
                symbol!("next_api::x"),
                symbol!("next_api::y"),
                symbol!("next_api::w"),
                symbol!("next_api::r"),
                symbol!("next_api::iz"),
                symbol!("next_api::id"),
                symbol!("next_api::unused"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let mut b = budget();
    let f = frame(&r, &["z^2-2"], vec![2, 3, 4], vec![2], vec![3, 4], &mut b);
    let SncProduction::Verified(ledger) =
        verify_initial_relative_snc(f.clone(), vec![], &mut b).unwrap()
    else {
        panic!("initial SNC")
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let source = Arc::new(MarkedIdeal::new(ideal(&r, &["y^2-(x-1)^3"], &mut b), 2, &b).unwrap());
    let mut state =
        BoundaryFreeFirstCenter::new(f, source, "next_api_implicit_center", true).unwrap();
    for _ in 0..100 {
        match state.advance(&mut b).unwrap() {
            RecursiveAdvance::Progress => {}
            RecursiveAdvance::Complete => break,
            other => panic!("{other:?}"),
        }
    }
    let RecursiveOutcome::Center(center) = state.outcome() else {
        panic!("recursive contact cover needs multiple opens")
    };
    assert_eq!(center.normals().len(), 2);
    let checked = CheckedRecursiveCenter::new(
        RecursiveCenterOrigin::BoundaryFree {
            center: center.clone(),
            history: history.clone(),
        },
        &mut b,
    )
    .unwrap();
    let adapted = adapt_recursive_center(checked, "next_api_implicit_adapt", &mut b).unwrap();
    assert!(!adapted.center_proof().algebraic_locus_empty());
    assert_eq!(
        adapted
            .opens()
            .iter()
            .filter(|o| o.identity_side().is_none())
            .count(),
        1
    );
    let output = blowup_recursive_center(adapted, "next_api_implicit_blowup", &mut b).unwrap();
    assert_eq!(
        output
            .charts()
            .iter()
            .filter(|c| c.geometry().pivot_normal().is_some())
            .count(),
        2
    );
    assert_eq!(
        output
            .charts()
            .iter()
            .filter(|c| c.geometry().pivot_normal().is_none())
            .count(),
        2
    );
    for (ci, chart) in output.charts().iter().enumerate() {
        assert!(chart.history().same_root(&history));
        assert_eq!(
            chart.history().stage(),
            u64::from(chart.geometry().pivot_normal().is_some())
        );
        assert_eq!(chart.parent().source().mark(), 2);
        assert_eq!(chart.parent().target().mark(), 2);
        assert!(chart.history().old_snapshot().is_empty());
        assert_eq!(chart.parent().generators().len(), 1);
        let contact = center.lift_receipt().unwrap().level().contact().clone();
        let support = transport_contact_support(
            chart.geometry().clone(),
            contact,
            &format!("next_api_implicit_support_{ci}"),
            &mut b,
        )
        .unwrap();
        assert_eq!(
            support.saturation().empty(),
            support.cover().algebraic_locus_empty()
        );
        for open in support.opens() {
            assert_eq!(open.frame().free_axes().len(), 1);
        }
    }
}
