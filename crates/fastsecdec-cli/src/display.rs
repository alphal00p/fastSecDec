//! Terminal rendering consumes public status snapshots, never computation state.
mod memory;

use std::{
    io::{self, IsTerminal},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
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
    interrupt: InterruptSignal,
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
        let interrupt = InterruptSignal::new()?;
        let terminal = if enabled && !json_status && io::stderr().is_terminal() {
            terminal::enable_raw_mode()?;
            if let Err(error) = execute!(io::stderr(), EnterAlternateScreen, cursor::Hide) {
                terminal::disable_raw_mode()?;
                return Err(error.into());
            }
            match Terminal::new(CrosstermBackend::new(io::stderr())) {
                Ok(terminal) => Some(terminal),
                Err(error) => {
                    let _ = terminal::disable_raw_mode();
                    let _ = execute!(io::stderr(), LeaveAlternateScreen, cursor::Show);
                    return Err(error.into());
                }
            }
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
                frame.render_widget(Paragraph::new("  q / Esc  cancel safely · ETA covers this stage; later work is discovered dynamically")
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

    /// Call before constructing the native integration snapshot. Forced events
    /// include stage/round boundaries and every final/cancelled/failed report.
    pub fn integration_due(&mut self, elapsed: Duration, force: bool) -> bool {
        self.integration_cadence.due(elapsed, force)
    }

    pub fn integration(&mut self, snapshot: &IntegrationSnapshot, elapsed: f64) -> CliResult<()> {
        if self.json_status {
            eprintln!(
                "{}",
                serde_json::to_string(&ScopedStatus {
                    snapshot,
                    scope: &self.scope
                })?
            );
            return Ok(());
        }
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(|frame| {
                if frame.area().width < 72 || frame.area().height < 22 {
                    compact(
                        frame,
                        "Integration",
                        format!("{} · {elapsed:.2} s\n{snapshot}", self.scope),
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
                            "{} / {} points    {:.2} s",
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
                frame.render_widget(
                    Paragraph::new(format!(
                        "  Worker time {:.2} s · checks {} · rescues {} · max {} bits\n  Weighted checks {} · additional replays {}\n  q / Esc stops{}",
                        snapshot.worker_seconds,
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.conditioning_checks),
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.rescues),
                        snapshot.evaluation_diagnostics.as_ref().map_or(53, |d|d.max_precision_bits),
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.weighted_checks),
                        snapshot.evaluation_diagnostics.as_ref().map_or(0, |d|d.additional_replays),
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
        if self.interrupt.flag.load(Ordering::Relaxed) {
            return true;
        }
        let pressed = if self.terminal.is_some() && event::poll(Duration::ZERO).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => match key.code {
                    KeyCode::Up | KeyCode::PageUp => {
                        self.generation_worker_offset
                            .set(self.generation_worker_offset.get().saturating_sub(1));
                        false
                    }
                    KeyCode::Down | KeyCode::PageDown => {
                        let maximum = self
                            .generation_workers
                            .as_ref()
                            .map_or(0, |work| work.workers.len().saturating_sub(1));
                        self.generation_worker_offset
                            .set((self.generation_worker_offset.get() + 1).min(maximum));
                        false
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => true,
                    _ => false,
                },
                _ => false,
            }
        } else {
            false
        };
        if pressed {
            self.request_cancel();
        }
        pressed
    }
}

struct InterruptSignal {
    flag: Arc<AtomicBool>,
    ids: [signal_hook::SigId; 2],
}
impl InterruptSignal {
    fn new() -> io::Result<Self> {
        let flag = Arc::new(AtomicBool::new(false));
        let force = signal_hook::flag::register_conditional_default(
            signal_hook::consts::SIGINT,
            Arc::clone(&flag),
        )?;
        match signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&flag)) {
            Ok(stop) => Ok(Self {
                flag,
                ids: [force, stop],
            }),
            Err(error) => {
                signal_hook::low_level::unregister(force);
                Err(error)
            }
        }
    }
}
impl Drop for InterruptSignal {
    fn drop(&mut self) {
        for id in self.ids {
            signal_hook::low_level::unregister(id);
        }
    }
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        if self.terminal.is_some() {
            let _ = terminal::disable_raw_mode();
            let _ = execute!(io::stderr(), LeaveAlternateScreen, cursor::Show);
        }
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
    lines.push(Line::raw("q / Esc cancels safely"));
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
}
