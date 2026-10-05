//! CLI presentation/cancellation adapter for the public diagnostic API.
use std::{io::IsTerminal, ops::ControlFlow};

use crossterm::style::{Color, Stylize};
use fastsecdec::diagnostics::{BoundaryScanProgress, BoundaryScanReport, DiagnosticProgress};

use crate::display::Dashboard;

pub fn observe(
    dashboard: &Dashboard,
    json_status: bool,
    progress: &DiagnosticProgress,
) -> ControlFlow<()> {
    if json_status {
        // All diagnostic numeric fields are validated finite by the library.
        if let Ok(message) = serde_json::to_string(progress) {
            eprintln!("{message}");
        }
    }
    if dashboard.cancelled() {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

pub fn observe_scan(
    dashboard: &Dashboard,
    json_status: bool,
    progress: &BoundaryScanProgress,
) -> ControlFlow<()> {
    if json_status && let Ok(message) = serde_json::to_string(progress) {
        eprintln!("{message}");
    }
    if dashboard.cancelled() {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

pub fn display_scan(report: &BoundaryScanReport, plain: bool) {
    let colored =
        crate::terminal_policy::ColorPolicy::for_stream(plain, std::io::stdout().is_terminal())
            .enabled();
    for line in report.to_string().lines() {
        if colored && (line.starts_with('╭') || line.starts_with('╰') || line.contains("Attempt"))
        {
            println!(
                "{}",
                line.with(Color::Rgb {
                    r: 68,
                    g: 210,
                    b: 188
                })
                .bold()
            );
        } else {
            println!("{line}");
        }
    }
}
