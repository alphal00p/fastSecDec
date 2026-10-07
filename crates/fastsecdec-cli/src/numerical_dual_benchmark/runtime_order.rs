//! A separate process is required to exercise native symbol restoration order.
use super::*;

#[test]
fn cold_runtime_artifacts_are_loaded_before_point_symbols() {
    const CHILD_CONFIGURATION: &str = "FASTSECDEC_DUAL_BENCHMARK_COLD_CONTROL";
    if let Some(path) = std::env::var_os(CHILD_CONFIGURATION) {
        let report = run(Path::new(&path)).unwrap();
        assert_eq!(report["agreement"], true);
        for row in report["comparisons"][0]["points"].as_array().unwrap() {
            let coordinate = row["chart_points"][0][0].as_f64().unwrap();
            let value = row["reference"][0].as_f64().unwrap();
            assert!((value - 2.5 * (1.0 + coordinate)).abs() < 1e-12);
        }
        for case in report["cases"].as_array().unwrap() {
            let load = case["loading_seconds"].as_f64().unwrap();
            let bind = case["binding_seconds"].as_f64().unwrap();
            assert!(load >= 0.0 && bind >= 0.0);
            assert_eq!(case["loading_and_binding_seconds"], json!(load + bind));
        }
        return;
    }

    use crate::artifact::{Provenance, dependencies};
    use fastsecdec::{
        generation::{GenerationOptions, generate},
        kernel::{CompilationSettings, EvaluatorBackend, PrecisionPolicy},
        parametric::{
            FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
        },
    };
    use std::{ops::ControlFlow, process::Command};
    use symbolica::{
        atom::{Atom, SymbolAttribute, SymbolBuilder},
        parse, symbol, wrap_symbol,
    };
    let parameter = SymbolBuilder::new(wrap_symbol!("dual_bench_cold::real_parameter"))
        .with_attributes(&[SymbolAttribute::Real])
        .build()
        .unwrap();
    let input = ParametricIntegrand::new(
        vec![symbol!("dual_bench_cold::x")],
        symbol!("dual_bench_cold::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::var(parameter),
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("1+dual_bench_cold::x"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let kernels = generated
        .compile_with_settings_parameters_and_progress(
            PrecisionPolicy::default(),
            &[parameter],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let artifact = Artifact::new(
        &kernels,
        Provenance {
            name: "cold benchmark runtime point".into(),
            sources: vec![],
            dependencies: dependencies(),
            domain: "UnitCube".into(),
            assume_no_threshold: false,
            dimension: "4-2*dual_bench_cold::eps".into(),
            regulator: "dual_bench_cold::eps".into(),
            measure: "native".into(),
            measure_multiplier: "1".into(),
            max_order: 0,
            integration: json!({}),
            family_preparation: None,
            model_parameter_defaults: BTreeMap::new(),
        },
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path().join("real.fsd");
    artifact.save(&base).unwrap();
    let parameters = directory.path().join("point.toml");
    fs::write(
        &parameters,
        "[parameters]\n\"dual_bench_cold::real_parameter\" = 2.5\n",
    )
    .unwrap();
    let path = directory.path().join("configuration.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "cases": (0..4).map(|index| json!({"name":format!("case{index}"),"artifact":base})).collect::<Vec<_>>(),
            "pairs":[["case0","case1"]],"parameters":parameters,
            "batch_rows":4,"timing_batches":1,"comparison_points":1,
        }))
        .unwrap(),
    )
    .unwrap();
    let result = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "numerical_dual_benchmark::runtime_order::cold_runtime_artifacts_are_loaded_before_point_symbols",
            "--nocapture",
        ])
        .env(CHILD_CONFIGURATION, &path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "fresh-process runtime restoration failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
