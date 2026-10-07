//! Final generation report. Native metadata remains typed until presentation;
//! tabled owns terminal-cell widths, wrapping, borders and ANSI styling.

use std::{collections::BTreeSet, io::IsTerminal, path::Path};

use fastsecdec::{
    Atom, AtomCore,
    kernel::KernelSet,
    status::{CoefficientComponent, GenerationTimings},
};
use tabled::{
    Table,
    builder::Builder,
    settings::{
        Alignment, Color, Padding, Style, Width,
        object::{Columns, Rows},
        style::BorderColor,
    },
};

use crate::{CliResult, artifact::Artifact, math_display, terminal_policy::ColorPolicy};

/// Presentation values copied from the generated artifact and native kernels.
/// Keeping this separate from terminal I/O also supports alternate terminal widths.
pub struct Summary<'a> {
    pub artifact: String,
    pub content_id: &'a str,
    pub sectors: usize,
    pub workers: Option<usize>,
    pub requested_coefficient_expansion: Option<fastsecdec::generation::CoefficientExpansionMethod>,
    pub evaluator: Option<fastsecdec::kernel::CompilationSettings>,
    pub runtime_inputs: usize,
    pub backends: Vec<String>,
    pub regulator: Atom,
    pub outputs: Vec<(i32, CoefficientComponent)>,
    pub timings: Option<&'a GenerationTimings>,
}

pub fn print(
    output: &Path,
    artifact: &Artifact,
    kernels: &KernelSet,
    plain: bool,
    json: bool,
) -> CliResult<()> {
    let relative = crate::artifact::relative_path(output, Path::new("."))?;
    if json {
        // Existing machine fields retain their types and values. Components are
        // additive and align one-to-one with orders, including complex outputs.
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "artifact": relative,
                "content_id": artifact.content_id,
                "sectors": kernels.sectors().len(),
                "orders": kernels.orders(),
                "components": kernels.components(),
                "generation_timings": artifact.generation_timings,
                "workers": artifact.generation.as_ref().map(|generation| generation.workers),
                "generation": artifact.generation,
            }))?
        );
        return Ok(());
    }
    let terminal = std::io::stdout().is_terminal();
    let width = if terminal {
        crossterm::terminal::size()
            .map(|(width, _)| usize::from(width))
            .unwrap_or(80)
    } else {
        80
    };
    let summary = Summary {
        artifact: relative.to_string_lossy().into_owned(),
        content_id: &artifact.content_id,
        sectors: kernels.sectors().len(),
        workers: artifact
            .generation
            .as_ref()
            .map(|generation| generation.workers),
        requested_coefficient_expansion: artifact
            .generation
            .as_ref()
            .map(|generation| generation.requested_coefficient_expansion),
        evaluator: artifact
            .generation
            .as_ref()
            .and_then(|record| record.evaluator),
        runtime_inputs: kernels.runtime_parameters().len(),
        backends: kernels
            .sectors()
            .iter()
            .map(|sector| sector.statistics().backend.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        regulator: crate::input::expression(&artifact.provenance.regulator)?,
        outputs: kernels
            .orders()
            .iter()
            .copied()
            .zip(kernels.components().iter().copied())
            .collect(),
        timings: artifact.generation_timings.as_ref(),
    };
    print!(
        "{}",
        render(&summary, width, ColorPolicy::for_stream(plain, terminal))
    );
    Ok(())
}

/// Saved generation controls, not values inferred from this process's defaults.
/// Reused by artifact inspection so both views describe the same producer.
pub(crate) fn evaluator_rows(
    settings: Option<&fastsecdec::kernel::CompilationSettings>,
) -> Vec<[String; 2]> {
    let Some(settings) = settings else {
        return vec![["Evaluator settings".into(), "Not recorded".into()]];
    };
    vec![
        [
            "Horner iterations".into(),
            settings.horner_iterations.to_string(),
        ],
        [
            "CPE rounds".into(),
            settings
                .cpe_rounds
                .map_or_else(|| "Unlimited".into(), |n| n.to_string()),
        ],
        [
            "Optimizer cores".into(),
            format!("{} (deterministic)", settings.cores),
        ],
        [
            "Horner variables".into(),
            format!("{} maximum", settings.max_horner_scheme_variables),
        ],
        [
            "CPE cache".into(),
            format!("{} entries maximum", settings.max_common_pair_cache_entries),
        ],
        [
            "Pair distance".into(),
            format!("{} (upstream inactive)", settings.max_common_pair_distance),
        ],
        [
            "Direct translation".into(),
            if settings.direct_translation {
                "Yes"
            } else {
                "No"
            }
            .into(),
        ],
        [
            "Verbose optimizer".into(),
            if settings.verbose { "Yes" } else { "No" }.into(),
        ],
    ]
}

pub fn render(summary: &Summary<'_>, width: usize, colors: ColorPolicy) -> String {
    let width = width.clamp(1, 120);
    let backend = if summary.sectors == 0 {
        "Exact coefficients only".to_owned()
    } else {
        summary
            .backends
            .iter()
            .map(|backend| match backend.as_str() {
                "symjit_o2" => "SymJIT · O2",
                "symbolica_interpreter" => "Symbolica interpreter",
                other => other,
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let runtime_inputs = if summary.runtime_inputs == 0 {
        "None".to_owned()
    } else {
        format!("{} (supply at integration)", summary.runtime_inputs)
    };
    let mut facts = vec![
        ["Artifact".into(), terminal_text(&summary.artifact)],
        ["Files".into(), ".json metadata + .dat evaluators".into()],
        ["ID (short)".into(), short_id(summary.content_id)],
        ["Sectors".into(), summary.sectors.to_string()],
        [
            "Workers".into(),
            summary
                .workers
                .map_or_else(|| "Not recorded".into(), |workers| workers.to_string()),
        ],
        ["Backend".into(), terminal_text(&backend)],
        ["Runtime inputs".into(), runtime_inputs],
    ];
    if let Some(method) = summary.requested_coefficient_expansion {
        facts.push([
            "Requested expansion".into(),
            expansion_method(method).into(),
        ]);
    }
    facts.extend(evaluator_rows(summary.evaluator.as_ref()));
    let mut result = heading("Generation complete", width, colors, Color::FG_GREEN);
    result.push_str(&facts_table_with_labels(facts, width, 20, colors));
    result.push_str("\n\n");
    result.push_str(&heading(
        "Laurent coefficients",
        width,
        colors,
        Color::FG_CYAN,
    ));
    let mut output_rows: Vec<[String; 3]> = Vec::new();
    for (order, component) in &summary.outputs {
        let basis = math_display::atom(&summary.regulator.pow(*order), colors, width);
        let order = math_display::atom(&Atom::num(*order), colors, width);
        let component = match component {
            CoefficientComponent::Real => "real",
            CoefficientComponent::Imag => "imaginary",
        };
        if let Some(last) = output_rows.last_mut()
            && last[0] == order
        {
            last[2].push_str(", ");
            last[2].push_str(component);
        } else {
            output_rows.push([order, basis, component.to_owned()]);
        }
    }
    if output_rows.is_empty() {
        result.push_str(&heading(
            "No outputs",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    } else {
        result.push_str(&coefficient_table(output_rows, width, colors));
    }
    if let Some(timings) = summary.timings {
        result.push_str("\n\n");
        result.push_str(&heading("Wall time", width, colors, Color::FG_CYAN));
        result.push_str(&data_table(
            ["Phase", "Elapsed"],
            timing_rows(timings),
            width,
            colors,
            true,
        ));
        result.push('\n');
        result.push_str(&heading(
            "Total excludes writing the artifact files.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    } else {
        result.push('\n');
    }
    result
}

fn coefficient_table(rows: Vec<[String; 3]>, width: usize, colors: ColorPolicy) -> String {
    if width < 32 {
        return rows
            .into_iter()
            .map(|[order, basis, components]| {
                facts_table(
                    vec![
                        ["Order".into(), order],
                        ["Basis".into(), basis],
                        ["Components".into(), components],
                    ],
                    width,
                    colors,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    let available = width - 10;
    let order_width = 5;
    let component_width = 16.min((available - order_width) / 2);
    let basis_width = available - order_width - component_width;
    let mut table = Builder::from_iter(
        [["Order".into(), "Basis".into(), "Components".into()]]
            .into_iter()
            .chain(rows),
    )
    .build();
    table
        .with(Style::rounded())
        .modify(Columns::first(), Width::wrap(order_width).keep_words(true))
        .modify(
            Columns::new(1..2),
            Width::wrap(basis_width).keep_words(true),
        )
        .modify(
            Columns::last(),
            Width::wrap(component_width).keep_words(true),
        );
    if colors.enabled() {
        table
            .with(BorderColor::filled(Color::FG_BRIGHT_BLACK))
            .modify(Rows::first(), Color::FG_CYAN | Color::BOLD);
    }
    table.to_string()
}

pub(crate) fn timing_rows(timings: &GenerationTimings) -> Vec<[String; 2]> {
    let mut rows = [
        ("Input", timings.input_seconds),
        ("Parametrization", timings.parametrization_seconds),
        ("Domain metadata", timings.domain_seconds),
        ("Geometry", timings.geometry_seconds),
        ("Sector mapping", timings.mapping_seconds),
        ("Sector equivalence", timings.symmetry_seconds),
        ("Subtraction", timings.subtraction_seconds),
        ("Laurent expansion", timings.laurent_seconds),
        (
            "Coefficient expansion",
            timings.coefficient_expansion_seconds,
        ),
        ("Compilation", timings.compilation_seconds),
    ]
    .into_iter()
    // A zero field can mean an unmeasured/dispatched subphase. Do not
    // present it as an independently measured zero-duration operation.
    .filter(|(_, seconds)| *seconds > 0.0)
    .map(|(label, seconds)| [label.to_owned(), duration(seconds)])
    .collect::<Vec<_>>();
    rows.push(["Generation total".into(), duration(timings.total_seconds)]);
    rows
}

pub(crate) fn expansion_method(
    method: fastsecdec::generation::CoefficientExpansionMethod,
) -> &'static str {
    use fastsecdec::generation::CoefficientExpansionMethod;
    match method {
        CoefficientExpansionMethod::NativeNamed => "Coefficient series",
        CoefficientExpansionMethod::Physical => "Full expression",
    }
}

pub(crate) fn short_id(id: &str) -> String {
    let prefix = id.chars().take(16).collect::<String>();
    if prefix.len() == id.len() {
        prefix
    } else {
        format!("{prefix}…")
    }
}

pub(crate) fn terminal_text(text: &str) -> String {
    text.chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

pub(crate) fn duration(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "unavailable".into();
    }
    if seconds == 0.0 {
        "0 s".into()
    } else if seconds < 0.001 {
        "<1 ms".into()
    } else if seconds < 1.0 {
        format!("{:.1} ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{seconds:.2} s")
    } else if seconds < 3600.0 {
        let rounded = (seconds * 10.0).round() / 10.0;
        format!("{} min {:.1} s", (rounded / 60.0).floor(), rounded % 60.0)
    } else {
        format!("{:.2} h", seconds / 3600.0)
    }
}

/// Table widths include padding and borders. Assign column budgets explicitly:
/// a whole-table Width::wrap can otherwise shrink the label column to zero.
pub(crate) fn facts_table(rows: Vec<[String; 2]>, width: usize, colors: ColorPolicy) -> String {
    facts_table_with_labels(rows, width, 14.min(width / 3), colors)
}

pub(crate) fn facts_table_with_labels(
    rows: Vec<[String; 2]>,
    width: usize,
    labels: usize,
    colors: ColorPolicy,
) -> String {
    if width < 32 {
        let mut table = Builder::from_iter(
            rows.into_iter()
                .map(|[label, value]| [format!("{label}:\n{value}")]),
        )
        .build();
        table
            .with(Style::empty())
            .with(Padding::zero())
            .modify(Columns::first(), Width::wrap(width).keep_words(true));
        return table.to_string();
    }
    let labels = labels.min((width - 7) / 2);
    let values = width - labels - 7;
    let mut table = Builder::from_iter(rows).build();
    table
        .with(Style::rounded().remove_horizontals())
        .modify(Columns::first(), Width::wrap(labels).keep_words(true))
        .modify(Columns::last(), Width::wrap(values).keep_words(true));
    if colors.enabled() {
        table
            .with(BorderColor::filled(Color::FG_BRIGHT_BLACK))
            .modify(Columns::first(), Color::FG_CYAN | Color::BOLD);
    }
    table.to_string()
}

fn data_table(
    header: [&str; 2],
    rows: Vec<[String; 2]>,
    width: usize,
    colors: ColorPolicy,
    timing: bool,
) -> String {
    if width < 32 {
        return facts_table(rows, width, colors);
    }
    let mut table: Table =
        Builder::from_iter([header.map(str::to_owned)].into_iter().chain(rows)).build();
    let right = if timing { 15 } else { 16 }.min((width - 7) / 2);
    let left = width - right - 7;
    table
        .with(Style::rounded())
        .modify(Columns::first(), Width::wrap(left).keep_words(true))
        .modify(Columns::last(), Width::wrap(right).keep_words(true));
    if timing {
        table.modify(Columns::last(), Alignment::right());
    }
    if colors.enabled() {
        table
            .with(BorderColor::filled(Color::FG_BRIGHT_BLACK))
            .modify(Rows::first(), Color::FG_CYAN | Color::BOLD);
        if timing {
            table.modify(Rows::last(), Color::FG_GREEN | Color::BOLD);
        }
    }
    table.to_string()
}

pub(crate) fn heading(text: &str, width: usize, colors: ColorPolicy, color: Color) -> String {
    let mut table = Builder::from_iter([[text]]).build();
    table
        .with(Style::empty())
        .with(Padding::zero())
        .modify(Columns::first(), Width::wrap(width).keep_words(true));
    if colors.enabled() {
        table.modify(Rows::first(), color | Color::BOLD);
    }
    format!("{table}\n")
}
