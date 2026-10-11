use super::*;
pub(super) fn source_centers(powers: [usize; 3], b: &mut Budget) -> Vec<Arc<CompanionCenter>> {
    let [za, xb, yc] = powers;
    let ns = format!("nested_family_{za}_{xb}_{yc}");
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
    let generators = vec![
        b.power(&z, za).unwrap() - b.power(&x, xb).unwrap(),
        b.power(&y, yc).unwrap(),
    ];
    let source = Arc::new(MarkedIdeal::new(Ideal::new(r, generators, b).unwrap(), 1, b).unwrap());
    let mut state = BoundaryFreeFirstCenter::new(f, source, &format!("{ns}_first"), false).unwrap();
    for _ in 0..100 {
        match state.advance(b).unwrap() {
            RecursiveAdvance::Complete => break,
            RecursiveAdvance::Incomplete { reason } => {
                panic!("family first-center incomplete: {reason}")
            }
            RecursiveAdvance::Progress => {}
        }
    }
    let RecursiveOutcome::Center(first) = state.outcome() else {
        panic!("first center outcome {:?}", state.outcome())
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
