//! Terminal rendering consumes public status snapshots, never computation state.
mod integration_activity;
mod memory;
mod terminal;
pub(crate) use integration_activity::IntegrationWorkerActivity;

use std::{
    io::{self, IsTerminal},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use fastsecdec::status::{GenerationSnapshot, IntegrationSnapshot};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
};

use crate::{CliResult, terminal_policy::ColorPolicy};

const TEAL: Color = Color::Rgb(68, 210, 188);
const GOLD: Color = Color::Rgb(243, 195, 91);

pub struct Dashboard {
    interrupt: terminal::Control,
    terminal: Option<Terminal<CrosstermBackend<io::Stderr>>>,
    last_frame: Instant,
    last_log: Instant,
    json_status: bool,
    scope: fastsecdec::results::ResultScope,
    integration_cadence: crate::status_policy::StatusCadence,
    generation_cadence: crate::status_policy::StatusCadence,
    generation_boundary: Option<crate::status_policy::GenerationBoundary>,
    color: ColorPolicy,
    generation_workers: Option<crate::generate::dispatch::Progress>,
    generation_worker_offset: std::cell::Cell<usize>,
    integration_workers: Vec<IntegrationWorkerActivity>,
    integration_worker_offset: std::cell::Cell<usize>,
    memory: memory::Monitor,
}

impl Dashboard {
    pub fn new(enabled: bool, json_status: bool) -> CliResult<Self> {
        Self::with_status_interval(enabled, json_status, 100)
    }

    pub fn with_status_interval(
        enabled: bool,
        json_status: bool,
        json_interval_ms: u64,
    ) -> CliResult<Self> {
        let interrupt = terminal::Control::new()?;
        let terminal = if enabled && !json_status && io::stderr().is_terminal() {
            interrupt.enter_terminal()?;
            Some(Terminal::new(CrosstermBackend::new(io::stderr()))?)
        } else {
            None
        };
        let interval = if json_status {
            Duration::from_millis(json_interval_ms)
        } else if terminal.is_some() {
            Duration::from_millis(40)
        } else {
            Duration::from_secs(1)
        };
        Ok(Self {
            interrupt,
            terminal,
            last_frame: Instant::now() - Duration::from_secs(2),
            last_log: Instant::now() - Duration::from_secs(2),
            json_status,
            scope: Default::default(),
            integration_cadence: crate::status_policy::StatusCadence::new(interval),
            generation_cadence: crate::status_policy::StatusCadence::new(interval),
            generation_boundary: None,
            color: ColorPolicy::for_stream(false, io::stderr().is_terminal()),
            generation_workers: None,
            generation_worker_offset: std::cell::Cell::new(0),
            integration_workers: Vec::new(),
            integration_worker_offset: std::cell::Cell::new(0),
            memory: memory::Monitor::new(),
        })
    }

    pub(crate) fn generation_workers(&mut self, progress: &crate::generate::dispatch::Progress) {
        self.generation_workers = Some(progress.clone());
    }

    pub(crate) fn generation_coordinator(&mut self) {
        self.generation_workers = None;
    }

    pub fn generation(&mut self, snapshot: &GenerationSnapshot) -> CliResult<()> {
        if !self.interrupt.terminal_active() {
            self.terminal = None;
        }
        let memory = self.memory.sample();
        if self.json_status {
            let boundary = crate::status_policy::GenerationBoundary::from(snapshot);
            let force = self.generation_boundary != Some(boundary);
            self.generation_boundary = Some(boundary);
            // Existing geometry and compilation events have their own semantic
            // boundaries. Coalesce only the new frequent coefficient polls.
            if snapshot.stage == fastsecdec::status::GenerationStage::CoefficientExpansion
                && !self
                    .generation_cadence
                    .due(Duration::from_secs_f64(snapshot.elapsed_seconds), force)
            {
                return Ok(());
            }
            #[derive(serde::Serialize)]
            struct GenerationStatus<'a> {
                #[serde(flatten)]
                snapshot: &'a GenerationSnapshot,
                #[serde(skip_serializing_if = "Option::is_none")]
                workload: Option<&'a crate::generate::dispatch::Progress>,
                memory: memory::Snapshot,
            }
            eprintln!(
                "{}",
                serde_json::to_string(&GenerationStatus {
                    snapshot,
                    workload: self.generation_workers.as_ref(),
                    memory,
                })?
            );
            return Ok(());
        }
        if let Some(terminal) = &mut self.terminal {
            let _output = io::stderr().lock();
            if !self.interrupt.terminal_active() {
                return Ok(());
            }
            if self.last_frame.elapsed() < Duration::from_millis(40)
                && snapshot.stage != fastsecdec::status::GenerationStage::Complete
            {
                return Ok(());
            }
            self.last_frame = Instant::now();
            terminal.draw(|frame| {
                if frame.area().width < 72 || frame.area().height < 22 {
                    compact(frame, "Generation", format!("{}\n{}\n{snapshot}", memory.process_line(), memory.system_line()), self.color);
                    return;
                }
                let chunks = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ]).split(frame.area());
                frame.render_widget(title("Generation", self.color), chunks[0]);
                let ratio = snapshot.total.filter(|n| *n > 0).map_or(0.0, |n| (snapshot.completed as f64 / n as f64).min(1.0));
                let workload = self.generation_workers.as_ref();
                let eta = workload.filter(|work| snapshot.total == Some(work.total) && snapshot.completed == work.completed)
                    .and_then(|work| work.eta_seconds())
                    .map_or_else(|| "unavailable".into(), |seconds| format!("{seconds:.1} s"));
                frame.render_widget(
                    Gauge::default().block(panel(&format!("{} · aggregate stage progress", snapshot.stage.label()), self.color))
                        .gauge_style(self.color.foreground(TEAL).add_modifier(Modifier::BOLD))
                        .ratio(ratio).label(if let Some(total) = snapshot.total {
                            format!("{:5.1}%  ·  {} / {} jobs  ·  elapsed {:.1} s  ·  stage ETA ≈ {}", ratio * 100.0, snapshot.completed, total, snapshot.elapsed_seconds, eta)
                        } else { format!("Coordinator in progress  ·  elapsed {:.1} s  ·  ETA unavailable", snapshot.elapsed_seconds) }),
                    chunks[1],
                );
                let running = workload.map_or(0, |work| work.running());
                let cores = workload.map_or(0, |work| work.workers.len());
                let rows = [
                    Row::new(vec!["Sectors".into(), snapshot.sectors.to_string(), "Kernels".into(), snapshot.kernels.to_string(), "Workers".into(), format!("{running} / {cores} busy")])
                        .style(self.color.foreground(GOLD)),
                    Row::new(vec!["Activity".into(), snapshot.detail.clone(), String::new(), String::new(), String::new(), String::new()]),
                ];
                frame.render_widget(Table::new(rows, [Constraint::Length(9), Constraint::Percentage(40), Constraint::Length(8), Constraint::Length(8), Constraint::Length(9), Constraint::Min(10)])
                    .block(panel("Coordinator", self.color)).column_spacing(1), chunks[2]);
                frame.render_widget(Paragraph::new(vec![
                    Line::styled(memory.process_line(), self.color.foreground(TEAL)),
                    Line::styled(memory.system_line(), self.color.foreground(GOLD)),
                ]).block(panel("Memory · entire process · sampled at most 2 Hz", self.color)), chunks[3]);
                let visible_workers = chunks[4].height.saturating_sub(3) as usize;
                let worker_offset = self.generation_worker_offset.get().min(cores.saturating_sub(visible_workers));
                let rows = workload.into_iter().flat_map(|work| &work.workers).skip(worker_offset).take(visible_workers).map(|worker| {
                    let color = if worker.busy { TEAL } else if worker.completed > 0 { Color::Rgb(126, 163, 243) } else { Color::DarkGray };
                    Row::new(vec![
                        format!("{:02}", worker.index + 1),
                        if worker.busy { "● running".into() } else { "○ idle".into() },
                        worker.completed.to_string(),
                        format!("{:.1} s", worker.active_seconds),
                        format!("{:.1} s", worker.busy_seconds + worker.active_seconds),
                        worker.activity.clone(),
                    ]).style(self.color.foreground(color))
                });
                frame.render_widget(Table::new(rows, [Constraint::Length(5), Constraint::Length(10), Constraint::Length(6), Constraint::Length(9), Constraint::Length(9), Constraint::Min(20)])
                    .header(Row::new(["Core", "State", "Done", "Job time", "Busy time", "Native activity"]).style(self.color.foreground(GOLD).add_modifier(Modifier::BOLD)))
                    .block(panel(&format!("Worker activity · cores {}–{} / {} · ↑/↓ scroll", (worker_offset + 1).min(cores), (worker_offset + visible_workers).min(cores), cores), self.color)).column_spacing(1), chunks[4]);
                frame.render_widget(Paragraph::new("  Ctrl-C / q / Esc  cancel safely · ETA covers this stage; later work is discovered dynamically")
                    .style(self.color.foreground(Color::Rgb(163, 143, 220))), chunks[5]);
            })?;
        } else if self.last_log.elapsed() >= Duration::from_secs(1)
            || snapshot.stage == fastsecdec::status::GenerationStage::Complete
        {
            eprintln!(
                "{snapshot}\n  {} · {}",
                memory.process_line(),
                memory.system_line()
            );
            self.last_log = Instant::now();
        }
        Ok(())
    }

    pub fn set_scope(&mut self, scope: fastsecdec::results::ResultScope) {
        self.scope = scope;
    }

    pub(crate) fn integration_work(&mut self, workers: Vec<IntegrationWorkerActivity>) {
        self.integration_workers = workers;
    }

    /// Call before constructing the native integration snapshot. Forced events
    /// include stage/round boundaries and every final/cancelled/failed report.
    pub fn integration_due(&mut self, elapsed: Duration, force: bool) -> bool {
        self.integration_cadence.due(elapsed, force)
    }

    pub fn integration(&mut self, snapshot: &IntegrationSnapshot, elapsed: f64) -> CliResult<()> {
        if !self.interrupt.terminal_active() {
            self.terminal = None;
        }
        let activity = integration_activity::summary(&self.integration_workers);
        if self.json_status {
            eprintln!(
                "{}",
                serde_json::to_string(&ScopedStatus {
                    snapshot,
                    scope: &self.scope,
                    in_flight_unaccepted: &self.integration_workers,
                })?
            );
            return Ok(());
        }
        if let Some(terminal) = &mut self.terminal {
            let _output = io::stderr().lock();
            if !self.interrupt.terminal_active() {
                return Ok(());
            }
            terminal.draw(|frame| {
                if frame.area().width < 72 || frame.area().height < 22 {
                    compact(
                        frame,
                        "Integration",
                        format!("{} · {elapsed:.2} s\n{activity}\n{snapshot}", self.scope),
                        self.color,
                    );
                    return;
                }
                let chunks = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(6),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(frame.area());
                frame.render_widget(
                    title(&format!("Integration · {}", self.scope), self.color),
                    chunks[0],
                );
                let ratio = if snapshot.planned_points == 0 {
                    1.0
                } else {
                    (snapshot.completed_points as f64 / snapshot.planned_points as f64).min(1.0)
                };
                frame.render_widget(
                    Gauge::default()
                        .block(panel(
                            &format!("{:?} · {:?}", snapshot.method, snapshot.stage),
                            self.color,
                        ))
                        .gauge_style(self.color.foreground(TEAL))
                        .ratio(ratio)
                        .label(format!(
                            "{} / {} accepted points    {:.2} s",
                            snapshot.completed_points, snapshot.planned_points, elapsed
                        )),
                    chunks[1],
                );
                let rows = snapshot
                    .estimate
                    .as_ref()
                    .map(|estimate| {
                        estimate
                            .orders
                            .iter()
                            .enumerate()
                            .map(|(i, order)| {
                                let value = estimate.mean[i];
                                let error = estimate.standard_error[i];
                                Row::new(vec![
                                    format!("ε^{order} {:?}", estimate.components[i]),
                                    format!("{value:+.10e}"),
                                    format!("{error:.3e}"),
                                    if value == 0.0 {
                                        "—".into()
                                    } else {
                                        format!("{:.2}%", 100.0 * error / value.abs())
                                    },
                                ])
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                frame.render_widget(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(10),
                            Constraint::Percentage(42),
                            Constraint::Percentage(25),
                            Constraint::Min(8),
                        ],
                    )
                    .header(
                        Row::new(["Order", "Value", "Std. error", "Relative"])
                            .style(self.color.foreground(GOLD).add_modifier(Modifier::BOLD)),
                    )
                    .block(panel(
                        &format!("Laurent coefficients · {:?}", snapshot.uncertainty),
                        self.color,
                    ))
                    .column_spacing(2),
                    chunks[2],
                );
                if !self.integration_workers.is_empty() {
                    let visible = chunks[3].height.saturating_sub(3) as usize;
                    let offset = self.integration_worker_offset.get()
                        .min(self.integration_workers.len().saturating_sub(visible));
                    let rows = self.integration_workers.iter().skip(offset).take(visible).map(|worker| {
                        Row::new(vec![
                            (worker.worker + 1).to_string(),
                            worker.batch.to_string(),
                            format!("{} / {}", worker.completed_points, worker.planned_points),
                            worker.sector.map_or_else(|| "—".into(), |id| id.to_string()),
                            if worker.completed_points == worker.planned_points { "Awaiting admission" }
                            else if worker.preparing_context { "Preparing evaluator" } else { "Evaluating" }.into(),
                        ]).style(self.color.foreground(if worker.preparing_context { GOLD } else { TEAL }))
                    });
                    frame.render_widget(Table::new(rows, [
                        Constraint::Length(7), Constraint::Length(8), Constraint::Percentage(30),
                        Constraint::Length(8), Constraint::Min(20),
                    ])
                    .header(Row::new(["Worker", "Batch", "Points", "Sector", "Activity"]).style(self.color.foreground(GOLD)))
                    .block(panel("In-flight batches · not yet accepted · ↑/↓ scroll", self.color)), chunks[3]);
                } else {
                let rows = snapshot.sectors.iter().map(|sector| {
                    Row::new(vec![
                        sector.id.to_string(),
                        sector.dimension.to_string(),
                        format!("{} / {}", sector.completed_points, sector.planned_points.map_or_else(|| "as allocated".into(), |points| points.to_string())),
                        format!("{} / {}", sector.complete_replicas, sector.planned_replicas),
                        format!("{:.2}", sector.worker_seconds),
                    ])
                });
                frame.render_widget(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(8),
                            Constraint::Length(6),
                            Constraint::Percentage(40),
                            Constraint::Percentage(22),
                            Constraint::Min(8),
                        ],
                    )
                    .header(
                        Row::new(["Sector", "Dim.", "Points", "Replicas", "CPU s"])
                            .style(self.color.foreground(GOLD)),
                    )
                    .block(panel("Sector coverage", self.color))
                    .column_spacing(2),
                    chunks[3],
                );
                }
                frame.render_widget(
                    Paragraph::new(format!(
                        "  Accepted worker time {:.2} s · recorded checks {} · rescues {} · max {} bits{}\n  {}\n  Ctrl-C / q / Esc stops{} · second Ctrl-C exits immediately",
                        snapshot.worker_seconds,
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.conditioning_checks),
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.rescues),
                        snapshot.evaluation_diagnostics.as_ref().map_or(53, |d|d.max_precision_bits),
                        if activity.is_empty() { "" } else { " · in-flight diagnostics pending" },
                        if self.interrupt.flag.load(Ordering::Relaxed) { "Cancelling: waiting for current evaluations to return".to_owned() }
                        else if activity.is_empty() { format!("Weighted checks {} · additional replays {}",
                            snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.weighted_checks),
                            snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.additional_replays)) }
                        else { activity.clone() },
                        if matches!(snapshot.method, fastsecdec::status::IntegrationMethod::HavanaMc | fastsecdec::status::IntegrationMethod::HavanaDiscreteMc)
                            && snapshot.stage == fastsecdec::status::IntegrationStage::Pilot {
                            "; restart this MC pilot to continue"
                        } else {" and saves a checkpoint"}
                    ))
                    .style(self.color.foreground(Color::DarkGray)),
                    chunks[4],
                );
            })?;
        } else {
            eprintln!("{} · {snapshot}", self.scope);
            if !activity.is_empty() {
                eprintln!("  {activity}");
            }
        }
        Ok(())
    }

    pub(crate) fn request_cancel(&self) {
        self.interrupt.flag.store(true, Ordering::Relaxed);
    }

    pub(crate) fn cancellation_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt.flag)
    }

    pub fn cancelled(&self) -> bool {
        self.interrupt.poll_interrupt();
        // Keep reading even after cancellation: the coordinator can be waiting
        // for a native point evaluation, and a second Ctrl-C must still work.
        for _ in 0..32 {
            if !self.interrupt.terminal_active() || !event::poll(Duration::ZERO).unwrap_or(false) {
                break;
            }
            if let Ok(Event::Key(key)) = event::read()
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Up | KeyCode::PageUp => {
                        let offset = if self.integration_workers.is_empty() {
                            &self.generation_worker_offset
                        } else {
                            &self.integration_worker_offset
                        };
                        offset.set(offset.get().saturating_sub(1));
                    }
                    KeyCode::Down | KeyCode::PageDown => {
                        let (offset, maximum) = if self.integration_workers.is_empty() {
                            (
                                &self.generation_worker_offset,
                                self.generation_workers
                                    .as_ref()
                                    .map_or(0, |work| work.workers.len().saturating_sub(1)),
                            )
                        } else {
                            (
                                &self.integration_worker_offset,
                                self.integration_workers.len().saturating_sub(1),
                            )
                        };
                        offset.set(offset.get().saturating_add(1).min(maximum));
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.interrupt.keyboard_interrupt()
                    }
                    KeyCode::Esc | KeyCode::Char('q') => self.request_cancel(),
                    _ => (),
                }
            }
        }
        self.interrupt.flag.load(Ordering::Relaxed)
    }
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        self.interrupt.restore();
    }
}

fn panel(title: &str, color: ColorPolicy) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(color.foreground(Color::Rgb(80, 125, 169)))
}
fn title(stage: &str, color: ColorPolicy) -> Paragraph<'_> {
    Paragraph::new(Line::from(vec![
        Span::styled(
            " FastSecDec ",
            color.foreground(TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" · {stage}")),
    ]))
    .block(panel("", color))
}

/// Reuse the public status display when panels would crowd out their contents.
fn compact(frame: &mut ratatui::Frame<'_>, stage: &str, status: String, color: ColorPolicy) {
    let mut lines = vec![Line::from(vec![
        Span::styled(
            "FastSecDec",
            color.foreground(TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" · {stage}")),
    ])];
    lines.extend(status.lines().map(|line| Line::raw(line.to_owned())));
    lines.push(Line::raw(
        "Ctrl-C / q / Esc cancels; second Ctrl-C forces exit",
    ));
    frame.render_widget(
        Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }),
        frame.area(),
    );
}

#[derive(serde::Serialize)]
struct ScopedStatus<'a> {
    #[serde(flatten)]
    snapshot: &'a IntegrationSnapshot,
    scope: &'a fastsecdec::results::ResultScope,
    #[serde(skip_serializing_if = "<[IntegrationWorkerActivity]>::is_empty")]
    in_flight_unaccepted: &'a [IntegrationWorkerActivity],
}
