mod mc;
mod qmc;
use super::{IntegrationReport, checkpoint::restore_checkpoint};
use crate::{CliResult, artifact::Artifact, config::IntegrationInput, display::Dashboard};
use fastsecdec::{
    integration::{IntegrationProblem, SectorSpec, Tolerance},
    kernel::{KernelSet, SectorKernel},
    status::EvaluationDiagnostics,
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
    restored: Option<(usize, EvaluationDiagnostics, Vec<u8>)>,
    diagnostics: EvaluationDiagnostics,
}

fn evaluate_tracked(
    kernel: &mut SectorKernel,
    point: &[f64],
    output: &mut [f64],
    diagnostics: &mut EvaluationDiagnostics,
) -> Result<(), String> {
    match kernel.evaluate_with_diagnostics(point, output) {
        Ok(report) => diagnostics
            .record(report)
            .map_err(|error| error.to_string()),
        Err(error) => {
            diagnostics
                .record_failure()
                .map_err(|error| error.to_string())?;
            Err(error.to_string())
        }
    }
}

pub(super) fn problem(artifact: &Artifact, kernels: &KernelSet) -> CliResult<IntegrationProblem> {
    Ok(IntegrationProblem::new_with_components(
        artifact.content_id.clone(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(id, kernel)| SectorSpec {
                id: id as u64,
                dimension: kernel.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )?)
}

fn cloned_kernels(kernels: &KernelSet) -> CliResult<Vec<SectorKernel>> {
    Ok(kernels
        .sectors()
        .iter()
        .map(SectorKernel::try_clone)
        .collect::<Result<Vec<_>, _>>()?)
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
    let problem = problem(artifact, kernels)?;
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
        .map(|(_, diagnostics, _)| diagnostics.clone())
        .unwrap_or_default();
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
    };
    match method.as_str() {
        "mc" | "adaptive_mc" => mc::run(context, &method),
        "qmc" | "adaptive_qmc" => qmc::run(context, &method),
        _ => Err("integration method must be qmc, adaptive_qmc, mc or adaptive_mc".into()),
    }
}
