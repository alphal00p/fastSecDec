use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_checkpoint,
    refinement::{adaptive_budget, qmc_design},
    report::{stopped, with_diagnostics},
};
use super::{Context, cloned_kernels, evaluate_tracked};
use crate::CliResult;
use fastsecdec::{
    integration::{Periodization, QmcSession, QmcSettings, QmcWorker, RuleSource},
    kernel::SectorKernel,
    status::{EvaluationDiagnostics, IntegrationStage},
};
use rayon::prelude::*;
use std::{collections::BTreeMap, time::Instant};

struct QmcSlot {
    kernels: Vec<SectorKernel>,
    workers: BTreeMap<u64, QmcWorker>,
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
    qmc_design(settings.points, settings.shifts, 0)?;
    let options = QmcSettings {
        points: settings.points,
        shifts: settings.shifts,
        seed: settings.seed,
        package_points: settings.package_points,
        periodization: match settings.periodization.as_str() {
            "none" => Periodization::None,
            "korobov3" => Periodization::Korobov3,
            _ => return Err("periodization must be none or korobov3".into()),
        },
        rule: RuleSource::Kuo,
    };
    let mut session = if let Some((_, _, bytes)) = &restored {
        QmcSession::restore(bytes, &problem)?
    } else if method == "adaptive_qmc" {
        QmcSession::adaptive(problem.clone(), options.clone())?
    } else {
        QmcSession::democratic(problem.clone(), options.clone())?
    };
    let mut cancelled = false;
    let mut round = restored.as_ref().map_or(0, |(round, _, _)| *round);
    loop {
        let mut slots = (0..settings.workers)
            .map(|_| {
                Ok(QmcSlot {
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
                .map(|_| session.next_work())
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("QMC scheduler has no work before completion".into());
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
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
                    &diagnostics,
                )?;
                return Err(error.into());
            }
            if last_checkpoint.elapsed().as_secs() >= 5 {
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
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
            let (seconds, minimum_shifts) = adaptive_budget(settings, round)?;
            let allocation = session.recommend_allocation(
                seconds,
                minimum_shifts,
                &vec![1.0; kernels.orders().len()],
            )?;
            session.freeze_production(allocation)?;
            continue;
        }
        if session.estimate()?.meets(tolerance)? || round + 1 >= settings.max_rounds {
            break;
        }
        round += 1;
        let mut next = options.clone();
        (next.points, next.shifts) = qmc_design(settings.points, settings.shifts, round)?;
        session = if method == "adaptive_qmc" {
            QmcSession::adaptive(problem.clone(), next)?
        } else {
            QmcSession::democratic(problem.clone(), next)?
        };
    }
    save_checkpoint(
        checkpoint,
        artifact,
        settings,
        round,
        session.checkpoint()?,
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
        stopping_reason: if cancelled {
            "cancelled"
        } else if converged {
            "accuracy reached"
        } else {
            "work limit"
        }
        .into(),
        estimate,
        snapshot: stopped(snapshot, cancelled, converged),
        resume_status: ResumeStatus::CheckpointSaved,
    })
}
