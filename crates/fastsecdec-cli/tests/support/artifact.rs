//! Test-side resolution of the public immutable data manifest.
pub fn data_path(base: &std::path::Path) -> std::path::PathBuf {
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(base.with_extension("fsd.json")).unwrap()).unwrap();
    base.parent().unwrap().join(
        manifest["programs"]["data_file"]
            .as_str()
            .or_else(|| manifest["indexed"]["data_file"].as_str())
            .unwrap(),
    )
}
