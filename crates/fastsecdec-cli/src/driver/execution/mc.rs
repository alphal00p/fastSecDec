use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_mc_checkpoint,
    refinement::mc_points,
    report::{stopped, with_diagnostics},
};
use super::{Context, cloned_kernels, evaluate_tracked};
use crate::CliResult;
use fastsecdec::{
    integration::mc::{HavanaSession, HavanaSettings, HavanaWorker},
    kernel::SectorKernel,
    status::{EvaluationDiagnostics, IntegrationStage},
};
use rayon::prelude::*;
use std::{collections::BTreeMap, time::Instant};

struct McSlot {
    kernels: Vec<SectorKernel>,
    workers: BTreeMap<u64, HavanaWorker>,
}

pub(super) fn run(context: Context<'_>, method: &str) -> CliResult<IntegrationReport> {
    let Context {
        artifact,
        kernels,
        settings,
        checkpoint,
        dashboard,
        pool,
        problem,
        tolerance,
        started,
        mut last_checkpoint,
        restored,
        mut diagnostics,
    } = context;
    let options = HavanaSettings {
        points_per_batch: settings.points.try_into()?,
        batches: settings.shifts,
        seed: settings.seed,
        ..HavanaSettings::default()
    };
    let mut session = if let Some((_, _, bytes)) = &restored {
        HavanaSession::restore(bytes, &problem)?
    } else if method == "adaptive_mc" {
        HavanaSession::pilot(problem.clone(), options.clone())?
    } else {
        HavanaSession::production(problem.clone(), options.clone())?
    };
    let mut cancelled = false;
    let mut round = restored.as_ref().map_or(0, |(round, _, _)| *round);
    let mut current_points = mc_points(settings.points, round)?;
    loop {
        let mut slots = (0..settings.workers)
            .map(|_| {
                Ok(McSlot {
                    kernels: cloned_kernels(kernels)?,
                    workers: session
                        .problem()
                        .sectors
                        .iter()
                        .map(|sector| Ok((sector.id, session.worker_context(sector.id)?)))
                        .collect::<CliResult<_>>()?,
                })
            })
            .collect::<CliResult<Vec<_>>>()?;
        while !session.is_complete() {
            let tasks = (0..settings.workers)
                .filter_map(|_| session.next_work())
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("MC scheduler has no work before completion".into());
            }
            let returns = pool.install(|| {
                slots
                    .par_iter_mut()
                    .zip(tasks.into_par_iter())
                    .map(|(slot, task)| {
                        let id = task.sector_id();
                        let kernel = &mut slot.kernels[id as usize];
                        let mut local = EvaluationDiagnostics::default();
                        let result = slot
                            .workers
                            .get_mut(&id)
                            .unwrap()
                            .evaluate(task, |point, output| {
                                evaluate_tracked(kernel, point, output, &mut local)
                            });
                        (result, local)
                    })
                    .collect::<Vec<_>>()
            });
            let mut failure = None;
            for (result, local) in returns {
                diagnostics.merge(&local)?;
                match result {
                    Ok(result) => session.submit(result)?,
                    Err(error) => {
                        failure.get_or_insert(error);
                    }
                }
            }
            dashboard.integration(
                &with_diagnostics(session.snapshot()?, &diagnostics),
                started.elapsed().as_secs_f64(),
            )?;
            if let Some(error) = failure {
                save_mc_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    &session,
                    &diagnostics,
                )?;
                return Err(error.into());
            }
            if last_checkpoint.elapsed().as_secs() >= 5 {
                save_mc_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    &session,
                    &diagnostics,
                )?;
                last_checkpoint = Instant::now();
            }
            if dashboard.cancelled() {
                cancelled = true;
                break;
            }
        }
        if cancelled {
            break;
        }
        if session.stage() == IntegrationStage::Pilot {
            session.freeze_production(0.5, current_points.try_into()?, settings.shifts)?;
            continue;
        }
        if session.estimate()?.meets(tolerance)? || round + 1 >= settings.max_rounds {
            break;
        }
        round += 1;
        current_points = mc_points(settings.points, round)?;
        let mut next = options.clone();
        next.points_per_batch = current_points.try_into()?;
        session = if method == "adaptive_mc" {
            HavanaSession::pilot(problem.clone(), next)?
        } else {
            HavanaSession::production(problem.clone(), next)?
        };
    }
    let resume_status = save_mc_checkpoint(
        checkpoint,
        artifact,
        settings,
        round,
        &session,
        &diagnostics,
    )?;
    let snapshot = with_diagnostics(session.snapshot()?, &diagnostics);
    let estimate = snapshot.estimate.clone();
    let converged = !cancelled
        && estimate
            .as_ref()
            .is_some_and(|estimate| estimate.meets(tolerance).unwrap_or(false));
    Ok(IntegrationReport {
        content_id: artifact.content_id.clone(),
        elapsed_seconds: started.elapsed().as_secs_f64(),
        loading_seconds: artifact.loading_seconds,
        generation_timings: artifact.generation_timings.clone(),
        converged,
        stopping_reason: if cancelled && resume_status == ResumeStatus::PilotRestartRequired {
            "cancelled during MC pilot; restart the pilot to continue"
        } else if cancelled {
            "cancelled"
        } else if converged {
            "accuracy reached"
        } else {
            "work limit"
        }
        .into(),
        estimate,
        snapshot: stopped(snapshot, cancelled, converged),
        resume_status,
    })
}
