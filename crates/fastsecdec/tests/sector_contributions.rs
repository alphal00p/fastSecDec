//! Numeric-only reporting contracts; these tests do not initialize Symbolica.
use fastsecdec::{
    integration::{
        ContributionReport, IntegrationProblem, Periodization, ProductionAllocation, QmcReturn,
        QmcSession, QmcSettings, QmcTask, ReplicaRelation, RuleSource, SectorSpec,
        mc::{HavanaSession, HavanaSettings, HavanaTask},
    },
    status::{CoefficientComponent, IntegrationStage, UncertaintyStatus},
};
use fastsecdec_qmc::QmcPartial;

fn problem() -> IntegrationProblem {
    IntegrationProblem::new_with_components(
        "additive-sector-diagnostic".into(),
        vec![0, 0],
        vec![CoefficientComponent::Real, CoefficientComponent::Imag],
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
        vec![5.0, -2.0],
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

fn qmc_tasks(session: &mut QmcSession) -> Vec<QmcTask> {
    let mut tasks = Vec::new();
    while let Some(task) = session.next_work().unwrap() {
        tasks.push(task);
    }
    tasks
}

fn submit_qmc(session: &mut QmcSession, task: QmcTask, offset: f64) {
    let worker = session.worker_context(task.sector_id()).unwrap();
    let sign = if task.sector_id() == 10 { 1.0 } else { -1.0 };
    let mut partial = QmcPartial::new(worker.plan(), task.work(), 2).unwrap();
    for point in task.work().start()..task.work().start() + task.point_count() {
        let value = offset + sign * (point / worker.plan().rule().points()) as f64;
        partial.push(&[value, -2.0 * value]).unwrap();
    }
    session
        .submit(QmcReturn::from_partial(task, partial, 0.1).unwrap())
        .unwrap();
}

fn check_additive(report: &ContributionReport, independent: bool) {
    let total = report.total.as_ref().unwrap();
    for (i, &exact) in report.exact_coefficients.iter().enumerate() {
        let sum = exact
            + report
                .sectors
                .iter()
                .map(|s| s.estimate.as_ref().unwrap().mean[i])
                .sum::<f64>();
        assert!((sum - total.mean[i]).abs() < 2e-14);
    }
    if independent {
        for (i, &covariance) in total.covariance_of_mean.iter().enumerate() {
            let sum = report
                .sectors
                .iter()
                .map(|s| s.estimate.as_ref().unwrap().covariance_of_mean[i])
                .sum::<f64>();
            assert!((sum - covariance).abs() < 2e-14);
        }
    }
}

#[test]
fn democratic_marginals_share_complete_support_and_keep_cancellation_covariance() {
    let mut session = QmcSession::democratic(problem(), settings()).unwrap();
    let tasks = qmc_tasks(&mut session);
    for task in tasks.iter().filter(|t| t.sector_id() == 10) {
        submit_qmc(&mut session, task.clone(), 0.0);
    }
    let waiting = session.contributions().unwrap();
    assert_eq!(waiting.uncertainty, UncertaintyStatus::WaitingForCoverage);
    assert!(
        waiting
            .sectors
            .iter()
            .all(|s| s.used_replicas == 0 && s.estimate.is_none())
    );
    let others = tasks
        .into_iter()
        .filter(|t| t.sector_id() == 20)
        .collect::<Vec<_>>();
    submit_qmc(&mut session, others[0].clone(), 0.0);
    let one = session.contributions().unwrap();
    assert!(one.total.is_none());
    assert!(
        one.sectors
            .iter()
            .all(|s| s.used_replicas == 1 && s.used_points == 8 && s.estimate.is_none())
    );
    submit_qmc(&mut session, others[1].clone(), 0.0);
    let partial = session.contributions().unwrap();
    assert_eq!(
        partial.replica_relation,
        ReplicaRelation::SharedAcrossSectors
    );
    assert_eq!(partial.sectors[0].progress.complete_replicas, 4);
    assert_eq!(partial.sectors[0].used_replicas, 2);
    assert_eq!(
        partial.sectors[0].estimate.as_ref().unwrap().mean,
        [0.5, -1.0]
    );
    assert!(
        partial
            .sectors
            .iter()
            .all(|s| !s.estimate.as_ref().unwrap().production_complete)
    );
    assert_eq!(partial.total, Some(session.estimate().unwrap()));
    assert_eq!(partial.total.as_ref().unwrap().mean, [5.0, -2.0]);
    assert_eq!(partial.total.as_ref().unwrap().covariance_of_mean, [0.0; 4]);
    assert!(
        partial
            .sectors
            .iter()
            .all(|s| s.estimate.as_ref().unwrap().standard_error[0] > 0.0)
    );
    check_additive(&partial, false);
    for task in others.into_iter().skip(2) {
        submit_qmc(&mut session, task, 0.0);
    }
    let complete = session.contributions().unwrap();
    assert!(complete.sectors.iter().all(|s| s.used_replicas == 4
        && s.used_points == 32
        && s.estimate.as_ref().unwrap().production_complete));
    assert_eq!(
        complete
            .sectors
            .iter()
            .map(|s| s.progress.completed_points)
            .sum::<u64>(),
        session.snapshot().unwrap().completed_points
    );
    assert_eq!(complete.total, Some(session.estimate().unwrap()));
    check_additive(&complete, false);
}

#[test]
fn adaptive_marginals_exclude_pilot_and_use_independent_local_allocations() {
    let mut session = QmcSession::adaptive(problem(), settings()).unwrap();
    for task in qmc_tasks(&mut session) {
        submit_qmc(&mut session, task, 1000.0);
    }
    let pilot = session.contributions().unwrap();
    assert_eq!(pilot.stage, IntegrationStage::Pilot);
    assert_eq!(pilot.uncertainty, UncertaintyStatus::PilotOnly);
    assert!(pilot.total.is_none());
    assert!(
        pilot
            .sectors
            .iter()
            .all(|s| s.progress.completed_points == 32
                && s.used_points == 0
                && s.estimate.is_none())
    );
    session
        .freeze_production(vec![
            ProductionAllocation {
                sector_id: 10,
                points: 8,
                shifts: 3,
            },
            ProductionAllocation {
                sector_id: 20,
                points: 8,
                shifts: 4,
            },
        ])
        .unwrap();
    let tasks = qmc_tasks(&mut session);
    for task in tasks.iter().filter(|t| t.sector_id() == 10) {
        submit_qmc(&mut session, task.clone(), 0.0);
    }
    let partial = session.contributions().unwrap();
    assert_eq!(
        partial.replica_relation,
        ReplicaRelation::IndependentAcrossSectors
    );
    assert!(partial.total.is_none());
    assert_eq!(partial.sectors[0].used_replicas, 3);
    assert_eq!(
        partial.sectors[0].estimate.as_ref().unwrap().mean,
        [1.0, -2.0]
    );
    assert!(partial.sectors[1].estimate.is_none());
    for task in tasks.into_iter().filter(|t| t.sector_id() == 20) {
        submit_qmc(&mut session, task, 0.0);
    }
    let complete = session.contributions().unwrap();
    assert_eq!(complete.total.as_ref().unwrap().mean, [4.5, -1.0]);
    assert_eq!(complete.total, Some(session.estimate().unwrap()));
    check_additive(&complete, true);
}

fn havana_settings() -> HavanaSettings {
    HavanaSettings {
        points_per_batch: 32,
        batches: 4,
        bins: 4,
        seed: 28,
        minimum_probability_density: 0.02,
    }
}

fn mc_tasks(session: &mut HavanaSession) -> Vec<HavanaTask> {
    let mut result = Vec::new();
    while let Some(task) = session.next_work() {
        result.push(task);
    }
    result
}

fn submit_mc(session: &mut HavanaSession, task: HavanaTask) {
    let value = session
        .worker_context(task.sector_id())
        .unwrap()
        .evaluate(task, |point, output| {
            let v = point.iter().product::<f64>();
            output.copy_from_slice(&[v, -2.0 * v]);
            Ok::<_, String>(())
        })
        .unwrap();
    session.submit(value).unwrap();
}

#[test]
fn havana_marginals_use_completed_independent_batches_and_keep_local_coverage() {
    let mut session = HavanaSession::production(problem(), havana_settings()).unwrap();
    let tasks = mc_tasks(&mut session);
    for task in tasks.iter().filter(|t| t.sector_id() == 10) {
        submit_mc(&mut session, task.clone());
    }
    let partial = session.contributions().unwrap();
    assert_eq!(
        partial.replica_relation,
        ReplicaRelation::IndependentAcrossSectors
    );
    assert!(partial.total.is_none());
    assert!(partial.sectors[0].estimate.is_some());
    assert_eq!(partial.sectors[0].used_points, 128);
    assert_eq!(
        partial.sectors[1].uncertainty,
        UncertaintyStatus::WaitingForCoverage
    );
    for task in tasks.into_iter().filter(|t| t.sector_id() == 20) {
        submit_mc(&mut session, task);
    }
    let complete = session.contributions().unwrap();
    assert_eq!(complete.total, Some(session.estimate().unwrap()));
    check_additive(&complete, true);
    let encoded = serde_json::to_vec(&complete).unwrap();
    let restored: ContributionReport = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(restored, complete);
    // Public deserialization permits caller-constructed records. Presentation
    // must reject an inconsistent layout without indexing past a short vector.
    let mut malformed = serde_json::to_value(&complete).unwrap();
    malformed["sectors"][0]["estimate"]["mean"] = serde_json::json!([]);
    let malformed: ContributionReport = serde_json::from_value(malformed).unwrap();
    assert!(malformed.to_string().contains("invalid coefficient layout"));
    let mut pilot = HavanaSession::pilot(problem(), havana_settings()).unwrap();
    for task in mc_tasks(&mut pilot) {
        submit_mc(&mut pilot, task);
    }
    let report = pilot.contributions().unwrap();
    assert_eq!(report.uncertainty, UncertaintyStatus::PilotOnly);
    assert!(report.total.is_none());
    assert!(report.sectors.iter().all(|s| s.estimate.is_none()
        && s.used_replicas == 0
        && s.progress.complete_replicas == 4));
}

#[test]
fn all_exact_reports_do_not_invent_stochastic_sector_rows() {
    let mut problem = problem();
    problem.sectors.clear();
    let reports = [
        QmcSession::democratic(problem.clone(), settings())
            .unwrap()
            .contributions()
            .unwrap(),
        HavanaSession::production(problem, havana_settings())
            .unwrap()
            .contributions()
            .unwrap(),
    ];
    for report in reports {
        assert_eq!(report.uncertainty, UncertaintyStatus::Exact);
        assert!(report.sectors.is_empty());
        let total = report.total.unwrap();
        assert_eq!(total.mean, report.exact_coefficients);
        assert_eq!(total.covariance_of_mean, vec![0.0; 4]);
        assert!(total.production_complete);
    }
}
