//! Terminal rendering consumes public status snapshots, never computation state.
mod diagnostic_summary;
mod generation_view;
pub(crate) use diagnostic_summary::diagnostic_summary;
mod integration_activity;
mod integration_view;
mod loading_view;
mod memory;
pub(crate) mod number;
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
    style::{Color, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{CliResult, terminal_policy::ColorPolicy};

const TEAL: Color = Color::Rgb(68, 210, 188);
const GOLD: Color = Color::Rgb(243, 195, 91);

pub struct Dashboard {
    interrupt: terminal::Control,
    terminal: Option<Terminal<CrosstermBackend<io::Stderr>>>,
    json_status: bool,
    scope: fastsecdec::results::ResultScope,
    integration_cadence: crate::status_policy::StatusCadence,
    generation_cadence: crate::status_policy::StatusCadence,
    loading_cadence: crate::status_policy::StatusCadence,
    generation_boundary: Option<crate::status_policy::GenerationBoundary>,
    color: ColorPolicy,
    generation_workers: Option<crate::generate::dispatch::Progress>,
    generation_plan: generation_view::Plan,
    generation_worker_offset: std::cell::Cell<usize>,
    integration_workers: Vec<IntegrationWorkerActivity>,
    cached_integration: Option<integration_view::Cached>,
    previous_completed: Option<integration_view::CompletedAllocation>,
    cached_generation: Option<GenerationSnapshot>,
    cached_loading: Option<loading_view::Cached>,
    generation_observed_at: Option<Instant>,
    integration_view: integration_view::View,
    live: Option<fastsecdec::integration::LiveObservation>,
    operational: fastsecdec::integration::OperationalMetrics,
    stability_mode: fastsecdec::kernel::StabilityMode,
    memory: memory::Monitor,
}

impl Dashboard {
    pub fn new(enabled: bool, json_status: bool) -> CliResult<Self> {
        Self::with_status_interval(enabled, json_status, 1000)
    }

    pub fn with_status_interval(
        enabled: bool,
        json_status: bool,
        interval_ms: u64,
    ) -> CliResult<Self> {
        let interrupt = terminal::Control::new()?;
        let terminal = if enabled && !json_status && io::stderr().is_terminal() {
            interrupt.enter_terminal()?;
            Some(Terminal::new(CrosstermBackend::new(io::stderr()))?)
        } else {
            None
        };
        let interval = Duration::from_millis(interval_ms);
        Ok(Self {
            interrupt,
            terminal,
            json_status,
            scope: Default::default(),
            integration_cadence: crate::status_policy::StatusCadence::new(interval),
            generation_cadence: crate::status_policy::StatusCadence::new(interval),
            loading_cadence: crate::status_policy::StatusCadence::new(interval),
            generation_boundary: None,
            color: ColorPolicy::for_stream(false, io::stderr().is_terminal()),
            generation_workers: None,
            generation_plan: Default::default(),
            generation_worker_offset: std::cell::Cell::new(0),
            integration_workers: Vec::new(),
            cached_integration: None,
            previous_completed: None,
            cached_generation: None,
            cached_loading: None,
            generation_observed_at: None,
            integration_view: Default::default(),
            live: None,
            operational: Default::default(),
            stability_mode: Default::default(),
            memory: memory::Monitor::new(),
        })
    }

    pub(crate) fn generation_workers(&mut self, progress: &crate::generate::dispatch::Progress) {
        self.generation_workers = Some(progress.clone());
    }

    pub(crate) fn generation_coordinator(&mut self) {
        self.generation_workers = None;
    }

    pub(crate) fn configure_generation(
        &mut self,
        mode: fastsecdec::generation::GenerationMode,
        method: fastsecdec::generation::CoefficientExpansionMethod,
    ) {
        self.generation_plan.configure(mode, method);
        self.generation_boundary = None;
    }

    pub(crate) fn generation_saving(&mut self, elapsed_seconds: f64) {
        self.generation_plan.saving(elapsed_seconds);
        self.generation_boundary = None;
    }

    pub fn generation(&mut self, snapshot: &GenerationSnapshot) -> CliResult<()> {
        self.generation_plan
            .observe(snapshot.stage, snapshot.elapsed_seconds);
        if let Some(formulas) = &snapshot.formula_preparation {
            self.generation_plan.formula_count(formulas.total);
        }
        self.cached_generation = Some(snapshot.clone());
        self.generation_observed_at = Some(Instant::now());
        let boundary = crate::status_policy::GenerationBoundary::from(snapshot);
        let force = self.generation_boundary != Some(boundary)
            || snapshot.stage == fastsecdec::status::GenerationStage::Complete;
        self.generation_boundary = Some(boundary);
        if self
            .generation_cadence
            .due(Duration::from_secs_f64(snapshot.elapsed_seconds), force)
        {
            self.render_generation(snapshot)?;
        }
        Ok(())
    }

    fn render_generation(&mut self, snapshot: &GenerationSnapshot) -> CliResult<()> {
        if !self.interrupt.terminal_active() {
            self.terminal = None;
        }
        let memory = self.memory.sample();
        if self.json_status {
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
            terminal.draw(|frame| {
                generation_view::render(
                    frame,
                    snapshot,
                    self.generation_workers.as_ref(),
                    &memory,
                    &self.generation_plan,
                    self.generation_worker_offset.get(),
                    self.color,
                );
            })?;
        } else {
            eprintln!(
                "{snapshot}\n  {} · {}",
                memory.process_line(),
                memory.system_line()
            );
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
    pub fn integration_interval(&self) -> Duration {
        self.integration_cadence.interval()
    }

    pub fn integration_due(&mut self, elapsed: Duration, force: bool) -> bool {
        self.integration_cadence.due(elapsed, force)
    }

    pub fn begin_integration(&mut self) {
        self.memory.begin_integration();
        self.previous_completed = None;
        self.cached_integration = None;
        self.cached_generation = None;
        self.cached_loading = None;
    }

    pub(crate) fn begin_loading(&mut self) {
        self.memory.begin_integration();
        self.cached_integration = None;
        self.cached_generation = None;
        self.cached_loading = None;
        self.loading_cadence =
            crate::status_policy::StatusCadence::new(self.integration_cadence.interval());
    }

    pub(crate) fn loading(&mut self, snapshot: &crate::loading::Snapshot) -> CliResult<()> {
        let cancelled = self.interrupt.flag.load(Ordering::Relaxed);
        let force = self.cached_loading.as_ref().is_none_or(|cached| {
            cached.snapshot.phase != snapshot.phase || cached.cancelled != cancelled
        });
        if !self
            .loading_cadence
            .due(Duration::from_secs_f64(snapshot.elapsed_seconds), force)
        {
            return Ok(());
        }
        if !self.interrupt.terminal_active() {
            self.terminal = None;
        }
        let cached = loading_view::Cached {
            snapshot: snapshot.clone(),
            memory: self.memory.sample(),
            observed_at: Instant::now(),
            cancelled,
        };
        if self.json_status {
            #[derive(serde::Serialize)]
            struct LoadingStatus<'a> {
                kind: &'static str,
                #[serde(flatten)]
                snapshot: &'a crate::loading::Snapshot,
                memory: memory::Snapshot,
                cancellation_requested: bool,
            }
            eprintln!(
                "{}",
                serde_json::to_string(&LoadingStatus {
                    kind: "artifact_loading",
                    snapshot,
                    memory: cached.memory,
                    cancellation_requested: cancelled,
                })?
            );
        } else if self.terminal.is_none() {
            eprintln!("{}", loading_view::plain(&cached));
        }
        self.cached_loading = Some(cached);
        self.redraw_cached()
    }

    pub fn set_stability_mode(&mut self, mode: fastsecdec::kernel::StabilityMode) {
        self.stability_mode = mode;
    }

    pub fn set_target_order(&mut self, order: Option<i32>) {
        self.integration_view.order = order;
    }

    pub fn set_live_observation(&mut self, live: Option<fastsecdec::integration::LiveObservation>) {
        self.live = live;
    }

    pub fn set_operational(&mut self, operational: fastsecdec::integration::OperationalMetrics) {
        self.operational = operational;
    }

    pub fn process_cpu_seconds(&mut self) -> Option<f64> {
        self.memory.finish().process_cpu_seconds
    }

    pub fn integration_observation(
        &mut self,
        observation: &fastsecdec::integration::IntegrationObservation,
        elapsed: f64,
    ) -> CliResult<()> {
        if !self.interrupt.terminal_active() {
            self.terminal = None;
        }
        let memory = if observation.snapshot.stop_reason.is_some() {
            self.memory.finish()
        } else {
            self.memory.sample()
        };
        integration_view::CompletedAllocation::update(
            &mut self.previous_completed,
            observation,
            &self.scope,
        );
        let cached = integration_view::Cached {
            previous_completed: self.previous_completed.clone(),
            observation: observation.clone(),
            workers: self.integration_workers.clone(),
            live: self.live.clone(),
            operational: self.operational.clone(),
            stability_mode: self.stability_mode,
            memory,
            elapsed,
            scope: self.scope.clone(),
        };
        if self.json_status {
            eprintln!(
                "{}",
                serde_json::to_string(&ScopedStatus {
                    snapshot: &observation.snapshot,
                    elapsed_seconds: elapsed,
                    scope: &self.scope,
                    in_flight_unaccepted: &self.integration_workers,
                    contributions: &observation.contributions,
                    live: self.live.as_ref(),
                    operational: &self.operational,
                    memory,
                })?
            );
        } else if self.terminal.is_none() {
            eprintln!(
                "{}",
                integration_view::plain(&cached, &self.integration_view)
            );
        }
        self.cached_integration = Some(cached);
        self.redraw_cached()
    }

    fn redraw_cached(&mut self) -> CliResult<()> {
        if let (Some(terminal), Some(cached)) = (&mut self.terminal, &self.cached_integration) {
            let _output = io::stderr().lock();
            if !self.interrupt.terminal_active() {
                return Ok(());
            }
            terminal.draw(|frame| {
                integration_view::render(
                    frame,
                    cached,
                    &mut self.integration_view,
                    self.color,
                    self.interrupt.flag.load(Ordering::Relaxed),
                )
            })?;
        } else if let (Some(terminal), Some(cached)) = (&mut self.terminal, &self.cached_loading) {
            let _output = io::stderr().lock();
            if !self.interrupt.terminal_active() {
                return Ok(());
            }
            terminal.draw(|frame| {
                loading_view::render(
                    frame,
                    cached,
                    self.color,
                    self.interrupt.flag.load(Ordering::Relaxed),
                )
            })?;
        } else if self.terminal.is_some()
            && let Some(mut snapshot) = self.cached_generation.clone()
        {
            if snapshot.stage != fastsecdec::status::GenerationStage::Complete
                && let Some(observed_at) = self.generation_observed_at
            {
                snapshot.elapsed_seconds += observed_at.elapsed().as_secs_f64();
            }
            self.render_generation(&snapshot)?;
        }
        Ok(())
    }

    pub(crate) fn request_cancel(&self) {
        self.interrupt.flag.store(true, Ordering::Relaxed);
    }

    pub(crate) fn cancellation_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt.flag)
    }

    pub fn cancelled(&mut self) -> bool {
        self.interrupt.poll_interrupt();
        let mut redraw = false;
        // Input polling stays independent of the observation/reduction cadence.
        for _ in 0..32 {
            if !self.interrupt.terminal_active() || !event::poll(Duration::ZERO).unwrap_or(false) {
                break;
            }
            match event::read() {
                Ok(Event::Resize(_, _)) => redraw = true,
                Ok(Event::Mouse(mouse)) => {
                    if let Some(cached) = &self.cached_integration {
                        redraw |= self.integration_view.mouse(mouse, cached);
                    }
                }
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.interrupt.keyboard_interrupt();
                        redraw = true;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.request_cancel();
                        redraw = true;
                    }
                    code => {
                        if let Some(cached) = &self.cached_integration {
                            redraw |= self.integration_view.key(code, cached);
                        } else {
                            let offset = &self.generation_worker_offset;
                            match code {
                                KeyCode::Up | KeyCode::PageUp => {
                                    offset.set(offset.get().saturating_sub(1));
                                    redraw = true;
                                }
                                KeyCode::Down | KeyCode::PageDown => {
                                    offset.set(
                                        (offset.get() + 1).min(
                                            self.generation_workers
                                                .as_ref()
                                                .map_or(0, |w| w.workers.len().saturating_sub(1)),
                                        ),
                                    );
                                    redraw = true;
                                }
                                _ => (),
                            }
                        }
                    }
                },
                _ => (),
            }
        }
        if redraw && self.redraw_cached().is_err() {
            self.request_cancel();
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

#[derive(serde::Serialize)]
struct ScopedStatus<'a> {
    elapsed_seconds: f64,
    #[serde(flatten)]
    snapshot: &'a IntegrationSnapshot,
    scope: &'a fastsecdec::results::ResultScope,
    #[serde(skip_serializing_if = "<[IntegrationWorkerActivity]>::is_empty")]
    in_flight_unaccepted: &'a [IntegrationWorkerActivity],
    contributions: &'a fastsecdec::integration::ContributionReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    live: Option<&'a fastsecdec::integration::LiveObservation>,
    operational: &'a fastsecdec::integration::OperationalMetrics,
    memory: memory::Snapshot,
}
