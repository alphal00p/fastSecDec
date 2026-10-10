use super::super::*;
use std::sync::Arc;
use symbolica::{parse, symbol};
fn budget() -> Budget {
    Budget::new(Limits {
        max_operations: 1_000_000,
        max_total_ideal_slots: 1_000_000,
        ..Limits::default()
    })
}
fn setup(
    powers: Vec<usize>,
    mark: usize,
    unit_combination: bool,
    b: &mut Budget,
) -> Arc<ProducedMonomialCenter> {
    let ring = Arc::new(
        Ring::new(
            vec![
                symbol!("iter_probe::p"),
                symbol!("iter_probe::x"),
                symbol!("iter_probe::y"),
            ],
            vec![0],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(ring.clone(), vec![], b).unwrap(),
        vec![1, 2],
        vec![],
        b,
    )
    .unwrap();
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![1, 2],
            determinant_inverse_axis: None,
        }
        .verify(b)
        .unwrap(),
    );
    let x = ring.coordinate(1).unwrap();
    let y = ring.coordinate(2).unwrap();
    let ledger = match verify_initial_relative_snc(
        frame,
        vec![
            InitialDivisor {
                id: BoundaryId(0),
                equation: x.clone(),
            },
            InitialDivisor {
                id: BoundaryId(1),
                equation: y.clone(),
            },
        ],
        b,
    )
    .unwrap()
    {
        SncProduction::Verified(v) => v,
        o => panic!("{o:?}"),
    };
    let xp = b.power(&x, powers[0]).unwrap();
    let yp = b.power(&y, powers[1]).unwrap();
    let m = b.mul(&xp, &yp).unwrap();
    let quotients = if unit_combination {
        vec![x, ring.atom(&parse!("1-iter_probe::x")).unwrap()]
    } else {
        vec![ring.one()]
    };
    let gens = quotients.iter().map(|q| b.mul(&m, q).unwrap()).collect();
    let source = Arc::new(MarkedIdeal::new(Ideal::new(ring, gens, b).unwrap(), mark, b).unwrap());
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
fn blowup(center: Arc<ProducedMonomialCenter>, ns: &str, b: &mut Budget) -> Arc<MonomialBlowup> {
    let history = ResolutionHistory::initial(center.ledger().clone()).unwrap();
    let cover =
        match produce_adapted_cover(center.ledger().clone(), &format!("{ns}_cover"), b).unwrap() {
            AdaptedProduction::Complete(v) => v,
            o => panic!("{o:?}"),
        };
    match produce_monomial_blowup(center, history, cover, ns, b).unwrap() {
        BlowupProduction::Complete(v) => Arc::from(v),
        o => panic!("{o:?}"),
    }
}
#[test]
fn monomial_iteration_carried_units_all_pivots_and_generated_ranks() {
    for (index, (powers, mark)) in [
        (vec![1, 1], 1),
        (vec![2, 2], 3),
        (vec![3, 2], 4),
        (vec![4, 1], 3),
    ]
    .into_iter()
    .enumerate()
    {
        let mut b = budget();
        let source = setup(powers, mark, true, &mut b);
        let step = blowup(source, &format!("iter_case{index}"), &mut b);
        for i in 0..step.charts().len() {
            let carried = carry_monomial_witness(step.clone(), i, &mut b).unwrap();
            assert!(carried.ranks().1 < carried.ranks().0);
            assert_eq!(
                carried.associations().len(),
                carried.witness().source.ideal().generators().len()
            );
            let checked = produce_monomial_center(
                carried.chart().history().ledger().clone(),
                carried.witness().clone(),
                &mut b,
            )
            .unwrap();
            assert!(!matches!(checked, MonomialProduction::Incomplete { .. }));
        }
    }
}
#[test]
fn monomial_iteration_invalid_selection_overflow_and_budget_are_explicit() {
    let mut b = budget();
    let source = setup(vec![2, 2], 3, false, &mut b);
    let step = blowup(source, "iter_bad", &mut b);
    assert!(carry_monomial_witness(step.clone(), usize::MAX, &mut b).is_err());
    assert!(matches!(
        exponent_rank(&[usize::MAX, 1]),
        Err(Error::ResourceIncomplete(_))
    ));
    let mut limited = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        carry_monomial_witness(step, 0, &mut limited),
        Err(Error::ResourceIncomplete(_))
    ));
}
fn finish(mut f: MonomialFrontier, reverse: bool) -> CompletedMonomialResolution {
    let mut b = budget();
    for _ in 0..500 {
        let path = {
            let paths = f.pending().cloned().collect::<Vec<_>>();
            if paths.is_empty() {
                break;
            }
            if reverse {
                paths.last().unwrap().clone()
            } else {
                paths[0].clone()
            }
        };
        assert!(matches!(
            f.advance(&path, &IterationLimits::default(), &mut b)
                .unwrap(),
            IterationAdvance::Progress { .. }
        ));
    }
    match f.try_complete().unwrap() {
        IterationCompletion::Complete(v) => v,
        IterationCompletion::Incomplete(_) => panic!("incomplete"),
    }
}
#[test]
fn monomial_iteration_caller_stepped_finite_tree_and_reordered_work() {
    let mut b = budget();
    let center = setup(vec![2, 2], 3, true, &mut b);
    let history = ResolutionHistory::initial(center.ledger().clone()).unwrap();
    let forward = finish(
        MonomialFrontier::new(
            history.clone(),
            center.witness().clone(),
            "iter_tree".into(),
        )
        .unwrap(),
        false,
    );
    let backward = finish(
        MonomialFrontier::new(history, center.witness().clone(), "iter_tree".into()).unwrap(),
        true,
    );
    let shape = |r: &CompletedMonomialResolution| {
        r.nodes()
            .iter()
            .map(|(p, n)| (p.clone(), n.rank(), n.terminal()))
            .collect::<Vec<_>>()
    };
    assert_eq!(shape(&forward), shape(&backward));
    assert_eq!(forward.initial_rank(), 4);
    assert!(forward.nodes().values().filter(|n| n.terminal()).count() > 1);
    assert!(forward.nodes().keys().all(|p| p.len() <= 4));
}
#[test]
fn monomial_iteration_limit_retains_frontier_and_resumes_with_new_budget() {
    let mut b = budget();
    let center = setup(vec![2, 2], 3, false, &mut b);
    let history = ResolutionHistory::initial(center.ledger().clone()).unwrap();
    let mut f =
        MonomialFrontier::new(history, center.witness().clone(), "iter_resume".into()).unwrap();
    let limits = IterationLimits {
        max_nodes: 1,
        max_depth: 1,
    };
    for _ in 0..3 {
        assert!(matches!(
            f.advance(&[], &limits, &mut b).unwrap(),
            IterationAdvance::Progress { .. }
        ));
    }
    assert!(matches!(
        f.advance(&[], &limits, &mut b).unwrap(),
        IterationAdvance::ResourceIncomplete("monomial node limit")
    ));
    assert!(f.nodes()[&vec![]].evidence().is_some());
    let IterationCompletion::Incomplete(f) = f.try_complete().unwrap() else {
        panic!("false completion")
    };
    let result = finish(f, false);
    assert!(result.nodes().len() > 1);
}
#[test]
fn monomial_iteration_quotient_ring_associations_do_not_assume_positions() {
    let mut b = budget();
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("iter_assoc::x"),
                symbol!("iter_assoc::y"),
                symbol!("iter_assoc::q"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let equation = r.atom(&parse!("2*iter_assoc::x-1")).unwrap();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![equation], &mut b).unwrap(),
        vec![0, 1],
        vec![],
        &mut b,
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
        .verify(&mut b)
        .unwrap(),
    );
    let ledger = match verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(0),
            equation: r.coordinate(1).unwrap(),
        }],
        &mut b,
    )
    .unwrap()
    {
        SncProduction::Verified(v) => v,
        o => panic!("{o:?}"),
    };
    let quotients = vec![
        r.coordinate(0).unwrap(),
        r.atom(&parse!("1-iter_assoc::x")).unwrap(),
    ];
    let m = b.power(&r.coordinate(1).unwrap(), 2).unwrap();
    let generators = quotients.iter().map(|q| b.mul(&m, q).unwrap()).collect();
    let source =
        Arc::new(MarkedIdeal::new(Ideal::new(r, generators, &mut b).unwrap(), 1, &b).unwrap());
    let center = match produce_monomial_center(
        ledger,
        MonomialWitness {
            source,
            powers: vec![2],
            quotients,
        },
        &mut b,
    )
    .unwrap()
    {
        MonomialProduction::Center(v) => Arc::new(*v),
        o => panic!("{o:?}"),
    };
    let step = blowup(center, "iter_assoc_step", &mut b);
    for index in 0..step.charts().len() {
        let carried = carry_monomial_witness(step.clone(), index, &mut b).unwrap();
        assert_eq!(carried.associations(), &[0, 0]);
        assert_eq!(carried.witness().quotients.len(), 2);
    }
}
#[test]
fn monomial_iteration_nonlinear_unit_factors_and_empty_exceptions_continue() {
    let mut b = budget();
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!("iter_guard::x"),
                symbol!("iter_guard::y"),
                symbol!("iter_guard::q"),
            ],
            vec![],
        )
        .unwrap(),
    );
    let unit = r.atom(&parse!("1+iter_guard::x*iter_guard::y")).unwrap();
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut b).unwrap(),
        vec![0, 1],
        vec![Guard {
            factor: unit.clone(),
            inverse_axis: 2,
        }],
        &mut b,
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
        .verify(&mut b)
        .unwrap(),
    );
    let h = b.mul(&r.coordinate(0).unwrap(), &unit).unwrap();
    let ledger = match verify_initial_relative_snc(
        frame,
        vec![InitialDivisor {
            id: BoundaryId(0),
            equation: h.clone(),
        }],
        &mut b,
    )
    .unwrap()
    {
        SncProduction::Verified(v) => v,
        o => panic!("{o:?}"),
    };
    let h3 = b.power(&h, 3).unwrap();
    let source = Arc::new(
        MarkedIdeal::new(Ideal::new(r.clone(), vec![h3], &mut b).unwrap(), 2, &b).unwrap(),
    );
    let witness = MonomialWitness {
        source,
        powers: vec![3],
        quotients: vec![r.one()],
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let complete = finish(
        MonomialFrontier::new(history, witness, "iter_nonlinear".into()).unwrap(),
        false,
    );
    assert!(
        complete
            .nodes()
            .values()
            .filter_map(|n| n.parent())
            .any(|p| p.chart().pivot().is_some() && p.chart().exceptional_is_empty())
    );
    assert!(
        complete
            .nodes()
            .values()
            .filter(|n| n.terminal())
            .all(|n| n.rank() < 3)
    );
}
