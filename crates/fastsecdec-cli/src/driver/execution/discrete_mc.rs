//! Caller-owned global batches with native discrete sector/continuous sampling.
use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_checkpoint,
    refinement::mc_points,
    report::{ExecutionOutcome, finish},
};
use super::{Context, final_report, observe};
use crate::CliResult;
use fastsecdec::{
    integration::{
        mc::HavanaSettings,
        mc_discrete::{HavanaDiscreteSession, HavanaDiscreteSettings, HavanaDiscreteWorker},
    },
    kernel::WeightedEvaluationContext,
    status::IntegrationStage,
};
mod wave;
use std::{collections::BTreeMap, time::Instant};
struct Slot {
    contexts: BTreeMap<u64, WeightedEvaluationContext>,
    worker: HavanaDiscreteWorker,
}
pub(super) fn run(context: Context<'_>) -> CliResult<IntegrationReport> {
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
    let steering = settings.discrete_mc.clone().unwrap_or_default();
    if steering.pilot_iterations == 0
        || [
            steering.discrete_learning_rate,
            steering.continuous_learning_rate,
        ]
        .iter()
        .any(|r| !r.is_finite() || *r < 0.0)
    {
        return Err(
            "discrete MC requires positive pilot iterations and finite nonnegative learning rates"
                .into(),
        );
    }
    let pilot = HavanaDiscreteSettings {
        batch: HavanaSettings {
            points_per_batch: steering.pilot_points,
            batches: steering.pilot_batches,
            seed: settings.seed,
            bins: steering.bins,
            minimum_probability_density: steering.minimum_probability_density,
        },
        maximum_sector_probability_ratio: steering.maximum_sector_probability_ratio,
    };
    pilot.validate()?;
    let mut production = pilot.clone();
    production.batch.points_per_batch = settings.points.try_into()?;
    production.batch.batches = settings.shifts;
    production.validate()?;
    let mut round = restored.as_ref().map_or(0, |c| c.round_index);
    let mut session = if let Some(stored) = &restored {
        HavanaDiscreteSession::restore(&stored.session, &problem)?
    } else {
        HavanaDiscreteSession::pilot(problem.clone(), pilot.clone())?
    };
    let mut pilot_iteration = 0usize;
    let mut cancelled = false;
    let mut failure = None;
    // Lazily cache only sectors actually selected by each caller-owned worker.
    // Caches survive grid adaptation; their worst-case size is all visited
    // sectors per worker, not a claim of one evaluator context per worker.
    let mut slots: Vec<Slot> = Vec::new();
    'rounds: loop {
        observe(
            dashboard,
            &diagnostics,
            started,
            true,
            || session.snapshot(),
            &mut failure,
        )?;
        cancelled |= dashboard.cancelled();
        if failure.is_some() || cancelled {
            break;
        }
        if slots.is_empty() && !problem.sectors.is_empty() {
            slots = (0..settings.workers)
                .map(|_| {
                    Ok(Slot {
                        contexts: BTreeMap::new(),
                        worker: session.worker_context()?,
                    })
                })
                .collect::<CliResult<Vec<_>>>()?;
        } else {
            for slot in &mut slots {
                slot.worker = session.worker_context()?;
            }
        }
        while !session.is_complete() {
            cancelled |= dashboard.cancelled();
            if cancelled {
                break 'rounds;
            }
            let tasks = (0..settings.workers)
                .filter_map(|_| session.next_work())
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("discrete MC scheduler has no work before completion".into());
            }
            for slot in &mut slots {
                for (id, context) in &mut slot.contexts {
                    context.merge_state(replay.state(*id as usize))?;
                }
            }
            let returns = wave::run(&pool, &mut slots, tasks, kernels, &replay, |activity| {
                dashboard.integration_work(activity);
                cancelled |= dashboard.cancelled();
                observe(
                    dashboard,
                    &diagnostics,
                    started,
                    cancelled || failure.is_some(),
                    || session.snapshot(),
                    &mut failure,
                )?;
                Ok(cancelled || failure.is_some())
            })?;
            dashboard.integration_work(Vec::new());
            for returned in returns {
                let completed = match returned {
                    Ok(completed) => completed,
                    Err(error) => {
                        failure
                            .get_or_insert_with(|| format!("discrete MC worker panicked: {error}"));
                        continue;
                    }
                };
                diagnostics.merge(&completed.diagnostics)?;
                if completed.aborted_prefix {
                    continue;
                }
                let wave::Completed { result, states, .. } = completed;
                let accepted = (|| -> CliResult<()> {
                    let value = result?;
                    let states = states.ok_or("successful global batch has no replay states")?;
                    // Validate and merge every sector into a private candidate;
                    // neither numerical coverage nor replay accepts a prefix.
                    let mut candidate = replay.clone();
                    for (id, state) in states {
                        candidate.accept(id, &state)?;
                    }
                    session.submit(value)?;
                    replay = candidate;
                    Ok(())
                })();
                if let Err(error) = accepted {
                    failure.get_or_insert_with(|| error.to_string());
                }
            }
            cancelled = dashboard.cancelled();
            observe(
                dashboard,
                &diagnostics,
                started,
                cancelled || failure.is_some() || session.is_complete(),
                || session.snapshot(),
                &mut failure,
            )?;
            if cancelled || failure.is_some() {
                break 'rounds;
            }
            if last_checkpoint.elapsed().as_secs() >= 5
                && session.stage() == IntegrationStage::Production
            {
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
        }
        if session.stage() == IntegrationStage::Pilot {
            pilot_iteration += 1;
            if pilot_iteration < steering.pilot_iterations {
                session.adapt_pilot(
                    steering.discrete_learning_rate,
                    steering.continuous_learning_rate,
                )?;
            } else {
                session.freeze_production(
                    steering.discrete_learning_rate,
                    steering.continuous_learning_rate,
                    mc_points(settings.points, round)?.try_into()?,
                    settings.shifts,
                )?;
            }
            continue;
        }
        let meets = match session.estimate().and_then(|e| e.meets(tolerance)) {
            Ok(v) => v,
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        };
        if meets || round + 1 >= settings.max_rounds {
            break;
        }
        round += 1;
        session = HavanaDiscreteSession::pilot(problem.clone(), pilot.clone())?;
        pilot_iteration = 0;
    }
    let resume_status = if session.stage() == IntegrationStage::Pilot {
        ResumeStatus::PilotRestartRequired
    } else {
        save_checkpoint(
            checkpoint,
            artifact,
            settings,
            round,
            session.checkpoint()?,
            &diagnostics,
            &replay,
        )?;
        ResumeStatus::CheckpointSaved
    };
    let report = finish(
        artifact,
        session.diagnostic_observation()?,
        &diagnostics,
        tolerance,
        started.elapsed().as_secs_f64(),
        ExecutionOutcome {
            scope: settings.scope.clone(),
            cancelled,
            failure,
            resume_status,
            qmc_design: None,
        },
    )?;
    final_report(dashboard, report)
}
