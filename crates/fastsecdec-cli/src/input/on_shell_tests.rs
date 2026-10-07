//! Exact kinematics must change native Newton support before sector discovery.
use super::*;
use fastsecdec::{
    generation::{CoefficientExpansionMethod, GenerationMode, GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, PrecisionPolicy},
};
use std::ops::ControlFlow;

fn triangle(directory: &Path, on_shell: bool) -> LoadedInput {
    fs::write(
        directory.join("graph.dot"),
        include_str!("../../../../examples/graphs/triangle.dot"),
    )
    .unwrap();
    fs::write(
        directory.join("model.json"),
        include_str!("../../../../examples/models/scalar.json"),
    )
    .unwrap();
    let first = if on_shell {
        "value = \"0\""
    } else {
        "symbol = \"on_shell::p1p1\""
    };
    let second = if on_shell {
        "value = \"0\""
    } else {
        "symbol = \"on_shell::p2p2\""
    };
    let point_virtualities = if on_shell {
        ""
    } else {
        "\"on_shell::p1p1\" = 0.0\n\"on_shell::p2p2\" = 0.0"
    };
    let card = format!(
        r#"
[input]
graph = "graph.dot"
model = "model.json"
[kinematics]
products = [
  {{left = 1, right = 1, {first}}},
  {{left = 2, right = 2, {second}}},
  {{left = 1, right = 2, symbol = "on_shell::p1p2"}},
]
[parameters]
"UFO::mt" = "0"
[integral]
dimension = "4-2*eps"
regulator = "eps"
[generation]
order = 0
# Runtime-only zeros must never alter the generated off-shell support.
[integration.parameters]
{point_virtualities}
"on_shell::p1p2" = -0.5
"#
    );
    fs::write(directory.join("run.toml"), card).unwrap();
    load(&directory.join("run.toml")).unwrap()
}

fn symanzik_f(input: &LoadedInput) -> &PolynomialFactor {
    let invariant = Atom::var(symbol("on_shell::p1p2").unwrap());
    let factors = input
        .integrand
        .terms()
        .iter()
        .flat_map(|term| term.factors())
        .filter(|factor| {
            factor.role() == FactorRole::Singularity
                && factor.polynomial().contains(invariant.as_view())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        factors.len(),
        1,
        "native scalar triangle has exactly one F factor"
    );
    factors[0]
}

#[test]
fn exact_on_shell_kinematics_remove_support_and_reveal_poles_before_both_generators() {
    let on_dir = tempfile::tempdir().unwrap();
    let off_dir = tempfile::tempdir().unwrap();
    let on = triangle(on_dir.path(), true);
    let off = triangle(off_dir.path(), false);
    let p11 = symbol("on_shell::p1p1").unwrap();
    let p22 = symbol("on_shell::p2p2").unwrap();
    let p12 = symbol("on_shell::p1p2").unwrap();
    assert_eq!(on.runtime_parameters, vec![p12]);
    assert_eq!(off.runtime_parameters, vec![p11, p22, p12]);
    let on_f = symanzik_f(&on);
    let off_f = symanzik_f(&off);
    let on_support = on_f.support(on.integrand.parameters()).unwrap();
    let off_support = off_f.support(off.integrand.parameters()).unwrap();
    assert_eq!(on_support.exponents().len(), 1);
    assert_eq!(off_support.exponents().len(), 3);
    assert!(
        on_support
            .exponents()
            .iter()
            .all(|row| off_support.exponents().contains(row))
    );
    let zeros = BTreeMap::from([(p11, Atom::Zero), (p22, Atom::Zero)]);
    assert_eq!(
        bind(off_f.polynomial(), &zeros).expand(),
        on_f.polynomial().expand()
    );
    for virtuality in [p11, p22] {
        assert!(off_f.polynomial().contains(Atom::var(virtuality).as_view()));
        assert!(
            !on.integrand
                .density()
                .contains(Atom::var(virtuality).as_view())
        );
    }
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let mut options = GenerationOptions {
            mode,
            ..Default::default()
        };
        options.coefficient_expansion.method = CoefficientExpansionMethod::NativeNamed;
        let on_generated =
            generate(&on.integrand, &options, |_| ControlFlow::Continue(())).unwrap();
        let off_generated =
            generate(&off.integrand, &options, |_| ControlFlow::Continue(())).unwrap();
        assert_eq!(
            on_generated.orders(),
            &[-2, -1, 0],
            "two massless external legs expose the triangle's double IR pole: {mode:?}"
        );
        assert_eq!(
            off_generated.orders(),
            &[0],
            "a generic off-shell massless triangle has no endpoint pole: {mode:?}"
        );
        let endpoint_subtractions = |generated: &fastsecdec::generation::GeneratedIntegral| {
            generated
                .metadata()
                .charts()
                .iter()
                .flat_map(|chart| chart.pre_subtraction().unwrap().terms())
                .flat_map(|term| term.powers())
                .map(|power| power.subtraction_count())
                .sum::<usize>()
        };
        assert!(endpoint_subtractions(&on_generated) > 0);
        assert_eq!(endpoint_subtractions(&off_generated), 0);
        let kernels = on_generated
            .compile_with_settings_parameters_and_progress(
                PrecisionPolicy::default(),
                &on.runtime_parameters,
                CompilationSettings {
                    backend: EvaluatorBackend::Eager,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        assert_eq!(kernels.runtime_parameters(), &[p12]);
    }
}

#[test]
fn native_gghh_cards_fix_the_incoming_gluon_virtualities_before_generation() {
    // Match ordinary CLI startup. The pinned native model fingerprint depends
    // on prior process-global symbol registration; a preceding unrelated
    // scalar-model generation can change its printed definition. A cold child
    // preserves the strict native fingerprint check without rewriting fixtures.
    const CHILD: &str = "FASTSECDEC_ON_SHELL_CARD_PROCESS";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "input::on_shell_tests::native_gghh_cards_fix_the_incoming_gluon_virtualities_before_generation",
                "--test-threads=1",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "strict native card child failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    // Import native graphs and parse their cards only. In particular, the
    // three-loop fixture is never contracted, parameterized or generated here.
    for (name, model_json, parameter_json, dot, run_toml, point_toml) in [
        (
            "D05",
            include_str!("../../../../examples/gghh_double_box/model.json"),
            include_str!("../../../../examples/gghh_double_box/parameters.json"),
            include_str!("../../../../examples/gghh_double_box/graph.dot"),
            include_str!("../../../../examples/gghh_double_box/run.toml"),
            include_str!("../../../../examples/gghh_double_box/point.toml"),
        ),
        (
            "D020",
            include_str!("../../../../examples/gghh_triple_box/model.json"),
            include_str!("../../../../examples/gghh_triple_box/parameters.json"),
            include_str!("../../../../examples/gghh_triple_box/graph.dot"),
            include_str!("../../../../examples/gghh_triple_box/run.toml"),
            include_str!("../../../../examples/gghh_triple_box/point.toml"),
        ),
        (
            "D068",
            include_str!("../../../../examples/gghh_triple_box_bis/model.json"),
            include_str!("../../../../examples/gghh_triple_box_bis/parameters.json"),
            include_str!("../../../../examples/gghh_triple_box_bis/graph.dot"),
            include_str!("../../../../examples/gghh_triple_box_bis/run.toml"),
            include_str!("../../../../examples/gghh_triple_box_bis/point.toml"),
        ),
    ] {
        let mut model = Model::from_json(model_json).unwrap();
        model
            .apply_parameter_card(&ParameterCard::from_json(parameter_json).unwrap())
            .unwrap();
        let diagram = feynkit_graph::FeynmanDiagram::from_dot(Arc::new(model), dot)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let run: RunCard = toml::from_str(run_toml).unwrap();
        let point: toml::Value = toml::from_str(point_toml).unwrap();
        let point = point["parameters"].as_table().unwrap();
        assert_eq!(diagram.loop_count(), if name == "D05" { 2 } else { 3 });
        if name != "D05" {
            assert_eq!(run.generation.mode, GenerationMode::NumericalDual, "{name}");
            assert_eq!(
                run.generation.contraction_mode,
                if name == "D020" {
                    fastsecdec::input::NumeratorContraction::Dots
                } else {
                    fastsecdec::input::NumeratorContraction::Minimal
                },
                "{name}"
            );
        }
        let gram_inputs = run
            .kinematics
            .products
            .iter()
            .filter_map(|product| product.symbol.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(gram_inputs.len(), 13, "{name}");
        for input in gram_inputs {
            assert!(point.contains_key(input), "{name}: missing runtime {input}");
        }
        assert_eq!(
            point.len(),
            19,
            "{name}: thirteen Gram and six model inputs"
        );
        let basis = diagram.loop_momentum_basis();
        for index in [0, 1] {
            let edge_id = basis.external_edges[index];
            assert!(!basis.dependent_externals.contains(&edge_id), "{name}");
            let (_, _, edge) = diagram.edges().find(|(id, _, _)| *id == edge_id).unwrap();
            let external = edge.external.as_ref().unwrap();
            assert_eq!(external.index, index, "{name}");
            assert_eq!(
                external.state,
                feynkit_graph::ExternalState::Incoming,
                "{name}"
            );
            assert_eq!(
                diagram
                    .model()
                    .particle_by_id(edge.particle)
                    .unwrap()
                    .pdg_code,
                21,
                "{name}"
            );
            let momentum = feynkit_graph::symbols::external_momentum().call(index);
            let products = run
                .kinematics
                .products
                .iter()
                .filter(|product| {
                    product.left.atom().unwrap() == momentum
                        && product.right.atom().unwrap() == momentum
                })
                .collect::<Vec<_>>();
            assert_eq!(
                products.len(),
                1,
                "{name}: exactly one declaration for P({index}) squared"
            );
            assert!(
                products[0].symbol.is_none(),
                "{name}: on-shell virtuality is not a runtime input"
            );
            assert!(
                expression(products[0].value.as_deref().unwrap())
                    .unwrap()
                    .is_zero(),
                "{name}: on-shell virtuality must be exact zero"
            );
            let removed = format!("p{index}p{index}");
            assert!(
                !point.contains_key(&removed)
                    && !point.contains_key(&format!("feynkit_graph::{removed}")),
                "{name}: fixed virtuality must not remain in the integration point"
            );
        }
    }
}
