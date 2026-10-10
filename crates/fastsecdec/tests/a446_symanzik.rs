//! Independent exact binding of the A446 corpus polynomial to a native graph.
//! This tests polynomial provenance, not a CAD or integration result.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use fastsecdec::{
    Atom, AtomCore, EdgeId, FeynmanDiagram, Kinematics, Model,
    input::{GraphIntegral, GraphSymanzik},
};
use feynkit_graph::symbols;
use serde::Deserialize;
use symbolica::{
    atom::{AtomView, Symbol},
    id::Pattern,
    parser::ParseSettings,
    symbol,
};

const DOT: &str = include_str!("../../../examples/no_deformation/a446/graph.dot");
const EXPECTED: &str = include_str!("../../../examples/no_deformation/a446/expected_uf.json");

fn p(text: &str) -> Atom {
    Atom::parse(text, "a446_fixture", ParseSettings::default()).unwrap()
}

fn scalar_symbol(text: &str) -> Symbol {
    match p(text).as_view() {
        AtomView::Var(variable) => variable.get_symbol(),
        _ => panic!("expected scalar symbol"),
    }
}

#[derive(Deserialize)]
struct Term {
    coefficient: String,
    exponents: Vec<u32>,
}

#[derive(Deserialize)]
struct Terms {
    terms: Vec<Term>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Expected {
    parameters: Vec<String>,
    U: Terms,
    F: Terms,
}

fn polynomial(terms: &Terms, parameters: &[String]) -> Atom {
    terms
        .terms
        .iter()
        .map(|term| {
            assert_eq!(term.exponents.len(), parameters.len());
            term.exponents
                .iter()
                .zip(parameters)
                .fold(p(&term.coefficient), |value, (degree, name)| {
                    value * p(name).pow(*degree)
                })
        })
        .sum::<Atom>()
        .expand()
}

fn expected() -> (Atom, Atom) {
    let expected: Expected = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(
        expected.parameters,
        ["x1", "x2", "x3", "x4", "x5", "x7", "x8"]
    );
    assert_eq!(expected.U.terms.len(), 15);
    assert_eq!(expected.F.terms.len(), 57);
    (
        polynomial(&expected.U, &expected.parameters),
        polynomial(&expected.F, &expected.parameters),
    )
}

fn substitute(value: &Atom, from: Atom, to: Atom) -> Atom {
    value
        .replace(Pattern::Literal(from))
        .with(Pattern::Literal(to))
        .expand()
}

fn kinematics() -> Kinematics {
    let momenta: Vec<_> = (0..4)
        .map(|i| symbols::external_momentum().call(i))
        .collect();
    let mut kinematics = Kinematics::in_dimension(&p("D")).unwrap();
    // P0,P1 incoming, P2,P3 outgoing. t=(P0-P3)^2 and
    // u=(P0-P2)^2=2*mz2-s-t. Define all Gram entries so native routing
    // may choose any one dependent external momentum without changing data.
    for (i, j, value) in [
        (0, 0, "0"),
        (1, 1, "0"),
        (2, 2, "mz2"),
        (3, 3, "mz2"),
        (0, 1, "s/2"),
        (0, 2, "(s+t-mz2)/2"),
        (0, 3, "(mz2-t)/2"),
        (1, 2, "(mz2-t)/2"),
        (1, 3, "(s+t-mz2)/2"),
        (2, 3, "(s-2*mz2)/2"),
    ] {
        kinematics = kinematics
            .with_scalar_product(&momenta[i], &momenta[j], p(value))
            .unwrap();
    }
    kinematics
}

fn graph(dot: &str, fixed: bool) -> GraphIntegral {
    let diagram =
        Arc::new(FeynmanDiagram::from_dot(Arc::new(Model::standard_model()), dot).unwrap());
    let mut point = BTreeMap::<Symbol, Atom>::from([(symbol!("UFO::WT"), Atom::Zero)]);
    if fixed {
        point.extend([
            (symbol!("UFO::MT"), Atom::num(1)),
            (scalar_symbol("mz2"), p("5/18")),
            (scalar_symbol("s"), Atom::num(10)),
            (scalar_symbol("t"), Atom::num(-3)),
        ]);
    }
    GraphIntegral::new_with_scalar_values(diagram, &kinematics(), &point).unwrap()
}

fn parameters() -> Vec<Atom> {
    ["x1", "x2", "x3", "x5", "x4", "x7", "x8"].map(p).to_vec()
}

#[test]
fn native_linnet_spanning_trees_independently_reconstruct_u() {
    let graph = graph(DOT, false);
    let native = graph.diagram().underlying();
    let trees = native.all_spanning_forests_of(&graph.diagram().internal_subgraph());
    assert_eq!(trees.len(), 15);
    let bindings: BTreeMap<_, _> = graph
        .propagator_edges()
        .iter()
        .copied()
        .zip(parameters())
        .collect();
    let mut sum = Atom::Zero;
    for tree in trees {
        let included: BTreeSet<_> = native
            .iter_edges_of(&tree)
            .map(|(_, edge, _)| EdgeId(edge.0))
            .collect();
        assert_eq!(included.len(), 5);
        let complement_product = bindings
            .iter()
            .filter(|(edge, _)| !included.contains(*edge))
            .fold(Atom::num(1), |product, (_, parameter)| product * parameter);
        sum += complement_product;
    }
    let (expected_u, _) = expected();
    assert!((sum - expected_u).expand().is_zero());
}

#[test]
fn native_a446_matches_all_symbolic_corpus_coefficients_with_the_physical_sign() {
    let graph = graph(DOT, false);
    let uf = GraphSymanzik::from_graph(&graph, parameters()).unwrap();
    assert_eq!(uf.loop_count(), 2);
    assert_eq!(
        graph.diagram().loop_momentum_basis().external_edges,
        [EdgeId(0), EdgeId(1), EdgeId(2), EdgeId(3)]
    );
    assert_eq!(
        uf.propagator_edges(),
        &[
            EdgeId(4),
            EdgeId(5),
            EdgeId(6),
            EdgeId(7),
            EdgeId(8),
            EdgeId(9),
            EdgeId(10)
        ]
    );
    assert_eq!(graph.diagram().underlying().n_nodes(), 6);
    assert_eq!(graph.diagram().underlying().n_edges(), 11);
    let (u, f) = expected();
    // The native model owns the mass, while the corpus names its square mt2.
    // Equality in the indeterminate MT proves every coefficient of mt2.
    let f = substitute(&f, p("mt2"), p("UFO::MT").pow(2));
    assert!((uf.u() - u).expand().is_zero());
    assert!(
        (uf.f() - &f).expand().is_zero(),
        "native F={}\nexpected F={f}",
        uf.f()
    );
    assert!(!(uf.f() + &f).expand().is_zero());
    let mut wrong_order = parameters();
    wrong_order.swap(1, 2);
    let wrong = GraphSymanzik::from_graph(&graph, wrong_order).unwrap();
    assert!(
        !(wrong.f() - &f).expand().is_zero(),
        "the asymmetric edge-map mutation must be detected"
    );
}

#[test]
fn fixed_a446_and_native_stable_dot_roundtrip_retain_the_same_exact_polynomial() {
    let graph = graph(DOT, true);
    let uf = GraphSymanzik::from_graph(&graph, parameters()).unwrap();
    let (u, mut f) = expected();
    for (from, to) in [("mt2", "1"), ("mz2", "5/18"), ("s", "10"), ("t", "-3")] {
        f = substitute(&f, p(from), p(to));
    }
    assert!((uf.u() - u).expand().is_zero());
    assert!((uf.f() - &f).expand().is_zero());
    let stable_dot = graph.diagram().to_dot().unwrap();
    let restored = crate::graph(&stable_dot, true);
    let again = GraphSymanzik::from_graph(&restored, parameters()).unwrap();
    assert_eq!(uf.propagator_edges(), again.propagator_edges());
    assert_eq!(uf.u(), again.u());
    assert_eq!(uf.f(), again.f());
    let raised = graph
        .with_powers(&BTreeMap::from([(EdgeId(4), 5)]))
        .unwrap();
    let raised_uf = GraphSymanzik::from_graph(&raised, parameters()).unwrap();
    assert_eq!(raised_uf.f(), uf.f());
}
