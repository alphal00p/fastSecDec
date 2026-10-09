use super::{Event, Job, Returned, TaskMetrics};
use crate::{CliResult, artifact::atomic_write, process::child};
use fastsecdec::{
    kernel::{KernelLoadOptions, WeightedEvaluationContext, indexed::ProgramArchiveReader},
    status::EvaluationDiagnostics,
};
use std::{
    collections::BTreeMap,
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

struct Active {
    sector: u64,
    data_path: PathBuf,
    catalogue_id: String,
    archive_id: String,
    recipe: fastsecdec::kernel::indexed::ProgramRecipe,
    settings_id: blake3::Hash,
    context: WeightedEvaluationContext,
    output_indices: Vec<usize>,
    local_orders: Vec<i32>,
    local_components: Vec<fastsecdec::status::CoefficientComponent>,
}

pub(crate) fn worker(run: String, lease: u64) -> CliResult<()> {
    let mut active: Option<Active> = None;
    child::serve::<PathBuf, Event>(run, lease, move |path, emit| {
        execute(&mut active, path, emit).map_err(|e| std::io::Error::other(e.to_string()))
    })?;
    Ok(())
}

fn execute(
    active: &mut Option<Active>,
    path: PathBuf,
    emit: &mut dyn FnMut(Event) -> std::io::Result<()>,
) -> CliResult<()> {
    let job: Job = serde_json::from_reader(File::open(path)?)?;
    let settings_id = blake3::hash(&serde_json::to_vec(&(
        &job.parameters,
        &job.policy,
        &job.stability,
        &job.contour,
        job.task.dimension(),
        job.task.output_count(),
    ))?);
    let identity = job.task.identity().clone();
    let mut load_seconds = 0.;
    if active.is_none() {
        let started = Instant::now();
        let mut archive = ProgramArchiveReader::from_reader(
            File::open(&job.data_path)?,
            KernelLoadOptions {
                validate: job.validate_artifact,
            },
        )?;
        if archive.catalogue().content_id != job.archive_id {
            return Err("resident worker archive identity differs".into());
        }
        let mut reader = archive.select(job.recipe)?;
        if reader.catalogue().content_id != job.catalogue_id {
            return Err("resident worker catalogue identity differs".into());
        }
        let descriptor = reader
            .catalogue()
            .sector(usize::try_from(job.task.sector_id())?)?
            .clone();
        let mut kernels = reader.load_sector(usize::try_from(job.task.sector_id())?)?;
        let mut parameters = BTreeMap::new();
        for (name, value) in &job.parameters {
            let symbol = crate::input::symbol(name)?;
            if kernels.runtime_parameters().contains(&symbol) {
                parameters.insert(symbol, *value);
            }
        }
        kernels.bind_parameters_with_contour(&parameters, &job.contour)?;
        let pilot = crate::contour_pilot::run(
            &mut kernels,
            &job.contour,
            job.validation_seed,
            None,
            |progress| {
                emit(Event::ContourPilot {
                    identity: identity.clone(),
                    progress: progress.clone(),
                })?;
                Ok(())
            },
        )?;
        if let Some(report) = pilot {
            emit(Event::ContourPilotComplete {
                identity: identity.clone(),
                report: crate::contour_pilot::provenance(&kernels, job.validation_seed, &report),
            })?;
        }
        kernels.set_stability_settings(&job.stability)?;
        let context = if let Some(replay) = &job.replay {
            kernels.restore_evaluation_context(0, job.policy.clone(), replay)?
        } else {
            kernels.evaluation_context(0, job.policy.clone())?
        };
        if context.dimension() != job.task.dimension()
            || context.output_count() != descriptor.output_indices.len()
            || descriptor
                .output_indices
                .iter()
                .any(|&i| i >= job.task.output_count())
        {
            return Err("resident worker local/global vector mapping differs".into());
        }
        *active = Some(Active {
            sector: job.task.sector_id(),
            data_path: job.data_path.clone(),
            catalogue_id: job.catalogue_id.clone(),
            archive_id: job.archive_id.clone(),
            recipe: job.recipe,
            settings_id,
            context,
            output_indices: descriptor.output_indices,
            local_orders: descriptor.receipt.orders,
            local_components: descriptor.receipt.components,
        });
        // kernels/reader drop here; the resident context is the only evaluator owner.
        load_seconds = started.elapsed().as_secs_f64();
        emit(Event::Loaded {
            identity: identity.clone(),
            seconds: load_seconds,
        })?;
    }
    let active = active.as_mut().unwrap();
    if active.sector != job.task.sector_id()
        || active.data_path != job.data_path
        || active.catalogue_id != job.catalogue_id
        || active.archive_id != job.archive_id
        || active.recipe != job.recipe
        || active.settings_id != settings_id
    {
        return Err("a resident integration process cannot switch sector; evict it first".into());
    }
    if let Some(replay) = &job.replay {
        active.context.merge_state(replay)?;
        active.context.set_reference_state(replay)?;
    }
    let mut diagnostics = EvaluationDiagnostics::default();
    let mut completed = 0u64;
    let mut publication = Instant::now();
    let mut local = Vec::new();
    let mut integrand_seconds = 0.;
    let mut maxima = BTreeMap::<i32, f64>::new();
    let sampling_started = Instant::now();
    let value = job.task.evaluate_weighted_batch(
        job.batch_size,
        |points, weights, output| -> CliResult<()> {
            let started = Instant::now();
            let local_width = active.output_indices.len();
            local.resize(
                weights
                    .len()
                    .checked_mul(local_width)
                    .ok_or("local evaluator matrix overflow")?,
                f64::NAN,
            );
            let evaluated = active
                .context
                .evaluate_weighted_batch(points, weights, &mut local);
            if let Some(delta) = active.context.take_contour_validation_report() {
                diagnostics.record_contour(
                    if identity.pilot {
                        fastsecdec::status::IntegrationStage::Pilot
                    } else {
                        fastsecdec::status::IntegrationStage::Production
                    },
                    &delta,
                )?;
            }
            let reports = match evaluated {
                Ok(reports) => reports,
                Err(error) => {
                    integrand_seconds += started.elapsed().as_secs_f64();
                    // A rejected allocation contributes no estimate, but its
                    // successfully checked prefix remains observational data.
                    emit(Event::Progress {
                        identity: identity.clone(),
                        completed,
                        planned: job.task.point_count(),
                        sampling_seconds: sampling_started.elapsed().as_secs_f64(),
                        metrics: Box::new(TaskMetrics {
                            diagnostics: diagnostics.clone(),
                            load_seconds,
                            worker_seconds: sampling_started.elapsed().as_secs_f64(),
                            integrand_seconds,
                            maxima: maxima.clone(),
                        }),
                    })?;
                    return Err(error.error.into());
                }
            };
            for report in reports {
                diagnostics.record_replay(report)?;
            }
            output.fill(0.);
            for (source, target) in local
                .chunks_exact(local_width)
                .zip(output.chunks_exact_mut(job.task.output_count()))
            {
                for (&value, &index) in source.iter().zip(&active.output_indices) {
                    target[index] = value;
                }
                let mut row = BTreeMap::<i32, f64>::new();
                for ((&value, &order), _component) in source
                    .iter()
                    .zip(&active.local_orders)
                    .zip(&active.local_components)
                {
                    let old = row.entry(order).or_default();
                    *old = old.hypot(value);
                }
                for (order, value) in row {
                    maxima
                        .entry(order)
                        .and_modify(|v| *v = v.max(value))
                        .or_insert(value);
                }
            }
            integrand_seconds += started.elapsed().as_secs_f64();
            completed = completed
                .checked_add(weights.len() as u64)
                .ok_or("worker point count overflow")?;
            if completed == job.task.point_count()
                || publication.elapsed() >= Duration::from_millis(100)
            {
                emit(Event::Progress {
                    identity: identity.clone(),
                    completed,
                    planned: job.task.point_count(),
                    sampling_seconds: sampling_started.elapsed().as_secs_f64(),
                    metrics: Box::new(TaskMetrics {
                        diagnostics: diagnostics.clone(),
                        load_seconds,
                        worker_seconds: sampling_started.elapsed().as_secs_f64(),
                        integrand_seconds,
                        maxima: maxima.clone(),
                    }),
                })?;
                publication = Instant::now();
            }
            Ok(())
        },
    )?;
    let metrics = TaskMetrics {
        worker_seconds: value.worker_seconds(),
        diagnostics,
        load_seconds,
        integrand_seconds,
        maxima,
    };
    let returned = Returned {
        value,
        replay: active.context.state().clone(),
        metrics,
    };
    atomic_write(&job.return_path, &serde_json::to_vec(&returned)?)?;
    emit(Event::Completed {
        identity,
        path: job.return_path,
    })?;
    Ok(())
}
