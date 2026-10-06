//! Readable native DOT without changing the finalized diagram payload.

use feynkit_graph::FeynmanDiagram;
use linnet::parser::DotGraph;

use super::Result;

pub fn pretty(diagram: &FeynmanDiagram) -> Result<String> {
    let parsed: DotGraph = DotGraph::from_string(&diagram.to_dot()?)?;
    if !parsed.global_data.edge_statements.is_empty()
        || !parsed.global_data.node_statements.is_empty()
    {
        return Err("unexpected defaults in the native FeynKit DOT export".into());
    }
    // Linnet owns parsing, attribute escaping and topology serialization. Its
    // global formatter already emits one attribute per line. Enclose exactly
    // that known formatter output in graph[...] rather than splitting DOT text
    // on commas, which also occur inside tensors and serialized routing data.
    let attributes = format!("{:4}\n", parsed.global_data);
    let mut bytes = Vec::new();
    parsed.write_io(&mut bytes)?;
    let native = String::from_utf8(bytes)?;
    let (header, body) = native.split_once('\n').ok_or("missing native DOT header")?;
    let body = body
        .strip_prefix(&attributes)
        .ok_or("unexpected native DOT attribute layout")?;
    let pretty = format!("{header}\n  graph [\n{attributes}  ];\n{body}");
    let loaded = FeynmanDiagram::from_dot(diagram.model_arc(), &pretty)?;
    if loaded.to_json()? != diagram.to_json()? {
        return Err("readable DOT changed the finalized native diagram payload".into());
    }
    Ok(pretty)
}
