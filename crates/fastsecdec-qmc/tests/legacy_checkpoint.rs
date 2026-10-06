#![cfg(feature = "serde")]

use fastsecdec_qmc::{QmcAccumulator, QmcEstimate, QmcPartial, QmcPlan, Rank1Rule};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixtures {
    source_revision: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    seed: u64,
    stream: u64,
    plan_json: String,
    plan_bytes: Vec<u8>,
    accumulator_json: String,
    accumulator_bytes: Vec<u8>,
    point_bits: Vec<u64>,
    estimate: QmcEstimate,
}

#[test]
fn numerica_checkpoints_and_randomizations_survive_the_crate_move_bit_for_bit() {
    let fixtures: Fixtures =
        serde_json::from_str(include_str!("fixtures/numerica-f6ecdac.json")).unwrap();
    assert_eq!(
        fixtures.source_revision,
        "f6ecdac8237a30adfcd1be5944a95c5160e474ce"
    );
    for case in fixtures.cases {
        let plan = QmcPlan::new(
            Rank1Rule::new(7, vec![1, 3]).unwrap(),
            3,
            case.seed,
            case.stream,
        )
        .unwrap();
        assert_eq!(serde_json::to_string(&plan).unwrap(), case.plan_json);
        assert_eq!(
            bincode::serde::encode_to_vec(&plan, bincode::config::standard()).unwrap(),
            case.plan_bytes
        );
        let (decoded, used): (QmcPlan, usize) =
            bincode::serde::decode_from_slice(&case.plan_bytes, bincode::config::standard())
                .unwrap();
        assert_eq!(used, case.plan_bytes.len());
        assert_eq!(decoded, plan);
        let mut point = vec![0.0; plan.dimension()];
        let mut actual_bits = Vec::new();
        for index in 0..plan.total_points() {
            plan.point(index, &mut point).unwrap();
            actual_bits.extend(point.iter().map(|value| value.to_bits()));
        }
        assert_eq!(actual_bits, case.point_bits);

        let json_state: QmcAccumulator = serde_json::from_str(&case.accumulator_json).unwrap();
        let (binary_state, used): (QmcAccumulator, usize) =
            bincode::serde::decode_from_slice(&case.accumulator_bytes, bincode::config::standard())
                .unwrap();
        assert_eq!(used, case.accumulator_bytes.len());
        for mut resumed in [json_state, binary_state] {
            assert_eq!(
                serde_json::to_string(&resumed).unwrap(),
                case.accumulator_json
            );
            assert_eq!(
                bincode::serde::encode_to_vec(&resumed, bincode::config::standard()).unwrap(),
                case.accumulator_bytes
            );
            for work in plan.packages(5).unwrap() {
                if resumed.completed_work_packages().any(|done| done == work) {
                    continue;
                }
                let mut partial = QmcPartial::new(&plan, work, 2).unwrap();
                for index in work.start()..work.start() + work.point_count() {
                    plan.point(index, &mut point).unwrap();
                    let value = point.iter().product::<f64>();
                    partial.push(&[value, -2.0 * value]).unwrap();
                }
                resumed.merge(partial.finish().unwrap()).unwrap();
            }
            assert_eq!(resumed.estimate().unwrap(), case.estimate);
        }
    }
}
