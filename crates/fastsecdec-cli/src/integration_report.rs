//! Final integration presentation shares the live dashboard's number policy.
use std::io::IsTerminal;

use fastsecdec::{integration::AccuracyTarget, status::CoefficientComponent};
use tabled::settings::Color;

use crate::{
    CliResult,
    display::number::{duration, superscript, uncertainty},
    driver::IntegrationReport,
    generation_report::{facts_table_with_labels, heading},
    reference::ReferenceReport,
    terminal_policy::ColorPolicy,
};

fn facts_table(rows: Vec<[String; 2]>, width: usize, colors: ColorPolicy) -> String {
    facts_table_with_labels(rows, width, 36, colors)
}

pub(crate) fn print(
    result: &IntegrationReport,
    comparison: Option<&ReferenceReport>,
    plain: bool,
    json: bool,
) -> CliResult<()> {
    if json {
        let mut value = serde_json::to_value(result)?;
        if let Some(comparison) = comparison {
            value["reference"] = serde_json::to_value(comparison)?;
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }
    let terminal = std::io::stdout().is_terminal();
    let colors = ColorPolicy::for_stream(plain, terminal);
    let width = if terminal {
        crossterm::terminal::size().map_or(80, |(w, _)| usize::from(w))
    } else {
        80
    }
    .clamp(1, 120);
    print!(
        "{}",
        heading("Integration result", width, colors, Color::FG_CYAN)
    );
    let target = match result.accuracy_target {
        AccuracyTarget::AllComponents => "All Laurent components".into(),
        AccuracyTarget::LaurentOrder(order) => {
            format!("ε{} · complex RMS uncertainty", superscript(order))
        }
    };
    println!(
        "{}",
        facts_table(
            vec![
                ["Stopping reason".into(), result.stopping_reason.clone()],
                ["Scope".into(), result.scope.to_string()],
                ["Accuracy target".into(), target],
                ["Wall elapsed".into(), duration(result.elapsed_seconds)],
                [
                    "Aggregate worker time".into(),
                    duration(result.operational.worker_seconds)
                ],
                [
                    "Process CPU time".into(),
                    result
                        .process_cpu_seconds
                        .map_or_else(|| "Unavailable".into(), duration)
                ],
            ],
            width,
            colors
        )
    );
    if let Some(estimate) = &result.estimate {
        print!(
            "\n{}",
            heading(
                "Accepted Laurent coefficients",
                width,
                colors,
                Color::FG_GREEN
            )
        );
        let rows = estimate
            .orders
            .iter()
            .enumerate()
            .map(|(i, order)| {
                let component = match estimate.components[i] {
                    CoefficientComponent::Real => "Re",
                    CoefficientComponent::Imag => "Im",
                };
                [
                    format!("ε{} · {component}", superscript(*order)),
                    uncertainty(estimate.mean[i], Some(estimate.standard_error[i])),
                ]
            })
            .collect();
        println!("{}", facts_table(rows, width, colors));
    } else {
        println!("No accepted production estimate is available.");
    }
    print!(
        "\n{}",
        heading(
            "Runtime diagnostics · this invocation",
            width,
            colors,
            Color::FG_CYAN
        )
    );
    println!(
        "{}",
        facts_table(
            crate::display::diagnostic_summary(&result.operational, result.stability_mode),
            width,
            colors
        )
    );
    if let Some(design) = &result.qmc_design {
        println!("\n{design}");
    }
    if let Some(comparison) = comparison {
        print!("\n{comparison}");
    }
    Ok(())
}
