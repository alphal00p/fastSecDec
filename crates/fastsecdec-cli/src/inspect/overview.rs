use super::tables::{bytes, section};
use crate::{
    CliResult,
    artifact::{Artifact, KernelSummary},
    generation_report::{
        duration, evaluator_rows, expansion_method, facts_table, facts_table_with_labels,
        formula_preparation_rows, generation_method_rows, heading, short_id, terminal_text,
        timing_rows,
    },
    math_display,
    terminal_policy::ColorPolicy,
};
use fastsecdec::{AtomCore, kernel::KernelSet, status::CoefficientComponent};
use std::{collections::BTreeSet, path::Path};
use tabled::settings::Color;

pub(super) fn render(
    path: &Path,
    artifact: &Artifact,
    kernels: &KernelSummary,
    restored: Option<&KernelSet>,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    let math =
        |value: &fastsecdec::Atom| math_display::atom(value, colors, width.saturating_sub(24));
    let sizes = kernels.evaluator_statistics.as_ref().map(|stats| {
        stats
            .iter()
            .map(|s| s.exact_program_bytes)
            .collect::<Vec<_>>()
    });
    let total = sizes.as_ref().map(|sizes| sizes.iter().sum::<usize>());
    let dimensions = kernels
        .dimensions
        .as_ref()
        .map(|values| values.iter().copied().collect::<BTreeSet<_>>());
    let dimensions = match dimensions.as_ref().map(|d| (d.first(), d.last())) {
        Some((Some(first), Some(last))) if first != last => format!("{first}–{last}"),
        Some((Some(first), _)) => first.to_string(),
        Some(_) => "None".into(),
        None => "Not recorded".into(),
    };
    let (json_path, _) = crate::artifact::paths(path)?;
    let data_path = artifact.data_path(path)?;
    let file_size = |path| {
        std::fs::metadata(path)
            .ok()
            .and_then(|m| usize::try_from(m.len()).ok())
            .map(bytes)
            .unwrap_or_else(|| "unavailable".into())
    };
    let json_size = file_size(json_path);
    let data_size = file_size(data_path);
    let charts = if let Some(metadata) = restored.and_then(|k| k.threshold_metadata()) {
        format!(
            "{} cells · {} endpoint charts",
            metadata.lineage().cells.len(),
            metadata.lineage().endpoint_charts.len()
        )
    } else if artifact
        .catalogue()
        .is_some_and(|c| c.threshold().is_some())
    {
        "Threshold lineage; see native record details".into()
    } else if let Some(native) = restored {
        native
            .generation_metadata()
            .map(|m| {
                format!(
                    "{} charts · {} representatives",
                    m.charts().len(),
                    m.charts()
                        .iter()
                        .map(|c| c.representative())
                        .collect::<BTreeSet<_>>()
                        .len()
                )
            })
            .unwrap_or_else(|| "Not retained".into())
    } else {
        artifact
            .inspection_index()
            .map(|index| {
                if index.retained_metadata_available {
                    format!(
                        "{} charts · {} representatives",
                        index.total_charts, index.total_representatives
                    )
                } else {
                    "Not retained".into()
                }
            })
            .unwrap_or_else(|| "Not indexed; --deep".into())
    };
    let mut facts = vec![
        [
            "Artifact".into(),
            terminal_text(
                &crate::artifact::relative_path(path, Path::new("."))?
                    .display()
                    .to_string(),
            ),
        ],
        ["Name".into(), terminal_text(&artifact.provenance.name)],
        ["ID (short)".into(), short_id(&artifact.content_id)],
        [
            "Saved files".into(),
            format!(".json {} · .dat {}", json_size, data_size),
        ],
        ["Sectors".into(), kernels.sectors.to_string()],
        ["Charts".into(), charts],
        ["Coordinates".into(), dimensions],
        [
            "Input domain".into(),
            match artifact.provenance.domain.as_str() {
                "ProjectiveSimplex" => "Projective simplex",
                "UnitCube" => "Unit cube",
                "PositiveOrthant" => "Positive orthant",
                other => other,
            }
            .into(),
        ],
        [
            "Spacetime D".into(),
            math(&crate::input::expression(&artifact.provenance.dimension)?),
        ],
        [
            "Evaluator sum".into(),
            total
                .map(|n| format!("{} ({n} bytes)", bytes(n)))
                .unwrap_or_else(|| "Not recorded".into()),
        ],
        [
            "Min/mean/max".into(),
            match sizes.as_ref() {
                Some(sizes) if !sizes.is_empty() => format!(
                    "{} / {} / {}",
                    bytes(*sizes.iter().min().unwrap()),
                    bytes(total.unwrap() / sizes.len()),
                    bytes(*sizes.iter().max().unwrap())
                ),
                Some(_) => "No numerical evaluators".into(),
                None => "Not recorded".into(),
            },
        ],
        [
            "Binding".into(),
            match kernels.runtime_parameters.as_ref() {
                Some(inputs) if inputs.is_empty() => "No runtime inputs",
                Some(_) => "Awaiting integration values",
                None => "Not recorded",
            }
            .into(),
        ],
        [
            "Inspection".into(),
            if artifact.validation.binary {
                "Deep · native binary validated"
            } else if restored.is_some() {
                "Deep · binary loaded; validation not requested"
            } else {
                "Metadata only · binary not read or validated"
            }
            .into(),
        ],
        [
            "Metadata identity".into(),
            if artifact.validation.metadata_identity {
                "Validated"
            } else {
                "Not checked; --validate-artifact"
            }
            .into(),
        ],
        [
            "Dependencies".into(),
            if artifact.dependencies_compatible() {
                "Match this build"
            } else {
                "Different from this build; deep loading unavailable"
            }
            .into(),
        ],
        ["Loaded in".into(), duration(artifact.loading_seconds)],
    ];
    if let Some(selection) = &kernels.source_selection {
        facts.push([
            "Generation extent".into(),
            format!(
                "Partial original integral · source charts {:?} of {}",
                selection.source_sectors(),
                selection.original_source_count()
            ),
        ]);
    }
    if let Some(programs) = &artifact.programs {
        facts.push([
            "Archive ID (short)".into(),
            short_id(&artifact.kernel_content_id),
        ]);
        facts.push([
            "Recipe ID (short)".into(),
            short_id(
                artifact
                    .catalogue()
                    .ok_or("missing selected recipe catalogue")?
                    .content_id,
            ),
        ]);
        facts.push([
            "Selected recipe".into(),
            artifact
                .selected_recipe()
                .ok_or("missing selected recipe")?
                .name()
                .into(),
        ]);
        facts.push([
            "Available recipes".into(),
            programs
                .catalogue
                .recipes
                .iter()
                .map(|r| r.recipe.name())
                .collect::<Vec<_>>()
                .join(", "),
        ]);
    }
    let mut out = heading("Artifact inspection", width, colors, Color::FG_GREEN);
    out.push_str(&facts_table(facts, width, colors));
    out.push_str("\n\n");
    out.push_str(&heading("Generation", width, colors, Color::FG_CYAN));
    let backend = kernels.evaluator_statistics.as_ref().map(|stats| {
        stats
            .iter()
            .map(|s| match s.backend.as_str() {
                "symjit_o2" => "SymJIT · O2",
                "symbolica_interpreter" => "Symbolica interpreter",
                other => other,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    });
    let mut generation_facts = vec![
        [
            "Workers".into(),
            artifact
                .generation
                .as_ref()
                .map(|record| record.workers.to_string())
                .unwrap_or_else(|| "Not recorded".into()),
        ],
        [
            "Requested expansion".into(),
            artifact
                .generation
                .as_ref()
                .map(|record| expansion_method(record.requested_coefficient_expansion))
                .unwrap_or("Not recorded")
                .into(),
        ],
        [
            "Backend".into(),
            backend.filter(|b| !b.is_empty()).unwrap_or_else(|| {
                if kernels.sectors == 0 {
                    "Exact coefficients only".into()
                } else {
                    "Not recorded".into()
                }
            }),
        ],
        [
            "Runtime inputs".into(),
            kernels
                .runtime_parameters
                .as_ref()
                .map(|v| v.len().to_string())
                .unwrap_or_else(|| "Not recorded".into()),
        ],
    ];
    generation_facts.extend(generation_method_rows(
        artifact.generation.as_ref().and_then(|record| record.mode),
        artifact
            .generation
            .as_ref()
            .and_then(|record| record.subtraction),
    ));
    generation_facts.extend(formula_preparation_rows(
        artifact
            .generation
            .as_ref()
            .and_then(|record| record.formula_preparation),
    ));
    if let Some(modes) = artifact
        .generation
        .as_ref()
        .and_then(|record| record.source_chart_modes.as_ref())
    {
        let mut counts = std::collections::BTreeMap::<_, usize>::new();
        for mode in modes.values() {
            *counts.entry(mode.name()).or_default() += 1;
        }
        generation_facts.push([
            "Actual chart modes".into(),
            if counts.is_empty() {
                "No source charts".into()
            } else {
                counts
                    .into_iter()
                    .map(|(mode, count)| format!("{mode}: {count}"))
                    .collect::<Vec<_>>()
                    .join(" · ")
            },
        ]);
    }
    generation_facts.extend(evaluator_rows(
        artifact
            .generation
            .as_ref()
            .and_then(|record| record.evaluator.as_ref()),
    ));
    out.push_str(&facts_table_with_labels(
        generation_facts,
        width,
        20,
        colors,
    ));
    out.push('\n');
    if let Some(timings) = &artifact.generation_timings {
        out.push_str(&section(
            "Wall time",
            vec!["Phase", "Elapsed"],
            timing_rows(timings).into_iter().map(Vec::from).collect(),
            width,
            colors,
        ));
        out.push_str(&heading(
            "Total excludes writing the artifact files.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    } else {
        out.push_str(&heading(
            "Phase timings were not recorded in this artifact.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    }
    out.push('\n');
    let regulator = crate::input::expression(&artifact.provenance.regulator)?;
    let mut outputs: Vec<Vec<String>> = Vec::new();
    for (&order, component) in kernels.orders.iter().zip(&kernels.components) {
        let basis = math(&regulator.pow(order));
        let component = match component {
            CoefficientComponent::Real => "real",
            CoefficientComponent::Imag => "imaginary",
        };
        if let Some(last) = outputs.last_mut()
            && last[1] == basis
        {
            last[2].push_str(", ");
            last[2].push_str(component);
        } else {
            outputs.push(vec![
                math(&fastsecdec::Atom::num(order)),
                basis,
                component.into(),
            ]);
        }
    }
    out.push_str(&section(
        "Laurent coefficients",
        vec!["Order", "Basis", "Components"],
        outputs,
        width,
        colors,
    ));
    Ok(out)
}
