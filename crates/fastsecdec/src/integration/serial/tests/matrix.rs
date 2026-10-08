use super::{evaluate, problem, settings};
use crate::{
    integration::{AccuracyTarget, IntegrationProblem, Tolerance, serial::*},
    status::{CoefficientComponent, UncertaintyStatus},
};
use std::collections::BTreeSet;

#[derive(Default)]
struct Traces {
    first: BTreeSet<Vec<u64>>,
    full: BTreeSet<Vec<u64>>,
}
impl Traces {
    fn evaluate(&mut self, task: &SerialTask) -> (SerialReturn, Vec<f64>) {
        // Serialization and a fresh worker reproduce the resident reload path.
        let task: SerialTask = serde_json::from_slice(&serde_json::to_vec(task).unwrap()).unwrap();
        let (value, coordinates) = evaluate(&task);
        let bits = coordinates.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        assert!(
            self.first.insert(bits[..task.dimension()].to_vec()),
            "fresh reservation repeated another replica's initial point"
        );
        assert!(
            self.full.insert(bits),
            "fresh reservation repeated a full coordinate sequence"
        );
        (value, coordinates)
    }
}
fn pair_with_restart(
    session: &mut SerialSession,
    traces: &mut Traces,
    capacity: usize,
) -> Vec<SerialTask> {
    session.set_max_in_flight(2).unwrap();
    let a = session.reserve(0).unwrap().unwrap();
    let b = session.reserve(0).unwrap().unwrap();
    let (ra, xa) = traces.evaluate(&a);
    let (rb, _) = traces.evaluate(&b);
    // Reverse completion order and simulate confirmed process reclamation.
    session.submit(rb).unwrap();
    session.release(&a).unwrap();
    *session = SerialSession::restore(&session.checkpoint().unwrap(), &problem(1)).unwrap();
    session.set_max_in_flight(capacity).unwrap();
    let retry = session.reserve(0).unwrap().unwrap();
    let (value, repeated) = evaluate(&retry);
    assert_eq!(
        xa, repeated,
        "abandoned work must resume its original stream"
    );
    assert_eq!(a.identity().stream, retry.identity().stream);
    assert_ne!(a.identity().run, retry.identity().run);
    assert_ne!(a.identity().lease, retry.identity().lease);
    assert!(session.submit(ra).is_err());
    session.submit(value.clone()).unwrap();
    assert!(session.submit(value).is_err());
    vec![a, b]
}

#[test]
fn actual_coordinate_matrix_survives_epochs_reverse_returns_reload_and_worker_resize() {
    for mc in [false, true] {
        for adaptive in [false, true] {
            for double in [false, true] {
                let mut options = settings(mc);
                options.max_rounds = Some(2);
                options.double_points = double;
                options.pilot_iterations = 2;
                if adaptive {
                    options.method = match options.method {
                        SerialMethod::Qmc(q) => SerialMethod::AdaptiveQmc(q),
                        SerialMethod::Mc(m) => SerialMethod::AdaptiveMc(m),
                        _ => unreachable!(),
                    };
                }
                let mut session = SerialSession::new(problem(1), options).unwrap();
                let mut traces = Traces::default();
                let mut pilot_pairs = 0;
                let first_production = loop {
                    let tasks = pair_with_restart(&mut session, &mut traces, 1);
                    if tasks[0].identity().pilot {
                        pilot_pairs += 1;
                        assert!(
                            session.estimate().is_err(),
                            "pilot evidence entered production"
                        );
                    } else {
                        break tasks;
                    }
                };
                assert_eq!(pilot_pairs, if adaptive { 2 } else { 0 });
                let accepted = session.estimate().unwrap();
                assert!(accepted.production_complete);
                let previous_epoch = first_production[0].identity().epoch;
                let previous_grid = match &first_production[0].sampler {
                    super::super::job::Sampler::Mc { grid, .. } => {
                        Some(serde_json::to_value(grid).unwrap())
                    }
                    _ => None,
                };
                let last_production = loop {
                    let tasks = pair_with_restart(&mut session, &mut traces, 3);
                    assert_eq!(tasks[0].point_count(), if double { 32 } else { 16 });
                    if tasks[0].identity().pilot {
                        assert!(adaptive && double);
                        assert_eq!(session.estimate().unwrap(), accepted);
                    } else {
                        break tasks;
                    }
                };
                assert!(session.exhausted());
                assert_eq!(
                    session.snapshot().unwrap().sectors[0].replicas,
                    if double { 2 } else { 4 }
                );
                if double {
                    assert!(last_production[0].identity().epoch > previous_epoch);
                } else {
                    assert_eq!(last_production[0].identity().epoch, previous_epoch);
                    if let (Some(previous), super::super::job::Sampler::Mc { grid, training }) =
                        (previous_grid, &last_production[0].sampler)
                    {
                        assert!(!training);
                        assert_eq!(
                            previous,
                            serde_json::to_value(grid).unwrap(),
                            "fixed-size Havana production changed its frozen grid"
                        );
                    }
                }
            }
        }
    }
}

fn constants(task: &SerialTask, values: &[f64]) -> SerialReturn {
    task.evaluate_weighted_batch(3, |_, weights, output| {
        for (weight, row) in weights.iter().zip(output.chunks_exact_mut(values.len())) {
            for (value, &constant) in row.iter_mut().zip(values) {
                *value = *weight * constant;
            }
        }
        Ok::<_, String>(())
    })
    .unwrap()
}

#[test]
fn complex_laurent_covariance_is_full_and_missing_or_pilot_zeros_are_not_exact() {
    let problem = IntegrationProblem::new_with_components(
        "complex-serial-control".into(),
        vec![-1, -1, 0, 0],
        vec![
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
        ],
        problem(2).sectors,
        vec![0.; 4],
    )
    .unwrap();
    let mut session = SerialSession::new(problem, settings(false)).unwrap();
    let samples = [
        [[1., 2., 3., 4.], [3., 6., 7., 8.]],
        [[-1., 3., 2., -4.], [1., 5., 6., 0.]],
    ];
    for (sector, rows) in samples.iter().enumerate() {
        for row in rows {
            let task = session.reserve(sector as u64).unwrap().unwrap();
            session.submit(constants(&task, row)).unwrap();
        }
    }
    let total = session.estimate().unwrap();
    assert_eq!(total.mean, [2., 8., 9., 4.]);
    let references = samples
        .map(|rows| fastsecdec_qmc::QmcEstimate::from_shift_means(&rows.map(Vec::from)).unwrap());
    for (i, &covariance) in total.covariance_of_mean.iter().enumerate() {
        assert_eq!(
            covariance,
            references[0].covariance_of_mean[i] + references[1].covariance_of_mean[i]
        );
    }
    assert!(total.covariance_of_mean[3] != 0.);
    let error = (total.covariance_of_mean[10] + total.covariance_of_mean[15]).sqrt();
    assert!(
        !total
            .meets_target(
                AccuracyTarget::LaurentOrder(0),
                Tolerance::new(error * 0.99, 0.).unwrap()
            )
            .unwrap()
    );
    assert!(
        total
            .meets_target(
                AccuracyTarget::LaurentOrder(0),
                Tolerance::new(error * 1.01, 0.).unwrap()
            )
            .unwrap()
    );

    let mut zero = SerialSession::new(super::problem(2), settings(false)).unwrap();
    for _ in 0..2 {
        let task = zero.reserve(0).unwrap().unwrap();
        zero.submit(constants(&task, &[0., 0.])).unwrap();
    }
    assert!(zero.estimate().is_err());
    assert!(!zero.meets_target().unwrap());
    let observation = zero.observation().unwrap();
    assert_eq!(
        observation.contributions.sectors[0].uncertainty,
        UncertaintyStatus::Available
    );
    assert_eq!(zero.problem().sectors.len(), 2);
    let mut adaptive = settings(true);
    adaptive.method = match adaptive.method {
        SerialMethod::Mc(m) => SerialMethod::AdaptiveMc(m),
        _ => unreachable!(),
    };
    let mut pilot = SerialSession::new(super::problem(1), adaptive).unwrap();
    for _ in 0..2 {
        let task = pilot.reserve(0).unwrap().unwrap();
        pilot.submit(constants(&task, &[0., 0.])).unwrap();
    }
    assert!(pilot.estimate().is_err());
    assert!(!pilot.meets_target().unwrap());
    assert_eq!(
        pilot.observation().unwrap().contributions.sectors[0].uncertainty,
        UncertaintyStatus::PilotOnly
    );
}

#[test]
fn identity_and_refinement_counter_exhaustion_are_transactional() {
    let assert_rejected = |session: &mut SerialSession| {
        let before = session.checkpoint().unwrap();
        assert!(session.reserve(0).is_err());
        assert_eq!(session.checkpoint().unwrap(), before);
    };
    let mut lease = SerialSession::new(problem(1), settings(false)).unwrap();
    lease.next_lease = u64::MAX;
    assert_rejected(&mut lease);
    let mut replica = SerialSession::new(problem(1), settings(false)).unwrap();
    replica.sectors[0].next_replica = u64::MAX;
    replica.sectors[0].target = u64::MAX;
    assert_rejected(&mut replica);
    for exhausted_epoch in [false, true] {
        let mut options = settings(false);
        options.double_points = true;
        let mut session = SerialSession::new(problem(1), options).unwrap();
        for _ in 0..2 {
            let task = session.reserve(0).unwrap().unwrap();
            session.submit(evaluate(&task).0).unwrap();
        }
        if exhausted_epoch {
            session.sectors[0].epoch = u64::MAX;
        } else {
            session.sectors[0].round = usize::MAX;
        }
        assert_rejected(&mut session);
    }
    let mut recovery = SerialSession::new(problem(1), settings(false)).unwrap();
    recovery.run = u64::MAX;
    assert!(SerialSession::restore(&recovery.checkpoint().unwrap(), &problem(1)).is_err());
}
