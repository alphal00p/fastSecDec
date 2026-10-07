use super::tables::{bytes, section};
use crate::{
    CliResult,
    artifact::Artifact,
    generation_report::{
        duration, evaluator_rows, expansion_method, facts_table, facts_table_with_labels, heading,
        short_id, terminal_text, timing_rows,
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
    kernels: &KernelSet,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    let math =
        |value: &fastsecdec::Atom| math_display::atom(value, colors, width.saturating_sub(24));
    let sizes = kernels
        .sectors()
        .iter()
        .map(|k| k.statistics().exact_program_bytes)
        .collect::<Vec<_>>();
    let total: usize = sizes.iter().sum();
    let dimensions = kernels
        .sectors()
        .iter()
        .map(|k| k.dimension())
        .collect::<BTreeSet<_>>();
    let dimensions = match (dimensions.first(), dimensions.last()) {
        (Some(first), Some(last)) if first != last => format!("{first}–{last}"),
        (Some(first), _) => first.to_string(),
        _ => "None".into(),
    };
    let (json_path, data_path) = crate::artifact::paths(path)?;
    let json_size = usize::try_from(std::fs::metadata(json_path)?.len())?;
    let data_size = usize::try_from(std::fs::metadata(data_path)?.len())?;
    let charts = kernels
        .generation_metadata()
        .map(|metadata| {
            format!(
                "{} charts · {} representatives",
                metadata.charts().len(),
                metadata
                    .charts()
                    .iter()
                    .map(|c| c.representative())
                    .collect::<BTreeSet<_>>()
                    .len()
            )
        })
        .unwrap_or_else(|| "Not retained".into());
    let facts = vec![
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
            format!(".json {} · .dat {}", bytes(json_size), bytes(data_size)),
        ],
        ["Sectors".into(), kernels.sectors().len().to_string()],
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
            format!("{} ({total} bytes)", bytes(total)),
        ],
        [
            "Min/mean/max".into(),
            match (sizes.iter().min(), sizes.iter().max()) {
                (Some(min), Some(max)) => format!(
                    "{} / {} / {}",
                    bytes(*min),
                    bytes(total / sizes.len()),
                    bytes(*max)
                ),
                _ => "No numerical evaluators".into(),
            },
        ],
        [
            "Binding".into(),
            if kernels.parameters_bound() {
                "Bound"
            } else {
                "Awaiting integration values"
            }
            .into(),
        ],
        ["Loaded in".into(), duration(artifact.loading_seconds)],
    ];
    let mut out = heading("Artifact inspection", width, colors, Color::FG_GREEN);
    out.push_str(&facts_table(facts, width, colors));
    out.push_str("\n\n");
    out.push_str(&heading("Generation", width, colors, Color::FG_CYAN));
    let backend = kernels
        .sectors()
        .iter()
        .map(|kernel| match kernel.statistics().backend.as_str() {
            "symjit_o2" => "SymJIT · O2",
            "symbolica_interpreter" => "Symbolica interpreter",
            other => other,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
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
            if backend.is_empty() {
                "Exact coefficients only".into()
            } else {
                backend
            },
        ],
        [
            "Runtime inputs".into(),
            kernels.runtime_parameters().len().to_string(),
        ],
    ];
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
    for (&order, component) in kernels.orders().iter().zip(kernels.components()) {
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
