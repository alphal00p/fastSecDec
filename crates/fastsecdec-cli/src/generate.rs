pub(crate) mod dispatch;
#[cfg(test)]
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
    artifact::{Artifact, GenerationRecord, Provenance, dependencies},
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
    workers: usize,
) -> CliResult<(Artifact, KernelSet)> {
    crate::artifact::paths(output)?;
    if workers == 0 {
        return Err("generation workers must be positive".into());
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
        detail: format!("Reading {}", crate::artifact::relative_display(path)),
    };
    dashboard.generation(&status)?;
    let loaded = input::load(path)?;
    let evaluator_settings = loaded.card.generation.evaluator;
    evaluator_settings.validate()?;
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
        let present = |progress: &GenerationProgress| {
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
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()?;
        let mut geometry_stage = 0;
        let mut geometry_dispatch =
            |jobs: &mut dyn ExactSizeIterator<Item = generation::GeometryJob>| {
                let label = if geometry_stage == 0 {
                    "charts"
                } else {
                    "cones"
                };
                geometry_stage += 1;
                dispatch::run(
                    &pool,
                    jobs,
                    &cancelled,
                    |job| format!("geometry {:?}", job.id()),
                    |job, observe| {
                        let id = job.id();
                        Ok(job.run(|p| {
                            observe(format!(
                                "{id:?} · {:?} · {}/{} constraints",
                                p.phase, p.completed_constraints, p.total_constraints
                            ))
                        }))
                    },
                    |progress| {
                        let mut ui = ui.borrow_mut();
                        let (dashboard, status, error) = &mut *ui;
                        status.stage = GenerationStage::Geometry;
                        status.completed = progress.completed;
                        status.total = Some(progress.total);
                        status.elapsed_seconds = started.elapsed().as_secs_f64();
                        status.detail = format!(
                            "{workers} workers · {label} · {} running · {} queued",
                            progress.running(),
                            progress
                                .total
                                .saturating_sub(progress.completed + progress.running())
                        );
                        dashboard.generation_workers(progress);
                        publish_generation(dashboard, status, error)
                    },
                )
                .map_err(|error| {
                    if error == "generation cancelled" {
                        generation::SectorError::Cancelled
                    } else {
                        generation::SectorError::Geometry(error)
                    }
                })
            };
        let mut symbolic_dispatch =
            |jobs: &mut dyn ExactSizeIterator<Item = generation::SymbolicJob>| {
                // The first job identifies the homogeneous native stage without
                // pre-running or collecting its remaining lazy work.
                let mut jobs = jobs.peekable();
                let stage = match jobs.peek().map(|job| job.id().stage) {
                    Some(generation::SymbolicStage::Symmetry) => GenerationStage::Symmetry,
                    Some(generation::SymbolicStage::Coefficients) => {
                        GenerationStage::CoefficientExpansion
                    }
                    _ => GenerationStage::Mapping,
                };
                dispatch::run(
                    &pool,
                    &mut jobs,
                    &cancelled,
                    |job| match job.id().stage {
                        generation::SymbolicStage::Symmetry => {
                            format!("Finding equivalent sectors · sector {}", job.id().index)
                        }
                        stage => format!("{stage:?} · sector {}", job.id().index),
                    },
                    |job, observe| {
                        let id = job.id();
                        job.run(|progress| {
                            let mut snapshot = GenerationSnapshot {
                                stage,
                                completed: 0,
                                total: None,
                                sectors: 0,
                                kernels: 0,
                                elapsed_seconds: 0.0,
                                timings: Default::default(),
                                coefficient_expansion: None,
                                detail: String::new(),
                            };
                            snapshot.observe_generation(options.max_order, progress);
                            observe(format!("sector {} · {}", id.index, snapshot.detail))
                        })
                        .map_err(|error| error.to_string())
                    },
                    |progress| {
                        let mut ui = ui.borrow_mut();
                        let (dashboard, status, error) = &mut *ui;
                        status.stage = stage;
                        status.completed = progress.completed;
                        status.total = Some(progress.total);
                        status.elapsed_seconds = started.elapsed().as_secs_f64();
                        status.detail = format!(
                            "{workers} workers · {} running · {} queued",
                            progress.running(),
                            progress
                                .total
                                .saturating_sub(progress.completed + progress.running())
                        );
                        if stage == GenerationStage::Symmetry {
                            status.detail = format!("Preparing comparisons · {}", status.detail);
                        }
                        dashboard.generation_workers(progress);
                        publish_generation(dashboard, status, error)
                    },
                )
                .map_err(|error| {
                    if error == "generation cancelled" {
                        generation::GenerationError::Cancelled
                    } else {
                        generation::GenerationError::Invariant(error)
                    }
                })
            };
        GenerationContext::new(0).generate_with_all_dispatch(
            &loaded.integrand,
            &options,
            &mut geometry_dispatch,
            &mut symbolic_dispatch,
            || cancelled.load(Ordering::Relaxed),
            |event| match event {
                GenerationEvent::Progress(progress) => present(progress),
                GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
            },
        )
    };
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let generated = generated?;
    let compilation_started = Instant::now();
    let kernels = {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()?;
        let ui = RefCell::new((&mut *dashboard, &mut status, &mut display_error));
        let mut compile_dispatch =
            |jobs: &mut dyn ExactSizeIterator<Item = fastsecdec::kernel::CompilationJob>| {
                dispatch::run(
                    &pool,
                    jobs,
                    &cancelled,
                    |job| format!("SymJIT O2 · sector {}", job.index()),
                    |job, _| job.run().map_err(|error| error.to_string()),
                    |progress| {
                        let mut ui = ui.borrow_mut();
                        let (dashboard, status, error) = &mut *ui;
                        status.stage = GenerationStage::Compilation;
                        status.completed = progress.completed;
                        status.total = Some(progress.total);
                        status.kernels = progress.completed;
                        status.elapsed_seconds = started.elapsed().as_secs_f64();
                        status.detail = format!(
                            "SymJIT O2 · {workers} workers · {} running",
                            progress.running()
                        );
                        dashboard.generation_workers(progress);
                        publish_generation(dashboard, status, error)
                    },
                )
                .map_err(fastsecdec::kernel::KernelError::Compilation)
            };
        generated.compile_with_settings_parameters_and_dispatch(
            fastsecdec::kernel::PrecisionPolicy::default(),
            &loaded.runtime_parameters,
            evaluator_settings,
            &mut compile_dispatch,
            |progress| {
                let mut ui = ui.borrow_mut();
                let (dashboard, status, error) = &mut *ui;
                status.observe_compilation(progress);
                status.elapsed_seconds = started.elapsed().as_secs_f64();
                publish_generation(dashboard, status, error)
            },
        )
    };
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let kernels = kernels?.with_runtime_mass_constraints(loaded.runtime_mass_constraints)?;
    drop(generated);
    if let Some(reference) = reference.filter(|_| kernels.runtime_parameters().is_empty()) {
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
        model_parameter_defaults: loaded.model_parameter_defaults,
    };
    dashboard.generation_coordinator();
    status.completed = 0;
    status.total = None;
    status.detail = "Preparing portable artifact; all kernels compiled".into();
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    dashboard.generation(&status)?;
    let mut artifact = Artifact::new(&kernels, provenance)?;
    artifact.reference = reference.map(|value| value.settings.clone());
    artifact.relocate_sources(
        path.parent().unwrap_or_else(|| Path::new(".")),
        output.parent().unwrap_or_else(|| Path::new(".")),
    )?;
    artifact.generation = Some(GenerationRecord {
        workers,
        contraction_mode: loaded
            .loops
            .map(|_| loaded.card.generation.contraction_mode),
        requested_coefficient_expansion: options.coefficient_expansion.method,
        evaluator: Some(evaluator_settings),
    });
    status.timings.total_seconds = started.elapsed().as_secs_f64();
    artifact.generation_timings = Some(status.timings.clone());
    status.detail = format!(
        "Saving portable artifact to {}",
        crate::artifact::relative_display(output)
    );
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    dashboard.generation(&status)?;
    artifact.save(output)?;
    status.stage = GenerationStage::Complete;
    status.kernels = kernels.sectors().len();
    status.completed = status.kernels;
    status.total = Some(status.kernels);
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    status.detail = format!("Saved {}", crate::artifact::relative_display(output));
    dashboard.generation(&status)?;
    Ok((artifact, kernels))
}
