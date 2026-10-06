mod discrete_mc;
mod mc;
mod qmc;
use super::{
    IntegrationReport,
    checkpoint::{RestoredCheckpoint, restore_checkpoint},
    replay::AcceptedReplay,
};
use crate::{CliResult, artifact::Artifact, config::IntegrationInput, display::Dashboard};
use fastsecdec::{
    integration::{IntegrationError, IntegrationProblem, Tolerance},
    kernel::{KernelSet, ReplayState, WeightedEvaluationContext},
    status::{EvaluationDiagnostics, IntegrationSnapshot},
};
use std::{path::Path, time::Instant};

struct Context<'a> {
    artifact: &'a Artifact,
    kernels: &'a KernelSet,
    settings: &'a IntegrationInput,
    checkpoint: &'a Path,
    dashboard: &'a mut Dashboard,
    pool: rayon::ThreadPool,
    problem: IntegrationProblem,
    tolerance: Tolerance,
    started: Instant,
    last_checkpoint: Instant,
    restored: Option<RestoredCheckpoint>,
    diagnostics: EvaluationDiagnostics,
    replay: AcceptedReplay,
}

/// Snapshot/reduction is observational and can be expensive for many sectors.
/// Package acceptance and cancellation remain checked by the caller at each
/// completion and bounded wait poll (or each batch for the MC path).
fn observe(
    dashboard: &mut Dashboard,
    diagnostics: &EvaluationDiagnostics,
    started: Instant,
    force: bool,
    snapshot: impl FnOnce() -> Result<IntegrationSnapshot, IntegrationError>,
    failure: &mut Option<String>,
) -> CliResult<()> {
    if dashboard.integration_due(started.elapsed(), force) {
        match snapshot() {
            Ok(snapshot) => dashboard.integration(
                &super::report::with_diagnostics(snapshot, diagnostics),
                started.elapsed().as_secs_f64(),
            )?,
            Err(error) => {
                failure.get_or_insert_with(|| error.to_string());
            }
        }
    }
    Ok(())
}

fn final_report(
    dashboard: &mut Dashboard,
    report: IntegrationReport,
) -> CliResult<IntegrationReport> {
    dashboard.integration_due(
        std::time::Duration::from_secs_f64(report.elapsed_seconds),
        true,
    );
    dashboard.integration(&report.snapshot, report.elapsed_seconds)?;
    Ok(report)
}

fn evaluate_tracked(
    kernel: &mut WeightedEvaluationContext,
    point: &[f64],
    weight: f64,
    output: &mut [f64],
    diagnostics: &mut EvaluationDiagnostics,
) -> Result<(), String> {
    match kernel.evaluate_weighted(point, weight, output) {
        Ok(report) => diagnostics
            .record_replay(report)
            .map_err(|error| error.to_string()),
        Err(error) => {
            diagnostics
                .record_failure()
                .map_err(|error| error.to_string())?;
            Err(error.to_string())
        }
    }
}

/// Evaluation failure or rejected numerical submission must not advance the
/// accepted replay envelope. Validate metadata before either accepted update.
pub(super) fn submit_package<T>(
    result: Result<T, IntegrationError>,
    sector: usize,
    state: Option<ReplayState>,
    replay: &mut AcceptedReplay,
    submit: impl FnOnce(T) -> Result<(), IntegrationError>,
) -> CliResult<()> {
    let result = result?;
    let state = state.ok_or("successful work package has no replay state")?;
    replay.validate_candidate(sector, &state)?;
    submit(result)?;
    replay.accept(sector, &state)
}

pub(super) fn problem(
    artifact: &Artifact,
    kernels: &KernelSet,
    scope: &fastsecdec::results::ResultScope,
) -> CliResult<IntegrationProblem> {
    Ok(
        fastsecdec::results::KernelResultManifest::from_kernels(kernels).integration_problem(
            scope,
            if kernels.runtime_parameters().is_empty() {
                artifact.content_id.clone()
            } else {
                format!("{}:{}", artifact.content_id, kernels.content_id())
            },
        )?,
    )
}

pub fn integrate(
    artifact: &Artifact,
    kernels: &KernelSet,
    settings: &IntegrationInput,
    checkpoint: &Path,
    resume: bool,
    dashboard: &mut Dashboard,
) -> CliResult<IntegrationReport> {
    if settings.workers == 0 || settings.max_rounds == 0 {
        return Err("workers and max_rounds must be positive".into());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(settings.workers)
        .build()?;
    let problem = problem(artifact, kernels, &settings.scope)?;
    dashboard.set_scope(settings.scope.clone());
    let tolerance = Tolerance::new(settings.absolute_tolerance, settings.relative_tolerance)?;
    let started = Instant::now();
    let last_checkpoint = Instant::now();
    let method = settings.method.replace('-', "_");
    let restored = if resume {
        Some(restore_checkpoint(checkpoint, artifact, settings)?)
    } else {
        None
    };
    let diagnostics = restored
        .as_ref()
        .map(|checkpoint| checkpoint.diagnostics.clone())
        .unwrap_or_default();
    let replay = if let Some(checkpoint) = &restored {
        checkpoint.replay.validate(kernels, &settings.replay)?;
        checkpoint.replay.clone()
    } else {
        AcceptedReplay::new(kernels, settings.replay.clone())?
    };
    let context = Context {
        artifact,
        kernels,
        settings,
        checkpoint,
        dashboard,
        pool,
        problem,
        tolerance,
        started,
        last_checkpoint,
        restored,
        diagnostics,
        replay,
    };
    match method.as_str() {
        "mc" | "adaptive_mc" => mc::run(context, &method),
        "discrete_mc" => discrete_mc::run(context),
        "qmc" | "adaptive_qmc" => qmc::run(context, &method),
        _ => Err(
            "integration method must be qmc, adaptive_qmc, mc, adaptive_mc or discrete_mc".into(),
        ),
    }
}
