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
    ContourValidation,
    Complete,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Snapshot {
    pub phase: Phase,
    pub completed: Option<usize>,
    pub total: Option<usize>,
    pub elapsed_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_evaluators: Option<PrimaryEvaluators>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PrimaryEvaluators {
    pub cache_restored: usize,
    pub rebuilt_missing_cache: usize,
    pub rebuilt_incompatible_cache: usize,
    pub eager: usize,
}
impl PrimaryEvaluators {
    fn from_kernels(kernels: &KernelSet) -> Self {
        use fastsecdec::kernel::PrimaryEvaluatorRestoration::*;
        let mut result = Self::default();
        for sector in kernels.sectors() {
            match sector.primary_evaluator_restoration() {
                CacheRestored => result.cache_restored += 1,
                CacheMissing | Generated => result.rebuilt_missing_cache += 1,
                CacheIncompatible => result.rebuilt_incompatible_cache += 1,
                Eager => result.eager += 1,
            }
        }
        result
    }
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
            primary_evaluators: None,
        }
    }
}
pub(crate) fn load(
    path: &Path,
    options: KernelLoadOptions,
    recipe: Option<fastsecdec::kernel::indexed::ProgramRecipe>,
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
            Artifact::load_recipe_observed(path, options, recipe, preflight, |event| {
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
    let loaded = result
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?
        .pop()
        .ok_or("artifact loader produced no result")?;
    dashboard.loading(&Snapshot {
        phase: Phase::Complete,
        completed: None,
        total: None,
        elapsed_seconds: started.elapsed().as_secs_f64(),
        primary_evaluators: Some(PrimaryEvaluators::from_kernels(&loaded.1)),
    })?;
    Ok(loaded)
}
