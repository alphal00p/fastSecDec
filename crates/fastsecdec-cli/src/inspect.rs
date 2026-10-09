//! Default inspection reads only human metadata. Explicit deep inspection
//! restores retained native metadata and evaluators, without sampling.
mod lightweight;
mod overview;
mod presentation;
mod selected;
mod tables;

use crate::{CliResult, artifact::Artifact, terminal_policy::ColorPolicy};
use fastsecdec::kernel::{KernelLoadOptions, KernelSet, PortableMetadata};
use std::{io::IsTerminal, path::Path};

#[allow(clippy::too_many_arguments)]
pub fn artifact(
    path: &Path,
    recipe: Option<fastsecdec::kernel::indexed::ProgramRecipe>,
    options: KernelLoadOptions,
    deep: bool,
    expressions: bool,
    sector: Option<usize>,
    plain: bool,
    json: bool,
) -> CliResult<()> {
    if !deep {
        let mut artifact = Artifact::load_metadata_with_options(path, options)?;
        if let Some(recipe) = recipe {
            artifact.select_recipe(recipe)?;
        }
        let summary = artifact.kernel_summary()?;
        lightweight::validate_sector(&summary, sector)?;
        if json {
            crate::report(&lightweight::document(&artifact, &summary, sector)?, true)?;
        } else {
            let (width, colors) = presentation_options(plain);
            print!(
                "{}",
                lightweight::render(path, &artifact, &summary, sector, width, colors)?
            );
        }
        return Ok(());
    }
    if let Some(id) = sector {
        let mut artifact = Artifact::load_metadata_with_options(path, options)?;
        if let Some(recipe) = recipe {
            artifact.select_recipe(recipe)?;
        }
        if artifact.catalogue().is_some() {
            return selected::inspect(path, artifact, options, id, expressions, plain, json);
        }
    }
    let (artifact, kernels) = if recipe.is_some() {
        Artifact::load_recipe_observed(
            path,
            options,
            recipe,
            |_| Ok(()),
            |_| std::ops::ControlFlow::Continue(()),
        )?
    } else {
        Artifact::load_with_options(path, options)?
    };
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
                "kernel_content_id":artifact.kernel_content_id,
                "selected_catalogue_content_id":artifact.catalogue().map(|c|c.content_id),
                "selected_recipe":artifact.selected_recipe(),
                "available_recipes":artifact.programs.as_ref().map(|p|p.catalogue.recipes.iter().map(|r|r.recipe).collect::<Vec<_>>()),
                "inspection_mode":"deep", "binary_loaded":true,
                "binary_validated":artifact.validation.binary,
                "metadata_identity_validated":artifact.validation.metadata_identity,
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
        let (width, colors) = presentation_options(plain);
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

fn presentation_options(plain: bool) -> (usize, ColorPolicy) {
    let terminal = std::io::stdout().is_terminal();
    let width = if terminal {
        crossterm::terminal::size()
            .map(|(w, _)| usize::from(w))
            .unwrap_or(100)
    } else {
        100
    };
    (
        width.clamp(1, 140),
        ColorPolicy::for_stream(plain, terminal),
    )
}

/// Join producer modes to the loaded kernel through stable source-chart IDs.
/// Generated-sector positions are not stable after exact-sector folding.
fn source_chart_modes(
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
) -> Option<std::collections::BTreeMap<usize, fastsecdec::generation::GenerationMode>> {
    source_chart_modes_mapped(artifact, kernels, id, None)
}

fn source_chart_modes_mapped(
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
    source_indices: Option<&[usize]>,
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
                    .get(&source_indices.map_or(chart.source_index(), |indices| {
                        indices[chart.source_index()]
                    }))
                    .map(|mode| {
                        (
                            source_indices.map_or(chart.source_index(), |indices| {
                                indices[chart.source_index()]
                            }),
                            *mode,
                        )
                    })
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
        "kernel_content_id":artifact.kernel_content_id,
        "selected_catalogue_content_id":artifact.catalogue().map(|c|c.content_id),
        "selected_recipe":artifact.selected_recipe(),
        "available_recipes":artifact.programs.as_ref().map(|p|p.catalogue.recipes.iter().map(|r|r.recipe).collect::<Vec<_>>()),
        "inspection_mode":"deep", "binary_loaded":true,
        "binary_validated":artifact.validation.binary,
        "metadata_identity_validated":artifact.validation.metadata_identity,
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
mod tests;
