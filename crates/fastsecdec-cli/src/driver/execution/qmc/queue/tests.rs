use super::*;
use crate::driver::execution::{problem, qmc::slot::tests::fixture};
use std::sync::{Condvar, Mutex, atomic::AtomicUsize};

// Exact native transport control: no alternate estimator, point generation or
// periodization. Two correlated components exercise the entire covariance.
fn transport(mut slot: QmcSlot, task: QmcTask, _: &AtomicBool) -> Completed {
    let sector = task.sector_id() as usize;
    let active = slot.active.as_mut().unwrap();
    let result = active.worker.evaluate_weighted(task, |point, weight, out| {
        out[0] = point[0] * weight;
        out[1] = 2.0 * out[0];
        Ok::<_, String>(())
    });
    let state = result.as_ref().ok().map(|_| active.context.state().clone());
    Completed {
        slot,
        sector,
        result,
        diagnostics: EvaluationDiagnostics::default(),
        state,
        aborted_prefix: false,
    }
}

#[test]
fn completed_worker_refills_before_straggler_and_preserves_full_covariance() {
    let (_dir, artifact, kernels, settings) = fixture();
    let problem = problem(&artifact, &kernels, &settings.scope).unwrap();
    let mut session =
        QmcSession::democratic(problem.clone(), settings.qmc_settings().unwrap()).unwrap();
    let mut replay = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let gate = (Mutex::new(false), Condvar::new());
    let active = AtomicUsize::new(0);
    let maximum = AtomicUsize::new(0);
    let completion_order = Mutex::new(Vec::new());
    let outcome = Phase {
        pool: &pool,
        kernels: &kernels,
        session: &mut session,
        workers: 2,
        diagnostics: &mut diagnostics,
        replay: &mut replay,
        operations: &crate::driver::execution::observations::Operations::new(
            2,
            kernels.orders(),
            Duration::from_secs(1),
        ),
    }
    .run_with(|_, _, _, _| Ok(Outcome::default()), &|slot, task, stop| {
        let count = active.fetch_add(1, Ordering::SeqCst) + 1;
        maximum.fetch_max(count, Ordering::SeqCst);
        let identity = (task.sector_id(), task.work().start());
        if identity == (0, 0) {
            let (ready, _) = gate
                .1
                .wait_timeout_while(gate.0.lock().unwrap(), Duration::from_secs(5), |ready| {
                    !*ready
                })
                .unwrap();
            assert!(
                *ready,
                "a returned worker did not refill while the first package remained in flight"
            );
        } else if task.work().start() > 0 {
            *gate.0.lock().unwrap() = true;
            gate.1.notify_all();
        }
        let completed = transport(slot, task, stop);
        completion_order.lock().unwrap().push(identity);
        active.fetch_sub(1, Ordering::SeqCst);
        completed
    })
    .unwrap();
    assert!(outcome.failure.is_none(), "{:?}", outcome.failure);
    assert!(!outcome.cancelled);
    assert_eq!(maximum.load(Ordering::SeqCst), 2);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert_eq!(completion_order.lock().unwrap()[0], (1, 0));
    assert!(session.is_complete());

    let mut serial = QmcSession::democratic(problem, settings.qmc_settings().unwrap()).unwrap();
    let mut slot = QmcSlot::default();
    while let Some(task) = serial.next_work().unwrap() {
        slot.prepare(task.sector_id(), &kernels, &serial, &replay)
            .unwrap();
        let completed = transport(slot, task, &AtomicBool::new(false));
        serial.submit(completed.result.unwrap()).unwrap();
        slot = completed.slot;
    }
    let got = session.estimate().unwrap();
    assert_eq!(got.mean.len(), 2);
    assert_eq!(got.covariance_of_mean.len(), 4);
    assert_eq!(got, serial.estimate().unwrap());
}

#[test]
fn cancellation_keeps_inflight_checkpoint_unaccepted_and_reissuable() {
    let (_dir, artifact, kernels, settings) = fixture();
    let problem = problem(&artifact, &kernels, &settings.scope).unwrap();
    let mut session =
        QmcSession::democratic(problem.clone(), settings.qmc_settings().unwrap()).unwrap();
    let mut replay = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let initial = replay.clone();
    let mut diagnostics = EvaluationDiagnostics::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let entered = AtomicUsize::new(0);
    let mut checkpoint = None;
    let outcome = Phase {
        pool: &pool,
        kernels: &kernels,
        session: &mut session,
        workers: 2,
        diagnostics: &mut diagnostics,
        replay: &mut replay,
        operations: &crate::driver::execution::observations::Operations::new(
            2,
            kernels.orders(),
            Duration::from_secs(1),
        ),
    }
    .run_with(
        |session, _, _, _| {
            let cancelled = entered.load(Ordering::SeqCst) == 2;
            if cancelled && checkpoint.is_none() {
                checkpoint = Some(session.checkpoint()?);
            }
            Ok(Outcome {
                cancelled,
                failure: None,
            })
        },
        &|slot, task, stop| {
            entered.fetch_add(1, Ordering::SeqCst);
            while !stop.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            evaluate(slot, task, stop)
        },
    )
    .unwrap();
    assert!(outcome.cancelled);
    assert!(outcome.failure.is_none());
    assert_eq!(diagnostics.evaluations, 0);
    assert_eq!(diagnostics.failures, 0);
    assert_eq!(replay, initial);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
    let mut restored = QmcSession::restore(&checkpoint.unwrap(), &problem).unwrap();
    let mut count = 0;
    while restored.next_work().unwrap().is_some() {
        count += 1;
    }
    assert_eq!(
        count, 8,
        "all unfinished canonical packages must be available again"
    );
}

#[test]
fn cancellation_drains_completed_returns_without_repeated_observation() {
    let (_dir, artifact, kernels, settings) = fixture();
    let problem = problem(&artifact, &kernels, &settings.scope).unwrap();
    let mut session =
        QmcSession::democratic(problem.clone(), settings.qmc_settings().unwrap()).unwrap();
    let mut replay = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();
    let completed = AtomicUsize::new(0);
    let mut cancellation_observed = false;
    let outcome = Phase {
        pool: &pool,
        kernels: &kernels,
        session: &mut session,
        workers: 8,
        diagnostics: &mut diagnostics,
        replay: &mut replay,
        operations: &crate::driver::execution::observations::Operations::new(
            8,
            kernels.orders(),
            Duration::from_secs(1),
        ),
    }
    .run_with(
        |_, _, _, _| {
            assert!(
                !cancellation_observed,
                "draining must not repeat expensive observational polling"
            );
            cancellation_observed = completed.load(Ordering::SeqCst) == 8;
            Ok(Outcome {
                cancelled: cancellation_observed,
                failure: None,
            })
        },
        &|slot, task, stop| {
            // Finish every native package before cancellation, but hold its
            // return until the coordinator sees the stop. No timed race is used.
            let value = transport(slot, task, stop);
            completed.fetch_add(1, Ordering::SeqCst);
            while !stop.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            value
        },
    )
    .unwrap();
    assert!(outcome.cancelled && outcome.failure.is_none());
    assert_eq!(completed.load(Ordering::SeqCst), 8);
    assert!(session.is_complete());
    assert_eq!(session.snapshot().unwrap().completed_points, 8192);
    assert_eq!(session.complete_shift_estimates().unwrap().len(), 4);
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.mean[1], 2. * estimate.mean[0]);
    assert!(estimate.covariance_of_mean[0] > 0.);
    assert_eq!(
        estimate.covariance_of_mean[1],
        2. * estimate.covariance_of_mean[0]
    );
    let restored = QmcSession::restore(&session.checkpoint().unwrap(), &problem).unwrap();
    assert!(restored.is_complete());
    assert_eq!(restored.estimate().unwrap(), estimate);
}

#[test]
fn real_failure_stops_refill_without_accepting_failed_or_cancelled_prefixes() {
    let (_dir, artifact, kernels, settings) = fixture();
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels, &settings.scope).unwrap(),
        settings.qmc_settings().unwrap(),
    )
    .unwrap();
    let mut replay = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let initial = replay.clone();
    let mut diagnostics = EvaluationDiagnostics::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let issued = AtomicUsize::new(0);
    let outcome = Phase {
        pool: &pool,
        kernels: &kernels,
        session: &mut session,
        workers: 2,
        diagnostics: &mut diagnostics,
        replay: &mut replay,
        operations: &crate::driver::execution::observations::Operations::new(
            2,
            kernels.orders(),
            Duration::from_secs(1),
        ),
    }
    .run_with(
        |_, _, _, _| Ok(Outcome::default()),
        &|mut slot, task, stop| {
            issued.fetch_add(1, Ordering::SeqCst);
            if task.sector_id() == 1 {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::yield_now();
                }
                return evaluate(slot, task, stop);
            }
            let active = slot.active.as_mut().unwrap();
            let mut local = EvaluationDiagnostics::default();
            let mut calls = 0;
            let result = active
                .worker
                .evaluate_weighted(task, |point, weight, output| {
                    calls += 1;
                    if calls == 3 {
                        local.record_failure().unwrap();
                        return Err("injected native callback failure".to_owned());
                    }
                    crate::driver::execution::evaluate_tracked(
                        &mut active.context,
                        point,
                        weight,
                        output,
                        &mut local,
                    )
                });
            assert!(active.context.state().verified());
            Completed {
                slot,
                sector: 0,
                result,
                diagnostics: local,
                state: None,
                aborted_prefix: false,
            }
        },
    )
    .unwrap();
    assert!(!outcome.cancelled);
    assert!(
        outcome
            .failure
            .unwrap()
            .contains("injected native callback failure")
    );
    assert_eq!(issued.load(Ordering::SeqCst), 2);
    // Diagnostics count the failed callback as attempted evaluation too;
    // accepted session coverage below remains zero for the whole prefix.
    assert_eq!(diagnostics.evaluations, 3);
    assert_eq!(diagnostics.failures, 1);
    assert_eq!(replay, initial);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
}

#[test]
fn worker_panic_notifies_coordinator_and_drains_other_jobs() {
    let (_dir, artifact, kernels, settings) = fixture();
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels, &settings.scope).unwrap(),
        settings.qmc_settings().unwrap(),
    )
    .unwrap();
    let mut replay = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let outcome = Phase {
        pool: &pool,
        kernels: &kernels,
        session: &mut session,
        workers: 2,
        diagnostics: &mut diagnostics,
        replay: &mut replay,
        operations: &crate::driver::execution::observations::Operations::new(
            2,
            kernels.orders(),
            Duration::from_secs(1),
        ),
    }
    .run_with(|_, _, _, _| Ok(Outcome::default()), &|slot, task, stop| {
        if task.sector_id() == 0 {
            panic!("injected worker panic");
        }
        while !stop.load(Ordering::Relaxed) {
            std::thread::yield_now();
        }
        evaluate(slot, task, stop)
    })
    .unwrap();
    assert!(
        outcome
            .failure
            .unwrap()
            .contains("QMC worker panicked: injected worker panic")
    );
    assert!(!outcome.cancelled);
    assert_eq!(diagnostics.failures, 0);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
}
