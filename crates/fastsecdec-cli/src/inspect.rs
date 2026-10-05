//! Artifact inspection delegates semantic transport and presentation to the
//! native metadata owner. No polynomial support or coordinate map is rebuilt.
use crate::{CliResult, artifact::Artifact};
use fastsecdec::kernel::{KernelSet, PortableMetadata};
use std::path::Path;

pub fn artifact(path: &Path, expressions: bool, json: bool) -> CliResult<()> {
    let (artifact, kernels) = Artifact::load(path)?;
    if json {
        crate::report(&document(&artifact, &kernels)?, true)?;
    } else {
        crate::report(&summary(&artifact, &kernels), false)?;
        if let Some(metadata) = kernels.generation_metadata() {
            println!("{}", metadata.display(expressions));
        } else {
            println!("Retained chart/domain metadata unavailable (legacy artifact).");
        }
    }
    Ok(())
}

fn summary(artifact: &Artifact, kernels: &KernelSet) -> serde_json::Value {
    serde_json::json!({
        "content_id":artifact.content_id,"provenance":artifact.provenance,
        "orders":kernels.orders(),"sectors":kernels.sectors().len(),
        "dimensions":kernels.sectors().iter().map(|k|k.dimension()).collect::<Vec<_>>(),
        "exact_coefficients":kernels.exact_coefficients(),
        "generation_timings":artifact.generation_timings,"loading_seconds":artifact.loading_seconds,
        "retained_metadata_available":kernels.generation_metadata().is_some()
    })
}

fn document(artifact: &Artifact, kernels: &KernelSet) -> CliResult<serde_json::Value> {
    let mut value = summary(artifact, kernels);
    value["generation_metadata"] = serde_json::to_value(
        kernels
            .generation_metadata()
            .map(PortableMetadata::from_native),
    )?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_legacy_artifact_inspection_keeps_metadata_explicitly_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let card = dir.path().join("input.toml");
        std::fs::write(&card, "[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['1']").unwrap();
        let (artifact, _) = crate::generate::generate(
            &card,
            &dir.path().join("new.json"),
            &mut crate::display::Dashboard::new(false, false).unwrap(),
            None,
        )
        .unwrap();
        // The historical expression fixture is independent of the current
        // native-IR serializer and deliberately has no retained metadata.
        let restored = KernelSet::from_bytes(include_bytes!(
            "../../fastsecdec/tests/fixtures/kernel-v1-triangle.json"
        ))
        .unwrap();
        let path = dir.path().join("legacy.json");
        Artifact::new(&restored, artifact.provenance)
            .unwrap()
            .save(&path)
            .unwrap();
        let (artifact, kernels) = Artifact::load(&path).unwrap();
        let view = document(&artifact, &kernels).unwrap();
        assert_eq!(view["retained_metadata_available"], false);
        assert!(view["generation_metadata"].is_null());
        assert_eq!(view["sectors"], 2);
    }
}
