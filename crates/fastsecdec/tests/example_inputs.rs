//! Native input coverage for the reference's scalar topology families.
use std::sync::Arc;

use fastsecdec::{
    Atom, AtomCore, Kinematics, Model,
    input::{GraphIntegral, default_algebra_settings},
};
use feynkit_graph::symbols;
use symbolica::parse;

#[test]
fn scalar_example_families_keep_native_topology_and_all_propagators() {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let kinematics = Kinematics::in_dimension(&parse!("D")).unwrap();
    let cases = [
        (
            "triangle",
            include_str!("../../../examples/graphs/triangle.dot"),
            1,
            3,
        ),
        (
            "box",
            include_str!("../../../examples/graphs/box.dot"),
            1,
            4,
        ),
        (
            "double_box",
            include_str!("../../../examples/graphs/double_box.dot"),
            2,
            7,
        ),
        (
            "triple_box",
            include_str!("../../../examples/graphs/triple_box.dot"),
            3,
            10,
        ),
        (
            "kite_2loop",
            include_str!("../../../examples/graphs/kite_2loop.dot"),
            2,
            5,
        ),
        (
            "self_energy_3loop",
            include_str!("../../../examples/graphs/self_energy_3loop.dot"),
            3,
            7,
        ),
        (
            "three_point_2loop",
            include_str!("../../../examples/graphs/three_point_2loop.dot"),
            2,
            5,
        ),
        (
            "three_point_2loop_6line",
            include_str!("../../../examples/graphs/three_point_2loop_6line.dot"),
            2,
            6,
        ),
        (
            "three_point_3loop",
            include_str!("../../../examples/graphs/three_point_3loop.dot"),
            3,
            7,
        ),
        (
            "three_point_3loop_8line",
            include_str!("../../../examples/graphs/three_point_3loop_8line.dot"),
            3,
            8,
        ),
    ];
    for (name, dot, loops, propagators) in cases {
        let integral = GraphIntegral::from_dot(Arc::clone(&model), dot, &kinematics)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(integral.diagram().loop_count(), loops, "{name}");
        assert_eq!(
            integral.family().denominators().len(),
            propagators,
            "{name}"
        );
        assert_eq!(integral.powers(), vec![1; propagators], "{name}");
    }
}

#[test]
fn numerator_fixtures_preserve_reference_denominators_in_native_routing() {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let kinematics = Kinematics::in_dimension(&parse!("D")).unwrap();
    let cases = [
        (
            "triangle",
            include_str!("../../../examples/graphs/triangle_numerator.dot"),
        ),
        (
            "box",
            include_str!("../../../examples/graphs/box_numerator.dot"),
        ),
        (
            "box_rank2",
            include_str!("../../../examples/graphs/box_rank2_numerator.dot"),
        ),
        (
            "box_high_rank",
            include_str!("../../../examples/graphs/box_high_rank_numerator.dot"),
        ),
        (
            "triple_box",
            include_str!("../../../examples/graphs/triple_box_offshell_rank2_numerator.dot"),
        ),
    ];
    let p = (0..4)
        .map(|i| symbols::external_momentum().call(i))
        .collect::<Vec<_>>();
    for (name, dot) in cases {
        let integral = GraphIntegral::from_dot(Arc::clone(&model), dot, &kinematics)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let family = integral.family();
        let k = family.loop_momenta();
        let dot = |a: &Atom, b: &Atom| family.kinematics().scalar_product(a, b).unwrap();
        let (momenta, numerator): (Vec<Atom>, Atom) = if name == "triangle" {
            (
                vec![k[0].clone(), &k[0] + &p[0], &k[0] + &p[0] - &p[1]],
                2 * dot(&k[0], &p[1]),
            )
        } else if name == "triple_box" {
            (
                vec![
                    &k[0] + &k[2] + &p[1],
                    &k[0] + &k[2],
                    k[0].clone(),
                    k[0].clone(),
                    k[1].clone(),
                    k[1].clone(),
                    &k[1] - &p[2],
                    &k[0] + &k[2] + &p[0] + &p[1],
                    k[2].clone(),
                    &k[0] - &k[1],
                ],
                dot(&k[0], &k[2]) + 2 * dot(&k[1], &p[0]) * dot(&k[1], &p[1]),
            )
        } else {
            // The historical box's k is minus the momentum on native edge 7.
            let q = -&k[0];
            let (a, b, c) = (dot(&q, &p[0]), dot(&q, &p[1]), dot(&q, &p[2]));
            let numerator = match name {
                "box" => 2 * &c,
                "box_rank2" => dot(&q, &q) + 3 * &a * &b,
                "box_high_rank" => {
                    -6 * a.pow(2) * b.pow(2) * &c
                        + 46 * a.pow(2) * &b * c.pow(2)
                        + 13 * &a * b.pow(2) * c.pow(2)
                        - 23 * &a * b.pow(3) * &c
                }
                _ => unreachable!(),
            };
            (
                vec![
                    q.clone(),
                    &q + &p[0],
                    &q + &p[0] + &p[1],
                    &q + &p[0] + &p[1] + &p[2],
                ],
                numerator,
            )
        };
        let mut expected = momenta
            .iter()
            .map(|q| (dot(q, q) - parse!("UFO::mt^2")).expand())
            .collect::<Vec<_>>();
        let mut actual = family
            .denominators()
            .iter()
            .map(|d| d.expand())
            .collect::<Vec<_>>();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected, "{name}: propagator multiset");
        let actual_numerator = integral
            .scalar_numerator(&default_algebra_settings())
            .unwrap();
        assert!(
            (actual_numerator - numerator).expand().is_zero(),
            "{name}: numerator"
        );
    }
}
