//! Presentation-only bridge for the library's typed progress events.
use std::{ops::ControlFlow, time::Instant};

use fastsecdec::{generation::GenerationProgress, status::GenerationSnapshot};

use crate::display::Dashboard;

pub(super) fn observe_generation(
    dashboard: &mut Dashboard,
    status: &mut GenerationSnapshot,
    display_error: &mut Option<String>,
    started: Instant,
    max_order: i32,
    progress: &GenerationProgress,
) -> ControlFlow<()> {
    status.observe_generation(max_order, progress);
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    publish_generation(dashboard, status, display_error)
}

pub(super) fn publish_generation(
    dashboard: &mut Dashboard,
    status: &GenerationSnapshot,
    display_error: &mut Option<String>,
) -> ControlFlow<()> {
    if let Err(error) = dashboard.generation(status) {
        display_error.get_or_insert_with(|| error.to_string());
        dashboard.request_cancel();
        return ControlFlow::Break(());
    }
    // Presentation may coalesce this event; cancellation still runs every time.
    if dashboard.cancelled() {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests;
