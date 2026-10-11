use super::*;
pub(super) fn actual_second_centers(kind: usize, b: &mut Budget) -> Vec<Arc<CompanionCenter>> {
    let ns = [
        "next_api_repeat_cusp",
        "next_api_repeat_nonprincipal",
        "next_api_repeat_monomials",
    ][kind];
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
    let f = Arc::new(
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
    let SncProduction::Verified(l) = verify_initial_relative_snc(f.clone(), vec![], b).unwrap()
    else {
        panic!()
    };
    let h = ResolutionHistory::initial(l).unwrap();
    let x = r.coordinate(0).unwrap();
    let y = r.coordinate(1).unwrap();
    let mut generators = vec![&y * &y - x.pow(3)];
    if kind == 1 {
        generators.push(y.pow(3));
    } else if kind == 2 {
        generators = vec![x.pow(2), y.pow(3)];
    }
    let source = Arc::new(MarkedIdeal::new(Ideal::new(r, generators, b).unwrap(), 1, b).unwrap());
    let mut state = BoundaryFreeFirstCenter::new(f, source, &format!("{ns}_first"), false).unwrap();
    for _ in 0..100 {
        if matches!(state.advance(b).unwrap(), RecursiveAdvance::Complete) {
            break;
        }
    }
    let RecursiveOutcome::Center(first) = state.outcome() else {
        panic!("first center")
    };
    let initial = first_coordinate_blowup(first.clone(), &format!("{ns}_first_blowup"), b).unwrap();
    let carried = carry_first_blowup(h, initial, b).unwrap();
    let mut centers = Vec::new();
    for (bi, chart) in carried.charts().iter().enumerate() {
        let mut factors = ComponentFactorFrontier::new(
            chart.history().clone(),
            chart.parent().clone(),
            format!("{ns}_factor{bi}"),
            b,
        )
        .unwrap();
        for _ in 0..100 {
            let Some(path) = factors.pending().next().cloned() else {
                break;
            };
            assert!(matches!(
                factors
                    .advance(&path, &ComponentFactorLimits::default(), b)
                    .unwrap(),
                FactorAdvance::Progress { .. }
            ));
        }
        let ComponentFactorCompletion::Complete(done) = factors.try_complete().unwrap() else {
            panic!("factor completion")
        };
        for (fi, leaf) in done.nodes().values().filter_map(|n| n.leaf()).enumerate() {
            let ComponentResidualProduction::Order(order) =
                produce_component_residual_order(leaf.clone(), b).unwrap()
            else {
                continue;
            };
            let order = Arc::new(*order);
            let drop = Arc::new(prove_first_residual_drop(chart.clone(), order.clone()).unwrap());
            if order.algebraic_maximum_on_cosupport() == 0 {
                continue;
            }
            let cycle = CycleSnapshot::after_first_drop(drop, b).unwrap();
            for oi in 0..order.upper_order_cover().opens().len() {
                let CompanionOpenProduction::Contact(open) = produce_companion_open(
                    order.clone(),
                    oi,
                    &format!("{ns}_upper{bi}_{fi}_{oi}"),
                    b,
                )
                .unwrap() else {
                    continue;
                };
                let open = Arc::new(*open);
                let local = open.contact().frame().local();
                for ci in 0..open.contact().candidates().len() {
                    let candidate = &open.contact().candidates()[ci];
                    if !local
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
                        .unwrap()
                        .contains(&local.ring().one(), local.unit_relations(), b)
                        .unwrap()
                    {
                        continue;
                    }
                    let CompanionCoefficientProduction::Coefficient(q) =
                        construct_companion_coefficient_for_cycle(
                            open.clone(),
                            ci,
                            [
                                symbol!(format!("{ns}_contact{bi}_{fi}_{oi}_{ci}")),
                                symbol!(format!("{ns}_inverse{bi}_{fi}_{oi}_{ci}")),
                            ],
                            cycle.clone(),
                            b,
                        )
                        .unwrap()
                    else {
                        continue;
                    };
                    let q = Arc::new(*q);
                    if q.coefficient().algebraic_maximum_old_count() == 0 {
                        continue;
                    }
                    let mut state = CompanionFirstCenter::new(
                        q,
                        &format!("{ns}_second{bi}_{fi}_{oi}_{ci}"),
                        false,
                    )
                    .unwrap();
                    for _ in 0..100 {
                        match state.advance(b).unwrap() {
                            RecursiveAdvance::Progress => {}
                            RecursiveAdvance::Complete => break,
                            e => panic!("{e:?}"),
                        }
                    }
                    let CompanionCenterOutcome::Center(c) = state.outcome() else {
                        panic!("second center")
                    };
                    centers.push(c.clone());
                    break;
                }
            }
        }
    }
    assert!(!centers.is_empty());
    centers
}
#[test]
fn relative_transition_actual_second_blowup_carries_original_c_and_j() {
    for nonprincipal in [0, 1, 2] {
        let mut b = budget();
        let centers = actual_second_centers(nonprincipal, &mut b);
        for (i, center) in centers.iter().enumerate() {
            let lower_cycle = EmbeddedChildCycle::new(center.clone(), &mut b).unwrap();
            assert_eq!(lower_cycle.initial_history().stage(), 0);
            let checked = CheckedRecursiveCenter::new(
                RecursiveCenterOrigin::Companion(center.clone()),
                &mut b,
            )
            .unwrap();
            assert_eq!(checked.history().stage(), 1);
            let a = adapt_recursive_center(
                checked,
                &format!("next_api_second_adapt_{nonprincipal}_{i}"),
                &mut b,
            )
            .unwrap();
            let charts = blowup_recursive_center(
                a,
                &format!("next_api_second_blowup_{nonprincipal}_{i}"),
                &mut b,
            )
            .unwrap();
            let mut actual = 0;
            for (j, chart) in charts.charts().iter().enumerate() {
                let carry = carry_companion_chart(
                    chart.clone(),
                    &format!("next_api_second_carry_{nonprincipal}_{i}_{j}"),
                    &mut b,
                )
                .unwrap();
                if chart.geometry().pivot_normal().is_some() {
                    actual += 1;
                    assert_eq!(chart.history().stage(), 2);
                }
                for (si, open) in carry.opens().iter().enumerate() {
                    let induced = induce_child_chart(
                        lower_cycle.clone(),
                        carry.clone(),
                        si,
                        &format!("next_api_induced_{nonprincipal}_{i}_{j}_{si}"),
                        &mut b,
                    )
                    .unwrap();
                    assert!(induced.history().same_root(lower_cycle.initial_history()));
                    assert!(induced.history().old_snapshot().is_empty());
                    assert_eq!(
                        induced.history().stage(),
                        u64::from(chart.geometry().pivot_normal().is_some())
                    );
                    let mut ff = ComponentFactorFrontier::new(
                        induced.history().clone(),
                        open.incidence_sum().target().clone(),
                        format!("next_api_lower_factor_{nonprincipal}_{i}_{j}_{si}"),
                        &mut b,
                    )
                    .unwrap();
                    for _ in 0..100 {
                        let Some(path) = ff.pending().next().cloned() else {
                            break;
                        };
                        assert!(matches!(
                            ff.advance(&path, &ComponentFactorLimits::default(), &mut b)
                                .unwrap(),
                            FactorAdvance::Progress { .. }
                        ));
                    }
                    let ComponentFactorCompletion::Complete(done) = ff.try_complete().unwrap()
                    else {
                        panic!("lower factor")
                    };
                    for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
                        match produce_induced_monomial_center(induced.clone(), leaf.clone(), &mut b)
                            .unwrap()
                        {
                            InducedCenterProduction::Center(third) => {
                                let checked = CheckedRecursiveCenter::new(
                                    RecursiveCenterOrigin::CarriedMonomial(third),
                                    &mut b,
                                )
                                .unwrap();
                                let adapted = adapt_recursive_center(
                                    checked,
                                    &format!("next_api_third_adapt_{nonprincipal}_{i}_{j}_{si}"),
                                    &mut b,
                                )
                                .unwrap();
                                let next = blowup_recursive_center(
                                    adapted,
                                    &format!("next_api_third_{nonprincipal}_{i}_{j}_{si}"),
                                    &mut b,
                                )
                                .unwrap();
                                assert!(
                                    next.charts()
                                        .iter()
                                        .filter(|c| c.geometry().pivot_normal().is_some())
                                        .count()
                                        >= 2
                                );
                                assert!(next.charts().iter().all(|c| c.history().stage()
                                    == chart.history().stage()
                                        + u64::from(c.geometry().pivot_normal().is_some())));
                                for (ni, nc) in next.charts().iter().enumerate() {
                                    let mut factor = ComponentFactorFrontier::new(
                                        nc.history().clone(),
                                        nc.parent().target().clone(),
                                        format!(
                                            "next_api_terminal_{nonprincipal}_{i}_{j}_{si}_{ni}"
                                        ),
                                        &mut b,
                                    )
                                    .unwrap();
                                    for _ in 0..100 {
                                        let Some(path) = factor.pending().next().cloned() else {
                                            break;
                                        };
                                        assert!(matches!(
                                            factor
                                                .advance(
                                                    &path,
                                                    &ComponentFactorLimits::default(),
                                                    &mut b
                                                )
                                                .unwrap(),
                                            FactorAdvance::Progress { .. }
                                        ));
                                    }
                                    let ComponentFactorCompletion::Complete(done) =
                                        factor.try_complete().unwrap()
                                    else {
                                        panic!("terminal factoring")
                                    };
                                    for (li, leaf) in
                                        done.nodes().values().filter_map(|n| n.leaf()).enumerate()
                                    {
                                        let terminal=certify_local_principalization(leaf.clone(),&format!("next_api_terminal_proof_{nonprincipal}_{i}_{j}_{si}_{ni}_{li}"),&mut b).unwrap();
                                        if nonprincipal != 1 {
                                            assert!(
                                                matches!(
                                                    terminal,
                                                    PrincipalizationProduction::Principal(_)
                                                ),
                                                "cusp/monomial control needs further resolution"
                                            );
                                        }
                                    }
                                }
                            }
                            InducedCenterProduction::Empty { .. } => {}
                            other => panic!("unexpected lower outcome: {other:?}"),
                        }
                    }
                    assert_eq!(
                        open.coefficient().source().ideal(),
                        center
                            .coefficient()
                            .contact()
                            .differential_coefficient()
                            .ideal()
                    );
                    assert_eq!(
                        open.incidence_sum().source().ideal(),
                        center.coefficient().coefficient().coefficient().ideal()
                    );
                    assert_eq!(
                        open.incidence_sum().target().mark(),
                        open.incidence_sum().source().mark()
                    );
                    assert!(Arc::ptr_eq(
                        open.incidence_sum().open(),
                        open.coefficient().open()
                    ));
                }
            }
            assert!(actual >= 2);
        }
    }
}

#[test]
fn relative_transition_reusable_caller_stepped_continuation_to_snc_terminals() {
    for kind in [0, 2, 1] {
        let mut b = budget();
        let centers = actual_second_centers(kind, &mut b);
        for (i, center) in centers.into_iter().enumerate() {
            let mut state = LocalCompanionContinuation::new(
                center,
                format!("next_api_driver_{kind}_{i}"),
                &mut b,
            )
            .unwrap();
            let before = state.nodes().len();
            let path = state.pending().next().unwrap().clone();
            assert!(matches!(
                state
                    .advance(
                        &path,
                        &ContinuationLimits {
                            max_nodes: 0,
                            ..ContinuationLimits::default()
                        },
                        &mut b
                    )
                    .unwrap(),
                ContinuationAdvance::Incomplete(_)
            ));
            assert_eq!(state.nodes().len(), before);
            for _ in 0..1000 {
                let path = state
                    .pending()
                    .find(|p| state.nodes().get(*p).unwrap().pending_reason().is_none())
                    .cloned();
                let Some(path) = path else { break };
                state
                    .advance(&path, &ContinuationLimits::default(), &mut b)
                    .unwrap();
            }
            match state.try_complete().unwrap() {
                ContinuationCompletion::Complete(done) => {
                    if kind == 1 {
                        assert!(
                            done.nodes()
                                .values()
                                .any(|n| n.chart().history().stage() >= 6)
                        );
                    }
                    assert!(
                        done.nodes()
                            .values()
                            .any(|n| n.chart().history().stage() == 3)
                    );
                    assert!(
                        done.nodes()
                            .values()
                            .filter(|n| n.terminal())
                            .all(|n| !n.certificates().is_empty())
                    );
                }
                ContinuationCompletion::Incomplete(state) => {
                    panic!(
                        "local carried driver unresolved: {:?}",
                        state
                            .nodes()
                            .values()
                            .filter_map(|n| n.pending_reason())
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}

#[test]
fn relative_transition_induced_history_rejects_owner_substitution_and_birth_reset() {
    let mut b = budget();
    let original = actual_second_centers(0, &mut b).remove(0);
    let wrong = actual_second_centers(0, &mut b).remove(0);
    let cycle = EmbeddedChildCycle::new(original.clone(), &mut b).unwrap();
    let wrong_cycle = EmbeddedChildCycle::new(wrong, &mut b).unwrap();
    let checked =
        CheckedRecursiveCenter::new(RecursiveCenterOrigin::Companion(original), &mut b).unwrap();
    let cover = adapt_recursive_center(checked, "next_api_mutation_adapt", &mut b).unwrap();
    let charts = blowup_recursive_center(cover, "next_api_mutation_blowup", &mut b).unwrap();
    let mut tested = false;
    for (i, chart) in charts.charts().iter().enumerate() {
        if chart.geometry().pivot_normal().is_none() {
            continue;
        }
        let carry = carry_companion_chart(
            chart.clone(),
            &format!("next_api_mutation_carry{i}"),
            &mut b,
        )
        .unwrap();
        if carry.opens().is_empty() {
            continue;
        }
        assert!(matches!(
            induce_child_chart(
                wrong_cycle.clone(),
                carry.clone(),
                0,
                "next_api_wrong_owner",
                &mut b
            ),
            Err(Error::Invalid(_))
        ));
        let induced =
            induce_child_chart(cycle.clone(), carry, 0, "next_api_valid_owner", &mut b).unwrap();
        assert_eq!(induced.history().stage(), 1);
        assert_eq!(induced.history().births().get(&BoundaryId(0)), Some(&1));
        let forged = ResolutionHistory::initial(induced.history().ledger().clone()).unwrap();
        assert!(!forged.same_root(cycle.initial_history()));
        // Birth authority is independent of the currently active SNC ledger:
        // the exceptional may already be a unit on this lower support.
        if induced
            .history()
            .ledger()
            .divisors()
            .iter()
            .any(|d| d.id == BoundaryId(0))
        {
            assert_eq!(forged.births().get(&BoundaryId(0)), Some(&0));
        } else {
            let exceptional = induced.ambient().chart().geometry().exceptional().unwrap();
            let pulled = induced
                .support()
                .incidence_sum()
                .open()
                .extension()
                .pull(exceptional, &mut b)
                .unwrap();
            assert!(
                super::super::helpers::unit(
                    induced.history().ledger().frame().local(),
                    &pulled,
                    &mut b
                )
                .unwrap()
            );
            assert_eq!(forged.births().get(&BoundaryId(0)), None);
        }
        // The legitimately issued transition keeps its birth even when absent.
        assert_eq!(induced.history().births().get(&BoundaryId(0)), Some(&1));
        let mut ff = ComponentFactorFrontier::new(
            forged,
            induced.support().incidence_sum().target().clone(),
            "next_api_forged_birth".into(),
            &mut b,
        )
        .unwrap();
        for _ in 0..100 {
            let Some(path) = ff.pending().next().cloned() else {
                break;
            };
            ff.advance(&path, &ComponentFactorLimits::default(), &mut b)
                .unwrap();
        }
        let ComponentFactorCompletion::Complete(done) = ff.try_complete().unwrap() else {
            panic!()
        };
        for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
            assert!(matches!(
                produce_induced_monomial_center(induced.clone(), leaf.clone(), &mut b),
                Err(Error::Invalid(_))
            ));
        }
        tested = true;
        break;
    }
    assert!(tested);
}

#[test]
fn relative_transition_reordered_work_retains_semantic_ancestry() {
    let mut b = budget();
    let center = actual_second_centers(0, &mut b).remove(0);
    let mut receipts = Vec::new();
    for reverse in [false, true] {
        let mut state = LocalCompanionContinuation::new(
            center.clone(),
            "relative_reordered_work".into(),
            &mut b,
        )
        .unwrap();
        for _ in 0..1000 {
            let mut pending = state.pending().cloned().collect::<Vec<_>>();
            if reverse {
                pending.reverse();
            }
            let Some(path) = pending.first() else { break };
            assert!(matches!(
                state
                    .advance(path, &ContinuationLimits::default(), &mut b)
                    .unwrap(),
                ContinuationAdvance::Progress
            ));
        }
        let ContinuationCompletion::Complete(done) = state.try_complete().unwrap() else {
            panic!("reordered continuation incomplete")
        };
        receipts.push(
            done.nodes()
                .iter()
                .map(|(p, n)| {
                    (
                        p.clone(),
                        n.chart().history().stage(),
                        n.chart().history().chart_path().to_vec(),
                        n.terminal(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(receipts[0], receipts[1]);
}
