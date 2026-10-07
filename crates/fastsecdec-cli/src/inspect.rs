//! Inspection reads retained native metadata; it never regenerates expressions,
//! reconstructs sector maps, or samples an evaluator.
mod overview;
mod presentation;
mod tables;

use crate::{CliResult, artifact::Artifact, terminal_policy::ColorPolicy};
use fastsecdec::kernel::{KernelSet, PortableMetadata};
use std::{io::IsTerminal, path::Path};

pub fn artifact(
    path: &Path,
    expressions: bool,
    sector: Option<usize>,
    plain: bool,
    json: bool,
) -> CliResult<()> {
    let (artifact, kernels) = Artifact::load(path)?;
    if let Some(id) = sector
        && id >= kernels.sectors().len()
    {
        return Err(format!(
            "unknown sector {id}; artifact contains {} numerical sectors (IDs start at 0)",
            kernels.sectors().len()
        )
        .into());
    }
    if json {
        let value = if let Some(id) = sector {
            serde_json::json!({
                "content_id":artifact.content_id,
                "generation":artifact.generation,
                "generation_timings":artifact.generation_timings,
                "loading_seconds":artifact.loading_seconds,
                "sectors":kernels.sectors().len(),
                "orders":kernels.orders(),"components":kernels.components(),
                "runtime_parameters":kernels.runtime_parameters().iter().map(|symbol|symbol.get_name()).collect::<Vec<_>>(),
                "parameters_bound":kernels.parameters_bound(),
                "selected_sector": {
                "id": id,
                "content_id": kernels.sector_content_id(id)?,
                "dimension": kernels.sectors()[id].dimension(),
                "source_chart_generation_modes": source_chart_modes(&artifact, &kernels, id),
                "evaluator_statistics": kernels.sectors()[id].statistics(),
                "charts": kernels.generation_metadata().map(|metadata| PortableMetadata::charts_for_sector(metadata,id)),
                }
            })
        } else {
            let mut value = document(&artifact, &kernels)?;
            value["largest_sectors"] = serde_json::to_value(ranked_sectors(&kernels))?;
            value
        };
        crate::report(&value, true)?;
    } else {
        let terminal = std::io::stdout().is_terminal();
        let width = if terminal {
            crossterm::terminal::size()
                .map(|(w, _)| usize::from(w))
                .unwrap_or(100)
        } else {
            100
        };
        let colors = ColorPolicy::for_stream(plain, terminal);
        print!(
            "{}",
            presentation::render(
                path,
                &artifact,
                &kernels,
                sector,
                expressions,
                width.clamp(1, 140),
                colors
            )?
        );
    }
    Ok(())
}

/// Join producer modes to the loaded kernel through stable source-chart IDs.
/// Generated-sector positions are not stable after exact-sector folding.
fn source_chart_modes(
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
) -> Option<std::collections::BTreeMap<usize, fastsecdec::generation::GenerationMode>> {
    let recorded = artifact.generation.as_ref()?.source_chart_modes.as_ref()?;
    Some(
        kernels
            .generation_metadata()?
            .charts()
            .iter()
            .filter(|chart| chart.kernel_sector() == Some(id))
            .filter_map(|chart| {
                recorded
                    .get(&chart.source_index())
                    .map(|mode| (chart.source_index(), *mode))
            })
            .collect(),
    )
}

fn ranked_sectors(kernels: &KernelSet) -> Vec<usize> {
    let mut ids = (0..kernels.sectors().len()).collect::<Vec<_>>();
    ids.sort_by_key(|&id| {
        (
            std::cmp::Reverse(kernels.sectors()[id].statistics().exact_program_bytes),
            id,
        )
    });
    ids.truncate(10);
    ids
}

fn summary(artifact: &Artifact, kernels: &KernelSet) -> serde_json::Value {
    serde_json::json!({
        "content_id":artifact.content_id,"provenance":artifact.provenance,
        "orders":kernels.orders(),"components":kernels.components(),"sectors":kernels.sectors().len(),
        "dimensions":kernels.sectors().iter().map(|k|k.dimension()).collect::<Vec<_>>(),
        "evaluator_statistics":kernels.sectors().iter().map(|k|k.statistics()).collect::<Vec<_>>(),
        "exact_coefficients":kernels.parameters_bound().then(|| kernels.exact_coefficients()),
        "runtime_parameters":kernels.runtime_parameters().iter().map(|symbol| symbol.get_name()).collect::<Vec<_>>(),
        "parameters_bound":kernels.parameters_bound(),
        "generation":artifact.generation,
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
            &dir.path().join("new.fsd"),
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
        let path = dir.path().join("legacy.fsd");
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
