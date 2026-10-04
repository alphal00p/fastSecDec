use super::{Guard, replay};
use crate::{
    Atom, Kinematics, Model, ParameterCard,
    generation::{GenerationError, GenerationOptions, GenerationProgress, generate},
    input::GraphIntegral,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow, path::PathBuf, sync::Arc};
use symbolica::{atom::AtomCore, parse, symbol};

#[test]
fn native_relative_depth_covers_high_poles_cancellation_and_negative_orders() {
    let epsilon = symbol!("laurent_replay_control::eps");
    for (expression, maximum) in [
        (
            parse!(
                "gamma(2*laurent_replay_control::eps)/laurent_replay_control::eps^4*(1+x)^laurent_replay_control::eps"
            ),
            0,
        ),
        (
            parse!(
                "(exp(laurent_replay_control::eps)-1-laurent_replay_control::eps-laurent_replay_control::eps^2/2)/laurent_replay_control::eps^6"
            ),
            0,
        ),
        (
            parse!(
                "gamma(2*laurent_replay_control::eps)*(exp(laurent_replay_control::eps)-1)/laurent_replay_control::eps^4"
            ),
            -3,
        ),
        (parse!("laurent_replay_control::eps^3"), -2),
        (Atom::Zero, 0),
    ] {
        let expected = super::super::expand_template(&expression, epsilon, maximum).unwrap();
        let (actual, _) = replay::relative(&expression, epsilon, maximum);
        assert_eq!(expected, replay::coefficients(&actual, maximum).unwrap());
    }
}

#[test]
fn fractional_native_series_is_explicitly_rejected() {
    let regulator = symbol!("fractional_capture::eps");
    let expression = parse!("fractional_capture::eps^(1/2)");
    assert!(matches!(
        super::super::expand_template(&expression, regulator, 1),
        Err(GenerationError::FractionalLaurent(_))
    ));
    let (native, _) = replay::relative(&expression, regulator, 1);
    assert!(matches!(
        replay::coefficients(&native, 1),
        Err(GenerationError::FractionalLaurent(_))
    ));
}

#[test]
fn capture_cancels_and_restores_normal_generation() {
    let temporary = tempfile::tempdir().unwrap();
    let input = ParametricIntegrand::new(
        vec![symbol!("capture_control::x"), symbol!("capture_control::y")],
        symbol!("capture_control::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::Zero, Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("capture_control::x+2*capture_control::y"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    {
        let capture = Guard::new(1, temporary.path().into());
        assert!(matches!(
            generate(&input, &GenerationOptions::default(), |_| {
                ControlFlow::Continue(())
            }),
            Err(GenerationError::Cancelled)
        ));
        assert!(capture.captured());
    }
    assert!(temporary.path().join("template.atom").exists());
    {
        let missing = Guard::new(usize::MAX, temporary.path().join("missing"));
        assert!(matches!(
            generate(&input, &GenerationOptions::default(), |_| {
                ControlFlow::Continue(())
            }),
            Err(GenerationError::Cancelled)
        ));
        assert!(!missing.captured());
    }
    let ordinary = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(ordinary.orders(), &[0]);
    assert_eq!(ordinary.sectors().len(), 2);
}

#[test]
#[ignore = "capture on-shell triple-box native template while explicitly skipping earlier Laurent expansions"]
fn capture_on_shell_triple_box_template() {
    let target = std::env::var("FASTSECDEC_LAURENT_INDEX")
        .unwrap_or_else(|_| "38".into())
        .parse::<usize>()
        .unwrap();
    let output = std::env::var_os("FASTSECDEC_LAURENT_CAPTURE")
        .map(PathBuf::from)
        .expect("explicit ignored output directory");
    let capture = Guard::new(target, output.clone());
    let input = on_shell_input();
    let mut chart_count = None;
    let mut representative_count = None;
    let result = generate(&input, &GenerationOptions::default(), |progress| {
        match progress {
            GenerationProgress::Factorization { total, .. } => chart_count = Some(*total),
            GenerationProgress::LaurentExpansion { total, .. } => {
                representative_count = Some(*total)
            }
            _ => {}
        }
        ControlFlow::Continue(())
    });
    assert!(matches!(result, Err(GenerationError::Cancelled)));
    assert!(capture.captured(), "target representative must exist");
    assert_eq!(chart_count, Some(2112));
    assert_eq!(representative_count, Some(1026));
    let fixture = serde_json::json!({
        "card": "examples/runs/triple_box.toml",
        "card_blake3": blake3::hash(include_bytes!("../../../../../../examples/runs/triple_box.toml")).to_hex().to_string(),
        "graph_blake3": blake3::hash(include_bytes!("../../../../../../examples/graphs/triple_box.dot")).to_hex().to_string(),
        "density_canonical_blake3": blake3::hash(input.density().to_canonical_string().as_bytes()).to_hex().to_string(),
        "geometry_charts": chart_count,
        "representatives": representative_count,
        "measure_multiplier": "1", "mass_squared": "0",
        "virtualities": [0,0,0,0], "s": -1, "t": -1,
    });
    std::fs::write(
        output.join("fixture.json"),
        serde_json::to_vec_pretty(&fixture).unwrap(),
    )
    .unwrap();
}

fn on_shell_input() -> ParametricIntegrand {
    // Native objects and exact values mirror examples/runs/triple_box.toml;
    // no parser, topology reduction, or special numerator algebra is introduced.
    let mut model = Model::from_json(include_str!(
        "../../../../../../examples/models/scalar.json"
    ))
    .unwrap();
    model
        .apply_parameter_card(
            &ParameterCard::from_json(include_str!(
                "../../../../../../examples/models/massless.json"
            ))
            .unwrap(),
        )
        .unwrap();
    let p = (0..3)
        .map(|i| feynkit_graph::symbols::external_momentum().call(i))
        .collect::<Vec<_>>();
    let mut kinematics = Kinematics::in_dimension(&parse!("feynkit_graph::D")).unwrap();
    for (i, row) in [[0, -1, 2], [-1, 0, -1], [2, -1, 0]].iter().enumerate() {
        for j in i..3 {
            kinematics = kinematics
                .with_scalar_product(&p[i], &p[j], Atom::num((row[j], 2)))
                .unwrap();
        }
    }
    assert_eq!(
        kinematics
            .scalar_product(&(-(&p[0] + &p[1] + &p[2])), &(-(&p[0] + &p[1] + &p[2])))
            .unwrap(),
        Atom::Zero
    );
    let graph = GraphIntegral::from_dot(
        Arc::new(model),
        include_str!("../../../../../../examples/graphs/triple_box.dot"),
        &kinematics,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::Zero)]))
    .unwrap();
    assert_eq!(graph.powers().len(), 10);
    ParametricIntegrand::from_graph(
        &graph,
        (0..10)
            .map(|i| symbol!(format!("fastsecdec::x{i}")))
            .collect(),
        symbol!("feynkit_graph::eps"),
        parse!("4-2*feynkit_graph::eps"),
    )
    .unwrap()
}
