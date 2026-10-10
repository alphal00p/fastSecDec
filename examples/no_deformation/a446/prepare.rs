//! Materialize a native Standard Model for the small tracked A446 run card.
//! No graph or polynomial reconstruction lives in this preparation helper.

use std::{fs, io::Write, path::Path};

pub fn prepare(output: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if output.exists() {
        return Err("A446 preparation requires a fresh output directory".into());
    }
    let model = fastsecdec::Model::standard_model().to_json()?;
    if model.len() > 32 * 1024 * 1024 {
        return Err("native model exceeds the exporter file bound".into());
    }
    let files = [
        ("standard-model.json", model.as_str()),
        ("graph.dot", include_str!("graph.dot")),
        ("run.toml", include_str!("run.toml")),
        ("point.toml", include_str!("point.toml")),
        ("expected_uf.json", include_str!("expected_uf.json")),
    ];
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output)?;
    let mut records = Vec::new();
    for (name, contents) in files {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        records.push(serde_json::json!({
            "file":name, "bytes":contents.len(),
            "blake3":blake3::hash(contents.as_bytes()).to_hex().to_string()
        }));
    }
    Ok(serde_json::json!({
        "schema":"fastsecdec.a446-native-fixture", "version":1,
        "directory":output, "files":records,
        "model_source":"Model::standard_model().to_json()",
        "coverage_claimed":false
    }))
}
