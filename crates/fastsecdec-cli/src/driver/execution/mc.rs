use super::super::{
    IntegrationReport,
    checkpoint::save_mc_checkpoint,
    refinement::mc_points,
    report::{ExecutionOutcome, finish, with_diagnostics},
};
use super::{Context, evaluate_tracked, submit_package};
use crate::CliResult;
use fastsecdec::{
    integration::mc::{HavanaSession, HavanaSettings, HavanaWorker},
    kernel::WeightedEvaluationContext,
    status::{EvaluationDiagnostics, IntegrationStage},
};
use rayon::prelude::*;
use std::{collections::BTreeMap, time::Instant};

struct McSlot {
    contexts: Vec<WeightedEvaluationContext>,
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
        mut replay,
    } = context;
    let options = HavanaSettings {
        points_per_batch: settings.points.try_into()?,
        batches: settings.shifts,
        seed: settings.seed,
        ..HavanaSettings::default()
    };
    let mut session = if let Some(checkpoint) = &restored {
        HavanaSession::restore(&checkpoint.session, &problem)?
    } else if method == "adaptive_mc" {
        HavanaSession::pilot(problem.clone(), options.clone())?
    } else {
        HavanaSession::production(problem.clone(), options.clone())?
    };
    let mut cancelled = false;
    let mut failure = None;
    let mut round = restored
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.round_index);
    let mut current_points = mc_points(settings.points, round)?;
    'rounds: loop {
        let mut slots = (0..settings.workers)
            .map(|_| {
                Ok(McSlot {
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
                .filter_map(|_| session.next_work())
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("MC scheduler has no work before completion".into());
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
                save_mc_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    &session,
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
            session.freeze_production(0.5, current_points.try_into()?, settings.shifts)?;
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
            resume_status,
            qmc_design: None,
        },
    )
}
