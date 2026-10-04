//! A diagnostic using only native Symbolica series and coefficient APIs.
use std::{collections::BTreeMap, fs::File, path::Path, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::atom::AtomField,
    poly::series::{Series, SeriesDepth},
};

pub(super) fn relative(
    expression: &Atom,
    regulator: Symbol,
    maximum: i32,
) -> (Series<AtomField>, serde_json::Value) {
    let started = Instant::now();
    let first = expression
        .series(regulator, 0, SeriesDepth::relative(1))
        .unwrap();
    let first_seconds = started.elapsed().as_secs_f64();
    let first_leading = first.get_trailing_exponent();
    let first_absolute = first.absolute_order();
    let first_relative = first.relative_order();
    // The native first series can already cover the requested absolute bound.
    // Otherwise the second call still owns all cancellations and truncation.
    // A failed bound is an explicit failed experiment, never a usable vector.
    let (series, requested) = if first_absolute > maximum {
        (first, None)
    } else {
        let depth =
            symbolica::domains::rational::Rational::from(i64::from(maximum) + 1) - &first_leading;
        let series = expression
            .series(regulator, 0, SeriesDepth::relative(depth.clone()))
            .unwrap();
        (series, Some(depth.to_string()))
    };
    assert!(
        series.absolute_order() > maximum,
        "native relative expansion did not cover requested absolute order"
    );
    let report = serde_json::json!({
        "first_seconds": first_seconds,
        "first_leading": first_leading.to_string(),
        "first_absolute_order": first_absolute.to_string(),
        "first_relative_order": first_relative.to_string(),
        "second_requested_relative_depth": requested,
        "final_absolute_order": series.absolute_order().to_string(),
        "total_seconds": started.elapsed().as_secs_f64(),
    });
    (series, report)
}

pub(super) fn coefficients(
    series: &Series<AtomField>,
    maximum: i32,
) -> Result<BTreeMap<i32, Atom>, super::GenerationError> {
    assert!(series.absolute_order() > maximum);
    let mut result = BTreeMap::new();
    for (order, coefficient) in series.terms() {
        if coefficient.is_zero() {
            continue;
        }
        if !order.is_integer() {
            return Err(super::GenerationError::FractionalLaurent(order.to_string()));
        }
        let order = i32::try_from(order.numerator().to_i64().unwrap()).unwrap();
        if order <= maximum {
            result.insert(order, coefficient.clone());
        }
    }
    Ok(result)
}

#[test]
#[ignore = "bounded replay of an exported native Laurent template; select strategy via test-only environment"]
fn replay_captured_template() {
    // Register native special-function callbacks before reading their symbols.
    // This also avoids State::import's lazy-initialization warning ambiguity.
    let _ = symbolica::transcendental::gamma();
    let directory = std::env::var("FASTSECDEC_LAURENT_CAPTURE").expect("capture directory");
    let path = Path::new(&directory);
    let mode = std::env::var("FASTSECDEC_LAURENT_REPLAY").expect("absolute or relative");
    assert!(["absolute", "relative"].contains(&mode.as_str()));
    let capture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("capture.json")).unwrap()).unwrap();
    let maximum = i32::try_from(capture["max_order"].as_i64().unwrap()).unwrap();
    let expression =
        Atom::import(&mut File::open(path.join("template.atom")).unwrap(), None).unwrap();
    let regulator =
        Atom::import(&mut File::open(path.join("regulator.atom")).unwrap(), None).unwrap();
    let AtomView::Var(regulator) = regulator.as_view() else {
        panic!("regulator symbol")
    };
    let started = Instant::now();
    let (series, details) = if mode == "relative" {
        relative(&expression, regulator.get_symbol(), maximum)
    } else {
        let series = expression
            .series(regulator.get_symbol(), 0, SeriesDepth::absolute(maximum))
            .unwrap();
        let details =
            serde_json::json!({"final_absolute_order": series.absolute_order().to_string()});
        (series, details)
    };
    let seconds = started.elapsed().as_secs_f64();
    let coefficients = coefficients(&series, maximum).unwrap();
    let mut images = BTreeMap::new();
    for entry in capture["images"].as_array().unwrap() {
        let read = |key: &str| {
            Atom::import(
                &mut File::open(path.join(entry[key].as_str().unwrap())).unwrap(),
                None,
            )
            .unwrap()
        };
        let symbol = read("symbol_file");
        let AtomView::Var(symbol) = symbol.as_view() else {
            panic!("image symbol")
        };
        images.insert(symbol.get_symbol(), read("file"));
    }
    // Replay the exact native restoration operations from laurent::expand,
    // separately timed from series. No polynomial expansion is introduced.
    let restoration_started = Instant::now();
    let restored = coefficients
        .iter()
        .map(|(order, coefficient)| {
            let restored = coefficient.replace_map(|term, _, out| {
                if let AtomView::Var(variable) = term
                    && let Some(value) = images.get(&variable.get_symbol())
                {
                    **out = value.clone();
                }
            });
            let restored = if restored.as_view().get_byte_size() <= 256
                && restored.is_polynomial(true, false).is_none()
            {
                restored.together()
            } else {
                restored
            };
            (*order, restored)
        })
        .collect::<BTreeMap<_, _>>();
    let restoration_seconds = restoration_started.elapsed().as_secs_f64();
    for (order, coefficient) in &coefficients {
        super::export(
            &path.join(format!("{mode}-order-{order}.atom")),
            coefficient,
        );
    }
    for (order, coefficient) in &restored {
        super::export(
            &path.join(format!("{mode}-restored-order-{order}.atom")),
            coefficient,
        );
    }
    let report = serde_json::json!({
        "mode": mode, "seconds": seconds, "details": details,
        "orders": coefficients.keys().collect::<Vec<_>>(),
        "coefficient_bytes": coefficients.values().map(|c| c.as_view().get_byte_size()).collect::<Vec<_>>(),
        "restoration_seconds": restoration_seconds,
        "restored_coefficient_bytes": restored.values().map(|c| c.as_view().get_byte_size()).collect::<Vec<_>>(),
        "template_bytes": expression.as_view().get_byte_size(),
        "scope": "native series and separately measured coordinate-image restoration; no multiplication by representative multiplicity, evaluator build or numerical integration",
    });
    std::fs::write(
        path.join(format!("replay-{mode}.json")),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("{report}");
}

#[test]
#[ignore = "exact native coefficient comparison after both bounded replay strategies finish"]
fn compare_captured_replays() {
    let _ = symbolica::transcendental::gamma();
    let directory = std::env::var("FASTSECDEC_LAURENT_CAPTURE").expect("capture directory");
    let path = Path::new(&directory);
    let read_report = |name| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(path.join(format!("replay-{name}.json"))).unwrap())
            .unwrap()
    };
    let absolute = read_report("absolute");
    let relative = read_report("relative");
    assert_eq!(absolute["orders"], relative["orders"]);
    for order in absolute["orders"].as_array().unwrap() {
        let read = |mode, restored| {
            Atom::import(
                &mut File::open(path.join(format!("{mode}-{restored}order-{order}.atom"))).unwrap(),
                None,
            )
            .unwrap()
        };
        assert_eq!(
            read("absolute", ""),
            read("relative", ""),
            "native canonical coefficient at order {order}"
        );
        assert_eq!(
            read("absolute", "restored-"),
            read("relative", "restored-"),
            "restored native coefficient at order {order}"
        );
    }
}
