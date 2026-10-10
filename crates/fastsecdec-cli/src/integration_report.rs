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
    if let Some(selection) = &result.source_selection {
        println!(
            "Generation extent: original source charts {:?} of {}; partial original integral",
            selection.source_sectors(),
            selection.original_source_count()
        );
    }
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
    if let Some(contour) = &result.contour {
        print!(
            "\n{}",
            heading("Contour validation", width, colors, Color::FG_CYAN)
        );
        println!("{}", facts_table(contour_rows(contour), width, colors));
    }
    if let Some(design) = &result.qmc_design {
        println!("\n{design}");
    }
    if let Some(comparison) = comparison {
        print!("\n{comparison}");
    }
    Ok(())
}

fn contour_rows(report: &fastsecdec::status::ContourRunReport) -> Vec<[String; 2]> {
    use fastsecdec::contour::{ContourMode, ContourValidation};
    let prescription = match report.deformation {
        ContourMode::Off => "Off".into(),
        ContourMode::Fixed { lambda } => format!("Fixed · λ = {lambda}"),
        ContourMode::Dynamical {
            safety_fraction,
            lambda_cap,
            displacement_cap,
            construction,
        } => {
            let construction = match construction {
                fastsecdec::contour::DynamicConstruction::Polynomial => "polynomial",
                fastsecdec::contour::DynamicConstruction::SignAware => "sign-aware",
            };
            format!(
                "Dynamic · S = {safety_fraction} · L = {lambda_cap} · R = {displacement_cap} · {construction}"
            )
        }
    };
    let policy = match report.validation.policy {
        ContourValidation::Always => "Pilot and production",
        ContourValidation::Pilot => "Pilot only; production unchecked",
        ContourValidation::Off => "Checks disabled",
    };
    let required: u128 = report
        .pilots
        .iter()
        .map(|p| p.required_charts.len() as u128)
        .sum();
    let validated: u128 = report
        .pilots
        .iter()
        .map(|p| p.validated_charts.len() as u128)
        .sum();
    let points: u128 = report.pilots.iter().map(|p| p.sampled_points as u128).sum();
    let pilot = if report.pilots.is_empty() {
        "No pilot evidence recorded".into()
    } else {
        format!("{validated}/{required} charts · {points} points")
    };
    let current = report.invocation_checks;
    let mut rows = vec![
        ["Deformation".into(), prescription],
        ["Runtime validation".into(), policy.into()],
        ["Recorded pilot coverage".into(), pilot],
        [
            "Production argument checks".into(),
            current.production.checked_arguments.to_string(),
        ],
        [
            "Adaptation argument checks".into(),
            current.adaptation.checked_arguments.to_string(),
        ],
    ];
    if current.production.checked_arguments > 0 {
        rows.push([
            "Production check precision".into(),
            format!("{} bits maximum", current.production.maximum_bits),
        ]);
    }
    if let Some(recorded) = report.recorded_checks {
        rows.push([
            "Recorded production checks".into(),
            recorded.production.checked_arguments.to_string(),
        ]);
    }
    rows.push([
        "Check scope".into(),
        "Polynomial map at checked arguments".into(),
    ]);
    rows
}
