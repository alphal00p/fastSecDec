use fastsecdec::{
    integration::{
        CoefficientComponent, IntegrationProblem, ReplicaRelation, SectorSpec,
        mc::HavanaSettings,
        mc_discrete::{HavanaDiscreteSession, HavanaDiscreteSettings, HavanaDiscreteTask},
    },
    status::{IntegrationStage, UncertaintyStatus},
};
fn problem() -> IntegrationProblem {
    IntegrationProblem::new_with_components(
        "discrete-scientific-fixture".into(),
        vec![-1, -1, 0, 0],
        vec![
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
        ],
        vec![
            SectorSpec {
                id: 4,
                dimension: 1,
            },
            SectorSpec {
                id: 9,
                dimension: 2,
            },
        ],
        vec![0.5, 0.0, 0.0, 0.0],
    )
    .unwrap()
}
fn settings() -> HavanaDiscreteSettings {
    HavanaDiscreteSettings {
        batch: HavanaSettings {
            points_per_batch: 2048,
            batches: 24,
            seed: 791,
            bins: 8,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn tasks(session: &mut HavanaDiscreteSession) -> Vec<HavanaDiscreteTask> {
    std::iter::from_fn(|| session.next_work()).collect()
}
fn callback(id: u64, x: &[f64], out: &mut [f64]) -> Result<(), String> {
    let value = if id == 4 { x[0] } else { 3.0 * x[0] * x[1] };
    out.copy_from_slice(&[value, -2.0 * value, 0.0, 3.0 * value]);
    Ok(())
}
fn finish(session: &mut HavanaDiscreteSession, reverse: bool) {
    let mut results = tasks(session)
        .into_iter()
        .map(|task| {
            session
                .worker_context()
                .unwrap()
                .evaluate(task, callback)
                .unwrap()
        })
        .collect::<Vec<_>>();
    if reverse {
        results.reverse();
    }
    for r in results {
        session.submit(r).unwrap();
    }
}
#[test]
fn nonuniform_actual_sampling_is_unbiased_and_retains_full_shared_covariance() {
    let mut s = HavanaDiscreteSession::production(problem(), settings())
        .unwrap()
        .with_sector_probabilities(&[0.2, 0.8])
        .unwrap();
    finish(&mut s, false);
    let o = s.diagnostic_observation().unwrap();
    let est = o.snapshot.estimate.unwrap();
    let count = o.snapshot.sectors[0].completed_points as f64;
    let n = o.snapshot.completed_points as f64;
    assert!((count / n - 0.2).abs() < 0.012);
    assert_eq!(
        o.contributions.replica_relation,
        ReplicaRelation::SharedAcrossSectors
    );
    assert!(
        o.snapshot
            .sectors
            .iter()
            .all(|row| row.planned_points.is_none() && row.discrete_allocation.is_some())
    );
    for (actual, expected, error) in est
        .mean
        .iter()
        .zip([1.75, -2.5, 0.0, 3.75])
        .zip(&est.standard_error)
        .map(|((a, b), c)| (a, b, c))
    {
        assert!(
            (*actual - expected).abs() <= 6.0 * error + 1e-13,
            "{actual} vs {expected} ±{error}"
        );
    }
    assert!(est.covariance_of_mean[1] < 0.0);
    assert!(est.covariance_of_mean[3] > 0.0);
    assert!((est.covariance_of_mean[5] - 4.0 * est.covariance_of_mean[0]).abs() < 1e-14);
    let marginal_sum = o
        .contributions
        .sectors
        .iter()
        .map(|r| r.estimate.as_ref().unwrap().covariance_of_mean[0])
        .sum::<f64>();
    assert!((marginal_sum - est.covariance_of_mean[0]).abs() > 1e-7);
}
#[test]
fn pilot_exclusion_independent_streams_and_out_of_order_reproducibility() {
    let mut a = HavanaDiscreteSession::pilot(problem(), settings()).unwrap();
    let mut b = a.clone();
    finish(&mut a, false);
    finish(&mut b, true);
    assert_eq!(
        a.snapshot().unwrap().uncertainty,
        UncertaintyStatus::PilotOnly
    );
    assert!(a.estimate().is_err());
    assert!(a.checkpoint().is_err());
    a.adapt_pilot(0.5, 0.5).unwrap();
    b.adapt_pilot(0.5, 0.5).unwrap();
    assert_eq!(a.sector_probabilities(), b.sector_probabilities());
    assert!((a.sector_probabilities()[0].1 - 0.5).abs() > 0.01);
    finish(&mut a, false);
    finish(&mut b, true);
    a.freeze_production(0.5, 0.5, 1024, 12).unwrap();
    b.freeze_production(0.5, 0.5, 1024, 12).unwrap();
    assert_eq!(a.stage(), IntegrationStage::Production);
    assert_eq!(a.snapshot().unwrap().completed_points, 0);
    finish(&mut a, false);
    finish(&mut b, true);
    assert_eq!(a.estimate().unwrap(), b.estimate().unwrap());
}
#[test]
fn failed_batch_retry_checkpoint_and_worker_count_change_preserve_result() {
    let mut s = HavanaDiscreteSession::production(problem(), settings()).unwrap();
    let first = s.next_work().unwrap();
    let mut worker = s.worker_context().unwrap();
    assert!(
        worker
            .evaluate(first.clone(), |_, _, _: &mut [f64]| Err::<(), _>(
                "deliberate failure"
            ))
            .is_err()
    );
    assert_eq!(s.snapshot().unwrap().completed_points, 0);
    s.retry(&first).unwrap();
    let again = s.next_work().unwrap();
    assert_eq!(first, again);
    let value = worker.evaluate(again.clone(), callback).unwrap();
    s.submit(value.clone()).unwrap();
    assert!(s.submit(value).is_err());
    assert!(s.retry(&again).is_err());
    let pending = s.next_work().unwrap();
    let bytes = s.checkpoint().unwrap();
    let mut resumed = HavanaDiscreteSession::restore(&bytes, &problem()).unwrap();
    s.retry(&pending).unwrap();
    finish(&mut s, false);
    finish(&mut resumed, true);
    assert_eq!(s.estimate().unwrap(), resumed.estimate().unwrap());
    let mut wrong = problem();
    wrong.content_id.push('!');
    assert!(HavanaDiscreteSession::restore(&bytes, &wrong).is_err());
}
#[test]
fn weighted_callback_applies_native_sector_weight_once_and_zero_hit_batches_are_valid() {
    let mut s = HavanaDiscreteSession::production(problem(), settings())
        .unwrap()
        .with_sector_probabilities(&[1e-15, 1.0 - 1e-15])
        .unwrap();
    for task in tasks(&mut s) {
        let value = s
            .worker_context()
            .unwrap()
            .evaluate_weighted(task, |_, _, weight, out| {
                assert!(weight > 0.0);
                out.copy_from_slice(&[1.0, -2.0, 0.0, 3.0]);
                Ok::<(), String>(())
            })
            .unwrap();
        s.submit(value).unwrap();
    }
    let e = s.estimate().unwrap();
    assert_eq!(e.mean, vec![1.5, -2.0, 0.0, 3.0]);
    assert_eq!(e.standard_error, vec![0.0; 4]);
    let c = s.contributions().unwrap();
    assert_eq!(c.sectors[0].used_points, 0);
    assert_eq!(c.sectors[0].uncertainty, UncertaintyStatus::Available);
    assert_eq!(c.sectors[0].used_replicas, 24);
}

#[test]
fn stale_epoch_and_tampered_checkpoint_are_rejected_without_accepting_work() {
    let mut session = HavanaDiscreteSession::pilot(problem(), settings()).unwrap();
    let old_task = session.next_work().unwrap();
    let old_return = session
        .worker_context()
        .unwrap()
        .evaluate(old_task.clone(), callback)
        .unwrap();
    session.submit(old_return.clone()).unwrap();
    finish(&mut session, false);
    session.freeze_production(0.5, 0.5, 1024, 4).unwrap();
    assert!(session.submit(old_return).is_err());
    assert!(session.retry(&old_task).is_err());
    assert_eq!(session.snapshot().unwrap().completed_points, 0);
    finish(&mut session, false);
    let bytes = session.checkpoint().unwrap();
    let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut invalid = original.clone();
    invalid["proposal"][0][0] = serde_json::json!(0.0);
    assert!(
        HavanaDiscreteSession::restore(&serde_json::to_vec(&invalid).unwrap(), &problem()).is_err()
    );
    let mut invalid = original.clone();
    invalid["seeds"][1] = invalid["seeds"][0].clone();
    assert!(
        HavanaDiscreteSession::restore(&serde_json::to_vec(&invalid).unwrap(), &problem()).is_err()
    );
    let mut invalid = original;
    invalid["records"][0]["counts"][0] = serde_json::json!(9999999);
    assert!(
        HavanaDiscreteSession::restore(&serde_json::to_vec(&invalid).unwrap(), &problem()).is_err()
    );
}
