//! Scientific composition gaps identified by the independent reference audit.
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};

use fastsecdec::{
    Atom, Kinematics, Model,
    generation::{GenerationError, GenerationOptions, generate},
    input::GraphIntegral,
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec},
    kernel::ReplayPolicy,
    parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
    status::CoefficientComponent,
};
use feynkit_graph::symbols;
use symbolica::{parse, symbol};

fn graph(dot: &str, triangle: bool, channel_sign: i64) -> GraphIntegral {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let p = (0..4)
        .map(|index| symbols::external_momentum().call(index))
        .collect::<Vec<_>>();
    let mut kin = Kinematics::in_dimension(&parse!("gap::D")).unwrap();
    if triangle {
        kin = kin
            .with_mass_squared(&p[1], Atom::Zero)
            .unwrap()
            .with_mass_squared(&p[2], Atom::Zero)
            .unwrap()
            .with_scalar_product(&p[1], &p[2], Atom::num((channel_sign, 2)))
            .unwrap();
    } else {
        let (s, t) = (channel_sign, 2 * channel_sign);
        let twice_gram = [
            [0, s, -s - t, t],
            [s, 0, t, -s - t],
            [-s - t, t, 0, s],
            [t, -s - t, s, 0],
        ];
        for i in 0..4 {
            for j in i..4 {
                kin = kin
                    .with_scalar_product(&p[i], &p[j], Atom::num((twice_gram[i][j], 2)))
                    .unwrap();
            }
        }
    }
    GraphIntegral::from_dot(model, dot, &kin)
        .unwrap()
        .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::Zero)]))
        .unwrap()
}

fn parametric(graph: &GraphIntegral) -> ParametricIntegrand {
    ParametricIntegrand::from_graph(
        graph,
        (0..graph.powers().len())
            .map(|i| symbol!(format!("gap::x{i}")))
            .collect(),
        symbol!("gap::eps"),
        parse!("4-2*gap::eps"),
    )
    .unwrap()
}

#[test]
fn native_timelike_massless_triangle_and_box_reject_even_with_an_assertion() {
    for (dot, triangle) in [
        (include_str!("../../../examples/graphs/triangle.dot"), true),
        (include_str!("../../../examples/graphs/box.dot"), false),
    ] {
        let input = parametric(&graph(dot, triangle, 1));
        for assume_no_threshold in [false, true] {
            let error = generate(
                &input,
                &GenerationOptions {
                    assume_no_threshold,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap_err();
            // Here F is negative throughout the open domain, rather than
            // changing sign inside it. Its regulator-dependent power needs
            // complex branch handling, which an assertion cannot override.
            assert!(
                matches!(error, GenerationError::ComplexBranch(_)),
                "{error}"
            );
        }
    }
}

#[test]
fn signed_scaled_gamma_prefactor_keeps_all_coefficients_through_order_four() {
    let input = ParametricIntegrand::new(
        vec![symbol!("gap::x")],
        symbol!("gap::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("-gamma(3+2*gap::eps)"),
            vec![Atom::one()],
            vec![],
        )],
    )
    .unwrap();
    let mut kernels = generate(
        &input,
        &GenerationOptions {
            max_order: 4,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap();
    assert_eq!(kernels.orders(), [0, 1, 2, 3, 4]);
    // Frozen independent values from Pathfinder test_integrals.py:5276,
    // commit 582d8c7f6dde9bf750750d4c2a2d85a94ce940cd. These are test
    // reference values; production obtains every constant from Symbolica.
    let expected = [
        -2.0,
        -3.691_137_340_393_868_6,
        -4.985_859_983_805_386,
        -4.599_953_351_364_898,
        -3.681_199_052_065_825,
    ];
    for x in [0.25, 0.75] {
        let mut total = kernels.exact_coefficients().to_vec();
        for kernel in kernels.sectors_mut() {
            let mut values = [0.0; 5];
            kernel.evaluate(&[x], &mut values).unwrap();
            for (sum, value) in total.iter_mut().zip(values) {
                *sum += value;
            }
        }
        for (actual, expected) in total.iter().zip(expected) {
            assert!((actual - x * expected).abs() < 2e-13, "{total:?}");
        }
    }
}

#[test]
fn negative_highest_order_integrates_only_the_native_triangle_poles() {
    let input = parametric(&graph(
        include_str!("../../../examples/graphs/triangle.dot"),
        true,
        -1,
    ));
    let mut kernels = generate(
        &input,
        &GenerationOptions {
            max_order: -1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap();
    assert_eq!(kernels.orders(), [-2, -1]);
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
            points: 1024,
            shifts: 16,
            seed: 7241,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|id| session.worker_context(id as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id() as usize;
        let result = workers[id]
            .evaluate(task, |point, output| {
                kernels.sectors_mut()[id].evaluate(point, output)
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert!(estimate.production_complete);
    assert_eq!(estimate.orders, [-2, -1]);
    assert_eq!(estimate.components, [CoefficientComponent::Real; 2]);
    // Analytic normalized massless triangle: -1/eps^2 + gamma_E/eps + O(1).
    for ((actual, error), expected) in estimate
        .mean
        .iter()
        .zip(&estimate.standard_error)
        .zip([-1.0, 0.577_215_664_901_532_9])
    {
        assert!(
            (actual - expected).abs() < 8.0 * error + 2e-6,
            "{estimate:?}"
        );
    }
}

#[test]
fn native_rank_two_numerator_replay_preserves_the_complete_real_laurent_vector() {
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(|| {
            let graph = graph(
                include_str!("../../../examples/graphs/box_rank2_numerator.dot"),
                false,
                -1,
            );
            let input = parametric(&graph);
            let mut kernels = generate(&input, &GenerationOptions::default(), |_| {
                ControlFlow::Continue(())
            })
            .unwrap()
            .compile()
            .unwrap();
            assert_eq!(kernels.orders().last(), Some(&0));
            assert!(kernels.orders().len() >= 2);
            assert!(
                kernels
                    .components()
                    .iter()
                    .all(|part| *part == CoefficientComponent::Real)
            );
            assert!(!kernels.sectors().is_empty());
            let mut ordinary_evaluations = 0;
            for id in 0..kernels.sectors().len() {
                let point = vec![0.37; kernels.sectors()[id].dimension()];
                let mut ordinary = vec![0.0; kernels.orders().len()];
                let report = kernels.sectors_mut()[id]
                    .evaluate_with_diagnostics(&point, &mut ordinary)
                    .unwrap();
                ordinary_evaluations += usize::from(!report.rescued);
                let mut replay = kernels
                    .evaluation_context(
                        id,
                        ReplayPolicy {
                            minimum_bits: 256,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                let mut precise = vec![0.0; ordinary.len()];
                let report = replay.evaluate_weighted(&point, 1.0, &mut precise).unwrap();
                assert!(report.weighted_check && report.precision.rescued);
                assert!(report.precision.bits >= 512);
                for (order, (actual, expected)) in
                    kernels.orders().iter().zip(precise.iter().zip(&ordinary))
                {
                    assert!(
                        (actual - expected).abs() < 1e-10 * (1.0 + expected.abs()),
                        "sector {id}, eps^{order}: MPFR {actual}, ordinary {expected}"
                    );
                }
            }
            assert!(ordinary_evaluations > 0);
        })
        .unwrap()
        .join()
        .unwrap();
}
