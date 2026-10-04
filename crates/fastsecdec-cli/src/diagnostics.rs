//! CLI presentation/cancellation adapter for the public diagnostic API.
use std::ops::ControlFlow;

use fastsecdec::diagnostics::DiagnosticProgress;

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
