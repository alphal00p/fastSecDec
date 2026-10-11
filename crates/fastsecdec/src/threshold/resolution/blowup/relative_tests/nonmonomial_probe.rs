use super::*;
pub(crate) fn nested_source_centers(kind: usize, b: &mut Budget) -> Vec<Arc<CompanionCenter>> {
    let ns = [
        "nonmonomial_nested_cusp",
        "nonmonomial_nested_nonprincipal",
        "nonmonomial_nested_monomials",
    ][kind];
    let r = Arc::new(
        Ring::new(
            vec![
                symbol!(format!("{ns}::x")),
                symbol!(format!("{ns}::y")),
                symbol!(format!("{ns}::z")),
            ],
            vec![],
        )
        .unwrap(),
    );
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], b).unwrap(),
        vec![0, 1, 2],
        vec![],
        b,
    )
    .unwrap();
    let f = Arc::new(
        EtaleCertificate {
            source: local,
            equations: vec![],
            dependent_axes: vec![],
            free_axes: vec![0, 1, 2],
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
    let z = r.coordinate(2).unwrap();
    let mut generators = vec![&z * &z - x.pow(3)];
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
fn nonmonomial_nested_native_progress_probe() {
    let mut b = budget();
    let centers = nested_source_centers(1, &mut b);
    println!("actual nested centers {}", centers.len());
    let mut detected = false;
    for (i, c) in centers.into_iter().enumerate() {
        let mut state =
            LocalCompanionContinuation::new(c, format!("nested_probe_{i}"), &mut b).unwrap();
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
        println!("nodes {}", state.nodes().len());
        for (p, n) in state.nodes() {
            if let Some(reason) = n.pending_reason() {
                println!(
                    "pending {p:?} stage {}: {reason}",
                    n.chart().history().stage()
                );
                detected |= reason.contains("nonmonomial");
            }
        }
    }
    assert!(
        detected,
        "control did not expose genuinely nonmonomial lower recursion"
    );
}
