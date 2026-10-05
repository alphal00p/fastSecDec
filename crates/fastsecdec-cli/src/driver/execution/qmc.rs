use super::super::{
    IntegrationReport, ResumeStatus,
    checkpoint::save_checkpoint,
    refinement::{adaptive_budget, qmc_design},
    report::{ExecutionOutcome, finish},
};
use super::{Context, final_report, observe};
use crate::CliResult;
use fastsecdec::{
    integration::QmcSession,
    status::{IntegrationMethod, IntegrationStage},
};
use std::time::Instant;

mod queue;
mod slot;
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
        observe(
            dashboard,
            &diagnostics,
            started,
            true,
            || session.snapshot(),
            &mut failure,
        )?;
        if failure.is_some() {
            break;
        }
        let outcome = queue::Phase {
            pool: &pool,
            kernels,
            session: &mut session,
            workers: settings.workers,
            diagnostics: &mut diagnostics,
            replay: &mut replay,
        }
        .run(|session, diagnostics, replay, force| {
            let cancelled = dashboard.cancelled();
            let mut failure = None;
            observe(
                dashboard,
                diagnostics,
                started,
                force || cancelled,
                || session.snapshot(),
                &mut failure,
            )?;
            if last_checkpoint.elapsed().as_secs() >= 5 {
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
                    diagnostics,
                    replay,
                )?;
                last_checkpoint = Instant::now();
            }
            Ok(queue::Outcome { cancelled, failure })
        })?;
        cancelled = outcome.cancelled;
        failure = outcome.failure;
        if cancelled || failure.is_some() {
            break 'rounds;
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
            resume_status: ResumeStatus::CheckpointSaved,
            qmc_design: Some(session.design()),
        },
    )?;
    final_report(dashboard, report)
}
