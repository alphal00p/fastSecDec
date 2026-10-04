//! Terminal rendering consumes public status snapshots, never computation state.
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
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
};

use crate::CliResult;

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
        })
    }

    pub fn generation(&mut self, snapshot: &GenerationSnapshot) -> CliResult<()> {
        if self.json_status {
            eprintln!("{}", serde_json::to_string(snapshot)?);
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
                let chunks = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(frame.area());
                frame.render_widget(title("Generation"), chunks[0]);
                let rows = [
                    Row::new(vec![
                        "Stage".into(),
                        format!("{:?}", snapshot.stage),
                        "Sectors".into(),
                        snapshot.sectors.to_string(),
                    ]),
                    Row::new(vec![
                        "Elapsed".into(),
                        format!("{:.2} s", snapshot.elapsed_seconds),
                        "Kernels".into(),
                        snapshot.kernels.to_string(),
                    ]),
                ];
                frame.render_widget(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(12),
                            Constraint::Percentage(40),
                            Constraint::Length(12),
                            Constraint::Min(8),
                        ],
                    )
                    .block(panel("Progress"))
                    .column_spacing(2),
                    chunks[1],
                );
                let ratio = snapshot
                    .total
                    .filter(|n| *n > 0)
                    .map_or(0.0, |n| (snapshot.completed as f64 / n as f64).min(1.0));
                let label = snapshot.total.map_or_else(
                    || format!("{} completed", snapshot.completed),
                    |n| format!("{} / {}", snapshot.completed, n),
                );
                frame.render_widget(
                    Gauge::default()
                        .block(panel("Current stage"))
                        .gauge_style(Style::default().fg(TEAL))
                        .ratio(ratio)
                        .label(label),
                    chunks[2],
                );
                frame.render_widget(
                    Paragraph::new(snapshot.detail.clone())
                        .block(panel("Activity"))
                        .wrap(ratatui::widgets::Wrap { trim: true }),
                    chunks[3],
                );
                frame.render_widget(
                    Paragraph::new("  q / Esc  cancel safely")
                        .style(Style::default().fg(Color::DarkGray)),
                    chunks[4],
                );
            })?;
        } else if self.last_log.elapsed() >= Duration::from_secs(1)
            || snapshot.stage == fastsecdec::status::GenerationStage::Complete
        {
            eprintln!("{snapshot}");
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
                let chunks = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(6),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(frame.area());
                frame.render_widget(title(&format!("Integration · {}", self.scope)), chunks[0]);
                let ratio = if snapshot.planned_points == 0 {
                    1.0
                } else {
                    (snapshot.completed_points as f64 / snapshot.planned_points as f64).min(1.0)
                };
                frame.render_widget(
                    Gauge::default()
                        .block(panel(&format!(
                            "{:?} · {:?}",
                            snapshot.method, snapshot.stage
                        )))
                        .gauge_style(Style::default().fg(TEAL))
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
                            .style(Style::default().fg(GOLD).add_modifier(Modifier::BOLD)),
                    )
                    .block(panel(&format!(
                        "Laurent coefficients · {:?}",
                        snapshot.uncertainty
                    )))
                    .column_spacing(2),
                    chunks[2],
                );
                let rows = snapshot.sectors.iter().map(|sector| {
                    Row::new(vec![
                        sector.id.to_string(),
                        sector.dimension.to_string(),
                        format!("{} / {}", sector.completed_points, sector.planned_points),
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
                            .style(Style::default().fg(GOLD)),
                    )
                    .block(panel("Sector coverage"))
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
                        if snapshot.method == fastsecdec::status::IntegrationMethod::HavanaMc
                            && snapshot.stage == fastsecdec::status::IntegrationStage::Pilot {
                            "; restart this MC pilot to continue"
                        } else {" and saves a checkpoint"}
                    ))
                    .style(Style::default().fg(Color::DarkGray)),
                    chunks[4],
                );
            })?;
        } else {
            eprintln!("{} · {snapshot}", self.scope);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn request_cancel(&self) {
        self.interrupt.flag.store(true, Ordering::Relaxed);
    }

    pub fn cancelled(&self) -> bool {
        self.interrupt.flag.load(Ordering::Relaxed)
            || (self.terminal.is_some()
                && event::poll(Duration::ZERO).unwrap_or(false)
                && matches!(event::read(),Ok(Event::Key(key)) if matches!(key.code,KeyCode::Esc|KeyCode::Char('q')|KeyCode::Char('c'))))
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

fn panel(title: &str) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
}
fn title(stage: &str) -> Paragraph<'_> {
    Paragraph::new(Line::from(vec![
        Span::styled(
            " FastSecDec ",
            Style::default().fg(TEAL).add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" · {stage}")),
    ]))
    .block(panel(""))
}

#[derive(serde::Serialize)]
struct ScopedStatus<'a> {
    #[serde(flatten)]
    snapshot: &'a IntegrationSnapshot,
    scope: &'a fastsecdec::results::ResultScope,
}
