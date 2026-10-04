use fastsecdec::{
    integration::{
        IntegrationProblem, SectorSpec, Tolerance,
        mc::{HavanaReturn, HavanaSession, HavanaSettings, HavanaTask},
    },
    status::{IntegrationStage, UncertaintyStatus},
};

fn problem() -> IntegrationProblem {
    IntegrationProblem::new(
        "havana-complete-vector".into(),
        vec![0, 1],
        vec![
            SectorSpec {
                id: 4,
                dimension: 3,
            },
            SectorSpec {
                id: 9,
                dimension: 1,
            },
        ],
        vec![0.0, 0.0],
    )
    .unwrap()
}

fn settings() -> HavanaSettings {
    HavanaSettings {
        points_per_batch: 256,
        batches: 16,
        bins: 8,
        seed: 51,
        minimum_probability_density: 0.02,
    }
}

fn tasks(session: &mut HavanaSession) -> Vec<HavanaTask> {
    let mut result = Vec::new();
    while let Some(task) = session.next_work() {
        result.push(task);
    }
    result
}

fn evaluate(session: &HavanaSession, task: HavanaTask) -> HavanaReturn {
    session
        .worker_context(task.sector_id())
        .unwrap()
        .evaluate(task, |x, out| {
            out[0] = x.iter().product();
            out[1] = -2.0 * out[0];
            Ok::<(), String>(())
        })
        .unwrap()
}

#[test]
fn havana_complete_vectors_match_cube_products_and_covariance() {
    let mut session = HavanaSession::production(problem(), settings()).unwrap();
    for task in tasks(&mut session) {
        session.submit(evaluate(&session, task)).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert!((estimate.mean[0] - 0.625).abs() < 6.0 * estimate.standard_error[0]);
    assert_eq!(estimate.mean[1], -2.0 * estimate.mean[0]);
    assert!((estimate.covariance_of_mean[1] + 2.0 * estimate.covariance_of_mean[0]).abs() < 1e-15);
    assert!((estimate.covariance_of_mean[3] - 4.0 * estimate.covariance_of_mean[0]).abs() < 1e-15);
    assert_eq!(session.snapshot().unwrap().completed_points, 2 * 256 * 16);
    assert!(estimate.meets(Tolerance::new(1.0, 0.0).unwrap()).unwrap());
}

#[test]
fn havana_waits_for_every_sector_before_reporting_uncertainty() {
    let mut session = HavanaSession::production(problem(), settings()).unwrap();
    let work = tasks(&mut session);
    for task in work.iter().filter(|t| t.sector_id() == 4) {
        session.submit(evaluate(&session, task.clone())).unwrap();
    }
    assert_eq!(
        session.snapshot().unwrap().uncertainty,
        UncertaintyStatus::WaitingForCoverage
    );
    let others: Vec<_> = work.into_iter().filter(|t| t.sector_id() == 9).collect();
    session
        .submit(evaluate(&session, others[0].clone()))
        .unwrap();
    assert!(session.estimate().is_err());
    session
        .submit(evaluate(&session, others[1].clone()))
        .unwrap();
    assert!(
        !session
            .estimate()
            .unwrap()
            .meets(Tolerance::new(100.0, 0.0).unwrap())
            .unwrap()
    );
}

#[test]
fn havana_pilot_adaptation_is_excluded_from_production() {
    let mut session = HavanaSession::pilot(problem(), settings()).unwrap();
    assert!(session.freeze_production(0.5, 256, 16).is_err());
    let work = tasks(&mut session);
    let stale = work[0].clone();
    // A different pilot integrand is intentional: a pilot can train on an
    // envelope, but its large values must not contaminate production means.
    for task in work.into_iter().rev() {
        let result = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task, |x, out| {
                out[0] = 10.0 + x[0].powi(4);
                out[1] = -out[0];
                Ok::<(), String>(())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    assert_eq!(
        session.snapshot().unwrap().uncertainty,
        UncertaintyStatus::PilotOnly
    );
    assert!(session.estimate().is_err());
    session.freeze_production(0.5, 512, 16).unwrap();
    assert_eq!(session.stage(), IntegrationStage::Production);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
    assert!(
        session
            .worker_context(stale.sector_id())
            .unwrap()
            .evaluate(stale, |_, _| Ok::<(), String>(()))
            .is_err()
    );
    let checkpoint = session.checkpoint().unwrap();
    let state: serde_json::Value = serde_json::from_slice(&checkpoint).unwrap();
    let partition = state["sectors"][0]["partitions"][0].as_array().unwrap();
    assert!(
        partition
            .iter()
            .enumerate()
            .any(|(i, x)| x.as_f64().unwrap() != i as f64 / 8.0)
    );
    let mut resumed = HavanaSession::restore(&checkpoint, &problem()).unwrap();
    for task in tasks(&mut session) {
        session.submit(evaluate(&session, task)).unwrap();
    }
    for task in tasks(&mut resumed).into_iter().rev() {
        resumed.submit(evaluate(&resumed, task)).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate, resumed.estimate().unwrap());
    assert!((estimate.mean[0] - 0.625).abs() < 6.0 * estimate.standard_error[0]);
}

#[test]
fn havana_checkpoint_replay_reissues_pending_batches() {
    let mut original = HavanaSession::production(problem(), settings()).unwrap();
    let work = tasks(&mut original);
    let values: Vec<_> = work.into_iter().map(|t| evaluate(&original, t)).collect();
    for value in values.iter().take(5) {
        original.submit(value.clone()).unwrap();
    }
    let bytes = original.checkpoint().unwrap();
    let mut restored = HavanaSession::restore(&bytes, &problem()).unwrap();
    for task in tasks(&mut restored).into_iter().rev() {
        restored.submit(evaluate(&restored, task)).unwrap();
    }
    for value in values.into_iter().skip(5) {
        original.submit(value).unwrap();
    }
    assert_eq!(original.estimate().unwrap(), restored.estimate().unwrap());
    let mut malformed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    malformed["sectors"][0]["partitions"][0][0] = (-1.0).into();
    assert!(HavanaSession::restore(&serde_json::to_vec(&malformed).unwrap(), &problem()).is_err());
}

#[test]
fn havana_batch_uncertainty_is_calibrated_across_independent_seeds() {
    let problem = IntegrationProblem::new(
        "calibration-product".into(),
        vec![0],
        vec![SectorSpec {
            id: 1,
            dimension: 3,
        }],
        vec![0.0],
    )
    .unwrap();
    let mut squared_z = 0.0;
    for seed in 0..32 {
        let mut session = HavanaSession::production(
            problem.clone(),
            HavanaSettings {
                seed,
                points_per_batch: 128,
                batches: 16,
                ..settings()
            },
        )
        .unwrap();
        let mut worker = session.worker_context(1).unwrap();
        while let Some(task) = session.next_work() {
            let value = worker
                .evaluate(task, |x, out| {
                    out[0] = x.iter().product();
                    Ok::<(), String>(())
                })
                .unwrap();
            session.submit(value).unwrap();
        }
        let estimate = session.estimate().unwrap();
        squared_z += ((estimate.mean[0] - 0.125) / estimate.standard_error[0]).powi(2);
    }
    // Independent seed ensemble with 15 variance degrees of freedom. This
    // broad deterministic check detects point-count/replica-count mistakes.
    assert!(
        (0.4..2.2).contains(&(squared_z / 32.0)),
        "mean squared z={}",
        squared_z / 32.0
    );
}

#[test]
fn havana_failures_duplicates_and_exact_integrals_are_explicit() {
    let mut session = HavanaSession::production(problem(), settings()).unwrap();
    let task = session.next_work().unwrap();
    assert!(
        session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task.clone(), |_, _| Err::<(), _>("numeric failure"))
            .is_err()
    );
    session.retry(&task).unwrap();
    let reissued = tasks(&mut session)
        .into_iter()
        .find(|t| t == &task)
        .unwrap();
    let value = evaluate(&session, reissued);
    session.submit(value.clone()).unwrap();
    assert!(session.submit(value).is_err());
    let exact = IntegrationProblem::new("havana-exact".into(), vec![0], vec![], vec![3.0]).unwrap();
    let mut session = HavanaSession::pilot(exact, settings()).unwrap();
    assert!(session.next_work().is_none());
    assert_eq!(session.estimate().unwrap().mean, vec![3.0]);
    assert!(
        session
            .estimate()
            .unwrap()
            .meets(Tolerance::new(0.0, 0.0).unwrap())
            .unwrap()
    );
}

#[test]
fn havana_caller_threads_preserve_ordered_batch_reduction() {
    let mut serial = HavanaSession::production(problem(), settings()).unwrap();
    let mut parallel = serial.clone();
    for task in tasks(&mut serial) {
        serial.submit(evaluate(&serial, task)).unwrap();
    }
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = tasks(&mut parallel)
            .into_iter()
            .map(|task| {
                let mut context = parallel.worker_context(task.sector_id()).unwrap();
                scope.spawn(move || {
                    context
                        .evaluate(task, |x, out| {
                            out[0] = x.iter().product();
                            out[1] = -2.0 * out[0];
                            Ok::<(), String>(())
                        })
                        .unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    for result in results.into_iter().rev() {
        parallel.submit(result).unwrap();
    }
    assert_eq!(serial.estimate().unwrap(), parallel.estimate().unwrap());
}
