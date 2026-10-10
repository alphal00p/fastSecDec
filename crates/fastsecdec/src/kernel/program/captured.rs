//! Explicit bounded diagnostic against the independent original-expression oracle.
mod ibp;
mod named;

use super::*;
use crate::kernel::{PrecisionPolicy, SectorKernel};
use std::{fs, fs::File, path::Path, time::Instant};
use symbolica::{
    atom::AtomView,
    coefficient::Coefficient,
    domains::float::{Float, Real, RealLike},
};

fn json(path: impl AsRef<Path>) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn atom(path: impl AsRef<Path>) -> Atom {
    Atom::import(&mut File::open(path).unwrap(), None).unwrap()
}
fn symbol(value: &Atom) -> Symbol {
    let AtomView::Var(value) = value.as_view() else {
        panic!("native symbol expected")
    };
    value.get_symbol()
}
fn hashes(record: &serde_json::Value) {
    for (path, expected) in record["source_blake3"].as_object().unwrap() {
        assert_eq!(
            blake3::hash(&fs::read(path).unwrap()).to_hex().as_str(),
            expected.as_str().unwrap(),
            "{path}"
        );
    }
}
fn rational(text: &str) -> Rational {
    let parsed = Atom::parse(text, "alias_gate", Default::default()).unwrap();
    let AtomView::Num(value) = parsed.as_view() else {
        panic!("rational point")
    };
    let Coefficient::Complex(value) = value.get_coeff_view().to_owned() else {
        panic!("rational point")
    };
    assert!(value.im.is_zero());
    value.re
}
fn mpfr(exact: &ExactProgram, point: &[Complex<Float>], bits: u32) -> Vec<Complex<Float>> {
    let mut evaluator = exact.clone().map_coeff_with_prec(
        &|c| {
            Complex::new(
                c.re.to_multi_prec_float(bits),
                c.im.to_multi_prec_float(bits),
            )
        },
        bits,
    );
    let mut result = vec![
        Complex::new(Float::with_val(bits, 0), Float::with_val(bits, 0));
        exact.get_output_len()
    ];
    evaluator.evaluate(point, &mut result);
    result
}
fn agrees(a: &Float, b: &Float) -> bool {
    let mut scale = a.norm();
    if b.norm() > scale {
        scale = b.norm();
    }
    if scale < Float::with_val(1024, 1) {
        scale = Float::with_val(1024, 1);
    }
    a.is_finite() && b.is_finite() && (a.clone() - b).norm() <= scale * Float::with_val(1024, 1e-70)
}

#[test]
#[ignore = "bounded captured-representative gate; requires source-bound original capture and all three independent oracle reports"]
fn production_alias_representative_matches_complete_oracles() {
    let _ = symbolica::transcendental::gamma();
    let config = json(
        std::env::var("FASTSECDEC_ALIAS_GATE_CONFIG").expect("explicit capture configuration"),
    );
    let path = Path::new(config["capture"].as_str().unwrap());
    let mapped = Path::new(config["mapped"].as_str().unwrap());
    let output = Path::new(config["output"].as_str().unwrap());
    fs::create_dir(output).expect("fresh diagnostic output");
    let started = Instant::now();
    let capture = json(path.join("capture.json"));
    assert_eq!(capture["representative_index"], 38);
    assert_eq!(capture["source_chart_index"], 46);
    assert_eq!(capture["multiplicity"], 2);
    assert_eq!(capture["max_order"], 0);
    let preparation = json(config["preparation"].as_str().unwrap());
    hashes(&preparation);
    assert_eq!(preparation["original_capture"], capture);
    assert_eq!(preparation["production_pieces"], 241);
    let rows: Vec<Vec<usize>> =
        serde_json::from_value(preparation["production_cancellation_terms"].clone()).unwrap();
    let cancellation = Cancellation::new(
        rows.iter().map(|r| r.iter().sum()).max().unwrap(),
        Some(rows.clone()),
        9,
    )
    .unwrap();
    let parameters = (0..9)
        .map(|i| symbol(&atom(mapped.join(format!("parameter-{i}.atom")))))
        .collect::<Vec<_>>();
    assert_eq!(
        parameters
            .iter()
            .map(|s| Atom::var(*s).to_canonical_string())
            .collect::<Vec<_>>(),
        serde_json::from_value::<Vec<String>>(capture["parameters"].clone()).unwrap()
    );
    let regulator = atom(path.join("regulator.atom"));
    assert_eq!(
        regulator.to_canonical_string(),
        capture["regulator"].as_str().unwrap()
    );
    let expression = atom(path.join("expression.atom"));
    let expansion_started = Instant::now();
    let captured =
        crate::generation::captured_coefficients(&expression, &parameters, symbol(&regulator), 0)
            .unwrap();
    let coefficients = captured.coefficients;
    let expansion_seconds = expansion_started.elapsed().as_secs_f64();
    assert_eq!(
        coefficients.keys().copied().collect::<Vec<_>>(),
        (-5..=0).collect::<Vec<_>>()
    );
    // Process-local symbol order can change traversal order and image handles.
    // Prove the complete image-body bijection before comparing renamed roots.
    // This substitutes literal handles only; no image body is expanded.
    let mut saved_bodies = std::collections::BTreeMap::new();
    for image in capture["images"].as_array().unwrap() {
        let body = atom(path.join(image["file"].as_str().unwrap()));
        let handle = atom(path.join(image["symbol_file"].as_str().unwrap()));
        assert!(matches!(handle.as_view(), AtomView::Var(_)));
        assert!(saved_bodies.insert(body, handle).is_none());
    }
    assert_eq!(
        saved_bodies
            .values()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        saved_bodies.len()
    );
    let saved_template = atom(path.join("template.atom"));
    let mut structural_equality = Vec::new();
    for (order, coefficient) in &coefficients {
        assert_eq!(coefficient.get_aliases().len(), saved_bodies.len());
        let mut fresh_bodies = std::collections::BTreeMap::new();
        let mut handles = std::collections::BTreeMap::new();
        for (handle, body) in coefficient.get_aliases() {
            assert!(matches!(handle.as_view(), AtomView::Var(_)));
            assert!(fresh_bodies.insert(body.clone(), handle.clone()).is_none());
            handles.insert(handle.clone(), saved_bodies.get(body).unwrap().clone());
        }
        assert!(fresh_bodies.keys().eq(saved_bodies.keys()));
        let rename = |value: &Atom| {
            value.replace_map(|term, _, out| {
                if matches!(term, AtomView::Var(_))
                    && let Some(handle) = handles.get(&term.to_owned())
                {
                    **out = handle.clone();
                }
            })
        };
        assert_eq!(rename(&captured.template), saved_template);
        let renamed = rename(coefficient.get_root());
        let saved = atom(path.join(format!("relative-order-{order}.atom")));
        structural_equality.push(serde_json::json!({
            "order":order,"root_identical_after_exact_handle_renaming":renamed == saved,
            "fresh_bytes":coefficient.get_root().as_view().get_byte_size(),"saved_bytes":saved.as_view().get_byte_size()
        }));
        coefficient
            .get_root()
            .export(&mut File::create(output.join(format!("fresh-order-{order}.atom"))).unwrap())
            .unwrap();
    }
    fs::write(
        output.join("structural-comparison.json"),
        serde_json::to_vec_pretty(&structural_equality).unwrap(),
    )
    .unwrap();
    let coefficients = coefficients.into_values().collect::<Vec<_>>();
    let build_started = Instant::now();
    let program = build(parameters.clone(), &coefficients, cancellation.clone()).unwrap();
    let encoded = encode(&program.exact).unwrap();
    let exact = decode(&encoded).unwrap();
    let mut kernel =
        SectorKernel::from_program(program, &PrecisionPolicy::default(), false).unwrap();
    let mut cold = SectorKernel::from_program(
        SectorProgram {
            symbolic_endpoint_contour_partials: None,
            parameters,
            runtime_parameters: Vec::new(),
            exact: exact.clone(),
            cancellation,
            exact_zero: vec![false; 6],
            real_coefficients: vec![false; 6],
        },
        &PrecisionPolicy::default(),
        false,
    )
    .unwrap();
    let build_seconds = build_started.elapsed().as_secs_f64();
    let oracles = config["oracles"].as_array().unwrap();
    assert_eq!(oracles.len(), 3);
    let mut evidence = Vec::new();
    for (index, oracle_path) in oracles.iter().enumerate() {
        let oracle = json(oracle_path.as_str().unwrap());
        hashes(&oracle);
        assert_eq!(oracle["capture"], capture);
        assert_eq!(oracle["complete"], true);
        assert_eq!(oracle["point_index"], index);
        let point = oracle["exact_point"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| rational(s.as_str().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(point.len(), 9);
        let input = |bits| {
            point
                .iter()
                .map(|p| Complex::new(p.to_multi_prec_float(bits), Float::with_val(bits, 0)))
                .collect::<Vec<_>>()
        };
        let low = mpfr(&exact, &input(512), 512);
        let high = mpfr(&exact, &input(1024), 1024);
        let oracle_rows = oracle["orders"].as_array().unwrap();
        assert_eq!(oracle_rows.len(), 6);
        for (offset, ((low, high), reference)) in low.iter().zip(&high).zip(oracle_rows).enumerate()
        {
            assert_eq!(reference["order"], offset as i32 - 5);
            assert_eq!(reference["precision_agreement"], true);
            let reference =
                Float::parse(reference["value_1024"].as_str().unwrap(), Some(1024)).unwrap();
            assert!(agrees(&low.re, &high.re) && agrees(&low.im, &high.im));
            assert!(
                agrees(&high.re, &reference) && agrees(&high.im, &Float::with_val(1024, 0)),
                "point {index}, order {}",
                offset as i32 - 5
            );
        }
        // Binary64 API inputs intentionally have their own MPFR reference.
        let rounded = point.iter().map(|p| p.to_f64()).collect::<Vec<_>>();
        let rounded_input = rounded
            .iter()
            .map(|p| Complex::new(Float::with_val(1024, *p), Float::with_val(1024, 0)))
            .collect::<Vec<_>>();
        let rounded_reference = mpfr(&exact, &rounded_input, 1024);
        let mut reports = Vec::new();
        for kernel in [&mut kernel, &mut cold] {
            for weight in [1.0, 1e40] {
                let mut values = vec![0.0; 6];
                let report = kernel
                    .evaluate_scaled(&rounded, &mut values, weight)
                    .unwrap();
                for (actual, reference) in values.iter().zip(&rounded_reference) {
                    let reference = (reference.re.clone() * Float::with_val(1024, weight)).to_f64();
                    assert!((actual - reference).abs() <= 2e-12 * reference.abs().max(1.0));
                }
                assert!(report.checked && report.rescued);
                reports.push(serde_json::json!({"weight":weight,"bits":report.bits,"rescued":report.rescued,"values":values}));
            }
        }
        evidence.push(serde_json::json!({"point_index":index,"exact_point":oracle["exact_point"],"all_six_orders_agree":true,
            "MPFR_1024":high.iter().map(|value| value.re.to_string()).collect::<Vec<_>>(),"production_reports":reports}));
    }
    fs::write(output.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "complete":true,"representative_index":38,"source_chart_index":46,"original_multiplicity":2,"multiplicity_applied":false,
        "orders":[-5,-4,-3,-2,-1,0],"production_pieces":241,"cancellation_terms":rows,
        "expansion_seconds":expansion_seconds,"build_encode_decode_two_backends_seconds":build_seconds,
        "encoded_bytes":encoded.len(),"elapsed_seconds":started.elapsed().as_secs_f64(),"points":evidence,
        "image_bijection_count":saved_bodies.len(),"pre_series_template_identical_after_handle_renaming":true,
        "coefficient_structural_comparison":structural_equality,
        "scope":"single captured representative; original subtraction expression, fresh production Laurent aliases and native kernels; no graph regeneration, chart metadata or integration claim"
    })).unwrap()).unwrap();
}
