use super::super::*;
use std::sync::Arc;
use symbolica::{atom::Atom, symbol};
fn b() -> Budget {
    Budget::new(Limits {
        max_operations: 8_000_000,
        max_total_ideal_slots: 8_000_000,
        ..Limits::default()
    })
}
fn fixture(
    formula: &str,
    mark: usize,
    ns: &'static str,
    b: &mut Budget,
) -> (Arc<ResolutionHistory>, Arc<MarkedIdeal>) {
    let r = Arc::new(
        Ring::new(
            vec![symbol!(format!("{ns}::x")), symbol!(format!("{ns}::y"))],
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
            id: BoundaryId(9),
            equation: r.coordinate(0).unwrap(),
        }],
        b,
    )
    .unwrap() else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let p = r
        .atom(&Atom::parse(formula, ns, Default::default()).unwrap())
        .unwrap();
    let source = Arc::new(MarkedIdeal::new(Ideal::new(r, vec![p], b).unwrap(), mark, b).unwrap());
    (history, source)
}
fn coefficient(
    h: Arc<ResolutionHistory>,
    s: Arc<MarkedIdeal>,
    ns: &str,
    b: &mut Budget,
) -> Arc<CompanionCoefficientChart> {
    let mut f = ComponentFactorFrontier::new(h, s, format!("{ns}_factor"), b).unwrap();
    for _ in 0..100 {
        let Some(path) = f.pending().next().cloned() else {
            break;
        };
        assert!(matches!(
            f.advance(&path, &ComponentFactorLimits::default(), b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = f.try_complete().unwrap() else {
        panic!()
    };
    for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
        let ComponentResidualProduction::Order(order) =
            produce_component_residual_order(leaf.clone(), b).unwrap()
        else {
            continue;
        };
        let order = Arc::new(*order);
        for oi in 0..order.upper_order_cover().opens().len() {
            let CompanionOpenProduction::Contact(open) =
                produce_companion_open(order.clone(), oi, &format!("{ns}_open{oi}"), b).unwrap()
            else {
                continue;
            };
            let open = Arc::new(*open);
            for ci in 0..open.contact().candidates().len() {
                let local = open.contact().frame().local();
                let candidate = &open.contact().candidates()[ci];
                let unit = local
                    .ideal()
                    .sum(
                        &Ideal::new(
                            local.ring().clone(),
                            vec![candidate.differential.clone()],
                            b,
                        )
                        .unwrap(),
                        b,
                    )
                    .unwrap();
                if !unit
                    .contains(&local.ring().one(), local.unit_relations(), b)
                    .unwrap()
                {
                    continue;
                }
                let fresh = [
                    symbol!(format!("{ns}_contact{oi}_{ci}")),
                    symbol!(format!("{ns}_inv{oi}_{ci}")),
                ];
                if let CompanionCoefficientProduction::Coefficient(c) =
                    construct_companion_coefficient(open.clone(), ci, fresh, b).unwrap()
                {
                    return Arc::new(*c);
                }
            }
        }
    }
    panic!("no global unit contact")
}
#[test]
fn bm_continuation_nonempty_old_boundary_actual_child() {
    for (k, formula, mark) in [(0, "x*(y^2-x)", 1), (1, "x^2*y", 3), (2, "x*(y^5-x)", 1)] {
        let ns = ["carried_probe0", "carried_probe1", "carried_probe2"][k];
        println!("case {k}: {formula}");
        let mut b = b();
        let (h, s) = fixture(formula, mark, ns, &mut b);
        let c = coefficient(h, s, ns, &mut b);
        assert_eq!(c.coefficient().algebraic_maximum_old_count(), 1);
        let mut state =
            super::CompanionFirstCenter::new(c.clone(), &format!("{ns}_child"), false).unwrap();
        for _ in 0..50 {
            match state.advance(&mut b).unwrap() {
                RecursiveAdvance::Progress => {}
                RecursiveAdvance::Complete => break,
                RecursiveAdvance::Incomplete { reason } => panic!("{reason}"),
            }
        }
        let super::CompanionCenterOutcome::Center(center) = state.outcome() else {
            panic!("{:?}", state.outcome());
        };
        let local = center.frame().local();
        let ring = local.ring();
        let expected = Ideal::new(
            ring.clone(),
            vec![ring.coordinate(0).unwrap(), ring.coordinate(1).unwrap()],
            &mut b,
        )
        .unwrap();
        for (a, z) in [(center.ideal(), &expected), (&expected, center.ideal())] {
            let z = local.ideal().sum(z, &mut b).unwrap();
            for p in a.generators() {
                assert!(z.contains(p, local.unit_relations(), &mut b).unwrap());
            }
        }
        assert!(Arc::ptr_eq(center.coefficient(), &c));
        assert_eq!(center.old_incidence(), 1);
    }
}
fn flat_source(
    ns: &'static str,
    formula: &str,
    mark: usize,
    b: &mut Budget,
) -> (Arc<EtaleFrame>, Arc<MarkedIdeal>) {
    let r = Arc::new(
        Ring::new(
            vec![symbol!(format!("{ns}::x")), symbol!(format!("{ns}::y"))],
            vec![],
        )
        .unwrap(),
    );
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(
                r.clone(),
                vec![
                    r.atom(&Atom::parse(formula, ns, Default::default()).unwrap())
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
    let local =
        LocalizedAlgebra::new(Ideal::new(r, vec![], b).unwrap(), vec![0, 1], vec![], b).unwrap();
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
    (frame, source)
}
#[test]
fn bm_continuation_contact_normalization_after_ambient_derivatives() {
    let mut b = b();
    let (frame, source) = flat_source("differentiation_order", "y^2+x*y", 2, &mut b);
    let r = frame.local().ring();
    let OrderProduction::ContactCover(cover) =
        produce_ordinary_contact_cover(frame.clone(), Arc::new(source.ideal().clone()), &mut b)
            .unwrap()
    else {
        panic!()
    };
    let index = cover
        .candidates()
        .iter()
        .position(|c| c.equation == r.coordinate(1).unwrap())
        .unwrap();
    let ContactProduction::Constructed(q) = construct_contact_quotient(
        Arc::new(cover),
        index,
        [
            symbol!("differentiation_order::z"),
            symbol!("differentiation_order::inv"),
        ],
        &mut b,
    )
    .unwrap() else {
        panic!()
    };
    let first = &q.progress().completed_jets[0];
    assert!(
        first
            .restricted_normalization
            .normalized()
            .ideal()
            .generators()
            .is_empty()
    );
    let second = &q.progress().completed_jets[1];
    assert!(
        !second
            .restricted_normalization
            .normalized()
            .ideal()
            .generators()
            .is_empty()
    );
    let x = q.contact().local().ring().coordinate(0).unwrap();
    let square = &x * &x;
    let expected = Ideal::new(
        q.contact().local().ring().clone(),
        vec![square.clone()],
        &mut b,
    )
    .unwrap();
    let actual = q.normal_coefficient();
    assert_eq!(actual.mark(), 2);
    let local = q.contact().local();
    assert!(
        local
            .ideal()
            .sum(actual.ideal(), &mut b)
            .unwrap()
            .contains(&square, local.unit_relations(), &mut b)
            .unwrap()
    );
    for f in actual.ideal().generators() {
        assert!(
            local
                .ideal()
                .sum(&expected, &mut b)
                .unwrap()
                .contains(f, local.unit_relations(), &mut b)
                .unwrap()
        );
    }
    for jet in &q.progress().completed_jets {
        assert_eq!(
            jet.restricted.mark(),
            jet.restricted_normalization.normalized().mark()
        );
        assert_eq!(
            jet.restricted.ideal(),
            jet.restricted_normalization.source().ideal()
        );
        assert!(Arc::ptr_eq(
            jet.restricted_normalization.normalizer().local(),
            q.contact().local()
        ));
    }
}
#[test]
fn bm_continuation_carried_coefficient_differs_from_recomputed() {
    let mut b = b();
    let (frame, source) = flat_source("carried_not_rebuilt", "y^2-x^3", 2, &mut b);
    let mut driver =
        BoundaryFreeFirstCenter::new(frame, source, "carried_not_rebuilt_recursion", false)
            .unwrap();
    for _ in 0..32 {
        if matches!(driver.advance(&mut b).unwrap(), RecursiveAdvance::Complete) {
            break;
        }
    }
    let RecursiveOutcome::Center(center) = driver.outcome() else {
        panic!()
    };
    let q = center.lift_receipt().unwrap().level().contact().clone();
    let norm = QuotientNormalizer::prepare(q.contact().local().clone(), &mut b).unwrap();
    let NormalizationOutcome::Complete(c) = norm
        .normalize(Arc::new(q.differential_coefficient().clone()), &mut b)
        .unwrap()
    else {
        panic!()
    };
    let FirstBlowup::MarkedResolved {
        extension, charts, ..
    } = first_coordinate_blowup(center.clone(), "carried_not_rebuilt_blowup", &mut b).unwrap()
    else {
        panic!()
    };
    let chart = charts.iter().find(|c| c.pivot_source_axis() == 0).unwrap();
    let t = chart.transform();
    let ring = extension.target();
    let target_axes = t.map().target().axes();
    let e = ring.coordinate(target_axes[0]).unwrap();
    let v = ring.coordinate(target_axes[1]).unwrap();
    let mut controlled = Vec::new();
    for f in c.normalized().ideal().generators() {
        let p = f
            .rearrange_with_growth(extension.source().one().variables())
            .unwrap();
        let p = extension.pull(&p, &mut b).unwrap();
        let a = t.center().frame().map().pull(&p, &mut b).unwrap();
        let total = t.map().pull(&a, &mut b).unwrap();
        let factor = e.pow(c.normalized().mark());
        let (quotient, remainder) = total.quot_rem(&factor, false);
        assert!(remainder.is_zero());
        assert_eq!(&quotient * &factor, total);
        controlled.push(quotient);
    }
    let carried = Ideal::new(ring.clone(), controlled, &mut b).unwrap();
    // Compare on the actual strict contact v=0. The carried ideal is (e),
    // whereas freshly differentiating I'=v²-e at mark2 yields a unit term.
    let support = Ideal::new(ring.clone(), vec![v], &mut b).unwrap();
    let ideal = support.sum(&carried, &mut b).unwrap();
    assert!(ideal.contains(&e, &[], &mut b).unwrap());
    assert!(!ideal.contains(&ring.one(), &[], &mut b).unwrap());
    for f in t.target().ideal().generators() {
        let dx = f.derivative(target_axes[0]);
        let rebuilt = Ideal::new(ring.clone(), vec![&dx * &dx], &mut b).unwrap();
        assert!(rebuilt.contains(&ring.one(), &[], &mut b).unwrap());
    }
    // Both marked supports are empty for weight2, but their ideals differ.
    assert!(chart.marked_cosupport_empty());
}
#[test]
fn bm_continuation_actual_history_carries_original_coefficient_and_proves_drop() {
    use super::{carry_first_blowup, prove_first_residual_drop};
    for nonprincipal in [false, true] {
        let mut b = b();
        let (frame, source) = flat_source("native_carried_history", "y^2-x^3", 1, &mut b);
        let source = if nonprincipal {
            let r = frame.local().ring();
            let mut gs = source.ideal().generators().to_vec();
            gs.push(r.coordinate(1).unwrap().pow(3));
            Arc::new(MarkedIdeal::new(Ideal::new(r.clone(), gs, &mut b).unwrap(), 1, &b).unwrap())
        } else {
            source
        };
        let SncProduction::Verified(ledger) =
            verify_initial_relative_snc(frame.clone(), vec![], &mut b).unwrap()
        else {
            panic!()
        };
        let initial = ResolutionHistory::initial(ledger).unwrap();
        let ns = if nonprincipal {
            "carried_history_nonprincipal"
        } else {
            "carried_history_principal"
        };
        let mut driver = BoundaryFreeFirstCenter::new(frame, source, ns, false).unwrap();
        for _ in 0..32 {
            if matches!(driver.advance(&mut b).unwrap(), RecursiveAdvance::Complete) {
                break;
            }
        }
        let RecursiveOutcome::Center(center) = driver.outcome() else {
            panic!()
        };
        let first =
            first_coordinate_blowup(center.clone(), &format!("{ns}_blowup"), &mut b).unwrap();
        let carried = carry_first_blowup(initial.clone(), first, &mut b).unwrap();
        assert_eq!(carried.charts().len(), 2);
        assert!(carried.charts().iter().all(|c| c.companion_resolved()));
        assert_eq!(
            carried
                .charts()
                .iter()
                .filter(|c| c.support_is_empty())
                .count(),
            1
        );
        for chart in carried.charts() {
            assert!(chart.history().same_root(&initial));
            assert_eq!(chart.history().stage(), 1);
            assert!(chart.history().old_snapshot().is_empty());
            assert_eq!(chart.history().births().get(&BoundaryId(0)), Some(&1));
            assert!(matches!(
                chart.history().chart_path().last(),
                Some(HistoryStep::CoordinateBlowup { .. })
            ));
            assert_eq!(
                chart.coefficient().mark(),
                chart.original_coefficient().normalized().mark()
            );
            let mut frontier = ComponentFactorFrontier::new(
                chart.history().clone(),
                chart.parent().clone(),
                format!("{ns}_factor{}", chart.transform().pivot_source_axis()),
                &mut b,
            )
            .unwrap();
            for _ in 0..100 {
                let Some(path) = frontier.pending().next().cloned() else {
                    break;
                };
                assert!(matches!(
                    frontier
                        .advance(&path, &ComponentFactorLimits::default(), &mut b)
                        .unwrap(),
                    FactorAdvance::Progress { .. }
                ));
            }
            let ComponentFactorCompletion::Complete(done) = frontier.try_complete().unwrap() else {
                panic!()
            };
            for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
                let order = produce_component_residual_order(leaf.clone(), &mut b).unwrap();
                if let ComponentResidualProduction::Order(order) = order {
                    let proof = prove_first_residual_drop(chart.clone(), Arc::new(*order)).unwrap();
                    assert!(proof.current().algebraic_maximum_on_cosupport() < 2);
                    assert!(Arc::ptr_eq(proof.prior(), chart));
                }
            }
        }
    }
}
#[test]
fn bm_continuation_new_cycle_keeps_geometric_births_and_derives_next_center() {
    use super::super::companion_stage::construct_companion_coefficient_for_cycle;
    use super::{
        CompanionCenterOutcome, CompanionFirstCenter, CycleSnapshot, carry_first_blowup,
        prove_first_residual_drop,
    };
    let mut b = b();
    let (frame, source) = flat_source("next_cycle", "y^2-x^3", 1, &mut b);
    let SncProduction::Verified(ledger) =
        verify_initial_relative_snc(frame.clone(), vec![], &mut b).unwrap()
    else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let mut state = BoundaryFreeFirstCenter::new(frame, source, "next_cycle_child", false).unwrap();
    for _ in 0..32 {
        if matches!(state.advance(&mut b).unwrap(), RecursiveAdvance::Complete) {
            break;
        }
    }
    let RecursiveOutcome::Center(center) = state.outcome() else {
        panic!()
    };
    let blowup = first_coordinate_blowup(center.clone(), "next_cycle_blowup", &mut b).unwrap();
    let carried = carry_first_blowup(history.clone(), blowup, &mut b).unwrap();
    let chart = carried
        .charts()
        .iter()
        .find(|c| c.transform().pivot_source_axis() == 0)
        .unwrap();
    let mut f = ComponentFactorFrontier::new(
        chart.history().clone(),
        chart.parent().clone(),
        "next_cycle_factor".into(),
        &mut b,
    )
    .unwrap();
    for _ in 0..100 {
        let Some(p) = f.pending().next().cloned() else {
            break;
        };
        assert!(matches!(
            f.advance(&p, &ComponentFactorLimits::default(), &mut b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = f.try_complete().unwrap() else {
        panic!()
    };
    let mut found = 0;
    let mut deferred = Vec::new();
    for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
        let ComponentResidualProduction::Order(order) =
            produce_component_residual_order(leaf.clone(), &mut b).unwrap()
        else {
            continue;
        };
        let order = Arc::new(*order);
        let proof = Arc::new(prove_first_residual_drop(chart.clone(), order.clone()).unwrap());
        let cycle = CycleSnapshot::after_first_drop(proof, &mut b).unwrap();
        assert_eq!(cycle.old_ids(), &[BoundaryId(0)]);
        assert!(cycle.birth_history().same_root(&history));
        assert_eq!(cycle.birth_history().stage(), 1);
        assert!(cycle.birth_history().old_snapshot().is_empty());
        for oi in 0..order.upper_order_cover().opens().len() {
            let CompanionOpenProduction::Contact(open) =
                produce_companion_open(order.clone(), oi, &format!("next_cycle_upper{oi}"), &mut b)
                    .unwrap()
            else {
                continue;
            };
            let open = Arc::new(*open);
            for ci in 0..open.contact().candidates().len() {
                let local = open.contact().frame().local();
                let p = &open.contact().candidates()[ci].differential;
                if !local
                    .ideal()
                    .sum(
                        &Ideal::new(local.ring().clone(), vec![p.clone()], &mut b).unwrap(),
                        &mut b,
                    )
                    .unwrap()
                    .contains(&local.ring().one(), local.unit_relations(), &mut b)
                    .unwrap()
                {
                    continue;
                }
                let CompanionCoefficientProduction::Coefficient(q) =
                    construct_companion_coefficient_for_cycle(
                        open.clone(),
                        ci,
                        [
                            symbol!(format!("next_cycle_c{oi}_{ci}")),
                            symbol!(format!("next_cycle_u{oi}_{ci}")),
                        ],
                        cycle.clone(),
                        &mut b,
                    )
                    .unwrap()
                else {
                    panic!()
                };
                let q = Arc::new(*q);
                assert!(Arc::ptr_eq(q.coefficient().cycle().unwrap(), &cycle));
                // These are overlapping upper-order opens. An open away from
                // the old boundary has a lower incidence and is deferred, not
                // an empty integral or a center to combine with this maximum.
                if q.coefficient().algebraic_maximum_old_count() == 0 {
                    deferred.push(q);
                    continue;
                }
                assert_eq!(q.coefficient().algebraic_maximum_old_count(), 1);
                let mut driver =
                    CompanionFirstCenter::new(q, &format!("next_cycle_recursive{oi}_{ci}"), false)
                        .unwrap();
                for _ in 0..32 {
                    match driver.advance(&mut b).unwrap() {
                        RecursiveAdvance::Progress => {}
                        RecursiveAdvance::Complete => break,
                        RecursiveAdvance::Incomplete { reason } => panic!("{reason}"),
                    }
                }
                let CompanionCenterOutcome::Center(next) = driver.outcome() else {
                    panic!()
                };
                let r = next.frame().local().ring();
                let axes = chart.transform().target_frame().free_axes();
                let expected = Ideal::new(
                    r.clone(),
                    axes.iter().map(|i| r.coordinate(*i).unwrap()).collect(),
                    &mut b,
                )
                .unwrap();
                let local = next.frame().local();
                for (a, z) in [(next.ideal(), &expected), (&expected, next.ideal())] {
                    let z = local.ideal().sum(z, &mut b).unwrap();
                    for p in a.generators() {
                        assert!(z.contains(p, local.unit_relations(), &mut b).unwrap());
                    }
                }
                found += 1;
            }
        }
    }
    assert!(found > 0);
    assert!(!deferred.is_empty());
    assert!(
        deferred
            .iter()
            .all(|q| q.coefficient().algebraic_maximum_old_count() == 0)
    );
}
#[test]
fn bm_continuation_owner_refusal_and_atomic_budget_retry() {
    use super::{CompanionCenterOutcome, CompanionFirstCenter, carry_first_blowup};
    let mut budget = b();
    let (h, s) = fixture("x*(y^2-x)", 1, "continuation_retry", &mut budget);
    let q = coefficient(h, s, "continuation_retry", &mut budget);
    let mut driver = CompanionFirstCenter::new(q, "continuation_retry_recursive", false).unwrap();
    let mut empty = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        driver.advance(&mut empty).unwrap(),
        RecursiveAdvance::Incomplete { .. }
    ));
    assert_eq!(driver.accepted_child_steps(), 0);
    assert!(matches!(driver.outcome(), CompanionCenterOutcome::Pending));
    for _ in 0..32 {
        if matches!(
            driver.advance(&mut budget).unwrap(),
            RecursiveAdvance::Complete
        ) {
            break;
        }
    }
    assert!(matches!(
        driver.outcome(),
        CompanionCenterOutcome::Center(_)
    ));
    let (frame, source) = flat_source("carry_owner_retry", "y^2-x^3", 1, &mut budget);
    let SncProduction::Verified(ledger) =
        verify_initial_relative_snc(frame.clone(), vec![], &mut budget).unwrap()
    else {
        panic!()
    };
    let initial = ResolutionHistory::initial(ledger).unwrap();
    let mut driver =
        BoundaryFreeFirstCenter::new(frame.clone(), source, "carry_owner_retry_recursive", false)
            .unwrap();
    for _ in 0..32 {
        if matches!(
            driver.advance(&mut budget).unwrap(),
            RecursiveAdvance::Complete
        ) {
            break;
        }
    }
    let RecursiveOutcome::Center(center) = driver.outcome() else {
        panic!()
    };
    let first =
        first_coordinate_blowup(center.clone(), "carry_owner_retry_blowup", &mut budget).unwrap();
    let copied = Arc::new(frame.as_ref().clone());
    let SncProduction::Verified(wrongledger) =
        verify_initial_relative_snc(copied, vec![], &mut budget).unwrap()
    else {
        panic!()
    };
    let wrong = ResolutionHistory::initial(wrongledger).unwrap();
    assert!(matches!(
        carry_first_blowup(wrong, first.clone(), &mut budget),
        Err(Error::Invalid(_))
    ));
    let mut empty = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        carry_first_blowup(initial.clone(), first.clone(), &mut empty),
        Err(Error::ResourceIncomplete(_))
    ));
    assert_eq!(initial.stage(), 0);
    assert!(initial.births().is_empty());
    let completed = carry_first_blowup(initial.clone(), first, &mut budget).unwrap();
    assert_eq!(completed.charts().len(), 2);
    assert_eq!(initial.stage(), 0);
}
