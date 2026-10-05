//! Test-only native program, weighted replay, and cold-process controls.
use super::*;
use crate::generation::{GenerationOptions, SubtractionStrategy};
use std::collections::BTreeSet;
use symbolica::{atom::AliasedAtom, parse, symbol};

fn points() -> Vec<Vec<Rational>> {
    [
        vec!["1/2", "1/3"],
        vec!["1/1000000000000", "37/100"],
        vec!["1/100000000", "1/100000000"],
    ]
    .iter()
    .map(|row| row.iter().map(|x| rational(x)).collect())
    .collect()
}

fn exact_input(point: &[Rational], bits: u32) -> Vec<Complex<Float>> {
    point
        .iter()
        .map(|x| Complex::new(x.to_multi_prec_float(bits), Float::with_val(bits, 0)))
        .collect()
}

fn check_program(
    exact: &ExactProgram,
    kernel: &mut SectorKernel,
    reference: &serde_json::Value,
) -> serde_json::Value {
    let started = Instant::now();
    assert_eq!(exact.get_output_len(), 4);
    assert_eq!(kernel.output_count(), 8, "paired complex full vector");
    let mut reports = Vec::new();
    for (index, point) in points().iter().enumerate() {
        assert_eq!(
            reference["points"][index]["exact_point"],
            serde_json::json!(point.iter().map(ToString::to_string).collect::<Vec<_>>())
        );
        let low = mpfr(exact, &exact_input(point, 512), 512);
        let high = mpfr(exact, &exact_input(point, 1024), 1024);
        for (offset, (low, high)) in low.iter().zip(&high).enumerate() {
            let row = &reference["points"][index]["exact_reference"][offset];
            let re = Float::parse(row["real"].as_str().unwrap(), Some(1024)).unwrap();
            let im = Float::parse(row["imag"].as_str().unwrap(), Some(1024)).unwrap();
            assert!(agrees(&low.re, &high.re) && agrees(&low.im, &high.im));
            assert!(agrees(&high.re, &re) && agrees(&high.im, &im));
        }
        let rounded = point.iter().map(|x| x.to_f64()).collect::<Vec<_>>();
        let input = rounded
            .iter()
            .map(|x| Complex::new(Float::with_val(1024, *x), Float::with_val(1024, 0)))
            .collect::<Vec<_>>();
        let expected = mpfr(exact, &input, 1024);
        for weight in [1.0, 1e40] {
            for forced in [false, true] {
                let mut values = vec![0.0; 8];
                let report = if forced {
                    kernel.replay_scaled(&rounded, &mut values, weight, 128)
                } else {
                    kernel.evaluate_scaled(&rounded, &mut values, weight)
                }
                .unwrap();
                if forced {
                    assert!(report.checked && report.rescued);
                }
                for (position, value) in expected.iter().enumerate() {
                    for (part, expected) in [&value.re, &value.im].into_iter().enumerate() {
                        let expected = (expected.clone() * Float::with_val(1024, weight)).to_f64();
                        let actual = values[2 * position + part];
                        assert!(actual.is_finite() && expected.is_finite());
                        assert!((actual - expected).abs() <= 2e-12 * expected.abs().max(1.0));
                    }
                }
                reports.push(serde_json::json!({"point":index,"weight":weight,"forced":forced,
                    "checked":report.checked,"rescued":report.rescued,"bits":report.bits,"values":values}));
            }
        }
    }
    serde_json::json!({"seconds":started.elapsed().as_secs_f64(),"reports":reports})
}

#[test]
#[ignore = "bounded named-coefficient writer; run as a separate process before cold reader"]
fn write_named_coefficient_programs() {
    let output = std::env::var("FASTSECDEC_NAMED_PROGRAM_OUTPUT").unwrap();
    let path = Path::new(&output);
    fs::create_dir(path).expect("new writer output");
    let x = symbol!("named_program_control::x");
    let y = symbol!("named_program_control::y");
    let eps = symbol!("named_program_control::eps");
    let parameters = vec![x, y];
    let epsilon = Atom::var(eps);
    let terms = vec![(
        parse!("gamma(2*named_program_control::eps)"),
        parse!(
            "(2+3*𝑖)*(1+named_program_control::x+named_program_control::y)^(-1-named_program_control::eps)"
        ),
        vec![&epsilon - 2, 2 * &epsilon - 1],
    )];
    for (i, parameter) in parameters.iter().enumerate() {
        Atom::var(*parameter)
            .export(&mut File::create(path.join(format!("parameter-{i}.atom"))).unwrap())
            .unwrap();
    }
    let mut records = Vec::new();
    for (label, strategy) in [
        ("Taylor", SubtractionStrategy::Taylor),
        ("IBP", SubtractionStrategy::IntegrateByParts),
    ] {
        let options = GenerationOptions {
            subtraction: strategy,
            ..GenerationOptions::default()
        };
        let (named, statistics) = crate::generation::captured_named_coefficients(
            terms.clone(),
            &parameters,
            eps,
            &options,
        )
        .unwrap();
        let (density, _, rows) =
            crate::generation::captured_subtraction(terms.clone(), &parameters, eps, &options)
                .unwrap();
        let expected = crate::generation::captured_coefficients(&density, &parameters, eps, 0)
            .unwrap()
            .coefficients;
        let union = named
            .keys()
            .chain(expected.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(union.iter().copied().collect::<Vec<_>>(), [-3, -2, -1, 0]);
        for order in &union {
            let actual = named
                .get(order)
                .cloned()
                .map(AliasedAtom::into_inner)
                .unwrap_or(Atom::Zero);
            let expected = expected
                .get(order)
                .cloned()
                .map(AliasedAtom::into_inner)
                .unwrap_or(Atom::Zero);
            let difference = actual - expected;
            assert!(difference.as_view().get_byte_size() < 128_000);
            assert!(
                difference.together().expand().is_zero(),
                "{label} order{order}"
            );
        }
        let orders = union.into_iter().collect::<Vec<_>>();
        let coefficients = orders
            .iter()
            .map(|o| named.get(o).cloned().unwrap_or_else(|| Atom::Zero.into()))
            .collect::<Vec<_>>();
        assert!(coefficients.iter().any(|c| !c.get_aliases().is_empty()));
        assert!(
            coefficients.iter().any(|c| !super::super::is_real(c)),
            "hidden complex native definitions retained"
        );
        assert!(
            coefficients.iter().any(|c| c
                .get_aliases()
                .values()
                .any(super::super::super::has_complex_coefficients)),
            "complex literal must occur in a retained native body"
        );
        let cancellation = Cancellation::new(
            rows.iter().map(|r| r.iter().sum()).max().unwrap_or(0),
            Some(rows.clone()),
            2,
        )
        .unwrap();
        let program_started = Instant::now();
        let program = build(parameters.clone(), &coefficients, cancellation.clone()).unwrap();
        let expected_coefficients = orders
            .iter()
            .map(|o| {
                expected
                    .get(o)
                    .cloned()
                    .unwrap_or_else(|| Atom::Zero.into())
            })
            .collect::<Vec<_>>();
        let baseline = build(
            parameters.clone(),
            &expected_coefficients,
            cancellation.clone(),
        )
        .unwrap();
        let exact = program.exact.clone();
        let encoded = encode(&exact).unwrap();
        let file = format!("{label}.native-ir.bin");
        fs::write(path.join(&file), &encoded).unwrap();
        let references = points().iter().map(|point| {
            let lower = mpfr(&baseline.exact,&exact_input(point,512),512);
            let values = mpfr(&baseline.exact,&exact_input(point,1024),1024);
            assert_eq!(lower.len(), 4);
            assert_eq!(values.len(), 4);
            for (low, high) in lower.iter().zip(&values) {
                assert!(agrees(&low.re, &high.re) && agrees(&low.im, &high.im));
            }
            serde_json::json!({"exact_point":point.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "baseline_precision_bits":[512,1024],"baseline_precision_agreement":true,
                "exact_reference":values.iter().map(|v| serde_json::json!({"real":v.re.to_string(),"imag":v.im.to_string()})).collect::<Vec<_>>()})
        }).collect::<Vec<_>>();
        let mut record = serde_json::json!({"strategy":label,"orders":orders,"cancellation":rows,"points":references,
            "program_file":file,"program_blake3":blake3::hash(&encoded).to_hex().to_string(),"native_ir_bytes":encoded.len(),"statistics":statistics});
        record["native_build_encode_baseline_reference_seconds"] =
            program_started.elapsed().as_secs_f64().into();
        let backend_started = Instant::now();
        let mut fresh =
            SectorKernel::from_program(program, &PrecisionPolicy::default(), true).unwrap();
        record["fresh_backend_seconds"] = backend_started.elapsed().as_secs_f64().into();
        record["fresh_checks"] = check_program(&exact, &mut fresh, &record);
        let mut worker = fresh.try_clone().unwrap();
        record["cloned_worker_checks"] = check_program(&exact, &mut worker, &record);
        let decoded = decode(&encoded).unwrap();
        let mut cold = SectorKernel::from_program(
            SectorProgram {
                parameters: parameters.clone(),
                exact: decoded.clone(),
                cancellation,
                exact_zero: vec![false; 4],
                real_coefficients: vec![false; 4],
            },
            &PrecisionPolicy::default(),
            true,
        )
        .unwrap();
        record["same_process_decoded_checks"] = check_program(&decoded, &mut cold, &record);
        records.push(record);
    }
    fs::write(path.join("writer.json"),serde_json::to_vec_pretty(&serde_json::json!({"complete":true,"records":records,
        "scope":"two small admitted densities, native complete vector and production weighted program checks; no actual graph or runtime claim"})).unwrap()).unwrap();
}

#[test]
#[ignore = "fresh-process native IR reader; no formal coefficient registry is constructed"]
fn read_named_coefficient_programs() {
    let input = std::env::var("FASTSECDEC_NAMED_PROGRAM_INPUT").unwrap();
    let output = std::env::var("FASTSECDEC_NAMED_READER_OUTPUT").unwrap();
    let source = Path::new(&input);
    let output = Path::new(&output);
    fs::create_dir(output).expect("new cold-reader output");
    let writer = json(source.join("writer.json"));
    assert_eq!(writer["complete"], true);
    assert_eq!(writer["records"].as_array().unwrap().len(), 2);
    assert_eq!(
        writer["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["strategy"].as_str().unwrap())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["Taylor", "IBP"])
    );
    let parameters = (0..2)
        .map(|i| symbol(&atom(source.join(format!("parameter-{i}.atom")))))
        .collect::<Vec<_>>();
    let mut records = Vec::new();
    for record in writer["records"].as_array().unwrap() {
        assert_eq!(record["orders"], serde_json::json!([-3, -2, -1, 0]));
        let bytes = fs::read(source.join(record["program_file"].as_str().unwrap())).unwrap();
        assert_eq!(
            blake3::hash(&bytes).to_hex().as_str(),
            record["program_blake3"].as_str().unwrap()
        );
        let exact = decode(&bytes).unwrap();
        let rows: Vec<Vec<usize>> = serde_json::from_value(record["cancellation"].clone()).unwrap();
        let cancellation = Cancellation::new(
            rows.iter().map(|r| r.iter().sum()).max().unwrap_or(0),
            Some(rows),
            2,
        )
        .unwrap();
        let mut kernel = SectorKernel::from_program(
            SectorProgram {
                parameters: parameters.clone(),
                exact: exact.clone(),
                cancellation,
                exact_zero: vec![false; 4],
                real_coefficients: vec![false; 4],
            },
            &PrecisionPolicy::default(),
            true,
        )
        .unwrap();
        records.push(serde_json::json!({"strategy":record["strategy"],"checks":check_program(&exact,&mut kernel,record)}));
    }
    fs::write(output.join("reader.json"),serde_json::to_vec_pretty(&serde_json::json!({"complete":true,"records":records,
        "scope":"fresh process; only native exact IR and source-bound expected values, no formal registry or callbacks"})).unwrap()).unwrap();
}
