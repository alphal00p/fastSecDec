//! Numeric-only failure observations; no Symbolica initialization.
use fastsecdec::{
    integration::{
        IntegrationProblem, Periodization, QmcSession, QmcSettings, RuleSource, SectorSpec,
        mc::{HavanaSession, HavanaSettings},
    },
    status::UncertaintyStatus,
};

fn problem(sectors: usize) -> IntegrationProblem {
    IntegrationProblem::new(
        "range-observation".into(),
        vec![0],
        (0..sectors)
            .map(|id| SectorSpec {
                id: id as u64,
                dimension: 1,
            })
            .collect(),
        vec![3.0],
    )
    .unwrap()
}

fn qmc(sectors: usize) -> QmcSession {
    QmcSession::democratic(
        problem(sectors),
        QmcSettings {
            points: 2,
            shifts: 2,
            package_points: 2,
            periodization: Periodization::None,
            rule: RuleSource::Supplied(vec![1]),
            ..QmcSettings::default()
        },
    )
    .unwrap()
}

fn fill(session: &mut QmcSession) {
    while let Some(task) = session.next_work().unwrap() {
        let shift = task.work().start() / 2;
        let sign = if (shift + task.sector_id()).is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let returned = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task, |_, out| {
                out[0] = sign * 1e200;
                Ok::<_, String>(())
            })
            .unwrap();
        session.submit(returned).unwrap();
    }
}

#[test]
fn total_failure_preserves_accepted_qmc_coverage_without_fake_estimate() {
    let mut session = qmc(1);
    fill(&mut session);
    assert!(session.snapshot().is_err());
    assert!(session.contributions().is_err());
    let observed = session.diagnostic_observation().unwrap();
    assert_eq!(observed.snapshot.completed_points, 4);
    assert!(observed.snapshot.estimate.is_none());
    assert!(matches!(
        observed.snapshot.uncertainty,
        UncertaintyStatus::StatisticalFailure { .. }
    ));
    let row = &observed.contributions.sectors[0];
    assert_eq!(
        (
            row.progress.complete_replicas,
            row.used_replicas,
            row.used_points
        ),
        (2, 2, 4)
    );
    assert!(row.estimate.is_none());
}

#[test]
fn correlated_huge_marginals_do_not_destroy_valid_total_cancellation() {
    let mut session = qmc(2);
    fill(&mut session);
    assert_eq!(session.estimate().unwrap().mean, vec![3.0]);
    assert!(session.contributions().is_err());
    let observed = session.diagnostic_observation().unwrap();
    assert_eq!(observed.snapshot.uncertainty, UncertaintyStatus::Available);
    let total = observed.contributions.total.unwrap();
    assert_eq!(total.mean, vec![3.0]);
    assert_eq!(total.standard_error, vec![0.0]);
    assert!(
        observed
            .contributions
            .sectors
            .iter()
            .all(|r| r.estimate.is_none()
                && matches!(r.uncertainty, UncertaintyStatus::StatisticalFailure { .. }))
    );
}

#[test]
fn accepted_havana_batches_remain_visible_when_batch_covariance_overflows() {
    let mut session = HavanaSession::production(
        problem(1),
        HavanaSettings {
            points_per_batch: 2,
            batches: 2,
            ..HavanaSettings::default()
        },
    )
    .unwrap();
    while let Some(task) = session.next_work() {
        let sign = if task.batch() == 0 { 1.0 } else { -1.0 };
        let returned = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate_weighted(task, |_, _, out| {
                out[0] = sign * 1e200;
                Ok::<_, String>(())
            })
            .unwrap();
        session.submit(returned).unwrap();
    }
    assert!(session.contributions().is_err());
    let observed = session.diagnostic_observation().unwrap();
    assert_eq!(observed.snapshot.completed_points, 4);
    assert!(matches!(
        observed.contributions.uncertainty,
        UncertaintyStatus::StatisticalFailure { .. }
    ));
    assert_eq!(observed.contributions.sectors[0].used_points, 4);
}
