use std::{collections::BTreeMap, sync::Arc};

use fastsecdec::{
    Atom, AtomCore, EdgeId, Error, FeynmanDiagram, Kinematics, Model,
    input::{GraphIntegral, default_algebra_settings},
    parametric::ScalarParametricIntegral,
};
use feynkit_graph::symbols;
use symbolica::{parse, symbol};

const MODEL: &str = include_str!("../../../examples/models/massless_phi3.json");
const BUBBLE: &str = include_str!("../../../examples/graphs/bubble.dot");

fn model() -> Arc<Model> {
    Arc::new(Model::from_json(MODEL).unwrap())
}

fn kinematics() -> Kinematics {
    Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        // Native routing chooses P(0) as the dependent external momentum.
        .with_mass_squared(&symbols::external_momentum().call(1), parse!("s"))
        .unwrap()
}

#[test]
fn native_dot_and_linnet_remain_the_graph_owner() {
    let diagram = Arc::new(FeynmanDiagram::from_dot(model(), BUBBLE).unwrap());
    let integral = GraphIntegral::new(Arc::clone(&diagram), &kinematics()).unwrap();
    assert!(Arc::ptr_eq(&diagram, integral.diagram()));
    let graph = integral.diagram().underlying();
    assert_eq!(graph.n_edges(), 4);
    assert_eq!(graph.n_nodes(), 2);
    assert_eq!(integral.diagram().loop_count(), 1);
    assert_eq!(integral.propagator_edges(), &[EdgeId(2), EdgeId(3)]);
    assert_eq!(integral.powers(), &[1, 1]);

    let exported = integral.diagram().to_dot().unwrap();
    let restored = GraphIntegral::from_dot(model(), &exported, &kinematics()).unwrap();
    assert_eq!(restored.diagram().to_dot().unwrap(), exported);
    assert_eq!(
        restored.family().denominators(),
        integral.family().denominators()
    );
}

#[test]
fn hepkit_symanzik_and_measure_have_the_expected_signs() {
    let integral = GraphIntegral::from_dot(model(), BUBBLE, &kinematics()).unwrap();
    let parametric = ScalarParametricIntegral::from_graph(
        &integral,
        vec![parse!("x"), parse!("y")],
        parse!("4-2*eps"),
    )
    .unwrap();
    assert_eq!(parametric.u(), &parse!("x+y"));
    assert!(
        (parametric.f() + parse!("s*x*y")).expand().is_zero(),
        "F = {}",
        parametric.f()
    );
    assert_eq!(parametric.u_exponent().expand(), parse!("-2+2*eps"));
    assert_eq!(parametric.f_exponent().expand(), parse!("-eps"));
    assert_eq!(parametric.prefactor(), &parse!("gamma(eps)"));
    let laurent = parametric
        .prefactor()
        .series(symbol!("eps"), 0, 0)
        .unwrap()
        .to_atom();
    assert_eq!(laurent, parse!("1/eps-euler_gamma"));
}

#[test]
fn raised_powers_use_stable_edges_and_gamma_normalization() {
    let integral = GraphIntegral::from_dot(model(), BUBBLE, &kinematics())
        .unwrap()
        .with_powers(&BTreeMap::from([(EdgeId(2), 2)]))
        .unwrap()
        .with_measure_multiplier(parse!("normalization"));
    let parametric = ScalarParametricIntegral::from_graph(
        &integral,
        vec![parse!("x"), parse!("y")],
        Atom::num(4),
    )
    .unwrap();
    assert_eq!(parametric.powers(), [2, 1]);
    assert_eq!(parametric.prefactor(), &parse!("-normalization"));
    assert_eq!(parametric.u_exponent(), &Atom::num(-1));
    assert_eq!(parametric.f_exponent(), &Atom::num(-1));
    assert!(matches!(
        integral
            .clone()
            .with_powers(&BTreeMap::from([(EdgeId(0), 2)])),
        Err(Error::UnknownPropagator(EdgeId(0)))
    ));
    assert!(matches!(
        integral.with_powers(&BTreeMap::from([(EdgeId(2), 0)])),
        Err(Error::InvalidPower { .. })
    ));
}

#[test]
fn graph_weights_are_applied_once_without_automorphism_division() {
    let dot = r#"digraph weighted_bubble {
        graph [num="3", overall_factor="5", projector="11"];
        ext [style=invis];
        a [num="2"];
        ext -> a [particle="phi"];
        b -> ext [particle="phi"];
        a -> b [particle="phi", num="7", lmb_id=0];
        b -> a [particle="phi"];
    }"#;
    let diagram = FeynmanDiagram::from_dot(model(), dot).unwrap();
    let stable_dot = diagram
        .to_dot()
        .unwrap()
        .replace("symmetry_factor=1,", "symmetry_factor=13,");
    let integral = GraphIntegral::from_dot(model(), &stable_dot, &kinematics()).unwrap();
    assert_eq!(integral.diagram().symmetry_factor(), 13);
    assert_eq!(
        integral
            .scalar_numerator(&default_algebra_settings())
            .unwrap(),
        Atom::num(2310)
    );
}

#[test]
fn tensor_contraction_reuses_native_momentum_and_dimension() {
    let diagram = FeynmanDiagram::from_dot(model(), BUBBLE)
        .unwrap()
        .with_numerator(parse!(
            "gammalooprs::K(0,spenso::mink(4,mu))*gammalooprs::K(0,spenso::mink(4,mu))"
        ))
        .unwrap();
    let integral = GraphIntegral::new(Arc::new(diagram), &kinematics()).unwrap();
    let numerator = integral
        .scalar_numerator(&default_algebra_settings())
        .unwrap();
    let k = symbols::loop_momentum().call(0);
    assert_eq!(
        numerator,
        integral
            .family()
            .kinematics()
            .scalar_product(&k, &k)
            .unwrap()
    );
    assert!(matches!(
        ScalarParametricIntegral::from_graph(
            &integral,
            vec![parse!("x"), parse!("y")],
            parse!("4-2*eps")
        ),
        Err(Error::LoopNumerator)
    ));
}

#[test]
fn residual_free_indices_are_rejected() {
    let diagram = FeynmanDiagram::from_dot(model(), BUBBLE)
        .unwrap()
        .with_numerator(parse!("gammalooprs::K(0,spenso::mink(4,mu))"))
        .unwrap();
    let integral = GraphIntegral::new(Arc::new(diagram), &kinematics()).unwrap();
    assert!(matches!(
        integral.scalar_numerator(&default_algebra_settings()),
        Err(Error::FreeTensorIndices)
    ));
}

#[test]
fn concrete_dimension_is_not_reinterpreted_as_a_symbolic_regulator() {
    let integral = GraphIntegral::from_dot(model(), BUBBLE, &Kinematics::new()).unwrap();
    assert!(matches!(
        ScalarParametricIntegral::from_graph(
            &integral,
            vec![parse!("x"), parse!("y")],
            parse!("4-2*eps")
        ),
        Err(Error::ConcreteDimensionMismatch)
    ));
}

fn modified_model(modify: impl FnOnce(&mut serde_json::Value)) -> Arc<Model> {
    let mut json: serde_json::Value = serde_json::from_str(MODEL).unwrap();
    modify(&mut json);
    Arc::new(Model::from_json(&json.to_string()).unwrap())
}

#[test]
fn widths_are_rejected_before_the_quadratic_family_can_omit_them() {
    let model = modified_model(|json| {
        json["parameters"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "width", "lhablock": null, "lhacode": null,
                "nature": "external", "parameter_type": "real",
                "value": [0.1, 0.0], "expression": null
            }));
        json["particles"][0]["width"] = serde_json::json!("width");
    });
    assert!(matches!(
        GraphIntegral::from_dot(model, BUBBLE, &kinematics()),
        Err(Error::UnsupportedWidth {
            edge: EdgeId(2),
            ..
        })
    ));
}

#[test]
fn complex_masses_are_rejected_before_the_quadratic_family_can_omit_them() {
    let model = modified_model(|json| {
        json["parameters"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name": "mass", "lhablock": null, "lhacode": null,
                "nature": "external", "parameter_type": "complex",
                "value": [1.0, 0.1], "expression": null
            }));
        json["particles"][0]["mass"] = serde_json::json!("mass");
    });
    assert!(matches!(
        GraphIntegral::from_dot(model, BUBBLE, &kinematics()),
        Err(Error::UnsupportedMass {
            edge: EdgeId(2),
            ..
        })
    ));
}

#[test]
fn custom_denominators_are_rejected_before_the_quadratic_family_can_omit_them() {
    let model = modified_model(|json| {
        json["propagators"][0]["denominator"] = serde_json::json!("(UFO::P(UFO::idx(1,1)))^4");
    });
    assert!(matches!(
        GraphIntegral::from_dot(model, BUBBLE, &kinematics()),
        Err(Error::UnsupportedDenominator {
            edge: EdgeId(2),
            ..
        })
    ));
}

#[test]
fn parameter_names_cannot_capture_external_weights_or_the_regulator() {
    let integral = GraphIntegral::from_dot(model(), BUBBLE, &kinematics())
        .unwrap()
        .with_measure_multiplier(parse!("x"));
    assert!(matches!(
        ScalarParametricIntegral::from_graph(
            &integral,
            vec![parse!("x"), parse!("y")],
            parse!("4-2*eps")
        ),
        Err(Error::ParameterCollision(_))
    ));
    let integral = GraphIntegral::from_dot(model(), BUBBLE, &kinematics()).unwrap();
    assert!(matches!(
        ScalarParametricIntegral::from_graph(
            &integral,
            vec![parse!("eps"), parse!("y")],
            parse!("4-2*eps")
        ),
        Err(Error::ParameterCollision(_))
    ));
}
