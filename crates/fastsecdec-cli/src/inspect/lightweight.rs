//! Inspection of producer-recorded JSON only. No native program restoration.
use super::tables::{bytes, section};
use crate::{
    CliResult,
    artifact::{Artifact, KernelSummary},
    generation_report::{facts_table, heading, terminal_text},
    terminal_policy::ColorPolicy,
};
use std::path::Path;
use tabled::settings::Color;

pub(super) fn validate_sector(summary: &KernelSummary, sector: Option<usize>) -> CliResult<()> {
    if let Some(id) = sector
        && id >= summary.sectors
    {
        return Err(format!(
            "unknown sector {id}; artifact contains {} numerical sectors (IDs start at 0)",
            summary.sectors
        )
        .into());
    }
    Ok(())
}
fn ranked(summary: &KernelSummary) -> Option<Vec<usize>> {
    let stats = summary.evaluator_statistics.as_ref()?;
    let mut ids = (0..summary.sectors).collect::<Vec<_>>();
    ids.sort_by_key(|&id| (std::cmp::Reverse(stats[id].exact_program_bytes), id));
    ids.truncate(10);
    Some(ids)
}
fn charts(
    artifact: &Artifact,
    id: usize,
) -> Option<Vec<&crate::artifact::inspection::ChartPreview>> {
    artifact.inspection_index().map(|index| {
        index
            .charts
            .iter()
            .filter(|c| c.kernel_sector == Some(id))
            .collect()
    })
}
fn chart_modes(
    artifact: &Artifact,
    id: usize,
) -> Option<std::collections::BTreeMap<usize, fastsecdec::generation::GenerationMode>> {
    let recorded = artifact.generation.as_ref()?.source_chart_modes.as_ref()?;
    Some(
        charts(artifact, id)?
            .into_iter()
            .filter_map(|c| {
                recorded
                    .get(&c.source_index)
                    .map(|mode| (c.source_index, *mode))
            })
            .collect(),
    )
}
pub(super) fn document(
    artifact: &Artifact,
    summary: &KernelSummary,
    sector: Option<usize>,
) -> CliResult<serde_json::Value> {
    validate_sector(summary, sector)?;
    let mut value = serde_json::json!({
        "content_id":artifact.content_id, "kernel_content_id":artifact.kernel_content_id,
        "provenance":artifact.provenance, "sectors":summary.sectors,
        "orders":summary.orders, "components":summary.components,
        "runtime_parameters":summary.runtime_parameters,
        "parameters_bound":summary.runtime_parameters.as_ref().map(|p|p.is_empty()),
        "generation":artifact.generation, "generation_timings":artifact.generation_timings,
        "loading_seconds":artifact.loading_seconds,
        "inspection_mode":"metadata_only", "binary_validated":false,
        "metadata_identity_validated":true,
        "dependencies_compatible":artifact.dependencies_compatible(),
        "inspection_note":"Binary data was not read. Optional producer-recorded inspection/generation observations are not checked against the binary.",
        "retained_metadata_available":artifact.inspection_index().map(|i|i.retained_metadata_available),
        "exact_coefficients":null,
    });
    if let Some(id) = sector {
        value["selected_sector"] = serde_json::json!({
            "id":id, "content_id":null,
            "dimension":summary.dimensions.as_ref().map(|v|v[id]),
            "evaluator_statistics":summary.evaluator_statistics.as_ref().map(|v|&v[id]),
            "source_chart_generation_modes":chart_modes(artifact,id),
            "chart_previews":charts(artifact,id),
            "omitted_index_charts":artifact.inspection_index().map(|i|i.omitted_charts),
            "details_note":"Per-sector content identity and complete chart metadata require --deep.",
        });
    } else {
        value["dimensions"] = serde_json::to_value(&summary.dimensions)?;
        value["evaluator_statistics"] = serde_json::to_value(&summary.evaluator_statistics)?;
        value["largest_sectors"] = serde_json::to_value(ranked(summary))?;
        value["inspection_index"] = serde_json::to_value(artifact.inspection_index())?;
    }
    Ok(value)
}
pub(super) fn render(
    path: &Path,
    artifact: &Artifact,
    summary: &KernelSummary,
    selected: Option<usize>,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    validate_sector(summary, selected)?;
    let mut out = super::overview::render(path, artifact, summary, None, width, colors)?;
    out.push('\n');
    let mut monomials_shown = false;
    if let Some(id) = selected {
        out.push_str(&heading(
            &format!("Sector {id}"),
            width,
            colors,
            Color::FG_GREEN,
        ));
        let mut facts = vec![
            [
                "Coordinates".into(),
                summary
                    .dimensions
                    .as_ref()
                    .map(|v| v[id].to_string())
                    .unwrap_or_else(|| "Not recorded".into()),
            ],
            ["Content ID".into(), "Not indexed; --deep".into()],
        ];
        if let Some(stats) = summary.evaluator_statistics.as_ref().map(|v| &v[id]) {
            facts.extend([
                ["Arithmetic".into(), terminal_text(&stats.arithmetic)],
                ["Backend".into(), terminal_text(&stats.backend)],
                [
                    "Evaluator data".into(),
                    format!(
                        "{} ({} bytes)",
                        bytes(stats.exact_program_bytes),
                        stats.exact_program_bytes
                    ),
                ],
                [
                    "Inputs / outputs".into(),
                    format!("{} / {}", stats.inputs, stats.outputs),
                ],
                [
                    "Add / multiply".into(),
                    format!(
                        "{} / {}",
                        stats.operations.additions, stats.operations.multiplications
                    ),
                ],
                [
                    "Invert / functions".into(),
                    format!(
                        "{} / {}",
                        stats.operations.inversions, stats.operations.function_calls
                    ),
                ],
            ]);
        }
        if let Some(modes) = chart_modes(artifact, id) {
            facts.push([
                "Chart modes".into(),
                modes
                    .into_iter()
                    .map(|(c, m)| format!("{c}: {}", m.name()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ]);
        }
        out.push_str(&facts_table(facts, width, colors));
        out.push('\n');
        if let Some(charts) = charts(artifact, id) {
            for chart in charts {
                out.push('\n');
                let Some(preview) = &chart.preview else {
                    out.push_str(&heading(
                        &format!(
                            "Chart {} → representative {}; full map: --deep",
                            chart.source_index, chart.representative
                        ),
                        width,
                        colors,
                        Color::FG_CYAN,
                    ));
                    continue;
                };
                monomials_shown |= !preview.leading_monomials.is_empty();
                out.push_str(&section(
                    &format!("Chart {} · pre-subtraction preview", chart.source_index),
                    vec!["Term", "Monomial factor"],
                    preview
                        .leading_monomials
                        .iter()
                        .enumerate()
                        .map(|(i, v)| vec![i.to_string(), preview_text(v)])
                        .collect(),
                    width,
                    colors,
                ));
                if preview.omitted_terms > 0 {
                    out.push_str(&heading(
                        &format!(
                            "{} further terms omitted; --deep for all retained terms.",
                            preview.omitted_terms
                        ),
                        width,
                        colors,
                        Color::FG_BRIGHT_BLACK,
                    ));
                }
                let mut rows = preview
                    .coordinate_images
                    .iter()
                    .map(|v| v.iter().map(preview_text).collect::<Vec<_>>())
                    .collect::<Vec<_>>();
                rows.push(vec![
                    "Positive measure".into(),
                    preview_text(&preview.positive_measure),
                ]);
                out.push_str(&section(
                    "Coordinate map preview",
                    vec!["Input", "Sector expression"],
                    rows,
                    width,
                    colors,
                ));
                if let Some(fixed) = preview.projective_fixed_parameter {
                    out.push_str(&heading(
                        &format!("Projective gauge: input parameter {fixed} is fixed to 1."),
                        width,
                        colors,
                        Color::FG_BRIGHT_BLACK,
                    ));
                }
                if preview.omitted_coordinate_images > 0 {
                    out.push_str(&heading(
                        &format!(
                            "{} coordinate images omitted; --deep for full maps.",
                            preview.omitted_coordinate_images
                        ),
                        width,
                        colors,
                        Color::FG_BRIGHT_BLACK,
                    ));
                }
            }
        }
    } else if let Some(ids) = ranked(summary) {
        let rows = ids
            .into_iter()
            .map(|id| {
                let preview = charts(artifact, id)
                    .and_then(|charts| charts.into_iter().find_map(|c| c.preview.as_ref()));
                monomials_shown |= preview.is_some_and(|p| !p.leading_monomials.is_empty());
                vec![
                    id.to_string(),
                    bytes(summary.evaluator_statistics.as_ref().unwrap()[id].exact_program_bytes),
                    summary
                        .dimensions
                        .as_ref()
                        .map(|v| v[id].to_string())
                        .unwrap_or_else(|| "?".into()),
                    preview
                        .and_then(|p| p.terms)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "?".into()),
                    preview
                        .map(|p| {
                            p.leading_monomials
                                .iter()
                                .map(preview_text)
                                .chain(
                                    (p.omitted_terms > 0)
                                        .then(|| format!("+ {} terms; --deep", p.omitted_terms)),
                                )
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "Not indexed; --deep".into()),
                ]
            })
            .collect();
        out.push_str(&section(
            "Largest sector evaluators · top 10",
            vec![
                "Sector",
                "Evaluator",
                "Vars",
                "Terms",
                "Pre-subtraction monomial",
            ],
            rows,
            width,
            colors,
        ));
        out.push_str(&heading(
            "Sorted by saved serialized evaluator bytes; IDs are zero-based kernel indices.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    } else {
        out.push_str(&heading(
            "Evaluator statistics were not recorded; --deep restores native details.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    }
    if let Some(index) = artifact.inspection_index() {
        if index.omitted_charts > 0 {
            out.push_str(&heading(
                &format!(
                    "{} charts omitted by the bounded preview index.",
                    index.omitted_charts
                ),
                width,
                colors,
                Color::FG_BRIGHT_BLACK,
            ));
        }
    } else {
        out.push_str(&heading("Chart previews were not indexed by this producer; use --deep for retained monomials and maps.",width,colors,Color::FG_BRIGHT_BLACK));
    }
    if monomials_shown {
        out.push_str(&heading("Monomial factors and coordinate maps describe the density before subtraction. The regular body may vanish; these factors do not establish surviving poles.",width,colors,Color::FG_BRIGHT_BLACK));
    }
    out.push_str(&heading(
        "--deep loads complete chart data; --expressions also loads retained expressions.",
        width,
        colors,
        Color::FG_BRIGHT_BLACK,
    ));
    Ok(out)
}
fn preview_text(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(terminal_text)
        .unwrap_or_else(|| "Not indexed; --deep".into())
}
