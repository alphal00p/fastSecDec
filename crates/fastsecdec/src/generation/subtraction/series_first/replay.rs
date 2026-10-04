//! Ignored real-representative replay. Inputs and outputs use native Atom export.
use super::*;
use std::{fs::File, path::Path};
use symbolica::{
    atom::AtomView,
    domains::float::{Float, RealLike},
};

fn read(path: &Path) -> Atom {
    Atom::import(&mut File::open(path).unwrap(), None).unwrap()
}
fn symbol_from(atom: Atom) -> Symbol {
    let AtomView::Var(variable) = atom.as_view() else {
        panic!("native symbol required")
    };
    variable.get_symbol()
}

#[test]
#[ignore = "bounded native Series-first replay of explicitly captured mapped input"]
fn replay_mapped_representative() {
    let _ = symbolica::transcendental::gamma();
    let directory =
        std::env::var("FASTSECDEC_SERIES_FIRST_CAPTURE").expect("mapped capture directory");
    let path = Path::new(&directory);
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("mapped.json")).unwrap()).unwrap();
    let dimension = metadata["parameters"].as_u64().unwrap() as usize;
    let regulator = symbol_from(read(&path.join("regulator.atom")));
    let parameters = (0..dimension)
        .map(|i| symbol_from(read(&path.join(format!("parameter-{i}.atom")))))
        .collect::<Vec<_>>();
    let terms = metadata["terms"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, term)| {
            assert_eq!(term["powers"].as_u64().unwrap() as usize, dimension);
            MappedTerm {
                regular: read(&path.join(format!("mapped-{i}-regular.atom"))),
                prefactor: read(&path.join(format!("mapped-{i}-prefactor.atom"))),
                powers: (0..dimension)
                    .map(|j| read(&path.join(format!("mapped-{i}-power-{j}.atom"))))
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    let options = GenerationOptions {
        max_order: metadata["max_order"].as_i64().unwrap() as i32,
        ..GenerationOptions::default()
    };
    let started = Instant::now();
    let (coefficients, attempts) = expand(&terms, &parameters, regulator, &options).unwrap();
    let seconds = started.elapsed().as_secs_f64();
    let rows = coefficients.iter().map(|(order, coefficient)| {
        let file = format!("series-first-order-{order}.atom");
        coefficient.export(&mut File::create(path.join(&file)).unwrap()).unwrap();
        serde_json::json!({"order": order, "bytes": coefficient.as_view().get_byte_size(), "file": file})
    }).collect::<Vec<_>>();
    let attempts = attempts
        .iter()
        .map(|attempt| {
            serde_json::json!({
            "route": attempt.route, "width": attempt.width, "absolute_bound": attempt.absolute_bound,
                "pieces": attempt.pieces, "seconds": attempt.seconds,
            })
        })
        .collect::<Vec<_>>();
    let report = serde_json::json!({"source": metadata, "seconds": seconds, "attempts": attempts, "coefficients": rows,
        "scope": "test-only native Series-first composition; no multiplicity, compilation or integration; all unknown remainder bounds retained"});
    std::fs::write(
        path.join("series-first.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("{}", serde_json::to_string(&report).unwrap());
}

#[test]
#[ignore = "bounded exact representation and matched MPFR checks of all captured Laurent orders"]
fn compare_mapped_representative() {
    let _ = symbolica::transcendental::gamma();
    let directory =
        std::env::var("FASTSECDEC_SERIES_FIRST_CAPTURE").expect("mapped capture directory");
    let reference =
        std::env::var("FASTSECDEC_SERIES_FIRST_REFERENCE").expect("original replay directory");
    let path = Path::new(&directory);
    let reference = Path::new(&reference);
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("mapped.json")).unwrap()).unwrap();
    let result: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("series-first.json")).unwrap()).unwrap();
    let old_capture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(reference.join("capture.json")).unwrap()).unwrap();
    for field in [
        "representative_index",
        "source_chart_index",
        "multiplicity",
        "max_order",
    ] {
        assert_eq!(
            metadata[field], old_capture[field],
            "capture association {field}"
        );
    }
    let old_result: serde_json::Value =
        serde_json::from_slice(&std::fs::read(reference.join("replay-relative.json")).unwrap())
            .unwrap();
    let actual_orders = result["coefficients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["order"].as_i64().unwrap())
        .collect::<Vec<_>>();
    let reference_orders = old_result["orders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|order| order.as_i64().unwrap())
        .collect::<Vec<_>>();
    let orders = actual_orders
        .iter()
        .chain(&reference_orders)
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let dimension = metadata["parameters"].as_u64().unwrap() as usize;
    let parameters = (0..dimension)
        .map(|i| read(&path.join(format!("parameter-{i}.atom"))))
        .collect::<Vec<_>>();
    let points = [
        (0..dimension)
            .map(|i| 1.0 / (i + 2) as f64)
            .collect::<Vec<_>>(),
        (0..dimension)
            .map(|i| if i == 0 { 1e-20 } else { 0.37 })
            .collect(),
        (0..dimension)
            .map(|i| if i % 2 == 0 { 1e-8 } else { 0.73 })
            .collect(),
    ];
    let mut rows = Vec::new();
    for order in orders {
        let started = Instant::now();
        let actual = if actual_orders.contains(&order) {
            read(&path.join(format!("series-first-order-{order}.atom")))
        } else {
            Atom::Zero
        };
        let expected = if reference_orders.contains(&order) {
            read(&reference.join(format!("relative-restored-order-{order}.atom")))
        } else {
            Atom::Zero
        };
        let exact_equal = actual == expected;
        let mut point_rows = Vec::new();
        for point in &points {
            let mut previous: Option<(Float, Float)> = None;
            for bits in [512, 1024] {
                let values = parameters
                    .iter()
                    .cloned()
                    .zip(point.iter().map(|value| Float::with_val(bits, *value)))
                    .collect::<std::collections::HashMap<_, _>>();
                let actual_value: Float = actual.evaluate_with_prec(&values, bits).unwrap();
                let expected_value: Float = expected.evaluate_with_prec(&values, bits).unwrap();
                let scale = actual_value
                    .to_f64()
                    .abs()
                    .max(expected_value.to_f64().abs())
                    .max(1.0);
                let difference = (actual_value.clone() - &expected_value).to_f64().abs();
                assert!(actual_value.to_f64().is_finite() && expected_value.to_f64().is_finite());
                assert!(
                    difference <= 1e-50 * scale,
                    "order {order}, bits {bits}, point {point:?}: {difference:e}"
                );
                if let Some((previous_actual, previous_expected)) = &previous {
                    for (now, before) in [
                        (&actual_value, previous_actual),
                        (&expected_value, previous_expected),
                    ] {
                        assert!(
                            (now.clone() - before).to_f64().abs() <= 1e-50 * scale,
                            "precision disagreement"
                        );
                    }
                }
                point_rows.push(serde_json::json!({"point": point, "bits": bits, "value": actual_value.to_string(), "reference": expected_value.to_string(), "absolute_difference_f64": difference}));
                previous = Some((actual_value, expected_value));
            }
        }
        rows.push(serde_json::json!({"order": order, "exact_atom_equal": exact_equal, "seconds": started.elapsed().as_secs_f64(), "points": point_rows}));
        std::fs::write(
            path.join("comparison-progress.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
        eprintln!("series-first comparison order {order}: exact={exact_equal}");
    }
    std::fs::write(path.join("comparison.json"), serde_json::to_vec_pretty(&serde_json::json!({"complete": true, "orders": rows, "scope": "exact Atom equality where available; otherwise 3 matched points at512/1024bits, no forced expansion"})).unwrap()).unwrap();
}
