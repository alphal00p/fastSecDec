use super::*;
#[test]
fn native_precision_controls_preserve_uncertainty_and_refuse_false_zero() {
    let normal = family(false);
    let continued = normal
        .continue_symbolically(&generation::GenerationOptions {
            mode: generation::GenerationMode::Symbolic,
            ..Default::default()
        })
        .unwrap();
    let mut kernels = Vec::new();
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        kernels.push(make_kernel(
            &normal,
            continued.expression(),
            continued.functions(),
            continued.profiles().iter().flatten().cloned().collect(),
            &[-1, 0, 1],
            backend,
        ));
    }
    let mut values = Vec::new();
    for kernel in &mut kernels {
        kernel.stability = StabilitySettings::validated();
        let mut rows = Vec::new();
        for x in [1e-8, 0.01, 0.2, 0.5, 0.9, 1.] {
            let mut out = vec![0.; kernel.output_count()];
            let report = kernel
                .evaluate_with_diagnostics(&[x, 0.37], &mut out)
                .unwrap();
            assert!(out.iter().all(|v| v.is_finite()));
            assert_eq!(out[0], 0.);
            assert_eq!(out[1], 0.);
            rows.push(serde_json::json!({"x":x,"values":out,"bits":report.bits,"checked":report.checked,"rescued":report.rescued}));
        }
        values.push(rows);
    }
    for (a, b) in values[0].iter().zip(&values[1]) {
        let a = a["values"].as_array().unwrap();
        let b = b["values"].as_array().unwrap();
        for (x, y) in a.iter().zip(b) {
            let x = x.as_f64().unwrap();
            let y = y.as_f64().unwrap();
            assert!((x - y).abs() < 2e-11 * (1. + x.abs()));
        }
    }
    let coordinate = Atom::var(normal.coordinates()[1]);
    let smooth = &continued.charts()[0] * (coordinate - Atom::num((1, 2)));
    // A nonstructural zero must be accepted through valid native tracking,
    // at a requested tolerance within the owner's finite error range.
    let mut zero_kernel = make_kernel_with_policy(
        &normal,
        &smooth,
        continued.functions(),
        continued.profiles()[0].clone(),
        &[0],
        EvaluatorBackend::Eager,
        &PrecisionPolicy {
            absolute_tolerance: 1e-20,
            ..Default::default()
        },
    );
    let mut zero = vec![0.; zero_kernel.output_count()];
    zero_kernel.evaluate(&[0.2, 0.5], &mut zero).unwrap();
    assert!(zero.iter().all(|v| *v == 0.));
    let mut tight_zero = make_kernel(
        &normal,
        &smooth,
        continued.functions(),
        continued.profiles()[0].clone(),
        &[0],
        EvaluatorBackend::Eager,
    );
    assert!(
        matches!(tight_zero.evaluate(&[0.2,0.5],&mut zero),Err(crate::kernel::KernelError::PrecisionEvaluation(message)) if message.contains("uncertainty") || message.contains("tracking range"))
    );
    let mut exact_zero = make_kernel(
        &normal,
        &Atom::zero(),
        continued.functions(),
        vec![],
        &[0],
        EvaluatorBackend::Eager,
    );
    exact_zero.evaluate(&[0.2, 0.5], &mut zero).unwrap();
    assert!(zero.iter().all(|v| *v == 0.));
    let close = family(true);
    let continued = close
        .continue_symbolically(&generation::GenerationOptions {
            mode: generation::GenerationMode::Symbolic,
            ..Default::default()
        })
        .unwrap();
    let mut close_kernels = Vec::new();
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        close_kernels.push(
            continued
                .charts()
                .iter()
                .zip(continued.profiles())
                .map(|(chart, profiles)| {
                    make_kernel(
                        &close,
                        chart,
                        continued.functions(),
                        profiles.clone(),
                        &[0],
                        backend,
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    drop(continued);
    drop(close);
    drop(normal);
    let mut collapsed = Vec::new();
    for kernels in &mut close_kernels {
        let mut rows = Vec::new();
        for kernel in kernels {
            kernel.stability = StabilitySettings::validated();
            let mut out = vec![0.; kernel.output_count()];
            let report = kernel
                .evaluate_with_diagnostics(&[1., 0.37], &mut out)
                .unwrap();
            println!("close output={out:?} report={report:?}");
            assert!(out[0] > 0. && out[1] == 0.);
            if out[0] < 0.5 {
                // P(1-w)=0 gives delta=6w−10w²+10w³−5w⁴+w⁵.
                // The correction to delta/6 is <1e−40; no subtraction of f64 roots.
                let expected = 1e-20 / 6.;
                assert!((out[0] / expected - 1.).abs() < 2e-12, "tiny width {out:?}");
                assert!(report.rescued && report.bits >= 128, "{report:?}");
            }
            rows.push(serde_json::json!({"values":out,"bits":report.bits,"checked":report.checked,"rescued":report.rescued}));
        }
        assert_eq!(rows.len(), 2);
        collapsed.push(rows);
    }
    let deep = family_digits(true, 200);
    let deep_continued = deep
        .continue_symbolically(&generation::GenerationOptions {
            mode: generation::GenerationMode::Symbolic,
            ..Default::default()
        })
        .unwrap();
    let mut deep_reports = Vec::new();
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Symjit] {
        let mut kernel = make_kernel(
            &deep,
            &deep_continued.charts()[1],
            deep_continued.functions(),
            deep_continued.profiles()[1].clone(),
            &[0],
            backend,
        );
        let mut out = vec![0.; kernel.output_count()];
        let failure = kernel
            .evaluate_with_diagnostics(&[1., 0.37], &mut out)
            .unwrap_err();
        assert!(
            matches!(&failure, crate::kernel::KernelError::PrecisionEvaluation(message) if message.contains("uncertainty") || message.contains("tracking range")),
            "{failure:?}"
        );
        deep_reports.push(format!("{failure:?}"));
        let limited = PrecisionPolicy {
            max_bits: 256,
            ..Default::default()
        };
        let mut kernel = make_kernel_with_policy(
            &deep,
            &deep_continued.charts()[1],
            deep_continued.functions(),
            deep_continued.profiles()[1].clone(),
            &[0],
            backend,
            &limited,
        );
        let failure = kernel.evaluate(&[1., 0.37], &mut out).unwrap_err();
        assert!(
            matches!(
                &failure,
                crate::kernel::KernelError::PrecisionExhausted { bits: 256 }
            ) || matches!(&failure, crate::kernel::KernelError::PrecisionEvaluation(message)
                    if message.contains("did not converge within 256 bits")
                        && message.contains("regular-section invalid corrected root")),
            "insufficient precision with retained root failure: {failure:?}"
        );
        deep_reports.push(format!("limited: {failure:?}"));
    }
    println!("deep root width {deep_reports:?}");
}

#[test]
fn small_prefix_point_recovers_nonzero_laurent_coefficient() {
    let family = family(false);
    let continued = family
        .continue_symbolically(&generation::GenerationOptions {
            mode: generation::GenerationMode::Symbolic,
            max_order: 1,
            ..Default::default()
        })
        .unwrap();
    let mut kernel = make_kernel(
        &family,
        &continued.charts()[1],
        continued.functions(),
        continued.profiles()[1].clone(),
        &[-1, 0, 1],
        EvaluatorBackend::Eager,
    );
    let mut output = vec![0.; kernel.output_count()];
    let report = kernel
        .evaluate_with_diagnostics(&[5.013127662430563e-11, 2.4768524871277252e-8], &mut output)
        .unwrap();
    let expected = -5.384119840425466e-33 / 3.4944883239153043e-12;
    assert!((output[4] / expected - 1.).abs() < 2e-12, "{output:?}");
    assert!(report.checked && report.rescued);
    let mut limited = make_kernel_with_policy(
        &family,
        &continued.charts()[1],
        continued.functions(),
        continued.profiles()[1].clone(),
        &[-1, 0, 1],
        EvaluatorBackend::Eager,
        &PrecisionPolicy {
            max_bits: 256,
            ..Default::default()
        },
    );
    let failure = limited
        .evaluate(&[5.013127662430563e-11, 2.4768524871277252e-8], &mut output)
        .unwrap_err();
    assert!(matches!(
        failure,
        crate::kernel::KernelError::PrecisionExhausted { bits: 256 }
    ));
}
