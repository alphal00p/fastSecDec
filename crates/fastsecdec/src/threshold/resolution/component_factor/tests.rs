use super::super::*;
use super::*;
use std::{collections::BTreeSet, sync::Arc};
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
                symbol!("factor_frontier::p"),
                symbol!("factor_frontier::x"),
                symbol!("factor_frontier::y"),
                symbol!("factor_frontier::u"),
                symbol!("factor_frontier::v"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "factor_frontier", Default::default()).unwrap())
        .unwrap()
}
fn flat(r: &Arc<Ring>, guards: Vec<Guard>, b: &mut Budget) -> Arc<EtaleFrame> {
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], b).unwrap(),
        vec![1, 2],
        guards,
        b,
    )
    .unwrap();
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
fn history(frame: Arc<EtaleFrame>, h: Vec<Poly>, b: &mut Budget) -> Arc<ResolutionHistory> {
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(
        frame,
        h.into_iter()
            .enumerate()
            .map(|(i, equation)| InitialDivisor {
                id: BoundaryId(i as u64 + 1),
                equation,
            })
            .collect(),
        b,
    )
    .unwrap() else {
        panic!("SNC gate")
    };
    ResolutionHistory::initial(ledger).unwrap()
}
fn source(r: &Arc<Ring>, f: Vec<Poly>, b: &mut Budget) -> Arc<MarkedIdeal> {
    Arc::new(MarkedIdeal::new(Ideal::new(r.clone(), f, b).unwrap(), 2, b).unwrap())
}
fn run(mut f: ComponentFactorFrontier, reverse: bool, b: &mut Budget) -> CompletedComponentFactors {
    for _ in 0..200 {
        let paths = f.pending().cloned().collect::<Vec<_>>();
        if paths.is_empty() {
            break;
        }
        let path = if reverse {
            paths.last().unwrap()
        } else {
            paths.first().unwrap()
        };
        assert!(matches!(
            f.advance(path, &ComponentFactorLimits::default(), b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = f.try_complete().unwrap() else {
        panic!("unfinished")
    };
    done
}
fn powers(done: &CompletedComponentFactors) -> BTreeSet<Vec<usize>> {
    done.nodes()
        .values()
        .filter_map(|n| n.leaf())
        .map(|l| l.data().powers().to_vec())
        .collect()
}
#[test]
fn component_factor_disconnected_powers_and_reordered_work() {
    let r = ring();
    let mut b = b();
    let h = history(
        flat(&r, vec![], &mut b),
        vec![p(&r, "factor_frontier::x*(factor_frontier::x-1)")],
        &mut b,
    );
    for (formula, expected) in [
        ("factor_frontier::x^2", vec![vec![0], vec![2]]),
        (
            "factor_frontier::x^2*(factor_frontier::x-1)^3",
            vec![vec![2], vec![3]],
        ),
    ] {
        let source = source(&r, vec![p(&r, formula)], &mut b);
        let a = run(
            ComponentFactorFrontier::new(
                h.clone(),
                source.clone(),
                "factor_ordered".into(),
                &mut b,
            )
            .unwrap(),
            false,
            &mut b,
        );
        let z = run(
            ComponentFactorFrontier::new(h.clone(), source, "factor_ordered".into(), &mut b)
                .unwrap(),
            true,
            &mut b,
        );
        assert_eq!(powers(&a), BTreeSet::from_iter(expected));
        assert_eq!(powers(&a), powers(&z));
        assert_eq!(
            a.nodes().keys().collect::<Vec<_>>(),
            z.nodes().keys().collect::<Vec<_>>()
        );
        for (path, n) in a.nodes() {
            assert_eq!(n.data().powers(), z.nodes()[path].data().powers());
            assert!(n.data().history().same_root(&h));
            assert_eq!(n.data().history().old_snapshot(), h.old_snapshot());
            n.data().verify(&mut b).unwrap();
        }
    }
}
#[test]
fn component_factor_multiple_generators_keep_original_association() {
    let r = ring();
    let mut b = b();
    let h = history(
        flat(&r, vec![], &mut b),
        vec![
            p(&r, "factor_frontier::x*(factor_frontier::x-1)"),
            p(&r, "factor_frontier::y"),
        ],
        &mut b,
    );
    let f = source(
        &r,
        vec![
            p(&r, "factor_frontier::x^3*factor_frontier::y^2"),
            p(&r, "factor_frontier::x^2*factor_frontier::y^3"),
            p(&r, "factor_frontier::x^3*factor_frontier::y^2"),
        ],
        &mut b,
    );
    assert_eq!(f.ideal().generators().len(), 2);
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_multigen".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![0, 2], vec![2, 2]]));
    for node in done.nodes().values() {
        assert_eq!(node.data().quotients().len(), 2);
        node.data().verify(&mut b).unwrap();
    }
}
#[test]
fn component_factor_guarded_parameters_and_higher_degree_boundaries() {
    let r = ring();
    let mut b = b();
    let h = history(
        flat(
            &r,
            vec![Guard {
                factor: p(&r, "factor_frontier::p"),
                inverse_axis: 3,
            }],
            &mut b,
        ),
        vec![p(
            &r,
            "factor_frontier::x*(factor_frontier::x-factor_frontier::p)*factor_frontier::u",
        )],
        &mut b,
    );
    let f = source(&r, vec![p(&r, "factor_frontier::x^2")], &mut b);
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_parametric".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![0], vec![2]]));
    for n in done.nodes().values() {
        assert!(
            n.data()
                .history()
                .ledger()
                .frame()
                .local()
                .ring()
                .is_parameter(0)
        );
    }
    let h = history(
        flat(&r, vec![], &mut b),
        vec![p(
            &r,
            "factor_frontier::x*(factor_frontier::x-1)*(factor_frontier::x^3-factor_frontier::x+1)",
        )],
        &mut b,
    );
    let f = source(
        &r,
        vec![p(
            &r,
            "factor_frontier::x^2*(factor_frontier::x^3-factor_frontier::x+1)^3",
        )],
        &mut b,
    );
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_degree5".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![0], vec![2], vec![3]]));
}
#[test]
fn component_factor_zero_components_are_not_integral_zero_and_units_are_absent() {
    let r = ring();
    let mut b = b();
    let local = LocalizedAlgebra::new(
        Ideal::new(
            r.clone(),
            vec![p(&r, "factor_frontier::y*(factor_frontier::y-1)")],
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
            determinant_inverse_axis: Some(4),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let h = history(frame, vec![p(&r, "factor_frontier::x")], &mut b);
    let f = source(
        &r,
        vec![p(&r, "factor_frontier::x^2*factor_frontier::y")],
        &mut b,
    );
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_zero_component".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(
        done.nodes()
            .values()
            .filter(|n| n.zero_ideal().is_some())
            .count(),
        1
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![2]]));
    let h = history(
        flat(
            &r,
            vec![Guard {
                factor: p(&r, "factor_frontier::x"),
                inverse_axis: 3,
            }],
            &mut b,
        ),
        vec![p(&r, "factor_frontier::x")],
        &mut b,
    );
    let f = source(&r, vec![p(&r, "factor_frontier::x^5")], &mut b);
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_unit".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![0]]));
    assert!(matches!(
        &done.nodes()[&vec![]].leaf().unwrap().maximality()[0],
        BoundaryMaximality::Unit { divisor: 0 }
    ));
}
#[test]
fn component_factor_resource_retry_preserves_accepted_work_and_coverage() {
    let r = ring();
    let mut b = b();
    let h = history(
        flat(&r, vec![], &mut b),
        vec![p(&r, "factor_frontier::x*(factor_frontier::x-1)")],
        &mut b,
    );
    let f = source(&r, vec![p(&r, "factor_frontier::x^3")], &mut b);
    let mut front = ComponentFactorFrontier::new(h, f, "factor_retry".into(), &mut b).unwrap();
    let mut tiny = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        front
            .advance(&[], &ComponentFactorLimits::default(), &mut tiny)
            .unwrap(),
        FactorAdvance::ResourceIncomplete(_)
    ));
    assert!(front.nodes()[&vec![]].evidence().is_some());
    assert!(matches!(
        front.clone().try_complete().unwrap(),
        ComponentFactorCompletion::Incomplete(_)
    ));
    // The root first certifies nonzero ambient components, then creates a mixed
    // boundary split. Node and depth caps must not drop either side of its cover.
    for _ in 0..2 {
        assert!(matches!(
            front
                .advance(&[], &ComponentFactorLimits::default(), &mut b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    for limits in [
        ComponentFactorLimits {
            max_nodes: 1,
            ..ComponentFactorLimits::default()
        },
        ComponentFactorLimits {
            max_depth: 0,
            ..ComponentFactorLimits::default()
        },
    ] {
        assert!(matches!(
            front.advance(&[], &limits, &mut b).unwrap(),
            FactorAdvance::ResourceIncomplete(_)
        ));
        assert_eq!(front.nodes().len(), 1);
    }
    let mut capped = front.clone();
    let mut hit = false;
    for _ in 0..50 {
        let Some(path) = capped.pending().next().cloned() else {
            break;
        };
        if matches!(
            capped
                .advance(
                    &path,
                    &ComponentFactorLimits {
                        max_power: 1,
                        ..ComponentFactorLimits::default()
                    },
                    &mut b
                )
                .unwrap(),
            FactorAdvance::ResourceIncomplete(_)
        ) {
            hit = true;
            break;
        }
    }
    assert!(hit);
    assert!(matches!(
        capped.clone().try_complete().unwrap(),
        ComponentFactorCompletion::Incomplete(_)
    ));
    let done = run(capped, true, &mut b);
    assert_eq!(powers(&done), BTreeSet::from([vec![0], vec![3]]));
}
#[test]
fn component_factor_implicit_degree_five_presentation() {
    let r = ring();
    let mut b = b();
    let local = LocalizedAlgebra::new(
        Ideal::new(
            r.clone(),
            vec![p(
                &r,
                "factor_frontier::y^5-factor_frontier::y-factor_frontier::x",
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
            determinant_inverse_axis: Some(4),
        }
        .verify(&mut b)
        .unwrap(),
    );
    let h = history(
        frame,
        vec![p(&r, "factor_frontier::y*(factor_frontier::y-1)")],
        &mut b,
    );
    let f = source(&r, vec![p(&r, "factor_frontier::y^2")], &mut b);
    let done = run(
        ComponentFactorFrontier::new(h, f, "factor_implicit_five".into(), &mut b).unwrap(),
        false,
        &mut b,
    );
    assert_eq!(powers(&done), BTreeSet::from([vec![0], vec![2]]));
    for n in done.nodes().values() {
        assert_eq!(n.data().history().ledger().frame().free_axes(), &[1]);
        n.data().verify(&mut b).unwrap();
    }
}
