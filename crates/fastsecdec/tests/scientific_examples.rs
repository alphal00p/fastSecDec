//! Independent integral identities through native graph input and complete QMC.
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};

use fastsecdec::{
    Atom, Kinematics, Model,
    generation::{GenerationOptions, SubtractionStrategy, generate},
    input::GraphIntegral,
    integration::{IntegrationProblem, QmcSession, QmcSettings, SectorSpec, VectorEstimate},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use feynkit_graph::symbols;
use symbolica::{parse, symbol};

fn integrate(input: &ParametricIntegrand, order: i32) -> VectorEstimate {
    let generated = generate(
        input,
        &GenerationOptions {
            max_order: order,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut kernels = generated.compile().unwrap();
    let problem = IntegrationProblem::new(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(index, kernel)| SectorSpec {
                id: index as u64,
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
            seed: 18931,
            ..Default::default()
        },
    )
    .unwrap();
    let mut workers = (0..kernels.sectors().len())
        .map(|index| session.worker_context(index as u64).unwrap())
        .collect::<Vec<_>>();
    while let Some(task) = session.next_work().unwrap() {
        let index = task.sector_id() as usize;
        let result = workers[index]
            .evaluate(task, |point, values| {
                kernels.sectors_mut()[index].evaluate(point, values)
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    session.estimate().unwrap()
}

#[test]
fn native_massless_box_matches_analytic_vector_and_frozen_external_target() {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let momenta = (0..4)
        .map(|index| symbols::external_momentum().call(index))
        .collect::<Vec<_>>();
    let twice_gram = [
        [0, -1, 2, -1],
        [-1, 0, -1, 2],
        [2, -1, 0, -1],
        [-1, 2, -1, 0],
    ];
    let mut kinematics = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_momenta(momenta.clone())
        .unwrap();
    for i in 0..4 {
        for j in i..4 {
            kinematics = kinematics
                .with_scalar_product(&momenta[i], &momenta[j], Atom::num((twice_gram[i][j], 2)))
                .unwrap();
        }
    }
    let graph = GraphIntegral::from_dot(
        model,
        include_str!("../../../examples/graphs/box.dot"),
        &kinematics,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::Zero)]))
    .unwrap();
    let input = ParametricIntegrand::from_graph(
        &graph,
        (0..4)
            .map(|index| symbol!(format!("box_audit::x{index}")))
            .collect(),
        symbol!("box_audit::eps"),
        parse!("4-2*box_audit::eps"),
    )
    .unwrap();
    let estimate = integrate(&input, 0);
    // At s=t=-1 the conventional massless box is
    // 4/eps^2 - 4*gamma_E/eps + 2*gamma_E^2 - 4*pi^2/3 + O(eps).
    // This is an integral-level check, independent of the sector partition.
    let gamma = std::f64::consts::EULER_GAMMA;
    let expected = [
        4.0,
        -4.0 * gamma,
        2.0 * gamma * gamma - 4.0 * std::f64::consts::PI.powi(2) / 3.0,
    ];
    let external: serde_json::Value =
        serde_json::from_str(include_str!("../../../examples/targets/box.json")).unwrap();
    assert_eq!(estimate.orders, [-2, -1, 0]);
    assert!(estimate.production_complete);
    for (index, target) in expected.into_iter().enumerate() {
        assert!((external["coefficients"][index]["re"].as_f64().unwrap() - target).abs() < 2e-8);
        assert!(
            (estimate.mean[index] - target).abs() <= 6.0 * estimate.standard_error[index] + 1e-9,
            "eps^{}: {} +/- {}, analytic {target}",
            estimate.orders[index],
            estimate.mean[index],
            estimate.standard_error[index]
        );
    }
}

#[test]
fn caller_coordinate_and_regulator_names_are_literal_symbols() {
    // Trailing underscores are valid Symbolica variable names. They become
    // wildcards only in patterns; the integral API must substitute them literally.
    let input = ParametricIntegrand::new(
        vec![symbol!("literal_audit::x_")],
        symbol!("literal_audit::eps_"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![parse!("-1+literal_audit::eps_")],
            vec![PolynomialFactor::new(
                parse!("1+literal_audit::x_"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let estimate = integrate(&input, 0);
    assert_eq!(estimate.orders, [-1, 0]);
    for (index, target) in [1.0, -2.0f64.ln()].into_iter().enumerate() {
        assert!(
            (estimate.mean[index] - target).abs() <= 6.0 * estimate.standard_error[index] + 1e-10,
            "literal symbol integral: {estimate:?}"
        );
    }
}

#[test]
fn high_order_taylor_cancellation_is_checked_away_from_the_endpoint_threshold() {
    let x = symbol!("cancellation_audit::x");
    let eps = symbol!("cancellation_audit::eps");
    for degree in [8, 16, 32, 48] {
        let input = ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::var(eps) - Atom::num(degree)],
                vec![PolynomialFactor::new(
                    Atom::one() + Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap();
        let generated = generate(
            &input,
            &GenerationOptions {
                subtraction: SubtractionStrategy::Taylor,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut kernels = generated.compile().unwrap();
        assert_eq!(kernels.orders(), [-1, 0]);
        let integrated_taylor = (0..degree - 1)
            .map(|k| (-1.0f64).powi(k) / f64::from(k - degree + 1))
            .sum::<f64>();
        for coordinate in [0.1, 0.02] {
            let mut total = kernels.exact_coefficients().to_vec();
            for kernel in kernels.sectors_mut() {
                let mut values = [0.0; 2];
                kernel.evaluate(&[coordinate], &mut values).unwrap();
                for (sum, value) in total.iter_mut().zip(values) {
                    *sum += value;
                }
            }
            // The geometric-series remainder is exactly (-x)^degree/(1+x).
            // This checks evaluation at ordinary interior points, independently
            // of any chosen boundary threshold or precision implementation.
            let expected = integrated_taylor + (-1.0f64).powi(degree) / (1.0 + coordinate);
            assert!(
                (total[0] - (-1.0f64).powi(degree - 1)).abs() < 1e-12
                    && (total[1] - expected).abs() < 1e-11,
                "degree {degree}, x={coordinate}: {total:?}, finite target {expected}"
            );
        }
    }
}

#[test]
fn three_axis_subtraction_preserves_the_complete_meromorphic_polynomial_integral() {
    let input = ParametricIntegrand::new(
        vec![
            symbol!("axis_audit::x"),
            symbol!("axis_audit::y"),
            symbol!("axis_audit::z"),
        ],
        symbol!("axis_audit::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![
                parse!("-2-2*axis_audit::eps"),
                parse!("-1-axis_audit::eps"),
                parse!("-2-2*axis_audit::eps"),
            ],
            vec![PolynomialFactor::new(
                parse!("(1+2*axis_audit::x)*(3+5*axis_audit::z)"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    // Integrate each monomial using int_0^1 x^a dx = 1/(a+1), then
    // analytically continue. The product is exactly
    // -(5+31*eps+48*eps^2)/(2*eps^3*(1+2*eps)^2).
    // This exercises intersecting subtraction faces, negative regulator slopes,
    // and the upper boundary terms introduced by repeated integration by parts.
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let generated = generate(
            &input,
            &GenerationOptions {
                max_order: 1,
                subtraction,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut kernels = generated.compile().unwrap();
        assert_eq!(kernels.orders(), [-3, -2, -1, 0, 1]);
        // A factored polynomial minus its Taylor polynomial may remain
        // structurally parameter-dependent. Do not require expansion merely
        // to classify that exact cancellation as a zero-dimensional sector.
        for point in [[0.23, 0.41, 0.67], [0.03, 0.61, 0.72], [1e-8, 0.2, 1e-7]] {
            let mut total = kernels.exact_coefficients().to_vec();
            for kernel in kernels.sectors_mut() {
                let mut values = [0.0; 5];
                kernel.evaluate(&point, &mut values).unwrap();
                for (sum, value) in total.iter_mut().zip(values) {
                    *sum += value;
                }
            }
            for (actual, expected) in total.iter().zip([-2.5, -5.5, 8.0, -10.0, 8.0]) {
                assert!(
                    (actual - expected).abs() < 1e-11,
                    "{subtraction:?} at {point:?}: {total:?}"
                );
            }
        }
    }
}
