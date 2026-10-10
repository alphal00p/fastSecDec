//! Loading observations contain no evaluator state and are redrawn from cache.
use std::time::Instant;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Modifier,
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Wrap},
};

use super::{GOLD, TEAL, memory, number, panel, title};
use crate::{loading::Phase, terminal_policy::ColorPolicy};

pub(super) struct Cached {
    pub snapshot: crate::loading::Snapshot,
    pub memory: memory::Snapshot,
    pub observed_at: Instant,
    pub cancelled: bool,
}

fn phase(snapshot: &crate::loading::Snapshot) -> &'static str {
    match snapshot.phase {
        Phase::Metadata => "Reading artifact metadata",
        Phase::ReadingBinary => "Reading evaluator data",
        Phase::Decoding => "Decoding saved programs",
        Phase::Restoring if snapshot.completed == snapshot.total && snapshot.total.is_some() => {
            "Finalizing loaded kernels"
        }
        Phase::Restoring => "Preparing executable evaluators",
        Phase::ContourValidation => "Checking contour pilot and subtraction faces",
        Phase::Complete => "Evaluators loaded",
    }
}

fn progress(snapshot: &crate::loading::Snapshot) -> String {
    match (snapshot.phase, snapshot.completed, snapshot.total) {
        (Phase::ReadingBinary, Some(done), total) => format!(
            "{} / {}",
            memory::bytes(Some(done as u64)),
            memory::bytes(total.map(|value| value as u64))
        ),
        (Phase::Restoring, Some(done), Some(total)) => {
            format!("{done} / {total} sectors")
        }
        (Phase::ContourValidation, Some(done), Some(total)) => {
            format!("{done} / {total} pilot points")
        }
        (Phase::Complete, _, _) => snapshot.primary_evaluators.as_ref().map_or_else(
            || "Complete".into(),
            |primary| {
                format!(
                    "JIT cache restored {}; rebuilt {} missing / {} incompatible; eager {}",
                    primary.cache_restored,
                    primary.rebuilt_missing_cache,
                    primary.rebuilt_incompatible_cache,
                    primary.eager,
                )
            },
        ),
        _ => "Working · progress unavailable".into(),
    }
}

fn duration(seconds: Option<f64>) -> String {
    match seconds {
        Some(value) if value.is_finite() && (0.0..0.01).contains(&value) => {
            format!("{value:.2} s")
        }
        Some(value) => number::duration(value),
        None => "—".into(),
    }
}

pub(super) fn plain(cached: &Cached) -> String {
    let snapshot = &cached.snapshot;
    format!(
        "Loading artifact · {} · {} · elapsed {} · CPU {}{}\n  {} · {}",
        phase(snapshot),
        progress(snapshot),
        duration(Some(snapshot.elapsed_seconds)),
        duration(cached.memory.process_cpu_seconds),
        if cached.cancelled {
            " · Cancelling; waiting for the current native operation"
        } else {
            ""
        },
        cached.memory.process_line(),
        cached.memory.system_line(),
    )
}

pub(super) fn render(frame: &mut Frame<'_>, cached: &Cached, color: ColorPolicy, cancelled: bool) {
    let snapshot = &cached.snapshot;
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);
    frame.render_widget(title("Loading artifact", color), chunks[0]);
    let elapsed = snapshot.elapsed_seconds
        + if snapshot.phase == Phase::Complete {
            0.0
        } else {
            cached.observed_at.elapsed().as_secs_f64()
        };
    let activity = Paragraph::new(vec![
        Line::from(Span::styled(
            phase(snapshot),
            color.foreground(TEAL).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Elapsed  ", color.foreground(GOLD)),
            Span::raw(duration(Some(elapsed))),
            Span::styled("    Process CPU  ", color.foreground(GOLD)),
            Span::raw(duration(cached.memory.process_cpu_seconds)),
        ]),
    ])
    .wrap(Wrap { trim: false })
    .block(panel(" Current operation ", color));
    frame.render_widget(activity, chunks[1]);
    if let (Some(done), Some(total)) = (snapshot.completed, snapshot.total)
        && total > 0
    {
        frame.render_widget(
            Gauge::default()
                .block(panel(" Phase progress ", color))
                .gauge_style(color.foreground(TEAL))
                .ratio((done as f64 / total as f64).clamp(0.0, 1.0))
                .label(progress(snapshot)),
            chunks[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new(progress(snapshot)).block(panel(" Phase progress ", color)),
            chunks[2],
        );
    }
    let memory = &cached.memory;
    let (free_label, free) = memory.free_or_available();
    let rows = [
        ("Process RSS", memory::bytes(memory.process_rss_bytes)),
        (
            "Observed peak",
            memory::bytes(memory.observed_peak_rss_bytes),
        ),
        (
            "System RAM",
            format!(
                "{} used / {}",
                memory::bytes(memory.system_used_bytes),
                memory::bytes(memory.system_total_bytes)
            ),
        ),
        (free_label, format!("{} (OS)", memory::bytes(free))),
    ];
    frame.render_widget(
        Paragraph::new(
            rows.into_iter()
                .map(|(label, value)| {
                    Line::from(vec![
                        Span::styled(format!("{label:<15}"), color.foreground(GOLD)),
                        Span::raw(value),
                    ])
                })
                .collect::<Vec<_>>(),
        )
        .wrap(Wrap { trim: false })
        .block(panel(" Memory ", color)),
        chunks[3],
    );
    frame.render_widget(
        Paragraph::new(if cancelled {
            "Cancelling · waiting for the current native operation"
        } else {
            "q / Esc / Ctrl+C  cancel"
        })
        .style(color.foreground(GOLD))
        .wrap(Wrap { trim: false }),
        chunks[4],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn view(phase: Phase, done: Option<usize>, total: Option<usize>) -> Cached {
        Cached {
            snapshot: crate::loading::Snapshot {
                phase,
                completed: done,
                total,
                elapsed_seconds: 90.0,
                primary_evaluators: None,
            },
            memory: memory::Snapshot::default(),
            observed_at: Instant::now(),
            cancelled: false,
        }
    }

    fn screen(cached: &Cached, width: u16, cancelled: bool) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    cached,
                    ColorPolicy::for_stream(false, false),
                    cancelled,
                )
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .chunks(width as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn unknown_work_has_no_fake_percentage_and_unavailable_memory_is_explicit() {
        let cached = view(Phase::Decoding, None, None);
        assert!(plain(&cached).contains("Decoding saved programs"));
        assert!(!plain(&cached).contains("validating"));
        for width in [40, 80, 120] {
            let output = screen(&cached, width, false);
            assert!(output.contains("progress unavailable"), "{output}");
            assert!(output.contains("unavailable"));
            assert!(output.contains("1.50 min"));
            assert!(!output.contains('%'));
            assert!(!output.contains("0.0 MiB"));
            assert!(!output.contains("·10"));
            let cancelled = screen(&cached, width, true);
            assert!(cancelled.contains("Cancelling"));
        }
    }

    #[test]
    fn byte_and_sector_progress_and_completion_are_distinct() {
        let reading = view(Phase::ReadingBinary, Some(1 << 30), Some(2 << 30));
        assert!(screen(&reading, 80, false).contains("1.0 GiB / 2.0 GiB"));
        let restoring = view(Phase::Restoring, Some(2), Some(304));
        assert!(plain(&restoring).contains("Preparing executable evaluators"));
        assert!(screen(&restoring, 80, false).contains("2 / 304 sectors"));
        let finalizing = view(Phase::Restoring, Some(304), Some(304));
        assert!(screen(&finalizing, 80, false).contains("Finalizing loaded kernels"));
        assert!(!plain(&finalizing).contains("Evaluators loaded"));
        let complete = view(Phase::Complete, None, None);
        assert!(screen(&complete, 80, false).contains("Evaluators loaded"));
        assert!(!plain(&complete).contains("progress unavailable"));
    }
}
