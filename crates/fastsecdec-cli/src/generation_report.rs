//! Final generation report. Native metadata remains typed until presentation;
//! tabled owns terminal-cell widths, wrapping, borders and ANSI styling.

use std::{collections::BTreeSet, io::IsTerminal, path::Path};

use fastsecdec::{
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

use crate::{CliResult, artifact::Artifact, terminal_policy::ColorPolicy};

/// Presentation values copied from the generated artifact and native kernels.
/// Keeping this separate from terminal I/O also supports alternate terminal widths.
pub struct Summary<'a> {
    pub artifact: String,
    pub content_id: &'a str,
    pub sectors: usize,
    pub workers: usize,
    pub runtime_inputs: usize,
    pub backends: Vec<String>,
    pub outputs: Vec<(i32, CoefficientComponent)>,
    pub timings: Option<&'a GenerationTimings>,
}

pub fn print(
    output: &Path,
    artifact: &Artifact,
    kernels: &KernelSet,
    workers: usize,
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
                "workers": workers,
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
        workers,
        runtime_inputs: kernels.runtime_parameters().len(),
        backends: kernels
            .sectors()
            .iter()
            .map(|sector| sector.statistics().backend.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
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
    let facts = vec![
        ["Artifact".into(), terminal_text(&summary.artifact)],
        ["Files".into(), ".json metadata + .dat evaluators".into()],
        ["ID (short)".into(), short_id(summary.content_id)],
        ["Sectors".into(), summary.sectors.to_string()],
        ["Workers".into(), summary.workers.to_string()],
        ["Backend".into(), terminal_text(&backend)],
        ["Runtime inputs".into(), runtime_inputs],
    ];
    let mut result = heading("Generation complete", width, colors, Color::FG_GREEN);
    result.push_str(&facts_table(facts, width, colors));
    result.push_str("\n\n");
    result.push_str(&heading(
        "Laurent coefficients",
        width,
        colors,
        Color::FG_CYAN,
    ));
    let mut output_rows: Vec<[String; 2]> = Vec::new();
    for (order, component) in &summary.outputs {
        let order = format!("ε^{order}");
        let component = match component {
            CoefficientComponent::Real => "real",
            CoefficientComponent::Imag => "imaginary",
        };
        if let Some(last) = output_rows.last_mut()
            && last[0] == order
        {
            last[1].push_str(", ");
            last[1].push_str(component);
        } else {
            output_rows.push([order, component.to_owned()]);
        }
    }
    if output_rows.is_empty() {
        output_rows.push(["Outputs".into(), "None".into()]);
    }
    result.push_str(&data_table(
        ["Order", "Components"],
        output_rows,
        width,
        colors,
        false,
    ));
    if let Some(timings) = summary.timings {
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
        result.push_str("\n\n");
        result.push_str(&heading("Wall time", width, colors, Color::FG_CYAN));
        result.push_str(&data_table(["Phase", "Elapsed"], rows, width, colors, true));
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

fn short_id(id: &str) -> String {
    let prefix = id.chars().take(16).collect::<String>();
    if prefix.len() == id.len() {
        prefix
    } else {
        format!("{prefix}…")
    }
}

fn terminal_text(text: &str) -> String {
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

fn duration(seconds: f64) -> String {
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
fn facts_table(rows: Vec<[String; 2]>, width: usize, colors: ColorPolicy) -> String {
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
    let labels = 14.min(width / 3);
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

fn heading(text: &str, width: usize, colors: ColorPolicy, color: Color) -> String {
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
