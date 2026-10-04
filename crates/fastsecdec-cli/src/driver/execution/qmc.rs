use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_checkpoint,
    refinement::{adaptive_budget, qmc_design},
    report::{ExecutionOutcome, finish, with_diagnostics},
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
    let mut failure = None;
    let mut round = restored
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.round_index);
    'rounds: loop {
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
            for (id, result, local, state) in returns {
                diagnostics.merge(&local)?;
                if let Err(error) = submit_package(result, id, state, &mut replay, |result| {
                    session.submit(result)
                }) {
                    failure.get_or_insert_with(|| error.to_string());
                }
            }
            match session.snapshot() {
                Ok(snapshot) => dashboard.integration(
                    &with_diagnostics(snapshot, &diagnostics),
                    started.elapsed().as_secs_f64(),
                )?,
                Err(error) => {
                    failure.get_or_insert_with(|| error.to_string());
                }
            }
            if failure.is_some() {
                break 'rounds;
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
            let allocation = match session.recommend_allocation(
                seconds,
                minimum_shifts,
                &vec![1.0; kernels.orders().len()],
            ) {
                Ok(allocation) => allocation,
                Err(error) => {
                    failure = Some(error.to_string());
                    break;
                }
            };
            session.freeze_production(allocation)?;
            continue;
        }
        let meets = match session
            .estimate()
            .and_then(|estimate| estimate.meets(tolerance))
        {
            Ok(meets) => meets,
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        };
        if meets || round + 1 >= settings.max_rounds {
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
    finish(
        artifact,
        session.diagnostic_observation()?,
        &diagnostics,
        tolerance,
        started.elapsed().as_secs_f64(),
        ExecutionOutcome {
            cancelled,
            failure,
            resume_status: ResumeStatus::CheckpointSaved,
            qmc_design: Some(session.design()),
        },
    )
}
