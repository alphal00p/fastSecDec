use super::*;
use crate::integration::{
    AccuracyTarget, IntegrationProblem, Periodization, QmcSettings, RuleSource, SectorSpec,
    Tolerance, mc::HavanaSettings,
};
use fastsecdec_qmc::QmcEstimate;
use numerica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};
use rand::RngCore;

mod matrix;

fn problem(sectors: usize) -> IntegrationProblem {
    IntegrationProblem::new(
        "serial-probe".into(),
        vec![-1, 0],
        (0..sectors)
            .map(|i| SectorSpec {
                id: i as u64,
                dimension: 2,
            })
            .collect(),
        vec![0., 0.],
    )
    .unwrap()
}
fn settings(mc: bool) -> SerialSettings {
    SerialSettings {
        method: if mc {
            SerialMethod::Mc(HavanaSettings {
                points_per_batch: 16,
                batches: 2,
                bins: 4,
                seed: 42,
                ..Default::default()
            })
        } else {
            SerialMethod::Qmc(QmcSettings {
                points: 16,
                shifts: 2,
                seed: 42,
                periodization: Periodization::None,
                rule: RuleSource::Supplied(vec![1, 3]),
                ..Default::default()
            })
        },
        double_points: false,
        max_rounds: None,
        max_in_flight: 2,
        target: AccuracyTarget::LaurentOrder(0),
        tolerance: Tolerance::new(0., 0.).unwrap(),
        pilot_iterations: 1,
        learning_rate: 0.5,
    }
}
fn evaluate(task: &SerialTask) -> (SerialReturn, Vec<f64>) {
    let mut points = Vec::new();
    let value = task
        .evaluate_weighted_batch(3, |x, w, out| {
            points.extend_from_slice(x);
            for ((point, weight), row) in x.chunks_exact(2).zip(w).zip(out.chunks_exact_mut(2)) {
                row[0] = point[0] * weight;
                row[1] = (point[0] * point[0] + point[1]) * weight;
            }
            Ok::<_, String>(())
        })
        .unwrap();
    (value, points)
}
#[test]
fn centered_vector_moments_reuse_native_numeric_precision_and_keep_covariance() {
    let mut moments = ReplicaMoments::new(2).unwrap();
    let rows = vec![
        vec![1e16, 1.],
        vec![1e16 + 2., 3.],
        vec![1e16 + 4., 5.],
        vec![1e16 + 6., 7.],
    ];
    for row in &rows {
        moments.add(row).unwrap();
    }
    let value = moments.estimate(&problem(0), true).unwrap();
    let expected = QmcEstimate::from_shift_means(&rows).unwrap();
    assert_eq!(value.mean, expected.mean);
    for (a, b) in value
        .covariance_of_mean
        .iter()
        .zip(expected.covariance_of_mean)
    {
        assert!((a - b).abs() < 1e-14);
    }
    let before = serde_json::to_vec(&moments).unwrap();
    assert!(moments.add(&[f64::NAN, 0.]).is_err());
    assert_eq!(serde_json::to_vec(&moments).unwrap(), before);
    let restored: ReplicaMoments = serde_json::from_slice(&before).unwrap();
    assert_eq!(restored.estimate(&problem(0), true).unwrap(), value);
}
#[test]
fn actual_coordinate_sequences_are_distinct_retries_are_identical_and_stale_leases_reject() {
    for mc in [false, true] {
        let mut session = SerialSession::new(problem(1), settings(mc)).unwrap();
        let a = session.reserve(0).unwrap().unwrap();
        let b = session.reserve(0).unwrap().unwrap();
        assert!(session.reserve(0).unwrap().is_none());
        let (ra, xa) = evaluate(&a);
        let (rb, xb) = evaluate(&b);
        assert_ne!(
            xa, xb,
            "two same-sector workers repeated a complete coordinate sequence"
        );
        session.release(&a).unwrap();
        let retry = session.reserve(0).unwrap().unwrap();
        assert_eq!(a.identity().stream, retry.identity().stream);
        assert_ne!(a.identity().lease, retry.identity().lease);
        let (retried, xretry) = evaluate(&retry);
        assert_eq!(xa, xretry);
        assert!(session.submit(ra).is_err());
        session.submit(rb.clone()).unwrap();
        assert!(session.submit(rb).is_err());
        session.submit(retried).unwrap();
        let next = session.reserve(0).unwrap().unwrap();
        let (_, xn) = evaluate(&next);
        assert_ne!(xa, xn);
        assert_ne!(xb, xn);
    }
}
#[test]
fn restart_changes_run_fence_but_preserves_exact_work_and_allows_smaller_pool() {
    for mc in [false, true] {
        let mut session = SerialSession::new(problem(1), settings(mc)).unwrap();
        let a = session.reserve(0).unwrap().unwrap();
        let b = session.reserve(0).unwrap().unwrap();
        let (old, xa) = evaluate(&a);
        let mut restored =
            SerialSession::restore(&session.checkpoint().unwrap(), &problem(1)).unwrap();
        restored.set_max_in_flight(1).unwrap();
        assert_eq!(restored.choose_sector(None, 0., 1., &[]).unwrap(), Some(0));
        let retry = restored.reserve(0).unwrap().unwrap();
        assert_eq!(retry.identity().stream, a.identity().stream);
        assert_ne!(retry.identity().run, a.identity().run);
        assert!(restored.reserve(0).unwrap().is_none());
        let (r, x) = evaluate(&retry);
        assert_eq!(x, xa);
        assert!(restored.submit(old).is_err());
        restored.submit(r).unwrap();
        let next = restored.reserve(0).unwrap().unwrap();
        assert_eq!(next.identity().stream, b.identity().stream);
        restored.submit(evaluate(&next).0).unwrap();
        let fresh = restored.reserve(0).unwrap().unwrap();
        assert!(fresh.identity().stream > b.identity().stream);
    }
}
#[test]
fn fixed_refinement_appends_and_size_growth_replaces_homogeneous_moments() {
    for mc in [false, true] {
        for double in [false, true] {
            let mut opts = settings(mc);
            opts.double_points = double;
            opts.max_rounds = Some(2);
            let mut session = SerialSession::new(problem(1), opts).unwrap();
            for _ in 0..2 {
                let task = session.reserve(0).unwrap().unwrap();
                session.submit(evaluate(&task).0).unwrap();
            }
            let previous = session.estimate().unwrap();
            assert!(previous.production_complete);
            let task = session.reserve(0).unwrap().unwrap();
            let state = session.snapshot().unwrap();
            assert_eq!(task.point_count(), if double { 32 } else { 16 });
            assert_eq!(state.sectors[0].replicas, if double { 0 } else { 2 });
            assert_eq!(state.sectors[0].previous.as_ref(), Some(&previous));
            session.submit(evaluate(&task).0).unwrap();
            let task = session.reserve(0).unwrap().unwrap();
            session.submit(evaluate(&task).0).unwrap();
            assert!(session.exhausted());
            assert!(session.reserve(0).unwrap().is_none());
            assert!(!session.meets_target().unwrap());
        }
    }
}
#[test]
fn resumed_smaller_pool_prioritizes_unissued_reservations_over_fresh_work() {
    let mut session = SerialSession::new(problem(3), settings(false)).unwrap();
    for sector in 0..3 {
        for _ in 0..2 {
            let task = session.reserve(sector).unwrap().unwrap();
            session.submit(evaluate(&task).0).unwrap();
        }
        session
            .choose_sector(Some(sector), 1., 1., &[sector])
            .unwrap();
    }
    assert!(session.first_coverage_complete());
    // The remaining fresh sector is intentionally by far the noisiest.
    session.sectors[2].moments = ReplicaMoments::new(2).unwrap();
    session.sectors[2].moments.add(&[0., -1e30]).unwrap();
    session.sectors[2].moments.add(&[0., 1e30]).unwrap();
    session.reserve(0).unwrap().unwrap();
    session.reserve(1).unwrap().unwrap();
    let mut restored = SerialSession::restore(&session.checkpoint().unwrap(), &problem(3)).unwrap();
    restored.set_max_in_flight(1).unwrap();
    for _ in 0..2 {
        let selected = restored.choose_sector(None, 0., 1., &[]).unwrap().unwrap();
        assert!(selected < 2, "fresh work displaced a fenced pending retry");
        let task = restored.reserve(selected).unwrap().unwrap();
        restored.submit(evaluate(&task).0).unwrap();
    }
    assert_eq!(restored.choose_sector(None, 0., 1., &[]).unwrap(), Some(2));
}
#[test]
fn supplied_lattice_size_caps_at_native_exact_index_limit() {
    let method = settings(false).method;
    assert_eq!(method.next_points(1 << 52, true).unwrap(), 1 << 53);
    assert_eq!(method.next_points(1 << 53, true).unwrap(), 1 << 53);
}
#[test]
fn first_sweep_is_distinct_and_residence_is_minimum_then_same_sector_can_win() {
    let mut session = SerialSession::new(problem(2), settings(false)).unwrap();
    let first = session.choose_sector(None, 0., 10., &[]).unwrap().unwrap();
    let a = session.reserve(first).unwrap().unwrap();
    let second = session
        .choose_sector(None, 0., 10., &[first])
        .unwrap()
        .unwrap();
    assert_ne!(first, second);
    let b = session.reserve(second).unwrap().unwrap();
    session.submit(evaluate(&a).0).unwrap();
    session.submit(evaluate(&b).0).unwrap();
    assert_eq!(
        session
            .choose_sector(Some(first), 20., 10., &[first, second])
            .unwrap(),
        Some(first)
    ); // need two replicas
    for id in [first, second] {
        let task = session.reserve(id).unwrap().unwrap();
        session.submit(evaluate(&task).0).unwrap();
    }
    assert_eq!(
        session
            .choose_sector(Some(first), 1., 10., &[first, second])
            .unwrap(),
        Some(first)
    );
    assert!(!session.first_coverage_complete());
    session
        .choose_sector(Some(first), 10., 10., &[first, second])
        .unwrap();
    session
        .choose_sector(Some(second), 10., 10., &[first, second])
        .unwrap();
    assert!(session.first_coverage_complete());
    let winner = session
        .choose_sector(None, 0., 10., &[first, second])
        .unwrap()
        .unwrap();
    let a = session.reserve(winner).unwrap().unwrap();
    assert_eq!(
        session
            .choose_sector(None, 0., 10., &[first, second])
            .unwrap(),
        Some(winner)
    );
    let b = session.reserve(winner).unwrap().unwrap();
    assert_ne!(evaluate(&a).1, evaluate(&b).1);
}
#[test]
fn minimum_residence_waits_for_a_concurrent_final_replica() {
    for mc in [false, true] {
        let mut session = SerialSession::new(problem(2), settings(mc)).unwrap();
        for id in [0, 1] {
            for _ in 0..2 {
                let task = session.reserve(id).unwrap().unwrap();
                session.submit(evaluate(&task).0).unwrap();
            }
            session.choose_sector(Some(id), 10., 10., &[id]).unwrap();
        }
        assert!(session.first_coverage_complete());
        let a = session.reserve(0).unwrap().unwrap();
        let b = session.reserve(0).unwrap().unwrap();
        session.submit(evaluate(&a).0).unwrap();
        assert!(session.reserve(0).unwrap().is_none());
        assert_eq!(
            session.choose_sector(Some(0), 1., 10., &[0, 0]).unwrap(),
            Some(0),
            "another worker's pending replica must not force early eviction"
        );
        session.submit(evaluate(&b).0).unwrap();
        assert!(session.reserve(0).unwrap().is_some());
    }
}
#[test]
fn elapsed_resident_can_yield_during_replacement_using_previous_evidence() {
    for adaptive in [false, true] {
        let mut options = settings(false);
        options.double_points = true;
        if adaptive {
            options.method = match options.method {
                SerialMethod::Qmc(qmc) => SerialMethod::AdaptiveQmc(qmc),
                _ => unreachable!(),
            };
        }
        let mut session = SerialSession::new(problem(2), options).unwrap();
        for id in 0..2 {
            loop {
                let a = session.reserve(id).unwrap().unwrap();
                let b = session.reserve(id).unwrap().unwrap();
                session.submit(evaluate(&a).0).unwrap();
                session.submit(evaluate(&b).0).unwrap();
                if !a.identity().pilot {
                    break;
                }
            }
            session.choose_sector(Some(id), 1., 1., &[id]).unwrap();
        }
        assert!(session.first_coverage_complete());
        let pending = session.reserve(0).unwrap().unwrap();
        session.submit(evaluate(&pending).0).unwrap();
        assert!(session.sectors[0].previous.is_some());
        assert_eq!(session.sectors[0].moments.count(), 1);
        session.sectors[1].moments = ReplicaMoments::new(2).unwrap();
        session.sectors[1].moments.add(&[0., -1e30]).unwrap();
        session.sectors[1].moments.add(&[0., 1e30]).unwrap();
        assert_eq!(
            session.choose_sector(Some(0), 2., 1., &[0]).unwrap(),
            Some(1)
        );
    }
}
#[test]
fn tiny_nonzero_replica_variance_cannot_underflow_into_exact_zero_error() {
    let mut moments = ReplicaMoments::new(1).unwrap();
    moments.add(&[0.]).unwrap();
    let before = serde_json::to_vec(&moments).unwrap();
    assert!(matches!(
        moments.add(&[1e-200]),
        Err(crate::integration::IntegrationError::NumericRange)
    ));
    assert_eq!(serde_json::to_vec(&moments).unwrap(), before);
    let mut constant = ReplicaMoments::new(1).unwrap();
    constant.add(&[1e-200]).unwrap();
    constant.add(&[1e-200]).unwrap();
    let scalar =
        IntegrationProblem::new("tiny-constant".into(), vec![0], Vec::new(), vec![0.]).unwrap();
    assert_eq!(
        constant.estimate(&scalar, true).unwrap().standard_error,
        [0.]
    );
}
#[test]
fn adaptive_havana_pilots_never_enter_production_or_retrain_fixed_continuation() {
    let mut opts = settings(true);
    let SerialMethod::Mc(grid) = opts.method else {
        unreachable!()
    };
    opts.method = SerialMethod::AdaptiveMc(grid);
    let mut session = SerialSession::new(problem(1), opts).unwrap();
    for _ in 0..2 {
        let task = session.reserve(0).unwrap().unwrap();
        assert!(task.identity().pilot);
        session.submit(evaluate(&task).0).unwrap();
    }
    assert!(session.estimate().is_err());
    let first = session.reserve(0).unwrap().unwrap();
    assert!(!first.identity().pilot);
    assert_eq!(session.snapshot().unwrap().sectors[0].replicas, 0);
    session.submit(evaluate(&first).0).unwrap();
    let second = session.reserve(0).unwrap().unwrap();
    session.submit(evaluate(&second).0).unwrap();
    let epoch = second.identity().epoch;
    let third = session.reserve(0).unwrap().unwrap();
    assert_eq!(epoch, third.identity().epoch);
    assert!(!third.identity().pilot);
}
#[test]
fn native_rng_draw_bound_probe_covers_every_havana_floor_branch() {
    #[derive(Clone)]
    struct Count {
        rng: MonteCarloRng,
        n: u64,
    }
    impl RngCore for Count {
        fn next_u64(&mut self) -> u64 {
            self.n += 1;
            self.rng.next_u64()
        }
        fn next_u32(&mut self) -> u32 {
            (self.next_u64() >> 32) as u32
        }
        fn fill_bytes(&mut self, d: &mut [u8]) {
            for c in d.chunks_mut(8) {
                let x = self.next_u64().to_le_bytes();
                c.copy_from_slice(&x[..c.len()]);
            }
        }
    }
    for floor in [0., 0.25, 1.] {
        let mut grid =
            ContinuousGrid::<f64>::new_with_min_probability_density(5, 8, 8, None, false, floor)
                .unwrap();
        let mut rng = Count {
            rng: MonteCarloRng::new(42, 0),
            n: 0,
        };
        let mut sample = Sample::new();
        for _ in 0..128 {
            grid.sample(&mut rng, &mut sample);
        }
        assert!(rng.n <= streams::mc_draws(128, 5).unwrap());
        assert_eq!(rng.n, 128 * if floor == 0.25 { 6 } else { 5 });
    }
    assert!(streams::mc_draws(u64::MAX, 1).is_err());
    let mut streams = streams::Streams::new(42);
    streams.next = u64::MAX;
    assert!(streams.reserve(1).is_err());
}
#[test]
fn failed_submission_and_checkpoint_corruption_are_transactional() {
    let mut session = SerialSession::new(problem(1), settings(false)).unwrap();
    let task = session.reserve(0).unwrap().unwrap();
    let mut returned = evaluate(&task).0;
    let bytes = session.checkpoint().unwrap();
    returned.mean[0] = f64::NAN;
    assert!(session.submit(returned).is_err());
    assert_eq!(session.checkpoint().unwrap(), bytes);
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["session"]["streams"]["next"] = 999.into();
    assert!(SerialSession::restore(&serde_json::to_vec(&value).unwrap(), &problem(1)).is_err());
}

#[test]
fn asynchronous_refinement_can_stop_from_complete_evidence_without_global_barrier() {
    let mut opts = settings(false);
    opts.tolerance = Tolerance::new(100., 0.).unwrap();
    opts.double_points = true;
    let mut session = SerialSession::new(problem(2), opts).unwrap();
    for _ in 0..2 {
        let task = session.reserve(0).unwrap().unwrap();
        session.submit(evaluate(&task).0).unwrap();
    }
    // The fast sector is already computing a new allocation when the slow
    // sector completes its first valid production evidence.
    let pending = session.reserve(0).unwrap().unwrap();
    for _ in 0..2 {
        let task = session.reserve(1).unwrap().unwrap();
        session.submit(evaluate(&task).0).unwrap();
    }
    assert!(session.meets_target().unwrap());
    assert_eq!(session.snapshot().unwrap().in_flight, 1);
    assert_eq!(
        session.choose_sector(Some(1), 0., 100., &[0, 1]).unwrap(),
        None
    );
    session.release(&pending).unwrap(); // caller confirms cancelled worker dead
}

#[test]
fn first_sweep_honors_residency_claims_between_replicas() {
    let mut session = SerialSession::new(problem(2), settings(false)).unwrap();
    let id = session.choose_sector(None, 0., 1., &[]).unwrap().unwrap();
    // Sector id is already loaded in a worker but has no task in flight.
    assert_eq!(
        session.choose_sector(None, 0., 1., &[id]).unwrap(),
        Some(1 - id)
    );
    assert_eq!(session.choose_sector(None, 0., 1., &[0, 1]).unwrap(), None);
}

#[test]
fn cross_sector_cancellation_retains_sub_ulp_centered_means_and_saved_epochs() {
    let mut session = SerialSession::new(problem(2), settings(false)).unwrap();
    for sector in [0, 1] {
        for replica in [0, 1] {
            let task = session.reserve(sector).unwrap().unwrap();
            let value = task
                .evaluate_weighted_batch(3, |_, weights, output| {
                    for (weight, row) in weights.iter().zip(output.chunks_exact_mut(2)) {
                        row[0] = 0.;
                        row[1] = *weight
                            * if sector == 0 {
                                1e16 + 2. * replica as f64
                            } else {
                                -1e16
                            };
                    }
                    Ok::<_, String>(())
                })
                .unwrap();
            session.submit(value).unwrap();
        }
    }
    assert_eq!(session.estimate().unwrap().mean[1], 1.);
    // Starting a replacement must save centered means, not rounded displays.
    let _pending = session.reserve(0).unwrap().unwrap();
    assert_eq!(session.estimate().unwrap().mean[1], 1.);
    let restored = SerialSession::restore(&session.checkpoint().unwrap(), &problem(2)).unwrap();
    assert_eq!(restored.estimate().unwrap().mean[1], 1.);
    assert_eq!(
        restored
            .observation()
            .unwrap()
            .contributions
            .total
            .unwrap()
            .mean[1],
        1.
    );
}
