mod geometry_dispatch;
mod progress;

use progress::{observe_generation, publish_generation};

use std::{cell::RefCell, ops::ControlFlow, path::Path, sync::atomic::Ordering, time::Instant};

use fastsecdec::{
    generation::{self, GenerationContext, GenerationEvent, GenerationOptions, GenerationProgress},
    kernel::KernelSet,
    status::{GenerationSnapshot, GenerationStage},
};

use crate::{
    CliResult,
    artifact::{Artifact, Provenance, dependencies},
    display::Dashboard,
    input,
};

#[cfg(test)]
pub fn generate(
    path: &Path,
    output: &Path,
    dashboard: &mut Dashboard,
    reference: Option<&crate::reference::PreparedReference>,
) -> CliResult<(Artifact, KernelSet)> {
    generate_with_workers(path, output, dashboard, reference, 1)
}

pub fn generate_with_workers(
    path: &Path,
    output: &Path,
    dashboard: &mut Dashboard,
    reference: Option<&crate::reference::PreparedReference>,
    geometry_workers: usize,
) -> CliResult<(Artifact, KernelSet)> {
    if geometry_workers == 0 {
        return Err("geometry workers must be positive".into());
    }
    let started = Instant::now();
    let mut status = GenerationSnapshot {
        stage: GenerationStage::Input,
        completed: 0,
        total: None,
        sectors: 0,
        kernels: 0,
        elapsed_seconds: 0.0,
        timings: Default::default(),
        coefficient_expansion: None,
        detail: format!("Reading {}", path.display()),
    };
    dashboard.generation(&status)?;
    let loaded = input::load(path)?;
    status.timings.input_seconds = loaded.input_seconds;
    status.timings.parametrization_seconds = loaded.parametrization_seconds;
    status.stage = GenerationStage::Parametrization;
    status.completed = 1;
    status.total = Some(1);
    status.detail = format!(
        "{} · {} parameters · {} terms",
        loaded.label,
        loaded.integrand.parameters().len(),
        loaded.integrand.terms().len()
    );
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    dashboard.generation(&status)?;
    let mut options = GenerationOptions {
        max_order: loaded.card.generation.order,
        assume_no_threshold: loaded.card.generation.assume_no_threshold,
        coefficient_expansion: loaded.card.generation.coefficient_expansion.clone(),
        ..GenerationOptions::default()
    };
    options.decomposition.max_sectors = loaded.card.generation.max_sectors;
    options.decomposition.max_support_pairs = loaded.card.generation.max_support_pairs;
    let mut display_error = None;
    let cancelled = dashboard.cancellation_handle();
    let generated = {
        let ui = RefCell::new((&mut *dashboard, &mut status, &mut display_error));
        let mut present = |progress: &GenerationProgress| {
            let mut ui = ui.borrow_mut();
            let (dashboard, status, error) = &mut *ui;
            observe_generation(
                dashboard,
                status,
                error,
                started,
                options.max_order,
                progress,
            )
        };
        if geometry_workers == 1 {
            generation::generate(&loaded.integrand, &options, &mut present)
        } else {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(geometry_workers)
                .build()?;
            let mut stage = 0;
            let mut dispatch = |jobs: &mut dyn ExactSizeIterator<
                Item = generation::GeometryJob,
            >| {
                let label = if stage == 0 { "chart" } else { "cone" };
                stage += 1;
                geometry_dispatch::run(&pool, geometry_workers, jobs, &cancelled, |progress| {
                    let mut ui = ui.borrow_mut();
                    let (dashboard, status, error) = &mut *ui;
                    status.stage = GenerationStage::Geometry;
                    status.completed = progress.completed;
                    status.total = Some(progress.total);
                    status.elapsed_seconds = started.elapsed().as_secs_f64();
                    status.detail = format!(
                        "{geometry_workers} geometry workers · {label} jobs returned ({} running); native admission pending{}",
                        progress.running,
                        progress
                            .latest
                            .as_ref()
                            .map_or_else(String::new, |(id, p)| format!(
                                " · {id:?} {:?} {}/{} local constraints",
                                p.phase, p.completed_constraints, p.total_constraints
                            ))
                    );
                    publish_generation(dashboard, status, error)
                })
            };
            GenerationContext::new(0).generate_with_dispatch(
                &loaded.integrand,
                &options,
                &mut dispatch,
                || cancelled.load(Ordering::Relaxed),
                |event| match event {
                    GenerationEvent::Progress(progress) => present(progress),
                    GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
                },
            )
        }
    };
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let generated = generated?;
    let compilation_started = Instant::now();
    let kernels = generated.compile_with_progress(|progress| {
        status.stage = GenerationStage::Compilation;
        status.completed = progress.completed;
        status.total = Some(progress.total);
        status.kernels = progress.completed;
        status.timings.compilation_seconds = progress.elapsed_seconds;
        status.elapsed_seconds = started.elapsed().as_secs_f64();
        status.detail = "Compiling portable SymJIT O2 kernels".into();
        if let Err(error) = dashboard.generation(&status) {
            display_error = Some(error.to_string());
            return ControlFlow::Break(());
        }
        if dashboard.cancelled() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let kernels = kernels?;
    if let Some(reference) = reference {
        reference.validate_identity(kernels.content_id())?;
    }
    status.timings.compilation_seconds = compilation_started.elapsed().as_secs_f64();
    let provenance = Provenance {
        name: loaded.label,
        sources: loaded.sources,
        dependencies: dependencies(),
        domain: format!("{:?}", loaded.integrand.domain()),
        assume_no_threshold: options.assume_no_threshold,
        dimension: loaded.card.integral.dimension,
        regulator: loaded.card.integral.regulator,
        measure: if loaded.loops.is_some() {
            "prod_l d^D k_l / (i*pi^(D/2)); propagators q^2-m^2+i0; no implicit scale factors"
        } else {
            "user-supplied direct density with the declared domain measure"
        }
        .into(),
        measure_multiplier: loaded.card.integral.measure_multiplier,
        max_order: options.max_order,
        integration: serde_json::to_value(loaded.card.integration)?,
        family_preparation: loaded.family_preparation,
    };
    let mut artifact = Artifact::new(&kernels, provenance)?;
    artifact.reference = reference.map(|value| value.settings.clone());
    status.timings.total_seconds = started.elapsed().as_secs_f64();
    artifact.generation_timings = Some(status.timings.clone());
    artifact.save(output)?;
    status.stage = GenerationStage::Complete;
    status.kernels = kernels.sectors().len();
    status.completed = status.kernels;
    status.total = Some(status.kernels);
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    status.detail = format!("Saved {}", output.display());
    dashboard.generation(&status)?;
    Ok((artifact, kernels))
}
