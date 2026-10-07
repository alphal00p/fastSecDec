//! CLI-owned background restoration with the existing bounded worker coordinator.
//! No native library loop or pool is introduced.
use crate::{
    CliResult,
    artifact::{Artifact, ArtifactLoadProgress},
    display::Dashboard,
};
use fastsecdec::kernel::{KernelLoadOptions, KernelLoadProgress, KernelSet};
use std::{
    ops::ControlFlow,
    path::Path,
    sync::{Mutex, atomic::Ordering},
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase {
    Metadata,
    ReadingBinary,
    Decoding,
    Restoring,
    Complete,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Snapshot {
    pub phase: Phase,
    pub completed: Option<usize>,
    pub total: Option<usize>,
    pub elapsed_seconds: f64,
}
impl Snapshot {
    fn event(event: &ArtifactLoadProgress) -> Self {
        let (phase, completed, total) = match event {
            ArtifactLoadProgress::Metadata => (Phase::Metadata, None, None),
            ArtifactLoadProgress::ReadingBinary { completed, total } => {
                (Phase::ReadingBinary, Some(*completed), *total)
            }
            ArtifactLoadProgress::Native(KernelLoadProgress::Decoding) => {
                (Phase::Decoding, None, None)
            }
            ArtifactLoadProgress::Native(KernelLoadProgress::Restoring(progress)) => (
                Phase::Restoring,
                Some(progress.completed),
                Some(progress.total),
            ),
            ArtifactLoadProgress::Native(KernelLoadProgress::Complete) => {
                (Phase::Complete, None, None)
            }
        };
        Self {
            phase,
            completed,
            total,
            elapsed_seconds: 0.0,
        }
    }
}
pub(crate) fn load(
    path: &Path,
    options: KernelLoadOptions,
    dashboard: &mut Dashboard,
    preflight: impl FnOnce(&Artifact) -> CliResult<()> + Send,
) -> CliResult<(Artifact, KernelSet)> {
    dashboard.begin_loading();
    let started = Instant::now();
    let latest = Mutex::new(Snapshot::event(&ArtifactLoadProgress::Metadata));
    let cancelled = dashboard.cancellation_handle();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build()?;
    let mut jobs = std::iter::once((path, preflight));
    let mut display_error = None;
    let result = crate::generate::dispatch::run(
        &pool,
        &mut jobs,
        &cancelled,
        |_| "Loading artifact".into(),
        |(path, preflight), _activity| {
            Artifact::load_observed_with_options(path, options, preflight, |event| {
                if cancelled.load(Ordering::Relaxed) {
                    return ControlFlow::Break(());
                }
                *latest.lock().unwrap_or_else(|p| p.into_inner()) = Snapshot::event(event);
                ControlFlow::Continue(())
            })
            .map_err(|error| error.to_string())
        },
        |_| {
            let mut snapshot = latest.lock().unwrap_or_else(|p| p.into_inner()).clone();
            snapshot.elapsed_seconds = started.elapsed().as_secs_f64();
            if let Err(error) = dashboard.loading(&snapshot) {
                display_error = Some(error);
                return ControlFlow::Break(());
            }
            if dashboard.cancelled() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    );
    if let Some(error) = display_error {
        return Err(error);
    }
    if cancelled.load(Ordering::Relaxed)
        && result
            .as_ref()
            .is_err_and(|error| error == "generation cancelled" || error.contains("cancelled"))
    {
        return Err("artifact loading cancelled".into());
    }
    result
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?
        .pop()
        .ok_or_else(|| "artifact loader produced no result".into())
}
