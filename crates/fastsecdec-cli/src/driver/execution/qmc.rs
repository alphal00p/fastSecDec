use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_checkpoint,
    refinement::{adaptive_budget, qmc_design},
    report::{stopped, with_diagnostics},
};
use super::{Context, evaluate_tracked, submit_package};
use crate::CliResult;
use fastsecdec::{
    integration::{QmcSession, QmcWorker},
    kernel::WeightedEvaluationContext,
    status::{EvaluationDiagnostics, IntegrationMethod, IntegrationStage},
};
use rayon::prelude::*;
use std::{collections::BTreeMap, time::Instant};

struct QmcSlot {
    contexts: Vec<WeightedEvaluationContext>,
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
        mut replay,
    } = context;
    let options = settings.qmc_settings()?;
    let mut session = if let Some(checkpoint) = &restored {
        let session = QmcSession::restore(&checkpoint.session, &problem)?;
        let mut expected = options.clone();
        (expected.points, expected.shifts) = qmc_design(settings, checkpoint.round_index)?;
        let expected_method = if method == "adaptive_qmc" {
            IntegrationMethod::AdaptiveQmc
        } else {
            IntegrationMethod::DemocraticQmc
        };
        if session.design().settings != expected || session.method() != expected_method {
            return Err(
                "checkpoint native QMC design differs from the outer settings or refinement round"
                    .into(),
            );
        }
        session
    } else if method == "adaptive_qmc" {
        QmcSession::adaptive(problem.clone(), options.clone())?
    } else {
        QmcSession::democratic(problem.clone(), options.clone())?
    };
    let mut cancelled = false;
    let mut round = restored
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.round_index);
    loop {
        let mut slots = (0..settings.workers)
            .map(|_| {
                Ok(QmcSlot {
                    contexts: replay.contexts(kernels)?,
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
            for (slot, task) in slots.iter_mut().zip(&tasks) {
                let id = task.sector_id() as usize;
                slot.contexts[id].merge_state(replay.state(id))?;
            }
            let returns = pool.install(|| {
                slots
                    .par_iter_mut()
                    .zip(tasks.into_par_iter())
                    .map(|(slot, task)| {
                        let id = task.sector_id();
                        let kernel = &mut slot.contexts[id as usize];
                        let mut local = EvaluationDiagnostics::default();
                        let result = slot.workers.get_mut(&id).unwrap().evaluate_weighted(
                            task,
                            |point, weight, output| {
                                evaluate_tracked(kernel, point, weight, output, &mut local)
                            },
                        );
                        let state = result.as_ref().ok().map(|_| kernel.state().clone());
                        (id as usize, result, local, state)
                    })
                    .collect::<Vec<_>>()
            });
            let mut failure = None;
            for (id, result, local, state) in returns {
                diagnostics.merge(&local)?;
                if let Err(error) = submit_package(result, id, state, &mut replay, |result| {
                    session.submit(result)
                }) {
                    failure.get_or_insert(error);
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
                    &replay,
                )?;
                return Err(error);
            }
            if last_checkpoint.elapsed().as_secs() >= 5 {
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
                    &diagnostics,
                    &replay,
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
        (next.points, next.shifts) = qmc_design(settings, round)?;
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
        &replay,
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
        qmc_design: Some(session.design()),
    })
}
