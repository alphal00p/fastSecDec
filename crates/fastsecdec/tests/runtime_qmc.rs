use fastsecdec::{
    integration::{
        IntegrationProblem, Periodization, ProductionAllocation, QmcReturn, QmcSession,
        QmcSettings, QmcTask, RuleSource, SectorSpec, Tolerance,
    },
    status::{IntegrationStage, UncertaintyStatus},
};
use numerica::numerical_integration::qmc::QmcPartial;

#[test]
fn callback_observes_periodization_weight_applied_exactly_once() {
    let mut configuration = settings();
    configuration.periodization = Periodization::Korobov3;
    let mut session = QmcSession::democratic(problem(), configuration).unwrap();
    let mut nontrivial = false;
    for task in tasks(&mut session) {
        let value = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate_with_weight(task, |point, weight, output| {
                assert!(point.iter().all(|x| (0.0..1.0).contains(x)));
                assert!(weight > 0.0 && weight.is_finite());
                nontrivial |= (weight - 1.0).abs() > 0.01;
                output[0] = 1.0 / weight;
                output[1] = -2.0 / weight;
                Ok::<_, String>(())
            })
            .unwrap();
        session.submit(value).unwrap();
    }
    assert!(nontrivial);
    let estimate = session.estimate().unwrap();
    // Two sectors plus the exact vector [1,-3]. The reciprocal cancels one
    // application of the supplied Jacobian, so missing/double weighting fails.
    assert!((estimate.mean[0] - 3.0).abs() < 1e-14);
    assert!((estimate.mean[1] + 7.0).abs() < 1e-14);
}

#[test]
fn complex_coefficients_retain_real_imaginary_covariance() {
    use fastsecdec::integration::CoefficientComponent::{Imag, Real};
    let problem = IntegrationProblem::new_with_components(
        "complex-prefactor".into(),
        vec![0, 0],
        vec![Real, Imag],
        vec![SectorSpec {
            id: 10,
            dimension: 1,
        }],
        vec![0.5, -0.5],
    )
    .unwrap();
    let mut session = QmcSession::democratic(problem, settings()).unwrap();
    for task in tasks(&mut session) {
        let value = synthetic(
            &session,
            task,
            |shift| vec![shift as f64, -2.0 * shift as f64],
            1.0,
        );
        session.submit(value).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.orders, vec![0, 0]);
    assert_eq!(estimate.components, vec![Real, Imag]);
    assert_eq!(estimate.mean, vec![2.0, -3.5]);
    assert_eq!(
        estimate.covariance_of_mean,
        vec![5.0 / 12.0, -5.0 / 6.0, -5.0 / 6.0, 5.0 / 3.0]
    );
    assert!(
        IntegrationProblem::new_with_components(
            "bad".into(),
            vec![0, 0],
            vec![Imag, Real],
            vec![],
            vec![0.0, 0.0]
        )
        .is_err()
    );
    assert!(
        IntegrationProblem::new("duplicate-real".into(), vec![0, 0], vec![], vec![0.0, 0.0])
            .is_err()
    );
}

fn problem() -> IntegrationProblem {
    IntegrationProblem::new(
        "test-complete-vector-v1".into(),
        vec![-1, 0],
        vec![
            SectorSpec {
                id: 10,
                dimension: 1,
            },
            SectorSpec {
                id: 20,
                dimension: 2,
            },
        ],
        vec![1.0, -3.0],
    )
    .unwrap()
}

fn settings() -> QmcSettings {
    QmcSettings {
        points: 8,
        shifts: 4,
        seed: 73,
        package_points: 8,
        periodization: Periodization::None,
        rule: RuleSource::Supplied(vec![1, 3]),
    }
}

fn tasks(session: &mut QmcSession) -> Vec<QmcTask> {
    let mut tasks = Vec::new();
    while let Some(task) = session.next_work().unwrap() {
        tasks.push(task);
    }
    tasks
}

fn evaluate(session: &QmcSession, task: QmcTask) -> QmcReturn {
    let sign = if task.sector_id() == 10 { 1.0 } else { -1.0 };
    session
        .worker_context(task.sector_id())
        .unwrap()
        .evaluate(task, |x, output| {
            output[0] = sign * x[0];
            output[1] = 2.0 * sign * x[0];
            Ok::<(), String>(())
        })
        .unwrap()
}

fn synthetic(
    session: &QmcSession,
    task: QmcTask,
    values: impl Fn(u64) -> Vec<f64>,
    seconds: f64,
) -> QmcReturn {
    let context = session.worker_context(task.sector_id()).unwrap();
    let mut partial =
        QmcPartial::new(context.plan(), task.work(), session.problem().orders.len()).unwrap();
    for index in task.work().start()..task.work().start() + task.point_count() {
        partial
            .push(&values(index / context.plan().rule().points()))
            .unwrap();
    }
    QmcReturn::from_partial(task, partial, seconds).unwrap()
}

#[test]
fn democratic_errors_wait_for_common_full_sector_coverage() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let work = tasks(&mut session);
    for task in work.iter().filter(|t| t.sector_id() == 10) {
        session.submit(evaluate(&session, task.clone())).unwrap();
    }
    assert_eq!(
        session.snapshot().unwrap().uncertainty,
        UncertaintyStatus::WaitingForCoverage
    );
    let others: Vec<_> = work.into_iter().filter(|t| t.sector_id() == 20).collect();
    session
        .submit(evaluate(&session, others[0].clone()))
        .unwrap();
    assert!(session.estimate().is_err());
    session
        .submit(evaluate(&session, others[1].clone()))
        .unwrap();
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.mean, vec![1.0, -3.0]);
    assert_eq!(estimate.standard_error, vec![0.0, 0.0]);
    assert!(!estimate.meets(Tolerance::new(0.0, 0.0).unwrap()).unwrap());
    for task in others.into_iter().skip(2) {
        session.submit(evaluate(&session, task)).unwrap();
    }
    assert!(
        session
            .estimate()
            .unwrap()
            .meets(Tolerance::new(0.0, 0.0).unwrap())
            .unwrap()
    );
}

#[test]
fn checkpoint_reissues_inflight_work_and_preserves_estimates() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let work = tasks(&mut session);
    let returns: Vec<_> = work.into_iter().map(|t| evaluate(&session, t)).collect();
    for value in returns.iter().take(3) {
        session.submit(value.clone()).unwrap();
    }
    let bytes = session.checkpoint().unwrap();
    let mut restored = QmcSession::restore(&bytes, &problem()).unwrap();
    for task in tasks(&mut restored).into_iter().rev() {
        restored.submit(evaluate(&restored, task)).unwrap();
    }
    for value in returns.into_iter().skip(3) {
        session.submit(value).unwrap();
    }
    assert!(restored.is_complete());
    assert_eq!(restored.estimate().unwrap(), session.estimate().unwrap());
    let mut other = problem();
    other.content_id.push_str("-different-numerator");
    assert!(QmcSession::restore(&bytes, &other).is_err());
}

#[test]
fn malformed_cross_integral_periodization_and_duplicate_returns_fail() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let task = session.next_work().unwrap().unwrap();
    let value = evaluate(&session, task.clone());
    session.submit(value.clone()).unwrap();
    assert!(session.submit(value).is_err());
    let mut other_problem = problem();
    other_problem.content_id.push('2');
    let mut other = QmcSession::democratic(other_problem, settings()).unwrap();
    let wrong = other.next_work().unwrap().unwrap();
    assert!(session.submit(evaluate(&other, wrong)).is_err());
    let mut options = settings();
    options.periodization = Periodization::Korobov3;
    let mut transformed = QmcSession::democratic(problem(), options).unwrap();
    let wrong = transformed.next_work().unwrap().unwrap();
    let mut clean = QmcSession::democratic(problem(), settings()).unwrap();
    let _ = clean.next_work().unwrap();
    assert!(clean.submit(evaluate(&transformed, wrong)).is_err());
    let bytes = session.checkpoint().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["settings"]["package_points"] = 0.into();
    assert!(QmcSession::restore(&serde_json::to_vec(&json).unwrap(), &problem()).is_err());
}

#[test]
fn checkpoint_rejects_changed_package_layout_periodization_and_cost_registry() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let task = session.next_work().unwrap().unwrap();
    session.submit(evaluate(&session, task)).unwrap();
    let state: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    for field in ["package_points", "periodization"] {
        let mut altered = state.clone();
        altered["settings"][field] = if field == "package_points" {
            3.into()
        } else {
            "Korobov3".into()
        };
        assert!(QmcSession::restore(&serde_json::to_vec(&altered).unwrap(), &problem()).is_err());
    }
    let mut missing_cost = state.clone();
    missing_cost["runs"][0]["costs"] = serde_json::json!({});
    assert!(QmcSession::restore(&serde_json::to_vec(&missing_cost).unwrap(), &problem()).is_err());
    let mut extra_cost = state;
    extra_cost["runs"][0]["costs"]["16"] = 1.0.into();
    assert!(QmcSession::restore(&serde_json::to_vec(&extra_cost).unwrap(), &problem()).is_err());
}

#[test]
fn failed_worker_can_be_reissued_without_accepting_partial_data() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let task = session.next_work().unwrap().unwrap();
    let mut worker = session.worker_context(task.sector_id()).unwrap();
    assert!(
        worker
            .evaluate(task.clone(), |_, _| Err::<(), _>("kernel failed"))
            .is_err()
    );
    session.retry(&task).unwrap();
    let reissued = tasks(&mut session)
        .into_iter()
        .find(|t| *t == task)
        .unwrap();
    session.submit(evaluate(&session, reissued)).unwrap();
    assert_eq!(session.snapshot().unwrap().completed_points, 8);
}

#[test]
fn cross_sector_and_exact_cancellations_retain_small_contributions() {
    let mut problem = problem();
    problem.exact_coefficients = vec![-1e16, 1e16];
    let mut session = QmcSession::democratic(problem, settings()).unwrap();
    for task in tasks(&mut session) {
        let value = if task.sector_id() == 10 {
            vec![1e16, -1e16]
        } else {
            vec![1.0, 2.0]
        };
        let result = synthetic(&session, task, |_| value.clone(), 1.0);
        session.submit(result).unwrap();
    }
    assert_eq!(session.estimate().unwrap().mean, vec![1.0, 2.0]);
    assert_eq!(session.estimate().unwrap().standard_error, vec![0.0, 0.0]);
}

#[test]
fn exact_problems_finish_without_work_and_tolerances_validate() {
    let exact = IntegrationProblem::new(
        "exact-integral".into(),
        vec![-2, -1, 0],
        vec![],
        vec![2.0, 0.0, -3.0],
    )
    .unwrap();
    let mut session = QmcSession::adaptive(exact, QmcSettings::default()).unwrap();
    assert!(session.next_work().unwrap().is_none());
    assert_eq!(
        session.snapshot().unwrap().uncertainty,
        UncertaintyStatus::Exact
    );
    assert!(
        session
            .estimate()
            .unwrap()
            .meets(Tolerance::new(0.0, 0.0).unwrap())
            .unwrap()
    );
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(Tolerance::new(bad, 0.0).is_err());
        assert!(Tolerance::new(0.0, bad).is_err());
    }
}

#[test]
fn adaptive_pilot_cost_allocation_and_production_are_separate() {
    let mut session = QmcSession::adaptive(problem(), settings()).unwrap();
    assert!(session.recommend_allocation(100.0, 2, &[1.0, 1.0]).is_err());
    for task in tasks(&mut session) {
        let factor = if task.sector_id() == 10 { 1.0 } else { 4.0 };
        let value = synthetic(
            &session,
            task,
            |shift| vec![factor * shift as f64, 0.0],
            factor,
        );
        session.submit(value).unwrap();
    }
    assert!(session.estimate().is_err());
    assert_eq!(
        session.snapshot().unwrap().uncertainty,
        UncertaintyStatus::PilotOnly
    );
    let allocation = session.recommend_allocation(100.0, 2, &[1.0, 1.0]).unwrap();
    // Sector two has sixteen times the variance but four times the cost:
    // its extra shift count is twice that of sector one.
    assert!(((allocation[1].shifts - 2) as i64 - 2 * (allocation[0].shifts - 2) as i64).abs() <= 1);
    assert!(session.recommend_allocation(1.0, 2, &[1.0, 1.0]).is_err());
    let pilot_shifts = session.worker_context(10).unwrap().plan().shifts().to_vec();
    session.freeze_production(allocation).unwrap();
    assert_eq!(session.stage(), IntegrationStage::Production);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
    let first = session.worker_context(10).unwrap();
    let second = session.worker_context(20).unwrap();
    assert_ne!(first.plan().shifts()[0], pilot_shifts[0]);
    assert_ne!(first.plan().shifts()[0][0], second.plan().shifts()[0][0]);
    for task in tasks(&mut session) {
        let value = synthetic(&session, task, |_| vec![5.0, -2.0], 1.0);
        session.submit(value).unwrap();
    }
    assert_eq!(session.estimate().unwrap().mean, vec![11.0, -7.0]);
    assert_eq!(session.estimate().unwrap().standard_error, vec![0.0, 0.0]);
}

#[test]
fn adaptive_covariance_sums_independent_complete_vectors() {
    let mut session = QmcSession::adaptive(problem(), settings()).unwrap();
    for task in tasks(&mut session) {
        let result = synthetic(&session, task, |_| vec![0.0, 0.0], 1.0);
        session.submit(result).unwrap();
    }
    session
        .freeze_production(vec![
            ProductionAllocation {
                sector_id: 10,
                points: 8,
                shifts: 2,
            },
            ProductionAllocation {
                sector_id: 20,
                points: 8,
                shifts: 3,
            },
        ])
        .unwrap();
    for task in tasks(&mut session) {
        let result = synthetic(
            &session,
            task,
            |shift| vec![shift as f64, -2.0 * shift as f64],
            1.0,
        );
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    let covariance = 0.25 + 1.0 / 3.0;
    assert_eq!(estimate.mean, vec![2.5, -6.0]);
    assert_eq!(
        estimate.covariance_of_mean,
        vec![
            covariance,
            -2.0 * covariance,
            -2.0 * covariance,
            4.0 * covariance
        ]
    );
}

#[test]
fn full_support_periodized_vector_matches_analytic_cube_integral() {
    let problem = IntegrationProblem::new(
        "three-dim-product".into(),
        vec![0, 1],
        vec![SectorSpec {
            id: 1,
            dimension: 3,
        }],
        vec![0.0, 0.0],
    )
    .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            shifts: 16,
            ..Default::default()
        },
    )
    .unwrap();
    let mut worker = session.worker_context(1).unwrap();
    while let Some(task) = session.next_work().unwrap() {
        let result = worker
            .evaluate(task, |x, out| {
                out[0] = x.iter().product();
                out[1] = 1.0;
                Ok::<(), String>(())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert!((estimate.mean[0] - 0.125).abs() < 6.0 * estimate.standard_error[0] + 1e-10);
    assert!((estimate.mean[1] - 1.0).abs() < 6.0 * estimate.standard_error[1] + 1e-10);
    assert_eq!(session.snapshot().unwrap().completed_points, 4096 * 16);
}

#[test]
fn caller_threads_and_reversed_arrivals_preserve_output_bits() {
    let mut serial = QmcSession::democratic(problem(), settings()).unwrap();
    let mut parallel = serial.clone();
    for task in tasks(&mut serial) {
        serial.submit(evaluate(&serial, task)).unwrap();
    }
    let returns = std::thread::scope(|scope| {
        let handles: Vec<_> = tasks(&mut parallel)
            .into_iter()
            .map(|task| {
                let mut worker = parallel.worker_context(task.sector_id()).unwrap();
                scope.spawn(move || {
                    let sign = if task.sector_id() == 10 { 1.0 } else { -1.0 };
                    worker
                        .evaluate(task, |x, output| {
                            output[0] = sign * x[0];
                            output[1] = 2.0 * sign * x[0];
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
    for value in returns.into_iter().rev() {
        parallel.submit(value).unwrap();
    }
    assert_eq!(serial.estimate().unwrap(), parallel.estimate().unwrap());
}
