//! Rebuild an archived ggHH fixture with the current native model and renderer.
//!
//! `cargo run -p fastsecdec --example refresh_gghh_fixture -- INPUT_DIR NEW_OUTPUT_DIR`
//! preserves the archived typed graph payload and explicit projection. It never
//! imports an annotated graph with a changed or missing fingerprint.

#[path = "gghh_double_box/dot.rs"]
mod native_dot;

use std::{path::PathBuf, sync::Arc};

use feynkit_graph::{DiagramEdge, DiagramVertex, FeynmanDiagram, NumeratorInterface};
use feynkit_model::{Model, ParameterCard};
use idenso::tensor::SymbolicTensor;
use linnet::parser::DotGraph;
use serde::de::DeserializeOwned;
use serde_json::Value;
use spenso::structure::partial::PartialStructure;
use symbolica::{
    atom::{Atom, AtomCore, UserData},
    parser::{ParseSettings, Token},
    state::{State, Workspace},
    with_default_namespace,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn decode<T: DeserializeOwned>(value: &Value) -> Result<T> {
    Ok(serde_json::from_value(value.clone())?)
}

fn expression(source: &str) -> Result<Atom> {
    // Match the native import's registered-symbol admission. Canonical strings
    // do not carry Spenso's user data; retain the owner's existing declarations.
    let mut symbols = State::symbol_iter()
        .filter(|(symbol, _)| !matches!(symbol.get_data(), UserData::None))
        .map(|(symbol, _)| (Atom::var(symbol).to_canonical_string().into(), symbol))
        .collect();
    let namespace = with_default_namespace!(source, "feynkit_graph");
    Ok(Workspace::get_local().with(|workspace| {
        Token::parse_with_atom_info(
            source,
            ParseSettings::default(),
            Some((&namespace, &mut symbols, workspace)),
        )?
        .to_atom(&namespace, &mut symbols, workspace)
    })?)
}

fn tensor(source: &str) -> Result<SymbolicTensor<PartialStructure>> {
    Ok(NumeratorInterface::admit(expression(source)?, None)?)
}

fn text(value: &Value) -> Result<&str> {
    value
        .as_str()
        .ok_or_else(|| "expected archived expression".into())
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().ok_or("expected INPUT_DIR NEW_OUTPUT_DIR")?);
    let output = PathBuf::from(args.next().ok_or("expected NEW_OUTPUT_DIR")?);
    if args.next().is_some() || output.exists() {
        return Err("supply exactly two arguments and a fresh output directory".into());
    }
    let model_source = std::fs::read_to_string(input.join("model.json"))?;
    let parameters_source = std::fs::read_to_string(input.join("parameters.json"))?;
    let raw_source = std::fs::read_to_string(input.join("raw-diagram.json"))?;
    let raw_dot_source = std::fs::read_to_string(input.join("raw-diagram.dot"))?;
    let projected_source = std::fs::read_to_string(input.join("graph.dot"))?;
    let raw: Value = serde_json::from_str(&raw_source)?;
    let projected: DotGraph = DotGraph::from_string(&projected_source)?;
    let raw_dot: DotGraph = DotGraph::from_string(&raw_dot_source)?;
    let old_model: [u8; 32] = decode(&raw["model"])?;
    let old_fingerprint = old_model
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect::<String>();
    for archived in [&raw_dot, &projected] {
        assert_eq!(
            archived.global_data.statements["model_fingerprint"],
            old_fingerprint
        );
    }
    // The native numerator setter moves the aggregate to the first vertex and
    // replaces other fragments with one. Compare all remaining transport here,
    // then check every projected local tensor and the native aggregate below.
    let mut raw_transport: DotGraph = DotGraph::from_string(&raw_dot_source)?;
    let mut projected_transport: DotGraph = DotGraph::from_string(&projected_source)?;
    for key in [
        "numerator",
        "numerator_prefactor",
        "projector",
        "overall_factor",
    ] {
        raw_transport.global_data.statements.remove(key);
        projected_transport.global_data.statements.remove(key);
    }
    for transport in [&mut raw_transport, &mut projected_transport] {
        for (_, _, vertex) in transport.iter_nodes_mut() {
            vertex.statements.remove("numerator");
        }
        let edges = transport
            .iter_edges()
            .map(|(_, id, _)| id)
            .collect::<Vec<_>>();
        for edge in edges {
            transport[edge].statements.remove("numerator");
        }
    }
    assert_eq!(raw_transport.debug_dot(), projected_transport.debug_dot());

    let mut model = Model::from_json(&model_source)?;
    model.apply_parameter_card(&ParameterCard::from_json(&parameters_source)?)?;
    let model = Arc::new(model);
    let attributes = &projected.global_data.statements;
    let mut builder = FeynmanDiagram::builder(Arc::clone(&model), text(&raw["name"])?)
        .symmetry_factor(decode(&raw["symmetry_factor"])?)
        .overall_factor(expression(&attributes["overall_factor"])?)
        .numerator(tensor(&attributes["numerator"])?)
        .numerator_prefactor(tensor(&attributes["numerator_prefactor"])?)
        .projector(tensor(&attributes["projector"])?)
        .half_edge_order(decode(&raw["half_edge_order"])?)
        .loop_momentum_basis(decode(&raw["loop_momentum_basis"])?)
        .cuts(decode(&raw["cuts"])?)
        .topology_threshold_candidates(decode(&raw["topology_threshold_candidates"])?);
    let vertices = raw["vertices"].as_array().ok_or("missing vertices")?;
    assert_eq!(vertices.len(), projected.iter_nodes().count());
    for (vertex, (_, _, archived)) in vertices.iter().zip(projected.iter_nodes()) {
        let interaction = decode::<Option<usize>>(&vertex["interaction"])?
            .map(|index| model.vertex_rule_id_at(index))
            .transpose()?;
        builder.add_vertex(DiagramVertex {
            name: text(&vertex["name"])?.into(),
            interaction,
            numerator: tensor(&archived.statements["numerator"])?,
        });
    }
    let mut orientations = Vec::new();
    let edges = raw["edges"].as_array().ok_or("missing edges")?;
    assert_eq!(edges.len(), projected.iter_edges().count());
    for (pair, (_, _, archived)) in edges.iter().zip(projected.iter_edges()) {
        let endpoints: feynkit_graph::EdgeEndpoints = decode(&pair[0])?;
        let stored = &pair[1];
        let source_slot = decode(&stored["source_slot"])?;
        let target_slot = decode(&stored["target_slot"])?;
        let mut edge = DiagramEdge::new(
            model.particle_id_at(decode(&stored["particle"])?)?,
            decode(&stored["directed"])?,
        )
        .with_slots(source_slot, target_slot);
        edge.external = decode(&stored["external"])?;
        edge.is_dummy = decode(&stored["is_dummy"])?;
        edge.propagator_power = stored
            .get("propagator_power")
            .map(decode)
            .transpose()?
            .unwrap_or(1);
        edge.numerator = tensor(&archived.data.statements["numerator"])?;
        builder.add_edge_with_slots(
            endpoints.source,
            endpoints.target,
            edge,
            source_slot,
            target_slot,
        )?;
        orientations.push(decode(&stored["orientation"])?);
    }
    let rebuilt = builder.edge_orientations(orientations).build()?;
    rebuilt.validate()?;
    for ((_, _, archived), (_, native)) in projected.iter_nodes().zip(rebuilt.vertices()) {
        assert_eq!(archived.statements["feynkit_name"], native.name);
        assert_eq!(tensor(&archived.statements["numerator"])?, native.numerator);
    }
    for ((_, archived_id, archived), (native_id, _, native)) in
        projected.iter_edges().zip(rebuilt.edges())
    {
        assert_eq!(archived_id.0, native_id.0);
        assert_eq!(
            tensor(&archived.data.statements["numerator"])?,
            native.numerator
        );
    }
    let mut original_payload = raw.clone();
    let mut rebuilt_payload: Value = serde_json::from_str(&rebuilt.to_json()?)?;
    for key in [
        "id",
        "model",
        "numerator",
        "num_interface",
        "numerator_prefactor",
        "numerator_prefactor_interface",
        "projector",
        "projector_interface",
        "overall_factor",
    ] {
        original_payload.as_object_mut().unwrap().remove(key);
        rebuilt_payload.as_object_mut().unwrap().remove(key);
    }
    for payload in [&mut original_payload, &mut rebuilt_payload] {
        for vertex in payload["vertices"].as_array_mut().unwrap() {
            vertex.as_object_mut().unwrap().remove("numerator");
            vertex.as_object_mut().unwrap().remove("num_interface");
        }
        for edge in payload["edges"].as_array_mut().unwrap() {
            edge[1].as_object_mut().unwrap().remove("numerator");
            edge[1].as_object_mut().unwrap().remove("num_interface");
            edge[1]
                .as_object_mut()
                .unwrap()
                .entry("propagator_power")
                .or_insert(Value::from(1));
        }
    }
    assert_eq!(original_payload, rebuilt_payload);
    let dot = native_dot::pretty(&rebuilt)?;
    let strict = FeynmanDiagram::from_dot(Arc::clone(&model), &dot)?;
    assert_eq!(strict.to_json()?, rebuilt.to_json()?);
    std::fs::create_dir(&output)?;
    std::fs::write(output.join("graph.dot"), &dot)?;
    std::fs::write(
        output.join("migration.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input": input, "old_model_fingerprint": old_fingerprint,
            "new_model_fingerprint": model.fingerprint().to_string(),
            "old_raw_json_blake3": blake3::hash(raw_source.as_bytes()).to_hex().to_string(),
            "old_projected_dot_blake3": blake3::hash(projected_source.as_bytes()).to_hex().to_string(),
            "new_projected_dot_blake3": blake3::hash(dot.as_bytes()).to_hex().to_string(),
            "native_diagram_id": rebuilt.id().to_string(),
            "native_payload_roundtrip": true, "archived_projection_transport_checked": true,
            "native_model_rebuilt": true, "native_import_guards_unchanged": true,
            "raw_archive_used_for_topology_only": true,
            "projected_numerator_reused_without_contraction": true,
        }))?,
    )?;
    Ok(())
}
