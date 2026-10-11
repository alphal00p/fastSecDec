use super::continuation::actual_second_centers;
use super::*;
#[test]
fn next_cycle_reordered_repeated_work_and_atomic_resource_retry() {
    let mut b = budget();
    let center = actual_second_centers(1, &mut b).remove(0);
    let mut receipts = Vec::new();
    for reverse in [false, true] {
        let mut state =
            LocalCompanionContinuation::new(center.clone(), "cycle_ordered".into(), &mut b)
                .unwrap();
        let mut retried = false;
        for _ in 0..2000 {
            let mut paths = state.pending().cloned().collect::<Vec<_>>();
            if reverse {
                paths.reverse();
            }
            let Some(path) = paths.first() else { break };
            if !retried && state.nodes()[path].chart().history().stage() >= 3 {
                let before = state.nodes().len();
                let old = state.nodes()[path].chart().clone();
                let mut exhausted = Budget::new(Limits {
                    max_operations: 0,
                    ..Limits::default()
                });
                assert!(matches!(
                    state
                        .advance(path, &ContinuationLimits::default(), &mut exhausted)
                        .unwrap(),
                    ContinuationAdvance::Incomplete(_)
                ));
                assert_eq!(state.nodes().len(), before);
                assert!(Arc::ptr_eq(state.nodes()[path].chart(), &old));
                assert_eq!(
                    state.pending().cloned().collect::<Vec<_>>().len(),
                    paths.len()
                );
                retried = true;
            }
            assert!(matches!(
                state
                    .advance(path, &ContinuationLimits::default(), &mut b)
                    .unwrap(),
                ContinuationAdvance::Progress
            ));
        }
        assert!(retried);
        let ContinuationCompletion::Complete(done) = state.try_complete().unwrap() else {
            panic!("repeated work left incomplete")
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
#[test]
fn next_cycle_original_coefficient_incidence_drop_and_birth_fences() {
    let mut b = budget();
    let centers = actual_second_centers(1, &mut b);
    for (i, center) in centers.into_iter().enumerate() {
        let mut state =
            LocalCompanionContinuation::new(center, format!("cycle_diag_{i}"), &mut b).unwrap();
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
        assert!(matches!(
            state.clone().try_complete().unwrap(),
            ContinuationCompletion::Complete(_)
        ));
        println!(
            "completed {} nodes, max physical stage {}",
            state.nodes().len(),
            state
                .nodes()
                .values()
                .map(|n| n.chart().history().stage())
                .max()
                .unwrap()
        );
        let mut repeat_controls = 0;
        for node in state.nodes().values() {
            if node.chart().history().stage() != 3 || node.terminal() {
                continue;
            }
            let chart = node.chart();
            if let RecursiveCenterOrigin::CarriedMonomial(center) =
                chart.geometry().center().origin()
            {
                repeat_controls += 1;
                let ind = center.induced();
                let presentation = super::super::presentation::EmbeddedPresentation::from_induced(
                    ind.clone(),
                    &mut b,
                )
                .unwrap();
                let transition =
                    super::super::presentation_transition::EmbeddedTransition::prepare(
                        presentation,
                        chart.clone(),
                        "cycle_owned_transition",
                        &mut b,
                    )
                    .unwrap();
                let next = transition.presentations(&mut b).unwrap();
                let mut missing_birth = (*transition).clone();
                let mut bad_open = (*missing_birth.opens[0]).clone();
                bad_open.born = None;
                missing_birth.opens[0] = Arc::new(bad_open);
                assert!(matches!(
                    Arc::new(missing_birth).presentations(&mut b),
                    Err(Error::Invalid(_))
                ));
                let mut wrong_coefficient = (*transition).clone();
                let mut bad_open = (*wrong_coefficient.opens[0]).clone();
                bad_open.coefficient = bad_open.incidence_sum.clone();
                wrong_coefficient.opens[0] = Arc::new(bad_open);
                assert!(matches!(
                    Arc::new(wrong_coefficient).presentations(&mut b),
                    Err(Error::Invalid(_))
                ));
                let mut reset = (*transition).clone();
                let mut reset_source = (*reset.source).clone();
                reset_source.history =
                    ResolutionHistory::initial(reset_source.history.ledger().clone()).unwrap();
                reset.source = Arc::new(reset_source);
                assert!(matches!(
                    Arc::new(reset).presentations(&mut b),
                    Err(Error::Invalid(_))
                ));
                let completion = super::super::incidence::CompletedEmbeddedChild::prove(
                    transition.clone(),
                    &mut b,
                )
                .unwrap()
                .unwrap();
                assert_eq!(next.len(), transition.support().opens().len());
                for p in &next {
                    assert!(p.history().same_root(ind.history()));
                    assert_eq!(p.history().stage(), ind.history().stage() + 1);
                    assert_eq!(p.history().births().len(), ind.history().births().len() + 1);
                    assert_eq!(p.old_ids(), transition.source().old_ids());
                    assert!(Arc::ptr_eq(p.original_cycle(), ind.origin()));
                    let super::super::incidence::IncidenceContinuation::Dropped(drop) =
                        super::super::incidence::IncidenceDrop::prepare(
                            p.clone(),
                            completion.clone(),
                            &mut b,
                        )
                        .unwrap()
                    else {
                        panic!("retained original C must require the incidence subcycle")
                    };
                    assert_eq!(drop.maximum(), 0);
                    let redefined = drop.presentation();
                    assert!(Arc::ptr_eq(redefined.coefficient(), p.coefficient()));
                    assert!(Arc::ptr_eq(redefined.history(), p.history()));
                    assert_eq!(redefined.old_ids(), p.old_ids());
                    assert_eq!(redefined.incidence_sum().ideal(), p.coefficient().ideal());
                }
                let embedding = super::super::embedding::SupportEmbedding::strict(
                    ind.ambient().support().clone(),
                    ind.support().incidence_sum().open().clone(),
                    &mut b,
                )
                .unwrap();
                let carried = super::super::support::transport_embedding(
                    chart.geometry().clone(),
                    embedding,
                    "cycle_probe_repeated_support",
                    &mut b,
                )
                .unwrap();
                assert!(!carried.opens().is_empty());
                let mut nonunit_c = 0;
                for (oi, open) in carried.opens().iter().enumerate() {
                    let c = super::super::coefficient::controlled_supported(
                        carried.clone(),
                        open.clone(),
                        ind.support().coefficient().target().clone(),
                        &format!("cycle_probe_C{oi}"),
                        &mut b,
                    )
                    .unwrap();
                    let j = super::super::coefficient::controlled_supported(
                        carried.clone(),
                        open.clone(),
                        ind.support().incidence_sum().target().clone(),
                        &format!("cycle_probe_J{oi}"),
                        &mut b,
                    )
                    .unwrap();
                    let center_source =
                        Arc::new(MarkedIdeal::new(center.child().center().clone(), 1, &b).unwrap());
                    let center_total = super::super::coefficient::controlled_supported(
                        carried.clone(),
                        open.clone(),
                        center_source,
                        &format!("cycle_probe_center{oi}"),
                        &mut b,
                    )
                    .unwrap();
                    let local = open.frame().local();
                    assert!(
                        local
                            .ideal()
                            .sum(center_total.target().ideal(), &mut b)
                            .unwrap()
                            .contains(&local.ring().one(), local.unit_relations(), &mut b)
                            .unwrap()
                    );
                    assert!(
                        local
                            .ideal()
                            .sum(j.target().ideal(), &mut b)
                            .unwrap()
                            .contains(&local.ring().one(), local.unit_relations(), &mut b)
                            .unwrap()
                    );
                    let c_unit = local
                        .ideal()
                        .sum(c.target().ideal(), &mut b)
                        .unwrap()
                        .contains(&local.ring().one(), local.unit_relations(), &mut b)
                        .unwrap();
                    nonunit_c += usize::from(!c_unit);
                    for div in ind.history().ledger().divisors() {
                        let old = super::super::coefficient::controlled_supported(
                            carried.clone(),
                            open.clone(),
                            Arc::new(
                                MarkedIdeal::new(
                                    Ideal::new(
                                        ind.history().ledger().frame().local().ring().clone(),
                                        vec![div.equation.clone()],
                                        &mut b,
                                    )
                                    .unwrap(),
                                    1,
                                    &b,
                                )
                                .unwrap(),
                            ),
                            &format!("cycle_probe_old{oi}"),
                            &mut b,
                        )
                        .unwrap();
                        assert!(
                            local
                                .ideal()
                                .sum(old.target().ideal(), &mut b)
                                .unwrap()
                                .contains(&local.ring().one(), local.unit_relations(), &mut b)
                                .unwrap()
                        );
                    }
                }
                assert!(nonunit_c > 0);
            }
        }
        assert!(repeat_controls > 0);
    }
}
