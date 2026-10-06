//! Import the requested labeled diagram through its native model and DOT owner.

use std::sync::Arc;

use feynkit_graph::FeynmanDiagram;
use feynkit_model::Model;
use linnet::parser::DotGraph;
use serde::Serialize;

use super::{Result, export};

pub const DOT: &str = include_str!("../../../../examples/gghh_double_box/source-diagram.dot");

#[derive(Serialize)]
pub struct Provenance {
    path: &'static str,
    blake3: String,
    original_model_fingerprint: String,
    physical_model_fingerprint: String,
    original_diagram_id: String,
    model_adaptation: &'static str,
    native_payload_preserved: bool,
}

pub fn physical_diagram(requested_model: &Model) -> Result<(FeynmanDiagram, Provenance)> {
    // The supplied D05 was exported with HEPKit's unmodified embedded SM.
    // Strict native import first verifies that exact model fingerprint, all
    // tensor fragments and the stored routing before applying our numeric card.
    let original_model = Arc::new(Model::standard_model());
    let original = FeynmanDiagram::from_dot(Arc::clone(&original_model), DOT)?;
    let mut model = original_model.as_ref().clone();
    model.apply_parameter_card(&export::parameter_card(&model)?)?;
    if requested_model.fingerprint() != original_model.fingerprint()
        && requested_model.fingerprint() != model.fingerprint()
    {
        return Err(
            "D05 requires the native Standard Model or this example's physical card".into(),
        );
    }

    // FeynmanDiagram has no public model-rebinding operation. Its native DOT
    // owner changes only the model identity after the native parameter update;
    // no topology, indices, expressions or momentum signatures are rewritten.
    let mut dot: DotGraph = DotGraph::from_string(&original.to_dot()?)?;
    dot.global_data
        .statements
        .insert("model_fingerprint".into(), model.fingerprint().to_string());
    let physical = FeynmanDiagram::from_dot(Arc::new(model), &dot.debug_dot())?;

    // Compare the complete native serialization, including local tensor
    // fragments, slots, half-edge ordering and routing. Only model-derived
    // identities are expected to change under the physical parameter card.
    let payload = |diagram: &FeynmanDiagram| -> Result<serde_json::Value> {
        let mut value: serde_json::Value = serde_json::from_str(&diagram.to_json()?)?;
        let fields = value
            .as_object_mut()
            .ok_or("invalid native diagram payload")?;
        fields.remove("model");
        fields.remove("id");
        Ok(value)
    };
    if payload(&original)? != payload(&physical)? {
        return Err("applying the physical card changed the supplied D05 diagram payload".into());
    }
    let provenance = Provenance {
        path: "source-diagram.dot",
        blake3: blake3::hash(DOT.as_bytes()).to_hex().to_string(),
        original_model_fingerprint: original_model.fingerprint().to_string(),
        physical_model_fingerprint: physical.model().fingerprint().to_string(),
        original_diagram_id: original.id().to_string(),
        model_adaptation: "native ParameterCard applied to the original SM; only model fingerprint and derived diagram ID change",
        native_payload_preserved: true,
    };
    Ok((physical, provenance))
}
