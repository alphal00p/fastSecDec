//! Ignored staging for kernel::program::captured::ibp. No production defaults.
use super::*;
use crate::generation::{GenerationOptions, SubtractionStrategy};
use std::collections::{BTreeMap, BTreeSet};

fn save(path: impl AsRef<Path>, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn export(path: impl AsRef<Path>, value: &Atom) {
    value.export(&mut File::create(path).unwrap()).unwrap();
}

fn expected_point(index: usize) -> Vec<Rational> {
    let text: Vec<String> = match index {
        0 => (2..=10).map(|d| format!("1/{d}")).collect(),
        1 => (0..9)
            .map(|i| {
                if i == 0 {
                    "1/100000000000000000000"
                } else {
                    "37/100"
                }
                .into()
            })
            .collect(),
        2 => (0..9)
            .map(|i| if i % 2 == 0 { "1/100000000" } else { "73/100" }.into())
            .collect(),
        _ => panic!("three prescribed exact points"),
    };
    text.iter().map(|v| rational(v)).collect()
}

fn config(stage: &str) -> serde_json::Value {
    let config =
        json(std::env::var("FASTSECDEC_IBP_GATE_CONFIG").expect("explicit frozen configuration"));
    assert_eq!(config["stage"], stage);
    hashes(&config);
    let _ = symbolica::transcendental::gamma();
    config
}

fn load_capture(config: &serde_json::Value) -> (serde_json::Value, Vec<Symbol>, Symbol) {
    let path = Path::new(config["capture"].as_str().unwrap());
    let capture = json(path.join("capture.json"));
    let mapped = json(path.join("mapped.json"));
    for key in [
        "representative_index",
        "source_chart_index",
        "multiplicity",
        "max_order",
    ] {
        assert_eq!(capture[key], mapped[key]);
    }
    assert_eq!(capture["representative_index"], 80);
    assert_eq!(capture["source_chart_index"], 119);
    assert_eq!(capture["multiplicity"], 4);
    assert_eq!(capture["max_order"], 0);
    assert_eq!(mapped["parameters"], 9);
    let fixture = json(path.join("fixture.json"));
    assert_eq!(fixture["geometry_charts"], 2112);
    assert_eq!(fixture["representatives"], 1026);
    assert_eq!(
        fixture["density_canonical_blake3"],
        "2950af91e8efe30160db08945a72c5131332bba4569269107e8313f1facd6ed7"
    );
    let parameters = (0..9)
        .map(|i| symbol(&atom(path.join(format!("parameter-{i}.atom")))))
        .collect::<Vec<_>>();
    assert_eq!(
        parameters
            .iter()
            .map(|p| Atom::var(*p).to_canonical_string())
            .collect::<Vec<_>>(),
        serde_json::from_value::<Vec<String>>(capture["parameters"].clone()).unwrap()
    );
    let regulator = atom(path.join("regulator.atom"));
    assert_eq!(
        regulator.to_canonical_string(),
        capture["regulator"].as_str().unwrap()
    );
    (capture, parameters, symbol(&regulator))
}

#[test]
#[ignore = "bounded source-bound native Taylor identity and existing IBP subtraction capture; no Laurent work"]
fn prepare_existing_ibp_subtraction() {
    let config = config("prepare");
    let source = Path::new(config["capture"].as_str().unwrap());
    let output = Path::new(config["output"].as_str().unwrap());
    fs::create_dir(output).expect("fresh preparation output");
    let started = Instant::now();
    let (capture, parameters, regulator) = load_capture(&config);
    let mapped = json(source.join("mapped.json"));
    let mut terms = Vec::new();
    let mut mapped_values = Vec::new();
    for i in 0..mapped["terms"].as_array().unwrap().len() {
        let prefactor = atom(source.join(format!("mapped-{i}-prefactor.atom")));
        let regular = atom(source.join(format!("mapped-{i}-regular.atom")));
        let powers = (0..parameters.len())
            .map(|j| atom(source.join(format!("mapped-{i}-power-{j}.atom"))))
            .collect::<Vec<_>>();
        mapped_values.push(serde_json::json!({"prefactor":prefactor.to_canonical_string(),
            "regular":regular.to_canonical_string(),"powers":powers.iter().map(|p| p.to_canonical_string()).collect::<Vec<_>>()}));
        terms.push((prefactor, regular, powers));
    }
    let mut report = serde_json::json!({"format":"native-existing-ibp-preparation","version":1,
        "complete":false,"stage":"Taylor identity","capture":capture,"mapped_values":mapped_values,
        "source_blake3":config["source_blake3"],"multiplicity_applied":false,
        "scope":"Exact captured mapped inputs, native existing subtraction only; Taylor and IBP are integral-equivalent, not pointwise-equal"});
    save(output.join("progress.json"), &report);
    let taylor_started = Instant::now();
    let (taylor, count, cancellation) = crate::generation::captured_subtraction(
        terms.clone(),
        &parameters,
        regulator,
        &GenerationOptions::default(),
    )
    .unwrap();
    assert_eq!(count, 872);
    let original = atom(source.join("expression.atom"));
    assert!(
        taylor == original,
        "native Taylor replay must exactly equal captured original expression"
    );
    report["Taylor_seconds"] = taylor_started.elapsed().as_secs_f64().into();
    report["Taylor_pieces"] = count.into();
    report["Taylor_cancellation_terms"] = serde_json::json!(cancellation);
    report["Taylor_native_expression_identity"] = true.into();
    drop(taylor);
    drop(original);
    report["stage"] = "existing IBP subtraction".into();
    save(output.join("progress.json"), &report);
    let ibp_started = Instant::now();
    let (ibp, count, cancellation) = crate::generation::captured_subtraction(
        terms,
        &parameters,
        regulator,
        &GenerationOptions {
            subtraction: SubtractionStrategy::IntegrateByParts,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!ibp.is_zero(), "refuse vacuous workload");
    report["IBP_seconds"] = ibp_started.elapsed().as_secs_f64().into();
    report["IBP_pieces"] = count.into();
    report["IBP_cancellation_terms"] = serde_json::json!(cancellation);
    report["IBP_expression_bytes"] = ibp.as_view().get_byte_size().into();
    export(output.join("ibp-expression.atom"), &ibp);
    export(output.join("regulator.atom"), &Atom::var(regulator));
    for (i, p) in parameters.iter().enumerate() {
        export(output.join(format!("parameter-{i}.atom")), &Atom::var(*p));
    }
    report["IBP_expression_blake3"] =
        blake3::hash(&fs::read(output.join("ibp-expression.atom")).unwrap())
            .to_hex()
            .to_string()
            .into();
    report["complete"] = true.into();
    report["stage"] = "complete".into();
    report["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
    save(output.join("prepared.json"), &report);
}

#[test]
#[ignore = "bounded existing-IBP production Laurent/native program plus three own-expression oracles and weighted kernels"]
fn compile_existing_ibp_complete_vector() {
    let config = config("compile");
    let output = Path::new(config["output"].as_str().unwrap());
    fs::create_dir(output).expect("fresh compile output");
    let started = Instant::now();
    let (capture, parameters, regulator) = load_capture(&config);
    let prepared_path = Path::new(config["prepared"].as_str().unwrap());
    let prepared = json(prepared_path.join("prepared.json"));
    hashes(&prepared);
    assert_eq!(prepared["complete"], true);
    assert_eq!(prepared["capture"], capture);
    assert_eq!(prepared["Taylor_native_expression_identity"], true);
    let expression_file = prepared_path.join("ibp-expression.atom");
    assert_eq!(
        blake3::hash(&fs::read(&expression_file).unwrap())
            .to_hex()
            .as_str(),
        prepared["IBP_expression_blake3"].as_str().unwrap()
    );
    let expression = atom(&expression_file);
    let mut report = serde_json::json!({"format":"native-existing-ibp-program","version":1,
        "complete":false,"stage":"production Laurent","capture":capture,"preparation":prepared,
        "source_blake3":config["source_blake3"],"multiplicity_applied":false});
    save(output.join("progress.json"), &report);
    let expansion_started = Instant::now();
    let captured =
        crate::generation::captured_coefficients(&expression, &parameters, regulator, 0).unwrap();
    assert!(!captured.coefficients.is_empty());
    let minimum = *captured.coefficients.first_key_value().unwrap().0;
    assert!(minimum <= 0);
    let orders = (minimum..=0).collect::<Vec<_>>();
    let coefficients = orders
        .iter()
        .map(|o| captured.coefficients.get(o).cloned().unwrap_or_default())
        .collect::<Vec<_>>();
    report["expansion_seconds"] = expansion_started.elapsed().as_secs_f64().into();
    report["orders"] = serde_json::json!(orders);
    report["template_bytes"] = captured.template.as_view().get_byte_size().into();
    report["coefficient_root_bytes"] = serde_json::json!(
        coefficients
            .iter()
            .map(|c| c.get_root().as_view().get_byte_size())
            .collect::<Vec<_>>()
    );
    report["coefficient_body_inclusive_bytes"] = serde_json::json!(
        coefficients
            .iter()
            .map(|c| c.get_byte_size())
            .collect::<Vec<_>>()
    );
    export(output.join("template.atom"), &captured.template);
    for (order, coefficient) in orders.iter().zip(&coefficients) {
        export(
            output.join(format!("order-{order}.atom")),
            coefficient.get_root(),
        );
    }
    report["stage"] = "native program and kernels".into();
    save(output.join("progress.json"), &report);
    let rows: Vec<Vec<usize>> =
        serde_json::from_value(prepared["IBP_cancellation_terms"].clone()).unwrap();
    let cancellation = Cancellation::new(
        rows.iter().map(|r| r.iter().sum()).max().unwrap_or(0),
        Some(rows),
        9,
    )
    .unwrap();
    let build_started = Instant::now();
    let program = build(parameters.clone(), &coefficients, cancellation.clone()).unwrap();
    let encoded = encode(&program.exact).unwrap();
    fs::write(output.join("native-ir.bin"), &encoded).unwrap();
    let exact = decode(&encoded).unwrap();
    let mut fresh =
        SectorKernel::from_program(program, &PrecisionPolicy::default(), false).unwrap();
    let mut decoded = SectorKernel::from_program(
        SectorProgram {
            parameters,
            exact: exact.clone(),
            cancellation,
            exact_zero: vec![false; orders.len()],
            real_coefficients: vec![false; orders.len()],
        },
        &PrecisionPolicy::default(),
        false,
    )
    .unwrap();
    report["build_encode_decode_seconds"] = build_started.elapsed().as_secs_f64().into();
    report["native_ir_bytes"] = encoded.len().into();
    assert_eq!(fresh.output_count(), orders.len());
    assert_eq!(decoded.output_count(), orders.len());
    report["stage"] = "complete-vector oracle and weighted checks".into();
    save(output.join("progress.json"), &report);
    let oracles = config["oracles"].as_array().unwrap();
    assert_eq!(oracles.len(), 3);
    let mut evidence = Vec::new();
    for (index, path) in oracles.iter().enumerate() {
        let oracle = json(path.as_str().unwrap());
        hashes(&oracle);
        assert_eq!(oracle["format"], "native-own-ibp-point-first-oracle");
        assert_eq!(oracle["version"], 1);
        assert_eq!(oracle["complete"], true);
        assert_eq!(oracle["multiplicity_applied"], false);
        assert_eq!(oracle["preparation"], prepared);
        assert!(rational(oracle["native_absolute_order"].as_str().unwrap()) > 0);
        assert_eq!(oracle["capture"], capture);
        assert_eq!(
            oracle["IBP_expression_blake3"],
            prepared["IBP_expression_blake3"]
        );
        assert_eq!(oracle["point_index"], index);
        let point = oracle["exact_point"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| rational(s.as_str().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(point.len(), 9);
        assert_eq!(point, expected_point(index));
        let input = |bits| {
            point
                .iter()
                .map(|p| Complex::new(p.to_multi_prec_float(bits), Float::with_val(bits, 0)))
                .collect::<Vec<_>>()
        };
        let low = mpfr(&exact, &input(512), 512);
        let high = mpfr(&exact, &input(1024), 1024);
        let reference = oracle["orders"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                assert_eq!(row["precision_agreement"], true);
                (
                    row["order"].as_i64().unwrap() as i32,
                    Float::parse(row["value_1024"].as_str().unwrap(), Some(1024)).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert!(!reference.is_empty());
        assert_eq!(
            reference.len(),
            oracle["orders"].as_array().unwrap().len(),
            "duplicate oracle order"
        );
        assert!(
            reference.contains_key(&0),
            "oracle must include requested finite coefficient"
        );
        let union = orders
            .iter()
            .copied()
            .chain(reference.keys().copied())
            .collect::<BTreeSet<_>>();
        for order in &union {
            let zero = Float::with_val(1024, 0);
            let expected = reference.get(order).unwrap_or(&zero);
            if let Some(position) = orders.iter().position(|o| o == order) {
                assert!(
                    agrees(&low[position].re, &high[position].re)
                        && agrees(&low[position].im, &high[position].im)
                );
                assert!(
                    agrees(&high[position].re, expected) && agrees(&high[position].im, &zero),
                    "own-IBP point {index}, order {order}"
                );
            } else {
                assert!(agrees(expected, &zero), "missing production order {order}");
            }
        }
        let rounded = point.iter().map(|p| p.to_f64()).collect::<Vec<_>>();
        let rounded_input = rounded
            .iter()
            .map(|p| Complex::new(Float::with_val(1024, *p), Float::with_val(1024, 0)))
            .collect::<Vec<_>>();
        let reference = mpfr(&exact, &rounded_input, 1024);
        let mut evaluations = Vec::new();
        for (label, kernel) in [
            ("fresh", &mut fresh),
            ("same-process decoded", &mut decoded),
        ] {
            for weight in [1.0, 1e40] {
                for forced in [false, true] {
                    let mut values = vec![0.0; orders.len()];
                    let precision = if forced {
                        kernel.replay_scaled(&rounded, &mut values, weight, 128)
                    } else {
                        kernel.evaluate_scaled(&rounded, &mut values, weight)
                    }
                    .unwrap();
                    for (actual, expected) in values.iter().zip(&reference) {
                        let expected =
                            (expected.re.clone() * Float::with_val(1024, weight)).to_f64();
                        assert!(actual.is_finite() && expected.is_finite());
                        assert!((actual - expected).abs() <= 2e-12 * expected.abs().max(1.0));
                    }
                    if forced {
                        assert!(precision.checked && precision.rescued);
                    }
                    evaluations.push(serde_json::json!({"kernel":label,"weight":weight,"forced_replay":forced,
                        "checked":precision.checked,"rescued":precision.rescued,"bits":precision.bits,"values":values}));
                }
            }
        }
        evidence.push(
            serde_json::json!({"point_index":index,"exact_point":oracle["exact_point"],
            "union_orders":union,"all_orders_agree":true,"production_evaluations":evaluations}),
        );
        report["points"] = serde_json::json!(evidence);
        save(output.join("progress.json"), &report);
    }
    report["complete"] = true.into();
    report["stage"] = "complete".into();
    report["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
    report["scope"] = "Own-IBP full-vector series/program and same-process decoded weighted checks; no Taylor pointwise equivalence, graph regeneration, multiplicity or integration claim".into();
    save(output.join("result.json"), &report);
}
