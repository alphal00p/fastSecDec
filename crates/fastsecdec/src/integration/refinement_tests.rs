//! Audit actual coordinates and covariance when appending frozen production.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn problem() -> IntegrationProblem {
    IntegrationProblem::new(
        "ordinary-append".into(),
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
        vec![0., 0.],
    )
    .unwrap()
}
fn qmc_settings() -> QmcSettings {
    QmcSettings {
        points: 16,
        shifts: 2,
        seed: 765,
        package_points: 7,
        rule: RuleSource::Supplied(vec![1, 3]),
        periodization: Periodization::None,
    }
}
fn qmc_finish(session: &mut QmcSession) -> BTreeMap<u64, Vec<(u64, Vec<u64>)>> {
    let mut returns = vec![];
    let mut samples = BTreeMap::<_, Vec<_>>::new();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id();
        let start = task.work().start();
        let mut coordinates = vec![];
        let value = session
            .worker_context(id)
            .unwrap()
            .evaluate(task, |x, out| {
                coordinates.extend(x.iter().map(|v| v.to_bits()));
                let sign = if id == 0 { 1. } else { -1. };
                out[0] = sign * x[0];
                out[1] = sign * (x[0] * x[0] + x[1]);
                Ok::<_, String>(())
            })
            .unwrap();
        samples.entry(id).or_default().push((start, coordinates));
        returns.push(value);
    }
    for value in returns.into_iter().rev() {
        session.submit(value).unwrap();
    }
    samples
}
#[test]
fn ordinary_qmc_appends_only_new_shift_coordinates_and_keeps_shared_covariance() {
    let mut session = QmcSession::democratic(problem(), qmc_settings()).unwrap();
    let first = qmc_finish(&mut session);
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.mean, [0., 0.]);
    assert_eq!(estimate.covariance_of_mean, [0.; 4]);
    session.extend_production_shifts(2).unwrap();
    assert_eq!(session.estimate().unwrap().mean, estimate.mean);
    let mut session = QmcSession::restore(&session.checkpoint().unwrap(), &problem()).unwrap();
    let appended = qmc_finish(&mut session);
    assert_eq!(
        appended[&0], appended[&1],
        "democratic sector coordinates intentionally stay shared"
    );
    assert!(appended.values().flatten().all(|(start, _)| *start >= 32));
    let initial_points = first[&0]
        .iter()
        .flat_map(|(_, x)| x.chunks_exact(2).map(|v| v.to_vec()))
        .collect::<BTreeSet<_>>();
    assert!(
        appended[&0]
            .iter()
            .flat_map(|(_, x)| x.chunks_exact(2))
            .all(|x| !initial_points.contains(x))
    );
    assert_eq!(session.estimate().unwrap().covariance_of_mean, [0.; 4]);
    assert_eq!(session.complete_shift_estimates().unwrap().len(), 4);
    let restored = QmcSession::restore(&session.checkpoint().unwrap(), &problem()).unwrap();
    assert!(restored.is_complete());
}
#[test]
fn ordinary_adaptive_qmc_appends_without_repeating_its_pilot() {
    let mut session = QmcSession::adaptive(problem(), qmc_settings()).unwrap();
    qmc_finish(&mut session);
    session
        .freeze_production(vec![
            ProductionAllocation {
                sector_id: 0,
                points: 16,
                shifts: 2,
            },
            ProductionAllocation {
                sector_id: 1,
                points: 16,
                shifts: 3,
            },
        ])
        .unwrap();
    qmc_finish(&mut session);
    session.extend_production_shifts(2).unwrap();
    assert_eq!(session.stage(), crate::status::IntegrationStage::Production);
    assert_eq!(
        session
            .design()
            .allocations
            .iter()
            .map(|a| a.shifts)
            .collect::<Vec<_>>(),
        [4, 6]
    );
    let samples = qmc_finish(&mut session);
    assert!(samples[&0].iter().all(|(start, _)| *start >= 32));
    assert!(samples[&1].iter().all(|(start, _)| *start >= 48));
    assert!(session.estimate().unwrap().production_complete);
}
fn mc_settings() -> mc::HavanaSettings {
    mc::HavanaSettings {
        points_per_batch: 16,
        batches: 2,
        seed: 783,
        bins: 4,
        ..Default::default()
    }
}
fn mc_finish(session: &mut mc::HavanaSession) -> Vec<Vec<u64>> {
    let mut returns = vec![];
    let mut sequences = vec![];
    while let Some(task) = session.next_work() {
        let mut points = vec![];
        let value = session
            .worker_context(task.sector_id())
            .unwrap()
            .evaluate(task, |x, out| {
                points.extend(x.iter().map(|v| v.to_bits()));
                out[0] = x[0];
                out[1] = 2. * x[0];
                Ok::<_, String>(())
            })
            .unwrap();
        sequences.push(points);
        returns.push(value);
    }
    for value in returns.into_iter().rev() {
        session.submit(value).unwrap();
    }
    sequences
}
#[test]
fn ordinary_havana_appends_independent_batches_on_the_frozen_grid_after_restore() {
    let mut session = mc::HavanaSession::pilot(problem(), mc_settings()).unwrap();
    let pilot = mc_finish(&mut session);
    session.freeze_production(0.5, 16, 2).unwrap();
    let first = mc_finish(&mut session);
    let previous = session.estimate().unwrap();
    let before: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    session.extend_production_batches(4).unwrap();
    assert_eq!(session.estimate().unwrap().mean, previous.mean);
    let after: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    for index in 0..2 {
        assert_eq!(
            before["sectors"][index]["partitions"],
            after["sectors"][index]["partitions"]
        );
    }
    let mut resumed =
        mc::HavanaSession::restore(&session.checkpoint().unwrap(), &problem()).unwrap();
    let second = mc_finish(&mut resumed);
    let sequences = pilot
        .into_iter()
        .chain(first)
        .chain(second)
        .collect::<Vec<_>>();
    assert_eq!(sequences.len(), 12);
    assert_eq!(
        sequences.iter().collect::<BTreeSet<_>>().len(),
        sequences.len()
    );
    let estimate = resumed.estimate().unwrap();
    assert!((estimate.covariance_of_mean[1] - 2. * estimate.covariance_of_mean[0]).abs() < 1e-14);
    let mut corrupt = after.clone();
    corrupt["streams"]["next"] = u64::MAX.into();
    assert!(
        mc::HavanaSession::restore(&serde_json::to_vec(&corrupt).unwrap(), &problem()).is_err()
    );
    let mut corrupt = after;
    corrupt["streams"]["state"] = corrupt["sectors"][0]["seeds"][0].clone();
    assert!(
        mc::HavanaSession::restore(&serde_json::to_vec(&corrupt).unwrap(), &problem()).is_err()
    );
}
fn discrete_finish(session: &mut mc_discrete::HavanaDiscreteSession) -> Vec<Vec<u64>> {
    let mut returns = vec![];
    let mut sequences = vec![];
    while let Some(task) = session.next_work() {
        let mut points = vec![];
        let value = session
            .worker_context()
            .unwrap()
            .evaluate(task, |id, x, out| {
                points.push(id);
                points.extend(x.iter().map(|v| v.to_bits()));
                out[0] = (id + 1) as f64 * x[0];
                out[1] = 2. * out[0];
                Ok::<_, String>(())
            })
            .unwrap();
        sequences.push(points);
        returns.push(value);
    }
    for value in returns.into_iter().rev() {
        session.submit(value).unwrap();
    }
    sequences
}
#[test]
fn ordinary_discrete_havana_appends_without_retraining_the_global_proposal() {
    let settings = mc_discrete::HavanaDiscreteSettings {
        batch: mc_settings(),
        ..Default::default()
    };
    let mut session = mc_discrete::HavanaDiscreteSession::pilot(problem(), settings).unwrap();
    let pilot = discrete_finish(&mut session);
    session.freeze_production(0.5, 0.5, 16, 2).unwrap();
    let first = discrete_finish(&mut session);
    let before: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    let previous = session.estimate().unwrap();
    session.extend_production_batches(4).unwrap();
    assert_eq!(session.estimate().unwrap().mean, previous.mean);
    let after: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    assert_eq!(before["proposal"], after["proposal"]);
    let mut restored =
        mc_discrete::HavanaDiscreteSession::restore(&session.checkpoint().unwrap(), &problem())
            .unwrap();
    let second = discrete_finish(&mut restored);
    let sequences = pilot
        .into_iter()
        .chain(first)
        .chain(second)
        .collect::<Vec<_>>();
    assert_eq!(sequences.len(), 6);
    assert_eq!(sequences.iter().collect::<BTreeSet<_>>().len(), 6);
    assert!(restored.estimate().unwrap().production_complete);
    let mut corrupt = after;
    corrupt["streams"]["state"] = corrupt["seeds"][0].clone();
    assert!(
        mc_discrete::HavanaDiscreteSession::restore(
            &serde_json::to_vec(&corrupt).unwrap(),
            &problem()
        )
        .is_err()
    );
}
