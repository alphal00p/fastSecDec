use super::*;
use crate::integration::{Periodization, QmcSettings, RuleSource, SectorSpec};

#[test]
fn nonconsecutive_common_shifts_preserve_covariance_after_reversed_returns() {
    let problem = IntegrationProblem::new(
        "common-shift-gaps".into(),
        vec![-1, 0],
        vec![
            SectorSpec {
                id: 0,
                dimension: 2,
            },
            SectorSpec {
                id: 1,
                dimension: 2,
            },
        ],
        vec![17., -3.],
    )
    .unwrap();
    let settings = QmcSettings {
        points: 16,
        shifts: 8,
        seed: 987,
        package_points: 16,
        rule: RuleSource::Supplied(vec![1, 3]),
        periodization: Periodization::None,
    };
    let mut session = QmcSession::democratic(problem.clone(), settings).unwrap();
    let mut returns = Vec::new();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id();
        returns.push(
            session
                .worker_context(id)
                .unwrap()
                .evaluate(task, |point, output| {
                    output[0] = if id == 0 { point[0] } else { 2. * point[0] };
                    output[1] = if id == 0 { point[1] } else { -point[1] };
                    Ok::<_, String>(())
                })
                .unwrap(),
        );
    }
    let mut common_only = session.clone();
    for value in &returns {
        if [2, 3, 7].contains(&(value.task().work().start() / 16)) {
            common_only.submit(value.clone()).unwrap();
        }
    }
    for value in returns.into_iter().rev() {
        let ids = if value.task().sector_id() == 0 {
            [0, 2, 3, 5, 7]
        } else {
            [1, 2, 3, 6, 7]
        };
        if ids.contains(&(value.task().work().start() / 16)) {
            session.submit(value).unwrap();
        }
    }
    assert_eq!(
        session
            .common_shift_rows()
            .unwrap()
            .iter()
            .map(|rows| rows.iter().map(|row| row.shift).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![2, 2], vec![3, 3], vec![7, 7]]
    );
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate, common_only.estimate().unwrap());
    assert_eq!(estimate.mean[1], -3.);
    assert!(estimate.covariance_of_mean[0] > 0.);
    assert_eq!(&estimate.covariance_of_mean[1..], &[0.; 3]);
    let report = session.diagnostic_observation().unwrap();
    assert!(
        report
            .contributions
            .sectors
            .iter()
            .all(|row| row.used_replicas == 3)
    );
    assert_eq!(report.contributions.total, Some(estimate.clone()));
    let restored = QmcSession::restore(&session.checkpoint().unwrap(), &problem).unwrap();
    assert_eq!(restored.estimate().unwrap(), estimate);
    assert_eq!(restored.complete_shift_estimates().unwrap().len(), 3);
}
