//! Public nonlinear contour sampling: native complete-vector statistics,
//! selective saved programs, and caller-owned QMC/Havana execution.
#[path = "contour_dynamic_sampling/common.rs"]
mod common;
use common::*;
use fastsecdec::{
    Atom, AtomCore,
    contour::dynamic::DynamicEnvelope,
    integration::{
        Periodization, QmcSession, QmcSettings, ReplicaRelation, RuleSource,
        mc::{HavanaSession, HavanaSettings},
        mc_discrete::{HavanaDiscreteSession, HavanaDiscreteSettings},
    },
    status::{IntegrationStage, UncertaintyStatus},
};
use std::collections::HashMap;
use symbolica::{parse, symbol};

#[test]
fn native_primitives_fix_the_cubic_reference_and_harmful_higher_term() {
    let input = cube(1);
    let [x, y, z] = <[_; 3]>::try_from(input.parameters()).unwrap();
    let ax = Atom::var(x);
    let ay = Atom::var(y);
    let left: Atom = 1 - 2 * &ax;
    let right: Atom = 2 * &ax - 1;
    let positive_primitive: Atom = &left * (left.log() - 1) / 2;
    let negative_primitive: Atom = -&right * (right.log() - 1) / 2;
    let ybase: Atom = 1 + &ay;
    let yprimitive: Atom = -&ybase * (ybase.log() - 1);
    assert!(
        (positive_primitive.derivative(x) + left.log())
            .together()
            .cancel()
            .is_zero()
    );
    assert!(
        (negative_primitive.derivative(x) + right.log())
            .together()
            .cancel()
            .is_zero()
    );
    assert!(
        (yprimitive.derivative(y) + ybase.log())
            .together()
            .cancel()
            .is_zero()
    );
    // t*log(t)->0. The x integral has real part 1 and lower-lip imaginary
    // part pi/2; each positive y/z factor contributes 1-2*log(2).
    // Scaling F by K adds exactly -log(K) to the finite coefficient.
    let f = input.terms()[0].factors()[0].polynomial().clone();
    assert_eq!(f.derivative(x).derivative(y).derivative(z), Atom::num(-2));
    let envelope = DynamicEnvelope::new(&[x, y, z], f.clone(), &[]).unwrap();
    assert_eq!(envelope.causal_terms().len(), 1);
    assert_eq!(envelope.causal_terms()[0].order(), 3);
    assert!(!envelope.causal_terms()[0].squared_bound().is_zero());
    let t = symbol!("sampling_cube::ray_t");
    let imaginary = parse!("𝑖");
    let replacements = [x, y, z]
        .into_iter()
        .zip(envelope.direction())
        .map(|(axis, v)| {
            (
                Atom::var(axis),
                Atom::var(axis) - &imaginary * Atom::var(t) * v,
            )
        })
        .collect::<Vec<_>>();
    let ray = f.replace_multiple(
        replacements
            .iter()
            .map(|(a, b)| symbolica::id::Replacement::new(a.to_pattern(), b.to_pattern())),
    );
    let coefficient = ray
        .series(t, 0, 3)
        .unwrap()
        .coefficient(3.into())
        .unwrap()
        .clone();
    let direction = envelope.direction();
    let harmful: Atom = -2 * &direction[0] * &direction[1] * &direction[2];
    assert!((coefficient - &imaginary * &harmful).expand().is_zero());
    assert!(
        harmful
            .evaluate_with_prec(
                &HashMap::from([
                    (Atom::var(x), 0.25_f64),
                    (Atom::var(y), 1. / 3.),
                    (Atom::var(z), 0.4)
                ]),
                53
            )
            .unwrap()
            > 0.
    );
}

#[test]
fn cubic_qmc_complete_vectors_cover_all_real_periodizations_and_saved_recipes() {
    // Fixed work and seeds, with an absolute error ceiling of 0.005 on each
    // component. Eight-sigma compatibility is a regression gate, not a claim
    // of per-mil production accuracy or a global floating-point certificate.
    for (scale, cap) in CASES {
        let bytes = archive(&cube(scale));
        for construction in CONSTRUCTIONS {
            for periodization in [
                Periodization::None,
                Periodization::Korobov2,
                Periodization::Korobov3,
            ] {
                let Fixture {
                    problem,
                    mut contexts,
                } = restored(&bytes, construction, cap);
                let points = if periodization == Periodization::None {
                    8192
                } else {
                    2048
                };
                let mut session = QmcSession::democratic(
                    problem,
                    QmcSettings {
                        points,
                        shifts: 8,
                        package_points: 128,
                        seed: 34723,
                        rule: RuleSource::Kuo,
                        periodization,
                    },
                )
                .unwrap();
                while let Some(task) = session.next_work().unwrap() {
                    let id = task.sector_id();
                    let result = session
                        .worker_context(id)
                        .unwrap()
                        .evaluate_weighted_batch(task, 64, |points, weights, output| {
                            contexts[id as usize]
                                .evaluate_weighted_batch(points, weights, output)
                                .map(|_| ())
                        })
                        .unwrap();
                    session.submit(result).unwrap();
                }
                assert!(session.is_complete());
                let estimate = session.estimate().unwrap();
                check_estimate(
                    &estimate,
                    |o, p| cube_reference(scale, o, p),
                    0.005,
                    &format!("QMC {construction:?} K={scale} L={cap} {periodization:?}"),
                );
                assert_eq!(
                    session.snapshot().unwrap().completed_points,
                    points * 8 * contexts.len() as u64
                );
                check_unchecked(&contexts);
            }
        }
    }
}

fn pilot_settings(seed: u64) -> HavanaSettings {
    HavanaSettings {
        points_per_batch: 512,
        batches: 4,
        bins: 8,
        seed,
        minimum_probability_density: 0.02,
    }
}

fn finish_havana(
    session: &mut HavanaSession,
    contexts: &mut [fastsecdec::kernel::WeightedEvaluationContext],
) {
    while let Some(task) = session.next_work() {
        let id = task.sector_id();
        let result = session
            .worker_context(id)
            .unwrap()
            .evaluate_weighted_batch_observed(
                task,
                64,
                |points, weights, output| {
                    contexts[id as usize]
                        .evaluate_weighted_batch(points, weights, output)
                        .map(|_| ())
                },
                |_| {},
            )
            .unwrap();
        session.submit(result).unwrap();
    }
}

fn finish_discrete(
    session: &mut HavanaDiscreteSession,
    contexts: &mut [fastsecdec::kernel::WeightedEvaluationContext],
) {
    while let Some(task) = session.next_work() {
        let result = session
            .worker_context()
            .unwrap()
            .evaluate_weighted_batch_observed(
                task,
                64,
                |id, points, weights, output| {
                    contexts[id as usize]
                        .evaluate_weighted_batch(points, weights, output)
                        .map(|_| ())
                },
                |_| {},
            )
            .unwrap();
        session.submit(result).unwrap();
    }
}

#[test]
fn cubic_havana_and_discrete_mc_train_then_freeze_complete_complex_statistics() {
    // The cube has one native sector: discrete MC still uses its real native
    // DiscreteGrid path, without fabricated extra sectors. The separate bubble
    // control below exercises selection between actual distinct native charts.
    // Production is fixed at 32768 (K=1) or 131072 (K=40000) draws
    // (per sector for Havana), and sigma must
    // stay below 0.03 for every component, independently of the 8-sigma oracle.
    for (scale, cap) in CASES {
        let points_per_batch = if scale == 1 { 2048 } else { 8192 };
        let production_points = (points_per_batch * 16) as u64;
        let bytes = archive(&cube(scale));
        for construction in CONSTRUCTIONS {
            let Fixture {
                problem,
                mut contexts,
            } = restored(&bytes, construction, cap);
            let mut session = HavanaSession::pilot(problem, pilot_settings(194731)).unwrap();
            finish_havana(&mut session, &mut contexts);
            assert_eq!(
                session.snapshot().unwrap().uncertainty,
                UncertaintyStatus::PilotOnly
            );
            assert!(session.estimate().is_err());
            session
                .freeze_production(0.5, points_per_batch, 16)
                .unwrap();
            assert_eq!(session.stage(), IntegrationStage::Production);
            assert_eq!(session.snapshot().unwrap().completed_points, 0);
            finish_havana(&mut session, &mut contexts);
            assert_eq!(
                session.snapshot().unwrap().completed_points,
                production_points * contexts.len() as u64
            );
            check_estimate(
                &session.estimate().unwrap(),
                |o, p| cube_reference(scale, o, p),
                0.03,
                &format!("Havana {construction:?} K={scale} L={cap}"),
            );
            check_unchecked(&contexts);

            let Fixture {
                problem,
                mut contexts,
            } = restored(&bytes, construction, cap);
            let mut session = HavanaDiscreteSession::pilot(
                problem,
                HavanaDiscreteSettings {
                    batch: pilot_settings(782291),
                    ..Default::default()
                },
            )
            .unwrap();
            finish_discrete(&mut session, &mut contexts);
            assert_eq!(
                session.snapshot().unwrap().uncertainty,
                UncertaintyStatus::PilotOnly
            );
            assert!(session.estimate().is_err());
            session
                .freeze_production(0.5, 0.5, points_per_batch, 16)
                .unwrap();
            assert_eq!(session.stage(), IntegrationStage::Production);
            assert_eq!(session.snapshot().unwrap().completed_points, 0);
            finish_discrete(&mut session, &mut contexts);
            assert_eq!(
                session.snapshot().unwrap().completed_points,
                production_points
            );
            let observation = session.diagnostic_observation().unwrap();
            assert_eq!(
                observation.contributions.replica_relation,
                ReplicaRelation::SharedAcrossSectors
            );
            check_estimate(
                observation.snapshot.estimate.as_ref().unwrap(),
                |o, p| cube_reference(scale, o, p),
                0.03,
                &format!("Discrete MC {construction:?} K={scale} L={cap}"),
            );
            check_unchecked(&contexts);
        }
    }
}

#[test]
fn unequal_mass_physical_bubble_discrete_mc_selects_real_native_sectors() {
    use fastsecdec::{
        parametric::{
            FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
            PolynomialFactor,
        },
        status::CoefficientComponent,
    };
    use symbolica::domains::float::Complex;
    let [x, y, eps] = [
        symbol!("sampling_bubble::x"),
        symbol!("sampling_bubble::y"),
        symbol!("sampling_bubble::eps"),
    ];
    let u: Atom = Atom::var(x) + Atom::var(y);
    let f: Atom = &u * (Atom::var(x) + 4 * Atom::var(y)) - 11 * Atom::var(x) * Atom::var(y);
    // Projective B0(11;1,4), mu^2=1: U^(2eps-2) F^(-eps)/eps.
    // On U=1 this is the usual Feynman-parameter integral. Its pole and finite
    // parts use the existing HEPKit OneLOop r_Gamma-normalized convention;
    // Gamma(eps)/r_Gamma=1/eps+O(eps) introduces no finite multiplier.
    let input = ParametricIntegrand::new(
        vec![x, y],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one() / Atom::var(eps),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(u, 2 * Atom::var(eps) - 2, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Positive),
                PolynomialFactor::new(f, -Atom::var(eps), FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let mut reference = [Complex::new(0., 0.); 3];
    oneloop::evaluate_with_backend(
        oneloop::ScalarIntegral::B0,
        &[11., 1., 4., 1.].map(|x| Complex::new(x, 0.)),
        &mut reference,
        oneloop::EvaluationBackend::Expression,
    )
    .unwrap();
    assert!((reference[1].re - 1.).abs() < 1e-14 && reference[1].im.abs() < 1e-14);
    assert!(reference[0].im > 0. && reference[2].re == 0. && reference[2].im == 0.);
    let bytes = archive(&input);
    for construction in CONSTRUCTIONS {
        let Fixture {
            problem,
            mut contexts,
        } = restored(&bytes, construction, 0.1);
        assert_eq!(
            contexts.len(),
            2,
            "unequal masses must retain two actual primary sectors"
        );
        let mut session = HavanaDiscreteSession::pilot(
            problem,
            HavanaDiscreteSettings {
                batch: pilot_settings(817109),
                ..Default::default()
            },
        )
        .unwrap();
        finish_discrete(&mut session, &mut contexts);
        assert!(session.estimate().is_err());
        session.freeze_production(0.5, 0.5, 2048, 16).unwrap();
        assert_eq!(session.snapshot().unwrap().completed_points, 0);
        finish_discrete(&mut session, &mut contexts);
        let observation = session.diagnostic_observation().unwrap();
        assert_eq!(
            observation.contributions.replica_relation,
            ReplicaRelation::SharedAcrossSectors
        );
        assert!(
            observation
                .snapshot
                .sectors
                .iter()
                .all(|s| s.completed_points > 0)
        );
        assert_eq!(
            observation
                .snapshot
                .sectors
                .iter()
                .map(|s| s.completed_points)
                .sum::<u64>(),
            32768
        );
        check_estimate(
            observation.snapshot.estimate.as_ref().unwrap(),
            |order, part| {
                let value = reference[(-order) as usize];
                match part {
                    CoefficientComponent::Real => value.re,
                    CoefficientComponent::Imag => value.im,
                }
            },
            0.02,
            &format!("B0 discrete MC {construction:?}"),
        );
        check_unchecked(&contexts);
    }
}
