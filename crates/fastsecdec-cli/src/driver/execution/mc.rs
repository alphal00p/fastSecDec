use super::super::{
    IntegrationReport,
    checkpoint::save_mc_checkpoint,
    refinement::mc_points,
    report::{ExecutionOutcome, finish},
};
use super::{Context, final_report, observe, submit_package};
mod wave;
use crate::CliResult;
use fastsecdec::{
    integration::mc::{HavanaSession, HavanaSettings, HavanaWorker},
    kernel::WeightedEvaluationContext,
    status::IntegrationStage,
};
use std::{collections::BTreeMap, time::Instant};

struct McSlot {
    contexts: BTreeMap<u64, WeightedEvaluationContext>,
    workers: BTreeMap<u64, HavanaWorker>,
    meter: super::observations::WorkerMeter,
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
        operations,
    } = context;
    let initialization = operations.coordinator(false);
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
    let mut resumed_phase = restored.is_some();
    drop(initialization);
    'rounds: loop {
        dashboard.set_live_observation(None);
        let frozen = replay.clone();
        let mut ledger = super::observations::McLedger::default();
        let source = if resumed_phase {
            fastsecdec::integration::LiveSource::SinceResume
        } else {
            fastsecdec::integration::LiveSource::CurrentIteration
        };
        observe(
            dashboard,
            &operations,
            &diagnostics,
            started,
            true,
            || session.diagnostic_observation(),
            &mut failure,
        )?;
        if failure.is_some() {
            break;
        }
        let mut slots = (0..settings.workers)
            .map(|id| McSlot {
                contexts: BTreeMap::new(),
                workers: BTreeMap::new(),
                meter: operations.worker(id),
            })
            .collect::<Vec<_>>();
        while !session.is_complete() {
            cancelled |= dashboard.cancelled();
            if cancelled {
                break 'rounds;
            }
            let schedule = operations.coordinator(false);
            let tasks = (0..settings.workers)
                .filter_map(|_| session.next_work())
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("MC scheduler has no work before completion".into());
            }
            drop(schedule);
            let returns = wave::run(
                &pool,
                &mut slots,
                tasks,
                &session,
                kernels,
                &replay,
                &frozen,
                |activity, collect_live, completed| {
                    dashboard.integration_work(activity);
                    if completed {
                        ledger.update(collect_live().into_iter());
                    }

                    cancelled |= dashboard.cancelled();
                    let _span = operations.coordinator(false);
                    super::observe_live(
                        dashboard,
                        &operations,
                        &diagnostics,
                        started,
                        cancelled,
                        || {
                            if !completed {
                                ledger.update(collect_live().into_iter());
                            }
                            fastsecdec::integration::mc_live_observation(
                                &problem,
                                session.stage(),
                                source,
                                &ledger.batches(),
                                false,
                            )
                        },
                        || session.diagnostic_observation(),
                        &mut failure,
                    )?;
                    Ok(cancelled || failure.is_some())
                },
            )?;
            dashboard.integration_work(Vec::new());
            let admission = operations.coordinator(false);
            for returned in returns {
                let completed = match returned {
                    Ok(value) => value,
                    Err(error) => {
                        failure.get_or_insert_with(|| format!("MC worker panicked: {error}"));
                        continue;
                    }
                };
                diagnostics.merge(&completed.diagnostics)?;
                if completed.aborted {
                    ledger.discard(Some(completed.sector), completed.batch);
                    continue;
                }
                if let Err(error) = submit_package(
                    completed.result,
                    completed.sector as usize,
                    completed.state,
                    &mut replay,
                    |value| session.submit(value),
                ) {
                    ledger.discard(Some(completed.sector), completed.batch);
                    failure.get_or_insert_with(|| error.to_string());
                }
            }
            drop(admission);

            cancelled = dashboard.cancelled();
            super::observe_live(
                dashboard,
                &operations,
                &diagnostics,
                started,
                cancelled || failure.is_some() || session.is_complete(),
                || {
                    fastsecdec::integration::mc_live_observation(
                        &problem,
                        session.stage(),
                        source,
                        &ledger.batches(),
                        false,
                    )
                },
                || session.diagnostic_observation(),
                &mut failure,
            )?;
            if failure.is_some() || cancelled {
                break 'rounds;
            }
            if last_checkpoint.elapsed().as_secs() >= 5 {
                let _checkpoint_span = operations.coordinator(false);
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
        }
        if cancelled {
            break;
        }
        resumed_phase = false;
        let _refinement_span = operations.coordinator(false);
        if session.stage() == IntegrationStage::Pilot {
            session.freeze_production(0.5, current_points.try_into()?, settings.shifts)?;
            continue;
        }
        let meets = match session
            .estimate()
            .and_then(|estimate| estimate.meets_target(settings.accuracy_target, tolerance))
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
    let checkpoint_span = operations.coordinator(false);
    let resume_status = save_mc_checkpoint(
        checkpoint,
        artifact,
        settings,
        round,
        &session,
        &diagnostics,
        &replay,
    )?;
    drop(checkpoint_span);
    let report = finish(
        artifact,
        session.diagnostic_observation()?,
        &diagnostics,
        tolerance,
        started.elapsed().as_secs_f64(),
        ExecutionOutcome {
            stability_mode: settings.stability.mode,
            accuracy_target: settings.accuracy_target,
            operational: operations.snapshot()?,
            scope: settings.scope.clone(),
            cancelled,
            failure,
            resume_status,
            qmc_design: None,
        },
    )?;
    final_report(dashboard, report)
}
