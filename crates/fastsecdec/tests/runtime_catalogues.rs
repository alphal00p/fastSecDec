use fastsecdec::integration::{
    IntegrationProblem, ProductionAllocation, PublishedLattice, QmcSession, QmcSettings,
    RuleSource, SectorSpec,
};

fn problem(dimension: usize) -> IntegrationProblem {
    IntegrationProblem::new(
        "catalogue-regression".into(),
        vec![0],
        vec![SectorSpec { id: 0, dimension }],
        vec![0.],
    )
    .unwrap()
}
fn settings() -> QmcSettings {
    QmcSettings {
        points: 1024,
        shifts: 2,
        package_points: 1024,
        rule: RuleSource::Published(PublishedLattice::HkknAlpha3),
        ..Default::default()
    }
}

#[test]
fn published_setting_checks_exact_only_counts_and_full_sector_dimensions() {
    let exact = IntegrationProblem::new("exact".into(), vec![0], vec![], vec![3.]).unwrap();
    let mut options = settings();
    options.points = 1025;
    assert!(options.validate().is_err());
    assert!(QmcSession::democratic(exact, options).is_err());
    assert!(QmcSession::democratic(problem(11), settings()).is_err());
    assert!(QmcSession::democratic(problem(10), settings()).is_ok());
    assert_eq!(
        serde_json::to_string(&RuleSource::default()).unwrap(),
        "\"Kuo\""
    );
}

#[test]
fn checkpoint_preserves_catalogue_and_reissues_its_missing_work() {
    let problem = problem(5);
    let mut session = QmcSession::democratic(problem.clone(), settings()).unwrap();
    let first = session.next_work().unwrap().unwrap();
    let mut worker = session.worker_context(0).unwrap();
    let result = worker
        .evaluate(first, |_, out| {
            out[0] = 1.;
            Ok::<_, String>(())
        })
        .unwrap();
    session.submit(result).unwrap();
    let mut restored = QmcSession::restore(&session.checkpoint().unwrap(), &problem).unwrap();
    assert_eq!(restored.design(), session.design());
    assert_eq!(
        restored.worker_context(0).unwrap().plan().rule().source(),
        numerica::numerical_integration::qmc::RuleSource::HkknAlpha3
    );
    while let Some(task) = restored.next_work().unwrap() {
        let result = worker
            .evaluate(task, |_, out| {
                out[0] = 1.;
                Ok::<_, String>(())
            })
            .unwrap();
        restored.submit(result).unwrap();
    }
    assert!(restored.is_complete());
    let mut value: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    value["settings"]["rule"] = serde_json::json!({"Published":"Kuo38005"});
    assert!(QmcSession::restore(&serde_json::to_vec(&value).unwrap(), &problem).is_err());
}

#[test]
fn adaptive_allocations_cannot_exceed_the_selected_catalogue_transactionally() {
    let mut session = QmcSession::adaptive(problem(5), settings()).unwrap();
    let mut worker = session.worker_context(0).unwrap();
    while let Some(task) = session.next_work().unwrap() {
        let result = worker
            .evaluate(task, |_, out| {
                out[0] = 1.;
                Ok::<_, String>(())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let before = session.checkpoint().unwrap();
    assert!(
        session
            .freeze_production(vec![ProductionAllocation {
                sector_id: 0,
                points: 1 << 21,
                shifts: 2
            }])
            .is_err()
    );
    assert_eq!(session.checkpoint().unwrap(), before);
    session
        .freeze_production(vec![ProductionAllocation {
            sector_id: 0,
            points: 2048,
            shifts: 2,
        }])
        .unwrap();
    let design = session.design();
    assert_eq!(design.settings.points, 1024);
    assert_eq!(design.allocations[0].points, 2048);
    assert_eq!(design.allocations[0].shifts, 2);
    assert_eq!(
        session.worker_context(0).unwrap().plan().rule().source(),
        numerica::numerical_integration::qmc::RuleSource::HkknAlpha3
    );
}
