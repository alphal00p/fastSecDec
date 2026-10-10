use super::super::component_factor::*;
use super::super::*;
use super::component_order::*;
use std::sync::Arc;
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
                symbol!("component_order::p"),
                symbol!("component_order::x"),
                symbol!("component_order::y"),
                symbol!("component_order::u"),
            ],
            vec![0],
        )
        .unwrap(),
    )
}
fn p(r: &Ring, s: &str) -> Poly {
    r.atom(&Atom::parse(s, "component_order", Default::default()).unwrap())
        .unwrap()
}
fn factors(
    formula: &str,
    mark: usize,
    boundaries: bool,
    parameter_guard: bool,
    b: &mut Budget,
) -> CompletedComponentFactors {
    let r = ring();
    let guards = if parameter_guard {
        vec![Guard {
            factor: p(&r, "component_order::p"),
            inverse_axis: 3,
        }]
    } else {
        vec![]
    };
    let local = LocalizedAlgebra::new(
        Ideal::new(r.clone(), vec![], b).unwrap(),
        vec![1, 2],
        guards,
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
    let divisors = if boundaries {
        vec![InitialDivisor {
            id: BoundaryId(1),
            equation: p(&r, "component_order::x*(component_order::x-1)"),
        }]
    } else {
        vec![]
    };
    let SncProduction::Verified(ledger) = verify_initial_relative_snc(frame, divisors, b).unwrap()
    else {
        panic!()
    };
    let history = ResolutionHistory::initial(ledger).unwrap();
    let source = Arc::new(
        MarkedIdeal::new(
            Ideal::new(r.clone(), vec![p(&r, formula)], b).unwrap(),
            mark,
            b,
        )
        .unwrap(),
    );
    let mut frontier =
        ComponentFactorFrontier::new(history, source, "component_order_fixture".into(), b).unwrap();
    for _ in 0..100 {
        let Some(path) = frontier.pending().next().cloned() else {
            break;
        };
        assert!(matches!(
            frontier
                .advance(&path, &ComponentFactorLimits::default(), b)
                .unwrap(),
            FactorAdvance::Progress { .. }
        ));
    }
    let ComponentFactorCompletion::Complete(done) = frontier.try_complete().unwrap() else {
        panic!()
    };
    done
}
#[test]
fn component_order_uses_current_chart_and_corrects_disconnected_whole_order() {
    let mut b = b();
    let done = factors(
        "component_order::x^2*component_order::y^2",
        2,
        true,
        false,
        &mut b,
    );
    let mut powers = Vec::new();
    for factor in done.nodes().values().filter_map(|n| n.leaf()) {
        let ComponentResidualProduction::Order(order) =
            produce_component_residual_order(factor.clone(), &mut b).unwrap()
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(order.factor(), factor));
        assert_eq!(order.algebraic_maximum_on_cosupport(), 2);
        assert_ne!(
            order.current_source().ideal().ring(),
            factor.data().source().ideal().ring()
        );
        assert_eq!(
            order.current_source().ideal().ring(),
            factor.data().history().ledger().frame().local().ring()
        );
        assert_eq!(order.current_source().mark(), 2);
        assert_eq!(order.companion_arithmetic().unwrap().mark(), 2);
        assert!(Arc::ptr_eq(
            order.upper_order_cover().algebra(),
            factor.data().history().ledger().frame().local()
        ));
        powers.push(factor.data().powers()[0]);
    }
    powers.sort();
    assert_eq!(powers, vec![0, 2]);
}
#[test]
fn component_order_weighted_companion_empty_and_monomial_lanes() {
    let mut b = b();
    let done = factors(
        "component_order::x^2*component_order::y",
        3,
        true,
        false,
        &mut b,
    );
    let mut empty = 0;
    let mut positive = 0;
    for factor in done.nodes().values().filter_map(|n| n.leaf()) {
        match produce_component_residual_order(factor.clone(), &mut b).unwrap() {
            ComponentResidualProduction::EmptyCosupport { .. } => empty += 1,
            ComponentResidualProduction::Order(order) => {
                positive += 1;
                assert_eq!(order.algebraic_maximum_on_cosupport(), 1);
                assert_eq!(order.companion_arithmetic().unwrap().mark(), 2);
            }
            _ => panic!(),
        }
    }
    assert_eq!((empty, positive), (1, 1));
    let done = factors("component_order::x^2", 2, true, false, &mut b);
    for factor in done.nodes().values().filter_map(|n| n.leaf()) {
        if let ComponentResidualProduction::Order(order) =
            produce_component_residual_order(factor.clone(), &mut b).unwrap()
        {
            assert_eq!(order.algebraic_maximum_on_cosupport(), 0);
            assert!(order.companion_arithmetic().is_none());
        }
    }
}
#[test]
fn component_order_parameters_and_resource_stops_are_not_genericized() {
    let mut b = b();
    let done = factors("component_order::p", 1, false, false, &mut b);
    let factor = done.nodes().values().find_map(|n| n.leaf()).unwrap();
    let mut cap = Budget::new(Limits {
        max_mark: 3,
        ..Limits::default()
    });
    let ComponentResidualProduction::Incomplete {
        factor: owner,
        progress,
        ..
    } = produce_component_residual_order(factor.clone(), &mut cap).unwrap()
    else {
        panic!("parameter locus silently genericized")
    };
    assert!(Arc::ptr_eq(factor, &owner));
    assert_eq!(progress.residual_layers.len(), 4);
    let mut cap = Budget::new(Limits {
        max_operations: 0,
        ..Limits::default()
    });
    assert!(matches!(
        produce_component_residual_order(factor.clone(), &mut cap).unwrap(),
        ComponentResidualProduction::Incomplete { .. }
    ));
    let done = factors("component_order::p", 1, false, true, &mut b);
    let factor = done.nodes().values().find_map(|n| n.leaf()).unwrap();
    assert!(matches!(
        produce_component_residual_order(factor.clone(), &mut b).unwrap(),
        ComponentResidualProduction::EmptyCosupport { .. }
    ));
}
