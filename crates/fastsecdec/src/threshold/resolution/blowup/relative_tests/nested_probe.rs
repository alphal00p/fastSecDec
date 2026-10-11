//! Diagnostic native anchoring/transport probe, not a new production owner.
use super::super::{
    coefficient::controlled_supported, embedding::SupportEmbedding, support::transport_embedding,
};
use super::*;
#[path = "family.rs"]
mod family;
fn prepared(b: &mut Budget) -> Arc<EmbeddedProblemCenter> {
    let family = std::env::var("FSD_NESTED_FAMILY").unwrap_or_else(|_| "2,3,3".into());
    let powers = family
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect::<Vec<usize>>();
    let center = family::source_centers([powers[0], powers[1], powers[2]], b).remove(0);
    let mut state =
        LocalCompanionContinuation::new(center, "nested_carry_parent".into(), b).unwrap();
    for _ in 0..1000 {
        let Some(path) = state
            .pending()
            .find(|p| state.nodes()[*p].pending_reason().is_none())
            .cloned()
        else {
            break;
        };
        state
            .advance(&path, &ContinuationLimits::default(), b)
            .unwrap();
    }
    for (i, node) in state.nodes().values().enumerate() {
        let RecursiveCenterOrigin::CarriedMonomial(c) = node.chart().geometry().center().origin()
        else {
            continue;
        };
        let source = EmbeddedPresentation::from_induced(c.induced().clone(), b).unwrap();
        let transition = EmbeddedTransition::prepare(
            source,
            node.chart().clone(),
            &format!("nested_carry_find{i}"),
            b,
        )
        .unwrap();
        let Some(completion) = CompletedEmbeddedChild::prove(transition.clone(), b).unwrap() else {
            continue;
        };
        for (j, p) in transition.presentations(b).unwrap().into_iter().enumerate() {
            let IncidenceContinuation::Dropped(drop) =
                IncidenceDrop::prepare(p, completion.clone(), b).unwrap()
            else {
                continue;
            };
            let problem = drop.initial_problem().unwrap();
            let mut driver = problem
                .recursion(format!("nested_carry_problem{i}_{j}"), b)
                .unwrap();
            for _ in 0..1000 {
                if matches!(
                    driver
                        .advance(&ComponentFactorLimits::default(), b)
                        .unwrap(),
                    RecursiveAdvance::Complete
                ) {
                    break;
                }
            }
            for record in driver.records() {
                if let ProblemRecursionRecord::Center(c) = record
                    && let EmbeddedProblemProduction::Center(c) = c.ascend(b).unwrap()
                {
                    return c;
                }
            }
        }
    }
    panic!("no issued nonmonomial center")
}
fn anchored(c: &Arc<EmbeddedProblemCenter>, b: &mut Budget) -> Vec<Arc<SupportEmbedding>> {
    let source = c.presentation();
    let original = c.child_cycle().original_tree();
    let base =
        SupportEmbedding::strict(source.support().clone(), source.open().clone(), b).unwrap();
    assert!(Arc::ptr_eq(base.frame(), source.history().ledger().frame()));
    let mut embedding = base
        .restrict_unit_open(
            c.prepared()
                .center()
                .coefficient()
                .source()
                .restriction()
                .clone(),
            b,
        )
        .unwrap();
    let mut q = Some(original.center().coefficient().contact().clone());
    let mut out = vec![];
    for level in original.levels() {
        embedding = SupportEmbedding::nested(embedding, q.take().unwrap(), b).unwrap();
        assert!(Arc::ptr_eq(embedding.ambient(), c.frame()));
        assert!(Arc::ptr_eq(embedding.frame(), level.center().frame()));
        assert!(Arc::ptr_eq(
            level.history().ledger().frame(),
            level.center().frame()
        ));
        let local = embedding.frame().local();
        let restriction = embedding.extension().ideal(c.ideal(), b).unwrap();
        assert!(equal(
            &local.ideal().sum(&restriction, b).unwrap(),
            &local.ideal().sum(level.center().ideal(), b).unwrap(),
            local.unit_relations(),
            b
        ));
        q = level
            .center()
            .lift_receipt()
            .map(|r| r.level().contact().clone());
        out.push(embedding.clone());
    }
    assert!(q.is_none());
    out
}
#[test]
fn nested_native_original_anchor_same_physical_pivots_and_actual_lower_state() {
    let mut b = budget();
    let c = prepared(&mut b);
    let embeddings = anchored(&c, &mut b);
    let tree = c.child_cycle().original_tree();
    let checked =
        CheckedRecursiveCenter::new(RecursiveCenterOrigin::EmbeddedProblem(c.clone()), &mut b)
            .unwrap();
    let adapt = adapt_recursive_center(checked, "nested_anchor_adapt", &mut b).unwrap();
    let blowup = blowup_recursive_center(adapt, "nested_anchor_blowup", &mut b).unwrap();
    let mut histories = 0;
    let mut old_presentations = 0;
    let mut empty = 0;
    let mut lower_nonempty = 0;
    let mut completed = 0;
    let mut further_drop = 0;
    let mut companion_resolved = 0;
    for (pi, chart) in blowup.charts().iter().enumerate() {
        let ancestor = EmbeddedTransition::prepare(
            c.presentation().clone(),
            chart.clone(),
            &format!("nested_ancestor{pi}"),
            &mut b,
        )
        .unwrap();
        let presentations = ancestor.presentations(&mut b).unwrap();
        old_presentations += presentations.len();
        if let Some(done) = CompletedEmbeddedChild::prove(ancestor.clone(), &mut b).unwrap() {
            completed += 1;
            for p in presentations {
                match IncidenceDrop::prepare(p, done.clone(), &mut b).unwrap() {
                    IncidenceContinuation::Dropped(_) => further_drop += 1,
                    IncidenceContinuation::CompanionResolved { .. } => companion_resolved += 1,
                }
            }
        }
        for (li, (level, embedding)) in tree.levels().iter().zip(&embeddings).enumerate() {
            let support = transport_embedding(
                chart.geometry().clone(),
                embedding.clone(),
                &format!("nested_support{pi}_{li}"),
                &mut b,
            )
            .unwrap();
            empty += usize::from(support.saturation().empty());
            let center_source =
                Arc::new(MarkedIdeal::new(level.center().ideal().clone(), 1, &b).unwrap());
            for (oi, open) in support.opens().iter().enumerate() {
                let source = controlled_supported(
                    support.clone(),
                    open.clone(),
                    level.center().source().clone(),
                    &format!("nested_source{pi}_{li}_{oi}"),
                    &mut b,
                )
                .unwrap();
                let center = controlled_supported(
                    support.clone(),
                    open.clone(),
                    center_source.clone(),
                    &format!("nested_center{pi}_{li}_{oi}"),
                    &mut b,
                )
                .unwrap();
                let history = level
                    .history()
                    .advance_first_embedded(level.center(), chart, &source, &center, &mut b)
                    .unwrap();
                assert!(history.same_root(level.history()));
                assert_eq!(
                    history.stage(),
                    level.history().stage() + u64::from(chart.geometry().exceptional().is_some())
                );
                histories += 1;
                let mut factors = ComponentFactorFrontier::new(
                    history,
                    source.target().clone(),
                    format!("nested_factors{pi}_{li}_{oi}"),
                    &mut b,
                )
                .unwrap();
                for _ in 0..1000 {
                    let Some(path) = factors.pending().next().cloned() else {
                        break;
                    };
                    assert!(matches!(
                        factors
                            .advance(&path, &ComponentFactorLimits::default(), &mut b)
                            .unwrap(),
                        FactorAdvance::Progress { .. }
                    ));
                }
                let ComponentFactorCompletion::Complete(done) = factors.try_complete().unwrap()
                else {
                    panic!()
                };
                for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
                    lower_nonempty += 1;
                    let order = produce_component_residual_order(leaf.clone(), &mut b).unwrap();
                    match order {
                        ComponentResidualProduction::Order(o) => println!(
                            "pivot{pi} level{li} open{oi} source mark{} residual maximum{} source{}",
                            source.target().mark(),
                            o.algebraic_maximum_on_cosupport(),
                            source
                                .target()
                                .ideal()
                                .generators()
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(";")
                        ),
                        other => println!("pivot{pi} level{li} open{oi} other{other:?}"),
                    }
                }
            }
        }
    }
    assert!(histories + empty > 0 && old_presentations > 0);
    println!(
        "native anchor: levels{} pivots{} histories{histories} outer presentations{old_presentations} empty supports{empty} nonempty factor leaves{lower_nonempty} completed parent charts{completed} further incidence drops{further_drop} companion resolved{companion_resolved}",
        embeddings.len(),
        blowup.charts().len()
    );
}
