//! Receipt-consuming initial problem checks, before recursive-driver integration.
use super::*;
fn order(problem: &Arc<InitialProblem>, ns: &str, b: &mut Budget) -> Arc<ComponentResidualOrder> {
    order_input(problem.history().clone(), problem.source().clone(), ns, b)
}
fn order_input(
    history: Arc<ResolutionHistory>,
    source: Arc<MarkedIdeal>,
    ns: &str,
    b: &mut Budget,
) -> Arc<ComponentResidualOrder> {
    let mut factor = ComponentFactorFrontier::new(history, source, ns.into(), b).unwrap();
    for _ in 0..100 {
        let Some(path) = factor.pending().next().cloned() else {
            break;
        };
        assert!(matches!(
            factor
                .advance(&path, &ComponentFactorLimits::default(), b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = factor.try_complete().unwrap() else {
        panic!()
    };
    let leaf = done.nodes().values().find_map(|n| n.leaf()).unwrap();
    let ComponentResidualProduction::Order(order) =
        produce_component_residual_order(leaf.clone(), b).unwrap()
    else {
        panic!()
    };
    Arc::new(*order)
}

#[test]
fn nonmonomial_problem_incidence_constructor_preserves_current_history_and_source() {
    let mut b = budget();
    let center = super::nonmonomial_probe::nested_source_centers(1, &mut b).remove(0);
    let mut state =
        LocalCompanionContinuation::new(center, "problem_incidence_state".into(), &mut b).unwrap();
    for _ in 0..1000 {
        let Some(path) = state
            .pending()
            .find(|p| state.nodes()[*p].pending_reason().is_none())
            .cloned()
        else {
            break;
        };
        state
            .advance(&path, &ContinuationLimits::default(), &mut b)
            .unwrap();
    }
    let mut constructed = 0;
    let mut positive = 0;
    for (i, node) in state.nodes().values().enumerate() {
        if node.chart().history().stage() != 3 {
            continue;
        }
        let RecursiveCenterOrigin::CarriedMonomial(c) = node.chart().geometry().center().origin()
        else {
            continue;
        };
        let source = EmbeddedPresentation::from_induced(c.induced().clone(), &mut b).unwrap();
        let transition = EmbeddedTransition::prepare(
            source,
            node.chart().clone(),
            &format!("problem_incidence_transition{i}"),
            &mut b,
        )
        .unwrap();
        let Some(completion) = CompletedEmbeddedChild::prove(transition.clone(), &mut b).unwrap()
        else {
            continue;
        };
        for (j, p) in transition
            .presentations(&mut b)
            .unwrap()
            .into_iter()
            .enumerate()
        {
            let IncidenceContinuation::Dropped(drop) =
                IncidenceDrop::prepare(p, completion.clone(), &mut b).unwrap()
            else {
                continue;
            };
            let problem = InitialProblem::incidence(drop.clone()).unwrap();
            constructed += 1;
            assert!(Arc::ptr_eq(problem.source(), drop.target()));
            assert!(Arc::ptr_eq(problem.history(), drop.source().history()));
            assert!(!problem.history().births().is_empty());
            let order = order(
                &problem,
                &format!("problem_incidence_factor{i}_{j}"),
                &mut b,
            );
            problem.check_order(&order).unwrap();
            let old = InitialProblem::original(drop.source().original_cycle().clone()).unwrap();
            assert!(matches!(old.check_order(&order), Err(Error::Invalid(_))));
            let late_order = order_input(
                problem.history().clone(),
                Arc::new(problem.source().as_ref().clone()),
                &format!("problem_incidence_foreign{i}_{j}"),
                &mut b,
            );
            assert!(matches!(
                problem.check_order(&late_order),
                Err(Error::Invalid(_))
            ));
            if order.algebraic_maximum_on_cosupport() > 0 {
                let snapshot = CycleSnapshot::initial(problem.clone(), order, &mut b).unwrap();
                assert!(snapshot.birth_history().same_root(drop.source().history()));
                assert_eq!(
                    snapshot.old_ids().len(),
                    drop.source().history().ledger().divisors().len()
                );
                positive += 1;
                if positive == 1 {
                    let mut recursion = InitialProblemRecursion::new(
                        problem.clone(),
                        "problem_reusable_recursion".into(),
                        &mut b,
                    )
                    .unwrap();
                    let mut exhausted = Budget::new(Limits {
                        max_operations: 0,
                        ..Limits::default()
                    });
                    assert!(matches!(
                        recursion
                            .advance(&ComponentFactorLimits::default(), &mut exhausted)
                            .unwrap(),
                        RecursiveAdvance::Incomplete { .. }
                    ));
                    assert_eq!(recursion.accepted_steps(), 0);
                    for _ in 0..1000 {
                        if matches!(
                            recursion
                                .advance(&ComponentFactorLimits::default(), &mut b)
                                .unwrap(),
                            RecursiveAdvance::Complete
                        ) {
                            break;
                        }
                    }
                    assert!(recursion.complete_inventory());
                    let centers = recursion
                        .records()
                        .iter()
                        .filter_map(|r| match r {
                            ProblemRecursionRecord::Center(c) => Some(c),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    assert!(!centers.is_empty());
                    for c in &centers {
                        assert!(Arc::ptr_eq(c.problem(), &problem));
                        assert!(
                            c.center()
                                .coefficient()
                                .source()
                                .restriction()
                                .history()
                                .same_root(problem.history())
                        );
                        c.cycle()
                            .check_open(c.center().coefficient().source())
                            .unwrap();
                        let EmbeddedProblemProduction::Center(lifted) = c.ascend(&mut b).unwrap()
                        else {
                            panic!("expected global unit-open ascent in control")
                        };
                        assert!(Arc::ptr_eq(lifted.prepared(), c));
                        assert!(Arc::ptr_eq(
                            lifted.presentation().history(),
                            problem.history()
                        ));
                        assert!(
                            lifted
                                .child_cycle()
                                .initial_history()
                                .ledger()
                                .divisors()
                                .is_empty()
                        );
                        let checked = CheckedRecursiveCenter::new(
                            RecursiveCenterOrigin::EmbeddedProblem(lifted.clone()),
                            &mut b,
                        )
                        .unwrap();
                        let adapted =
                            adapt_recursive_center(checked, "problem_nonmonomial_adapt", &mut b)
                                .unwrap();
                        let physical =
                            blowup_recursive_center(adapted, "problem_nonmonomial_blowup", &mut b)
                                .unwrap();
                        let mut carried = 0;
                        for (pi, chart) in physical.charts().iter().enumerate() {
                            let transition = EmbeddedTransition::prepare(
                                lifted.presentation().clone(),
                                chart.clone(),
                                &format!("problem_nonmonomial_carry{pi}"),
                                &mut b,
                            )
                            .unwrap();
                            for next in transition.presentations(&mut b).unwrap() {
                                assert!(next.history().same_root(lifted.presentation().history()));
                                assert!(Arc::ptr_eq(
                                    next.original_cycle(),
                                    lifted.presentation().original_cycle()
                                ));
                                assert!(Arc::ptr_eq(
                                    next.previous_transition().unwrap().0.source().coefficient(),
                                    lifted.presentation().coefficient()
                                ));
                                carried += 1;
                            }
                        }
                        assert!(carried > 0);
                        println!(
                            "actual nonmonomial physical pivots{} carried original presentations{carried}",
                            physical.charts().len()
                        );
                    }
                    println!(
                        "caller-stepped nonmonomial recursion steps{} records{} centers{}",
                        recursion.accepted_steps(),
                        recursion.records().len(),
                        centers.len()
                    );
                }
            }
        }
    }
    assert!(constructed > 0 && positive > 0);
    println!(
        "genuine incidence constructions{constructed} positive nonmonomial initial cycles{positive}"
    );
}
#[test]
fn nonmonomial_problem_original_constructor_and_localized_nested_owner() {
    let mut b = budget();
    let center = super::nonmonomial_probe::nested_source_centers(1, &mut b).remove(0);
    let original = EmbeddedChildCycle::new(center.clone(), &mut b).unwrap();
    let problem = InitialProblem::original(original.clone()).unwrap();
    let other =
        InitialProblem::original(EmbeddedChildCycle::new(center.clone(), &mut b).unwrap()).unwrap();
    let order = order(&problem, "new_problem_factor", &mut b);
    assert!(matches!(
        CycleSnapshot::initial(other, order.clone(), &mut b),
        Err(Error::Invalid(_))
    ));
    let cycle = CycleSnapshot::initial(problem.clone(), order.clone(), &mut b).unwrap();
    assert!(Arc::ptr_eq(
        cycle.birth_history(),
        original.initial_history()
    ));
    assert!(cycle.old_ids().is_empty());
    let parent = super::super::embedding::SupportEmbedding::contact(
        center.coefficient().contact().clone(),
        &mut b,
    )
    .unwrap();
    let physical_cover = PhysicalSupportCover::prepare(
        parent.clone(),
        center.parent_source().clone(),
        Arc::new(order.upper_order_cover().clone()),
        &mut b,
    )
    .unwrap();
    assert_eq!(
        physical_cover.lower().opens().len(),
        order.upper_order_cover().opens().len()
    );
    let mut complements = 0;
    for index in 0..physical_cover.opens().len() {
        let actual = physical_cover
            .localize(
                center
                    .coefficient()
                    .source()
                    .restriction()
                    .history()
                    .clone(),
                index,
                &format!("new_problem_physical_open{index}"),
                &mut b,
            )
            .unwrap();
        if let CoverLocalizationProduction::Complete(localized) = actual {
            let localized = Arc::new(*localized);
            assert!(
                localized
                    .history()
                    .same_root(center.coefficient().source().restriction().history())
            );
            complements += usize::from(matches!(
                physical_cover.opens()[index],
                PhysicalSupportOpen::BelowMark { .. }
            ));
            match physical_cover
                .restrict_support(
                    localized.clone(),
                    &format!("new_problem_restricted{index}"),
                    &mut b,
                )
                .unwrap()
            {
                PhysicalSupportRestriction::Support(restricted) => {
                    let (actual_cover, actual_open, map) =
                        restricted.physical_localization().unwrap();
                    assert!(Arc::ptr_eq(actual_cover, &physical_cover));
                    assert!(Arc::ptr_eq(actual_open, &localized));
                    assert!(Arc::ptr_eq(
                        restricted.ambient(),
                        localized.history().ledger().frame()
                    ));
                    assert!(Arc::ptr_eq(map.source(), parent.frame()));
                    assert!(restricted.descends_from(&parent));
                    assert!(map.checked_derivatives() > 0);
                }
                PhysicalSupportRestriction::EmptySupport {
                    cover,
                    localization,
                    equations,
                    unit_relations,
                } => {
                    assert!(Arc::ptr_eq(&cover, &physical_cover));
                    assert!(Arc::ptr_eq(&localization, &localized));
                    assert!(
                        equations
                            .contains(&equations.ring().one(), &unit_relations, &mut b)
                            .unwrap()
                    );
                }
            }
        }
    }
    assert!(complements > 0);
    let fake = Arc::new(
        OpenCoverCertificate {
            algebra: parent.frame().local().clone(),
            support: Ideal::new(
                parent.frame().local().ring().clone(),
                vec![parent.frame().local().ring().one()],
                &mut b,
            )
            .unwrap(),
            opens: vec![],
        }
        .verify(&mut b)
        .unwrap(),
    );
    assert!(matches!(
        PhysicalSupportCover::prepare(parent.clone(), center.parent_source().clone(), fake, &mut b),
        Err(Error::Invalid(_))
    ));
    let mut accepted = 0;
    for index in 0..order.upper_order_cover().opens().len() {
        let CompanionOpenProduction::Contact(open) = produce_companion_open(
            order.clone(),
            index,
            &format!("new_problem_open{index}"),
            &mut b,
        )
        .unwrap() else {
            continue;
        };
        let open = Arc::new(*open);
        let localized = match super::super::embedding::SupportEmbedding::localized(
            parent.clone(),
            open.restriction().clone(),
            &mut b,
        ) {
            Ok(owner) => owner,
            Err(Error::ResourceIncomplete(reason)) => {
                println!(
                    "retained pending physical open {index}: {} / {reason}",
                    open.restriction().open().factor()
                );
                continue;
            }
            Err(e) => panic!("{e:?}"),
        };
        assert!(Arc::ptr_eq(
            localized.localized_origin().unwrap().0,
            &parent
        ));
        assert!(localized.descends_from(&parent));
        for ci in 0..open.contact().candidates().len() {
            let local = open.contact().frame().local();
            let differential = &open.contact().candidates()[ci].differential;
            if !local
                .ideal()
                .sum(
                    &Ideal::new(local.ring().clone(), vec![differential.clone()], &mut b).unwrap(),
                    &mut b,
                )
                .unwrap()
                .contains(&local.ring().one(), local.unit_relations(), &mut b)
                .unwrap()
            {
                println!("retained contact open {index}/{ci} requiring physical restriction");
                continue;
            }
            let q = construct_companion_coefficient_for_cycle(
                open.clone(),
                ci,
                [
                    symbol!(format!("new_problem_q{index}_{ci}")),
                    symbol!(format!("new_problem_i{index}_{ci}")),
                ],
                cycle.clone(),
                &mut b,
            )
            .unwrap();
            let CompanionCoefficientProduction::Coefficient(q) = q else {
                continue;
            };
            let q = Arc::new(*q);
            let nested = super::super::embedding::SupportEmbedding::nested(
                localized.clone(),
                q.contact().clone(),
                &mut b,
            )
            .unwrap();
            assert_eq!(nested.codimension(), 2);
            assert!(nested.descends_from(&parent));
            let mut driver = CompanionFirstCenter::new(
                q.clone(),
                &format!("new_problem_child{index}_{ci}"),
                false,
            )
            .unwrap();
            for _ in 0..100 {
                let result = driver.advance(&mut b);
                if let Err(e) = &result {
                    println!(
                        "failed center open{index} ci{ci} parentmark{} max{} source{:?} qmark{} q{:?} error{e:?}",
                        order.current_source().mark(),
                        order.algebraic_maximum_on_cosupport(),
                        order.current_source().ideal().generators(),
                        q.coefficient().coefficient().mark(),
                        q.coefficient().coefficient().ideal().generators()
                    );
                }
                if matches!(result.unwrap(), RecursiveAdvance::Complete) {
                    break;
                }
            }
            assert!(matches!(
                driver.outcome(),
                CompanionCenterOutcome::Center(_)
            ));
            accepted += 1;
        }
    }
    assert!(accepted > 0);
    println!("original nonmonomial initial cycles and actual nested centers {accepted}");
}

#[test]
fn nonmonomial_problem_original_tree_all_physical_pivots_and_no_late_roots() {
    let mut b = budget();
    let center = super::nonmonomial_probe::nested_source_centers(1, &mut b).remove(0);
    let origin = EmbeddedChildCycle::new(center.clone(), &mut b).unwrap();
    let tree = origin.original_tree();
    assert!(tree.levels().len() >= 2);
    assert!(Arc::ptr_eq(
        tree.levels()[0].history(),
        origin.initial_history()
    ));
    let checked =
        CheckedRecursiveCenter::new(RecursiveCenterOrigin::Companion(center), &mut b).unwrap();
    let cover = adapt_recursive_center(checked, "problem_tree_adapt", &mut b).unwrap();
    let blowup = blowup_recursive_center(cover, "problem_tree_blowup", &mut b).unwrap();
    let mut history_count = 0;
    let mut empty_count = 0;
    for (i, chart) in blowup.charts().iter().enumerate() {
        let pulled = OriginalTreePullback::prepare(
            tree.clone(),
            chart.clone(),
            &format!("problem_tree_pull{i}"),
            &mut b,
        )
        .unwrap();
        assert!(Arc::ptr_eq(pulled.original(), tree));
        let histories = pulled.histories(&mut b).unwrap();
        for (j, (level, histories)) in pulled.levels().iter().zip(histories).enumerate() {
            assert!(Arc::ptr_eq(level.original(), &tree.levels()[j]));
            empty_count += usize::from(level.support().saturation().empty());
            for history in histories {
                history_count += 1;
                assert!(history.same_root(tree.levels()[j].history()));
                assert_eq!(
                    history.stage(),
                    u64::from(chart.geometry().exceptional().is_some())
                );
                assert_eq!(
                    history.births().len(),
                    usize::from(chart.geometry().exceptional().is_some())
                );
                assert!(matches!(
                    OriginalRecursionTree::new(tree.center().clone(), history.clone(), &mut b),
                    Err(Error::Invalid(_))
                ));
                assert!(matches!(
                    CycleSnapshot::initial(
                        InitialProblem::original(origin.clone()).unwrap(),
                        order(
                            &InitialProblem::original(
                                EmbeddedChildCycle::new(tree.center().clone(), &mut b).unwrap()
                            )
                            .unwrap(),
                            "problem_wrong_prior_root",
                            &mut b
                        ),
                        &mut b,
                    ),
                    Err(Error::Invalid(_))
                ));
            }
        }
    }
    assert!(history_count > 0 && empty_count > 0);
    println!(
        "original recursion levels{} physical charts{} issued lower histories{history_count} empty supports{empty_count}",
        tree.levels().len(),
        blowup.charts().len()
    );
}

#[test]
fn nonmonomial_problem_zero_auxiliary_source_remains_incomplete_with_factor_evidence() {
    let mut b = budget();
    let r = Arc::new(
        Ring::new(
            vec![symbol!("zero_problem::x"), symbol!("zero_problem::y")],
            vec![],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], &mut b).unwrap(),
        vec![0, 1],
        vec![],
        &mut b,
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
        .verify(&mut b)
        .unwrap(),
    );
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(f, vec![], &mut b).unwrap()
    else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![r.coordinate(0).unwrap()], &mut b).unwrap(),
            1,
            &b,
        )
        .unwrap(),
    );
    let order = order_input(history, source, "zero_problem_factor", &mut b);
    let mut issued = 0;
    for i in 0..order.upper_order_cover().opens().len() {
        let CompanionOpenProduction::Contact(open) =
            produce_companion_open(order.clone(), i, &format!("zero_problem_open{i}"), &mut b)
                .unwrap()
        else {
            continue;
        };
        let open = Arc::new(*open);
        for j in 0..open.contact().candidates().len() {
            let CompanionCoefficientProduction::Coefficient(q) = construct_companion_coefficient(
                open.clone(),
                j,
                [
                    symbol!(format!("zero_problem_q{i}_{j}")),
                    symbol!(format!("zero_problem_inv{i}_{j}")),
                ],
                &mut b,
            )
            .unwrap() else {
                continue;
            };
            let mut first =
                CompanionFirstCenter::new(Arc::new(*q), "zero_problem_first", false).unwrap();
            for _ in 0..32 {
                if matches!(first.advance(&mut b).unwrap(), RecursiveAdvance::Complete) {
                    break;
                }
            }
            let CompanionCenterOutcome::Center(center) = first.outcome() else {
                continue;
            };
            let child = EmbeddedChildCycle::new(center.clone(), &mut b).unwrap();
            let initial = child.initial_problem().unwrap();
            assert!(initial.source().ideal().generators().iter().all(|p| {
                initial
                    .history()
                    .ledger()
                    .frame()
                    .local()
                    .zero(p, &mut b)
                    .unwrap()
            }));
            let mut driver = initial
                .recursion("zero_problem_recursion".into(), &mut b)
                .unwrap();
            for _ in 0..32 {
                match driver
                    .advance(&ComponentFactorLimits::default(), &mut b)
                    .unwrap()
                {
                    RecursiveAdvance::Progress => {}
                    RecursiveAdvance::Incomplete { .. } => break,
                    RecursiveAdvance::Complete => {
                        panic!("zero auxiliary source was promoted to completion")
                    }
                }
            }
            assert!(driver.unresolved_zero_ideal_component());
            assert!(!driver.complete_inventory());
            assert!(driver.records().is_empty());
            let factors = driver.factors().unwrap().clone();
            assert!(factors.nodes().values().any(|n| n.zero_ideal().is_some()));
            let steps = driver.accepted_steps();
            for _ in 0..3 {
                assert!(matches!(
                    driver
                        .advance(&ComponentFactorLimits::default(), &mut b)
                        .unwrap(),
                    RecursiveAdvance::Incomplete { .. }
                ));
                assert!(Arc::ptr_eq(driver.factors().unwrap(), &factors));
                assert_eq!(driver.accepted_steps(), steps);
                assert!(!driver.complete_inventory());
            }
            assert!(Arc::ptr_eq(driver.problem(), &initial));
            assert!(Arc::ptr_eq(initial.history(), child.initial_history()));
            issued += 1;
        }
    }
    assert!(
        issued > 0,
        "control failed to issue a genuine zero coefficient child"
    );
}
