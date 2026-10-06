//! Native graph integrals compared with HEPKit's Rust OneLoopMaster provider.
//! OneLOop uses mu^(2 eps)/r_Gamma relative to our normalized loop measure,
//! where r_Gamma = Gamma(1+eps) Gamma(1-eps)^2 / Gamma(1-2eps).
//! Apply that explicit multiplier to the actual graph integral, then compare
//! its complete Laurent vector with the master's native numerical API.
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};

use fastsecdec::{
    Atom, AtomCore, Kinematics, Model,
    generation::{GenerationOptions, generate},
    input::GraphIntegral,
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec, VectorEstimate},
    parametric::ParametricIntegrand,
};
use feynkit_graph::symbols;
use oneloop::{EvaluationBackend, JitEvaluator, ScalarEvaluator, ScalarIntegral};
use symbolica::{domains::float::Complex, parse, symbol};

fn integrate(graph: GraphIntegral, mu_squared: i64) -> VectorEstimate {
    let epsilon = symbol!("hepkit_master::eps");
    let multiplier = Atom::num(mu_squared).pow(Atom::var(epsilon))
        * parse!(
            "gamma(1-2*hepkit_master::eps)/(gamma(1+hepkit_master::eps)*gamma(1-hepkit_master::eps)^2)"
        );
    let graph = graph.with_measure_multiplier(multiplier);
    let input = ParametricIntegrand::from_graph(
        &graph,
        (0..graph.powers().len())
            .map(|i| symbol!(format!("hepkit_master::x{i}")))
            .collect(),
        epsilon,
        parse!("4-2*hepkit_master::eps"),
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let mut kernels = generated.compile().unwrap();
    let problem = IntegrationProblem::new_with_components(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(id, kernel)| SectorSpec {
                id: id as u64,
                dimension: kernel.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )
    .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 2048,
            shifts: 32,
            seed: 48703,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|id| session.worker_context(id as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id() as usize;
        let value = workers[id]
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[id].evaluate(point, output)
            })
            .unwrap();
        session.submit(value).unwrap();
    }
    session.estimate().unwrap()
}

fn graph(dot: &str, kinematics: &Kinematics, mass: i64) -> GraphIntegral {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    GraphIntegral::from_dot(model, dot, kinematics)
        .unwrap()
        .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::num(mass))]))
        .unwrap()
}

fn compare(family: ScalarIntegral, arguments: &[i64], actual: &VectorEstimate) {
    let input = arguments
        .iter()
        .map(|value| Complex::new(*value as f64, 0.0))
        .collect::<Vec<_>>();
    let mut expected = [Complex::new(0.0, 0.0); 3];
    // Expression means the provider's Rust numerical interpreter, independently
    // of FastSecDec's generated SymJIT kernels. No Python or Fortran is linked.
    oneloop::evaluate_with_backend(family, &input, &mut expected, EvaluationBackend::Expression)
        .unwrap();
    assert!(actual.production_complete);
    assert!(
        actual
            .components
            .iter()
            .all(|component| { *component == fastsecdec::status::CoefficientComponent::Real })
    );
    assert_eq!(actual.orders.last(), Some(&0));
    assert!(actual.orders.iter().all(|order| (-2..=0).contains(order)));
    for (order, expected) in [0, -1, -2].into_iter().zip(expected) {
        let (mean, error) = actual
            .orders
            .iter()
            .position(|candidate| *candidate == order)
            .map(|index| (actual.mean[index], actual.standard_error[index]))
            .unwrap_or((0.0, 0.0));
        assert!(
            expected.re.is_finite() && expected.im.is_finite() && expected.im.abs() < 1e-10,
            "{} {arguments:?}: non-Euclidean or nonfinite native master {expected:?}",
            family.name()
        );
        assert!(
            (mean - expected.re).abs() <= 8.0 * error + 2e-8 * expected.re.abs().max(1.0),
            "{} {arguments:?}, eps^{order}: {mean} +/- {error}; HEPKit native {}",
            family.name(),
            expected.re
        );
    }
}

fn with_stack(work: impl FnOnce() + Send + 'static) {
    // The provider documents this stack size for building its lazy C0/D0 graph.
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn native_master_jit_cache_round_trip_uses_current_backend_and_rejects_old_version() {
    with_stack(|| {
        // Rebuild explicitly from native expressions: no bundled evaluator built
        // with the previous SymJIT release participates in this check.
        let family = ScalarIntegral::B0;
        let evaluator = ScalarEvaluator::rebuild(family).unwrap();
        let bytes = evaluator.to_bytes().unwrap();
        drop(evaluator);
        // The family wrapper prefixes one discriminator byte. The remainder is
        // the provider's public portable JitEvaluator cache, including functions.
        let mut restored = JitEvaluator::from_bytes(&bytes[1..]).unwrap();
        assert_eq!(restored.input_count(), family.arity());
        assert_eq!(restored.output_count(), 3);
        for arguments in [[-1.0, 0.0, 0.0, 1.0], [-2.0, 1.0, 1.0, 4.0]] {
            let input = arguments.map(|value| Complex::new(value, 0.0));
            let mut actual = [Complex::new(f64::NAN, f64::NAN); 3];
            let mut expected = actual;
            restored.evaluate(&input, &mut actual).unwrap();
            oneloop::evaluate_with_backend(
                family,
                &input,
                &mut expected,
                EvaluationBackend::Expression,
            )
            .unwrap();
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert!(actual.re.is_finite() && actual.im.is_finite());
                assert!(expected.re.is_finite() && expected.im.is_finite());
                assert!((actual.re - expected.re).abs() < 1e-12);
                assert!((actual.im - expected.im).abs() < 1e-12);
            }
        }
        let current = b"oneloop-evaluator-v3:";
        let mut stale = bytes[1..].to_vec();
        let offset = stale
            .windows(current.len())
            .position(|window| window == current)
            .expect("native cache must identify the Symbolica source-IR schema");
        stale[offset..offset + current.len()].copy_from_slice(b"oneloop-evaluator-v0:");
        assert!(matches!(
            JitEvaluator::from_bytes(&stale),
            Err(error) if error.contains("incompatible evaluator cache")
        ));
    });
}

#[test]
fn native_bubble_matches_hepkit_b0_at_massless_massive_and_scaleless_points() {
    with_stack(|| {
        for (s, mass, mu_squared) in [(-1, 0, 1), (-3, 0, 4), (-2, 1, 1), (0, 0, 1)] {
            let kin = Kinematics::in_dimension(&parse!("D"))
                .unwrap()
                .with_mass_squared(&symbols::external_momentum().call(1), Atom::num(s))
                .unwrap();
            let actual = integrate(
                graph(
                    include_str!("../../../examples/graphs/bubble.dot"),
                    &kin,
                    mass,
                ),
                mu_squared,
            );
            compare(
                ScalarIntegral::B0,
                &[s, mass * mass, mass * mass, mu_squared],
                &actual,
            );
        }
    });
}

#[test]
fn native_triangle_matches_hepkit_c0_across_euclidean_virtualities_and_masses() {
    with_stack(|| {
        for (virtualities, mass, mu_squared) in [
            ([0, 0, -1], 0, 1),
            ([0, 0, -4], 0, 4),
            ([-1, -2, -4], 0, 1),
            ([0, 0, -1], 1, 1),
        ] {
            let [a, b, c] = virtualities;
            let p = symbols::external_momentum().call(1);
            let q = symbols::external_momentum().call(2);
            let kin = Kinematics::in_dimension(&parse!("D"))
                .unwrap()
                .with_mass_squared(&p, Atom::num(a))
                .unwrap()
                .with_mass_squared(&q, Atom::num(b))
                .unwrap()
                .with_scalar_product(&p, &q, Atom::num((c - a - b, 2)))
                .unwrap();
            let actual = integrate(
                graph(
                    include_str!("../../../examples/graphs/triangle.dot"),
                    &kin,
                    mass,
                ),
                mu_squared,
            );
            compare(
                ScalarIntegral::C0,
                &[a, b, c, mass * mass, mass * mass, mass * mass, mu_squared],
                &actual,
            );
        }
    });
}

#[test]
fn native_box_matches_hepkit_d0_across_euclidean_channels_and_masses() {
    with_stack(|| {
        for (s, t, mass, mu_squared) in [(-1, -1, 0, 1), (-2, -3, 0, 4), (-1, -2, 1, 1)] {
            let actual = integrate(box_graph(s, t, mass), mu_squared);
            compare(
                ScalarIntegral::D0,
                &[
                    0,
                    0,
                    0,
                    0,
                    s,
                    t,
                    mass * mass,
                    mass * mass,
                    mass * mass,
                    mass * mass,
                    mu_squared,
                ],
                &actual,
            );
        }
    });
}

fn box_graph(s: i64, t: i64, mass: i64) -> GraphIntegral {
    let momenta = (0..4)
        .map(|index| symbols::external_momentum().call(index))
        .collect::<Vec<_>>();
    let twice_gram = [
        [0, s, -s - t, t],
        [s, 0, t, -s - t],
        [-s - t, t, 0, s],
        [t, -s - t, s, 0],
    ];
    let mut kin = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_momenta(momenta.clone())
        .unwrap();
    for i in 0..4 {
        for j in i..4 {
            kin = kin
                .with_scalar_product(&momenta[i], &momenta[j], Atom::num((twice_gram[i][j], 2)))
                .unwrap();
        }
    }
    graph(include_str!("../../../examples/graphs/box.dot"), &kin, mass)
}

#[test]
fn native_box_massive_shipped_point_matches_hepkit_d0() {
    with_stack(|| {
        // The shipped massive card has s=t=-1, whereas the existing massive
        // family test uses s=-1,t=-2. Keep this distinct point explicit.
        let actual = integrate(box_graph(-1, -1, 1), 1);
        compare(
            ScalarIntegral::D0,
            &[0, 0, 0, 0, -1, -1, 1, 1, 1, 1, 1],
            &actual,
        );
    });
}
