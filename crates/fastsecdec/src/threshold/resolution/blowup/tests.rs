use super::super::{matrix, *};
use super::*;
use std::sync::Arc;
use symbolica::{atom::Symbol, parse, symbol};
fn budget() -> Budget {
    Budget::new(Limits {
        max_operations: 1_000_000,
        max_total_ideal_slots: 1_000_000,
        ..Limits::default()
    })
}
fn ring() -> Arc<Ring> {
    let symbols: [Symbol; 5] = [
        symbol!("blow_probe::p"),
        symbol!("blow_probe::x"),
        symbol!("blow_probe::y"),
        symbol!("blow_probe::q"),
        symbol!("blow_probe::w"),
    ];
    Arc::new(Ring::new(symbols.to_vec(), vec![0]).unwrap())
}
fn p(r: &Ring, a: symbolica::atom::Atom) -> Poly {
    r.atom(&a).unwrap()
}
fn affine(r: Arc<Ring>, axes: Vec<usize>, guards: Vec<Guard>, b: &mut Budget) -> Arc<EtaleFrame> {
    let local =
        LocalizedAlgebra::new(Ideal::new(r, vec![], b).unwrap(), axes.clone(), guards, b).unwrap();
    Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: axes,
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    )
}
fn setup(
    frame: Arc<EtaleFrame>,
    divisors: Vec<Poly>,
    powers: Vec<usize>,
    quotients: Vec<Poly>,
    mark: usize,
    b: &mut Budget,
) -> Arc<ProducedMonomialCenter> {
    let r = frame.local().ring();
    let mut m = r.one();
    for (h, p) in divisors.iter().zip(&powers) {
        let power = b.power(h, *p).unwrap();
        m = b.mul(&m, &power).unwrap();
    }
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(
                r.clone(),
                quotients.iter().map(|q| b.mul(&m, q).unwrap()).collect(),
                b,
            )
            .unwrap(),
            mark,
            b,
        )
        .unwrap(),
    );
    let ledger = match verify_initial_relative_snc(
        frame,
        divisors
            .into_iter()
            .enumerate()
            .map(|(i, equation)| InitialDivisor {
                id: BoundaryId(i as u64),
                equation,
            })
            .collect(),
        b,
    )
    .unwrap()
    {
        SncProduction::Verified(v) => v,
        o => panic!("{o:?}"),
    };
    match produce_monomial_center(
        ledger,
        MonomialWitness {
            source,
            powers,
            quotients,
        },
        b,
    )
    .unwrap()
    {
        MonomialProduction::Center(v) => Arc::new(*v),
        o => panic!("{o:?}"),
    }
}
fn cover(s: &Arc<ProducedMonomialCenter>, ns: &str, b: &mut Budget) -> Arc<AdaptedCover> {
    match produce_adapted_cover(s.ledger().clone(), ns, b).unwrap() {
        AdaptedProduction::Complete(v) => v,
        o => panic!("{o:?}"),
    }
}
fn run(s: Arc<ProducedMonomialCenter>, ns: &str, b: &mut Budget) -> Box<MonomialBlowup> {
    let c = cover(&s, &format!("{ns}_adapt"), b);
    let h = ResolutionHistory::initial(s.ledger().clone()).unwrap();
    match produce_monomial_blowup(s, h, c, ns, b).unwrap() {
        BlowupProduction::Complete(v) => v,
        o => panic!("{o:?}"),
    }
}
#[test]
fn all_origin_charts_history_and_residual_unit() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let s = setup(
        frame,
        vec![r.coordinate(1).unwrap(), r.coordinate(2).unwrap()],
        vec![2, 2],
        vec![r.coordinate(1).unwrap(), p(&r, parse!("1-blow_probe::x"))],
        3,
        &mut b,
    );
    let out = run(s, "blow_origin", &mut b);
    assert_eq!(out.born(), (BoundaryId(2), 1));
    let actual = out
        .charts()
        .iter()
        .filter(|c| c.pivot().is_some())
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), 2);
    assert!(actual.iter().any(|c| c.pivot() == Some(0)));
    assert!(actual.iter().any(|c| c.pivot() == Some(1)));
    assert!(out.charts().iter().any(|c| c.pivot().is_none()));
    for c in out.charts() {
        assert_eq!(c.recombinations(), 2);
        assert_eq!(c.target().mark(), 3);
        assert_eq!(c.history().old_snapshot(), &[BoundaryId(0), BoundaryId(1)]);
        assert_eq!(c.history().births()[&BoundaryId(2)], 1);
        assert_eq!(c.history().stage(), 1);
        assert!(c.frame().local().ring().is_parameter(0));
        if let Some(e) = c.exceptional() {
            assert_eq!(c.jacobian(), e);
            assert!(
                c.divisors()
                    .iter()
                    .find(|d| d.id.0 == c.pivot().unwrap() as u64)
                    .unwrap()
                    .absent
            );
        }
    }
}
#[test]
fn nonlinear_graph_and_cartier_identity() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let s = setup(
        frame,
        vec![p(&r, parse!("blow_probe::x+blow_probe::y^2"))],
        vec![3],
        vec![r.one()],
        2,
        &mut b,
    );
    let out = run(s, "blow_cartier", &mut b);
    let actual = out
        .charts()
        .iter()
        .filter(|c| c.pivot().is_some())
        .collect::<Vec<_>>();
    assert!(!actual.is_empty());
    for c in actual {
        assert_eq!(c.jacobian(), &c.frame().local().ring().one());
        assert_eq!(c.target().ideal().generators().len(), 1);
        assert!(c.divisors()[0].absent);
        assert_eq!(c.history().ledger().divisors().len(), 1);
        assert_eq!(c.history().ledger().divisors()[0].id, BoundaryId(1));
    }
}
#[test]
fn unit_clearing_and_parameters() {
    let r = ring();
    let mut b = budget();
    let frame = affine(
        r.clone(),
        vec![1, 2],
        vec![Guard {
            factor: p(&r, parse!("1+blow_probe::y^2")),
            inverse_axis: 3,
        }],
        &mut b,
    );
    let s = setup(
        frame,
        vec![
            p(&r, parse!("blow_probe::q*blow_probe::x")),
            r.coordinate(2).unwrap(),
        ],
        vec![2, 2],
        vec![r.one()],
        3,
        &mut b,
    );
    let out = run(s, "blow_guarded", &mut b);
    assert_eq!(
        out.charts().iter().filter(|c| c.pivot().is_some()).count(),
        2
    );
    for c in out.charts() {
        for j in 0..c.frame().free_axes().len() {
            assert!(
                c.frame()
                    .derivative(j, &c.frame().local().ring().coordinate(0).unwrap(), &mut b)
                    .unwrap()
                    .is_zero()
            );
        }
    }
}
#[test]
fn resource_frontier_and_wrong_owners() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let s = setup(
        frame,
        vec![r.coordinate(1).unwrap(), r.coordinate(2).unwrap()],
        vec![2, 2],
        vec![r.one()],
        3,
        &mut b,
    );
    let c = cover(&s, "blow_limit_adapt", &mut b);
    let h = ResolutionHistory::initial(s.ledger().clone()).unwrap();
    let other_owner = Arc::new((**s.ledger()).clone());
    let other_history = ResolutionHistory::initial(other_owner).unwrap();
    assert!(
        produce_monomial_blowup(s.clone(), other_history, c.clone(), "wrong_owner", &mut b)
            .is_err()
    );
    let mut complete_budget = budget();
    let BlowupProduction::Complete(complete) = produce_monomial_blowup(
        s.clone(),
        h.clone(),
        c.clone(),
        "budget_reference",
        &mut complete_budget,
    )
    .unwrap() else {
        panic!("reference")
    };
    let mut partial_budget = Budget::new(Limits {
        max_operations: complete.progress().operations - 1,
        ..budget().limits
    });
    let BlowupProduction::Incomplete {
        progress: partial, ..
    } = produce_monomial_blowup(
        s.clone(),
        h.clone(),
        c.clone(),
        "budget_reference",
        &mut partial_budget,
    )
    .unwrap()
    else {
        panic!("partial")
    };
    assert!(!partial.completed.is_empty());
    assert!(partial.completed.len() < complete.charts().len());
    let mut limited = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    let BlowupProduction::Incomplete {
        source,
        history,
        cover,
        progress,
        ..
    } = produce_monomial_blowup(s.clone(), h.clone(), c.clone(), "blow_limit", &mut limited)
        .unwrap()
    else {
        panic!("limit")
    };
    assert!(Arc::ptr_eq(&source, &s));
    assert!(Arc::ptr_eq(&history, &h));
    assert!(Arc::ptr_eq(&cover, &c));
    assert!(progress.completed.is_empty());
    let mut short = Budget::new(Limits {
        max_total_ideal_slots: 0,
        ..Limits::default()
    });
    assert!(matches!(
        produce_adapted_cover(s.ledger().clone(), "blow_limit_cover", &mut short).unwrap(),
        AdaptedProduction::Incomplete { .. }
    ));
}
#[test]
fn multiple_rank_opens_and_independent_native_cover() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let h = p(&r, parse!("blow_probe::x^2+blow_probe::y^2-1"));
    let source = setup(frame, vec![h], vec![2], vec![r.one()], 1, &mut b);
    let out = run(source, "blow_rank", &mut b);
    assert_eq!(
        out.charts().iter().filter(|c| c.pivot().is_some()).count(),
        2
    );
    assert_eq!(out.cover().proof().opens().len(), 3);
    let failed = OpenCoverCertificate {
        algebra: out.cover().proof().algebra().clone(),
        support: out.cover().proof().support().clone(),
        opens: vec![out.cover().proof().opens()[0].clone()],
    }
    .verify(&mut b);
    assert!(failed.is_err());
    for c in out.charts() {
        // The separately retained source-to-adapted determinant is not
        // generally one, even though Cartier blow-up Jacobians are one.
        assert_eq!(c.jacobian(), &c.frame().local().ring().one());
    }
}
#[test]
fn relative_algebraic_equations_survive_every_pivot() {
    let r = ring();
    let mut b = budget();
    let equation = p(&r, parse!("blow_probe::x^2+blow_probe::y^2-1"));
    let relations = Ideal::new(r.clone(), vec![equation.clone()], &mut b).unwrap();
    let local = LocalizedAlgebra::new(relations, vec![1, 2, 4], vec![], &mut b).unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![0],
            dependent_axes: vec![2],
            free_axes: vec![1, 4],
            determinant_inverse_axis: Some(3),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let source = setup(
        frame,
        vec![r.coordinate(1).unwrap(), r.coordinate(4).unwrap()],
        vec![2, 2],
        vec![r.one()],
        3,
        &mut b,
    );
    let out = run(source, "blow_relative", &mut b);
    assert_eq!(
        out.charts().iter().filter(|c| c.pivot().is_some()).count(),
        2
    );
    for c in out.charts() {
        let in_graph = c.open().extension.pull(&equation, &mut b).unwrap();
        let pulled = c.pull_adapted(&in_graph, &mut b).unwrap();
        assert!(c.frame().local().zero(&pulled, &mut b).unwrap());
        assert!(c.frame().dependent_axes().contains(&2));
    }
}
#[test]
fn repeated_checked_births_keep_fixed_old_snapshot() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let source = setup(
        frame,
        vec![r.coordinate(1).unwrap(), r.coordinate(2).unwrap()],
        vec![2, 2],
        vec![r.one()],
        3,
        &mut b,
    );
    let first = run(source, "blow_repeat_first", &mut b);
    let chart = first
        .charts()
        .iter()
        .find(|c| c.pivot() == Some(0))
        .unwrap();
    let ledger = chart.history().ledger().clone();
    assert_eq!(
        ledger.divisors().iter().map(|d| d.id).collect::<Vec<_>>(),
        vec![BoundaryId(1), BoundaryId(2)]
    );
    let witness = MonomialWitness {
        source: chart.target().clone(),
        powers: vec![2, 1],
        quotients: vec![chart.frame().local().ring().one()],
    };
    let second_source = match produce_monomial_center(ledger, witness, &mut b).unwrap() {
        MonomialProduction::Center(v) => Arc::new(*v),
        o => panic!("{o:?}"),
    };
    let second_cover = cover(&second_source, "blow_repeat_second_cover", &mut b);
    let second = match produce_monomial_blowup(
        second_source,
        chart.history().clone(),
        second_cover,
        "blow_repeat_second",
        &mut b,
    )
    .unwrap()
    {
        BlowupProduction::Complete(v) => v,
        o => panic!("{o:?}"),
    };
    assert_eq!(second.born(), (BoundaryId(3), 2));
    for c in second.charts() {
        assert_eq!(c.history().old_snapshot(), &[BoundaryId(0), BoundaryId(1)]);
        assert_eq!(c.history().births()[&BoundaryId(2)], 1);
        assert_eq!(c.history().births()[&BoundaryId(3)], 2);
        assert!(!c.history().old_snapshot().contains(&BoundaryId(2)));
        assert!(!c.history().old_snapshot().contains(&BoundaryId(3)));
    }
}
#[test]
fn exhausted_ids_and_fresh_symbol_collision_are_rejected() {
    let r = ring();
    let mut b = budget();
    let frame = affine(r.clone(), vec![1, 2], vec![], &mut b);
    let ledger = match verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(u64::MAX),
            equation: r.coordinate(1).unwrap(),
        }],
        &mut b,
    )
    .unwrap()
    {
        SncProduction::Verified(v) => v,
        o => panic!("{o:?}"),
    };
    assert!(matches!(
        ResolutionHistory::initial(ledger),
        Err(Error::ResourceIncomplete(_))
    ));
    let symbols = vec![symbol!("blow_collision::v0"), symbol!("blow_collision::v1")];
    let r = Arc::new(Ring::new(symbols, vec![]).unwrap());
    assert!(matches!(
        helpers::Extension::new(r, 2, "blow_collision", &mut b),
        Err(Error::Invalid(_))
    ));
}
#[test]
fn empty_exceptional_on_minor_open_is_not_an_active_divisor() {
    let r = ring();
    let mut b = budget();
    let frame = affine(
        r.clone(),
        vec![1, 2],
        vec![Guard {
            factor: p(&r, parse!("1+blow_probe::x*blow_probe::y")),
            inverse_axis: 3,
        }],
        &mut b,
    );
    let source = setup(
        frame,
        vec![p(
            &r,
            parse!("blow_probe::x*(1+blow_probe::x*blow_probe::y)"),
        )],
        vec![2],
        vec![r.one()],
        1,
        &mut b,
    );
    let out = run(source, "blow_empty_exception", &mut b);
    let missed = out
        .charts()
        .iter()
        .find(|c| c.pivot().is_some() && c.exceptional_is_empty())
        .expect("dy minor misses center");
    assert!(missed.history().ledger().divisors().is_empty());
    assert!(missed.exceptional().is_some());
    assert_eq!(missed.history().births()[&BoundaryId(1)], 1);
    for c in out.charts() {
        assert!(c.history().same_root(out.prior_history()));
        assert_eq!(c.history().chart_path().len(), 1);
        assert_eq!(
            c.history().birth_context(BoundaryId(1)).unwrap().center,
            vec![BoundaryId(0)]
        );
    }
}
#[test]
fn native_matrix_shape_and_ring_are_checked() {
    let r = ring();
    let mut b = budget();
    assert!(matrix::determinant(&r, vec![r.one()], 0, &mut b).is_err());
    let foreign = Arc::new(Ring::new(vec![symbol!("blow_foreign::x")], vec![]).unwrap());
    assert!(matrix::determinant(&r, vec![foreign.one()], 1, &mut b).is_err());
}
