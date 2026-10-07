//! Mode-aware CLI generation plan alongside aggregate progress and worker rows.
mod plan;
#[cfg(test)]
mod tests;
use super::{GOLD, TEAL, memory, panel, title};
use crate::{generate::dispatch::Progress, terminal_policy::ColorPolicy};
use fastsecdec::status::{GenerationSnapshot, GenerationStage};
pub(super) use plan::Plan;
use plan::State;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Row, Table},
};

pub(super) fn render(
    frame: &mut Frame<'_>,
    snapshot: &GenerationSnapshot,
    workload: Option<&Progress>,
    memory: &memory::Snapshot,
    plan: &Plan,
    worker_offset: usize,
    color: ColorPolicy,
) {
    let small = frame.area().width < 80 || frame.area().height < 22;
    let chunks = Layout::vertical(if small {
        vec![
            Constraint::Length(1),
            Constraint::Length(10),
            Constraint::Length(4),
            Constraint::Min(3),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Length(11),
            Constraint::Min(5),
            Constraint::Length(1),
        ]
    })
    .split(frame.area());
    if small {
        frame.render_widget(
            Paragraph::new(Line::styled(
                " FastSecDec · Generation",
                color.foreground(TEAL).add_modifier(Modifier::BOLD),
            )),
            chunks[0],
        );
        steps(frame, chunks[1], plan, snapshot.elapsed_seconds, color);
        let (memory_label, capacity) = memory.free_or_available();
        frame.render_widget(
            Paragraph::new(vec![
                Line::styled(
                    progress_label(snapshot, workload, true),
                    color.foreground(TEAL),
                ),
                Line::raw(snapshot.detail.clone()),
                Line::styled(
                    format!(
                        "RSS {} · peak {}",
                        memory::bytes(memory.process_rss_bytes),
                        memory::bytes(memory.observed_peak_rss_bytes)
                    ),
                    color.foreground(TEAL),
                ),
                Line::styled(
                    format!(
                        "RAM {}/{} · {memory_label} {}",
                        memory::bytes(memory.system_used_bytes),
                        memory::bytes(memory.system_total_bytes),
                        memory::bytes(capacity)
                    ),
                    color.foreground(GOLD),
                ),
            ]),
            chunks[2],
        );
        workers(frame, chunks[3], workload, worker_offset, color);
    } else {
        frame.render_widget(title("Generation", color), chunks[0]);
        let columns = Layout::horizontal([
            Constraint::Length(
                plan.labels()
                    .iter()
                    .enumerate()
                    .map(|(index, label)| {
                        Line::from(*label).width()
                            + timer(plan, index, snapshot.elapsed_seconds).len()
                            + 5
                    })
                    .max()
                    .unwrap_or(33) as u16,
            ),
            Constraint::Min(20),
        ])
        .split(chunks[1]);
        steps(frame, columns[0], plan, snapshot.elapsed_seconds, color);
        let details = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(columns[1]);
        let ratio = snapshot
            .total
            .filter(|n| *n > 0)
            .map_or(0.0, |n| (snapshot.completed as f64 / n as f64).min(1.0));
        frame.render_widget(
            Gauge::default()
                .block(panel(plan.active_label(), color))
                .gauge_style(color.foreground(TEAL).add_modifier(Modifier::BOLD))
                .ratio(ratio)
                .label(progress_label(snapshot, workload, details[0].width < 80)),
            details[0],
        );
        let running = workload.map_or(0, Progress::running);
        let cores = workload.map_or(0, |work| work.workers.len());
        frame.render_widget(
            Paragraph::new(vec![
                Line::styled(
                    format!(
                        "Sectors {} · kernels {} · workers {running}/{cores}",
                        snapshot.sectors, snapshot.kernels
                    ),
                    color.foreground(GOLD),
                ),
                Line::raw(snapshot.detail.clone()),
            ])
            .block(panel("Coordinator", color)),
            details[1],
        );
        let (memory_label, capacity) = memory.free_or_available();
        frame.render_widget(
            Paragraph::new(vec![
                Line::styled(
                    format!(
                        "RSS {} · peak {}",
                        memory::bytes(memory.process_rss_bytes),
                        memory::bytes(memory.observed_peak_rss_bytes)
                    ),
                    color.foreground(TEAL),
                ),
                Line::styled(
                    format!(
                        "RAM {}/{} · {memory_label} {}",
                        memory::bytes(memory.system_used_bytes),
                        memory::bytes(memory.system_total_bytes),
                        memory::bytes(capacity)
                    ),
                    color.foreground(GOLD),
                ),
            ])
            .block(panel("Memory · entire process", color)),
            details[2],
        );
        workers(frame, chunks[2], workload, worker_offset, color);
    }
    let footer = chunks.last().copied().unwrap_or_default();
    frame.render_widget(
        Paragraph::new(if frame.area().width < 80 {
            " q/Esc/Ctrl-C cancel · ↑/↓ workers"
        } else {
            " q/Esc/Ctrl-C cancel · ↑/↓ workers · ✓ done ▶ active ○ pending — not needed"
        })
        .style(color.foreground(Color::Rgb(163, 143, 220))),
        footer,
    );
}

fn timer(plan: &Plan, index: usize, elapsed: f64) -> String {
    plan.seconds(index, elapsed)
        .map(|seconds| {
            // The shared formatter already selects s/min/h/d. Its tiny-time
            // scientific form is useful in reports, but steps stay plain.
            let time = if seconds < 0.01 {
                format!("{seconds:.2} s")
            } else {
                super::number::duration(seconds)
            };
            format!(" [{time}]")
        })
        .unwrap_or_default()
}

fn steps(frame: &mut Frame<'_>, area: Rect, plan: &Plan, elapsed: f64, color: ColorPolicy) {
    let rows = plan
        .labels()
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let (marker, ink) = match plan.state(index) {
                State::Complete => ("✓", TEAL),
                State::Current => ("▶", GOLD),
                State::Pending => ("○", Color::DarkGray),
                State::NotNeeded => ("—", Color::Rgb(130, 144, 158)),
            };
            let mut style = color.foreground(ink);
            if plan.state(index) == State::Current {
                style = style.add_modifier(Modifier::BOLD);
            }
            let time = timer(plan, index, elapsed);
            let available = usize::from(area.width.saturating_sub(5)).saturating_sub(time.len());
            // All plan labels are ASCII. Keep the complete duration visible on
            // narrow terminals and make any shortened label explicit.
            let label = if label.len() > available {
                format!("{}…", &label[..available.saturating_sub(1)])
            } else {
                label.to_owned()
            };
            Line::from(vec![
                Span::styled(format!(" {marker} "), style),
                Span::styled(label, style),
                Span::styled(time, style),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(rows).block(panel(plan.description(), color)),
        area,
    );
}

fn progress_label(
    snapshot: &GenerationSnapshot,
    workload: Option<&Progress>,
    compact: bool,
) -> String {
    let ratio = snapshot
        .total
        .filter(|n| *n > 0)
        .map_or(0.0, |n| (snapshot.completed as f64 / n as f64).min(1.0));
    let eta = workload
        .filter(|work| snapshot.total == Some(work.total) && snapshot.completed == work.completed)
        .and_then(Progress::eta_seconds)
        .map_or_else(|| "n/a".into(), |seconds| format!("{seconds:.1}s"));
    if snapshot.stage == GenerationStage::FormulaPreparation && snapshot.total == Some(0) {
        format!(
            "No formulas required · {:.1}s elapsed",
            snapshot.elapsed_seconds
        )
    } else if let Some(total) = snapshot.total {
        if compact {
            format!(
                "{:.0}% · {}/{} · {:.1}s · ETA {eta}",
                ratio * 100.,
                snapshot.completed,
                total,
                snapshot.elapsed_seconds
            )
        } else {
            format!(
                "{:.1}% · {} / {} {} · elapsed {:.1}s · stage ETA {eta}",
                ratio * 100.,
                snapshot.completed,
                total,
                if snapshot.stage == GenerationStage::FormulaPreparation {
                    "formulas"
                } else {
                    "jobs"
                },
                snapshot.elapsed_seconds
            )
        }
    } else {
        format!("In progress · {:.1}s · ETA n/a", snapshot.elapsed_seconds)
    }
}

fn workers(
    frame: &mut Frame<'_>,
    area: Rect,
    workload: Option<&Progress>,
    offset: usize,
    color: ColorPolicy,
) {
    let cores = workload.map_or(0, |work| work.workers.len());
    let visible = area.height.saturating_sub(3) as usize;
    let offset = offset.min(cores.saturating_sub(visible));
    let rows = workload
        .into_iter()
        .flat_map(|work| &work.workers)
        .skip(offset)
        .take(visible)
        .map(|worker| {
            let ink = if worker.busy {
                TEAL
            } else if worker.completed > 0 {
                Color::Rgb(126, 163, 243)
            } else {
                Color::DarkGray
            };
            Row::new(vec![
                format!("{:02}", worker.index + 1),
                if worker.busy {
                    "● running".into()
                } else {
                    "○ idle".into()
                },
                worker.completed.to_string(),
                format!("{:.1}s", worker.active_seconds),
                format!("{:.1}s", worker.busy_seconds + worker.active_seconds),
                worker.activity.clone(),
            ])
            .style(color.foreground(ink))
        });
    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Min(10),
            ],
        )
        .header(
            Row::new([
                "Core",
                "State",
                "Done",
                "Job time",
                "Busy time",
                "Native activity",
            ])
            .style(color.foreground(GOLD).add_modifier(Modifier::BOLD)),
        )
        .block(panel(
            &format!(
                "Worker activity · cores {}–{} / {} · ↑/↓ scroll",
                (offset + 1).min(cores),
                (offset + visible).min(cores),
                cores
            ),
            color,
        ))
        .column_spacing(1),
        area,
    );
}
