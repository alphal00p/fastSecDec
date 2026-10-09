//! Complete CLI paths for exact offsets, native runtime inputs and serial run cards.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn coefficients(value: &Value) -> Vec<f64> {
    value["estimate"]["mean"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_f64().unwrap())
        .collect()
}

#[test]
fn run_serial_rebinds_hepkit_runtime_inputs_for_exact_and_sampled_integrals() {
    let directory = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs/bubble.toml");
    for dimension in [2, 4] {
        // Reuse the actual HEPKit graph/model/card. At order zero this bubble
        // is exact in D=2-2eps and retains a sampled sector in D=4-2eps.
        let mut card: toml::Value = toml::from_str(&fs::read_to_string(&source).unwrap()).unwrap();
        for key in ["graph", "model", "parameter_card"] {
            let relative = card["input"][key].as_str().unwrap();
            card["input"][key] = source
                .parent()
                .unwrap()
                .join(relative)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .into();
        }
        card["integral"]["dimension"] = format!("{dimension}-2*eps").into();
        let input = directory.path().join(format!("bubble-{dimension}.toml"));
        fs::write(&input, toml::to_string(&card).unwrap()).unwrap();
        let artifact = directory.path().join(format!("bubble-{dimension}.fsd"));
        let first = success(
            cli()
                .arg("run")
                .arg(&input)
                .arg("--output")
                .arg(&artifact)
                .args([
                    "--serial",
                    "0.001",
                    "--generation-workers",
                    "2",
                    "--workers",
                    "2",
                    "--points",
                    "1024",
                    "--shifts",
                    "4",
                    "--max-rounds",
                    "1",
                    "--seed",
                    "456",
                ])
                .output()
                .unwrap(),
        );
        assert!(artifact.with_extension("fsd.generation").is_dir());
        let metadata_path = artifact.with_extension("fsd.json");
        let metadata_bytes = fs::read(&metadata_path).unwrap();
        let metadata: Value = serde_json::from_slice(&metadata_bytes).unwrap();
        assert_eq!(
            metadata["kernel"]["runtime_parameters"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let data_path = directory
            .path()
            .join(metadata["programs"]["data_file"].as_str().unwrap());
        let data_bytes = fs::read(&data_path).unwrap();
        let second = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args([
                    "--serial",
                    "0.001",
                    "--workers",
                    "1",
                    "--points",
                    "1024",
                    "--shifts",
                    "4",
                    "--max-rounds",
                    "1",
                    "--seed",
                    "456",
                    "--parameter",
                    "p1_dot_p1=-4",
                ])
                .output()
                .unwrap(),
        );
        let first_mean = coefficients(&first);
        let second_mean = coefficients(&second);
        assert_eq!(
            first["estimate"]["orders"],
            serde_json::json!([-1, -1, 0, 0])
        );
        assert_eq!(
            first["estimate"]["components"],
            serde_json::json!(["Real", "Imag", "Real", "Imag"])
        );
        let scale = if dimension == 2 { 0.25 } else { 1.0 };
        let pole = if dimension == 2 { -2.0 } else { 1.0 };
        assert!((first_mean[0] - pole).abs() < 1e-9);
        assert!((second_mean[0] - scale * first_mean[0]).abs() < 1e-9);
        // I(-4) = 4^(D/2-2) I(-1), retaining the regulator in D.
        assert!(
            (second_mean[2] - scale * (first_mean[2] - first_mean[0] * 4f64.ln())).abs() < 1e-9
        );
        for report in [&first, &second] {
            assert!(coefficients(report)[1].abs() < 1e-12);
            assert!(coefficients(report)[3].abs() < 1e-12);
            if dimension == 2 {
                assert!(report["snapshot"]["sectors"].as_array().unwrap().is_empty());
                assert_eq!(report["operational"]["evaluations"], 0);
                assert_eq!(report["snapshot"]["uncertainty"], "Exact");
                assert_eq!(
                    report["contributions"]["exact_coefficients"],
                    report["estimate"]["mean"]
                );
            } else {
                assert!(!report["snapshot"]["sectors"].as_array().unwrap().is_empty());
                assert!(report["operational"]["evaluations"].as_u64().unwrap() > 0);
            }
        }
        assert_eq!(fs::read(&metadata_path).unwrap(), metadata_bytes);
        assert_eq!(fs::read(&data_path).unwrap(), data_bytes);
    }
}

#[test]
fn card_enabled_serial_integration_adds_exact_offsets_once_after_ordinary_generation() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("mixed.toml");
    fs::write(
        &input,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["-1+eps","0"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1"
[[direct.terms.factors]]
polynomial="x+y"
exponent="1"
role="polynomial"
[generation]
order=0
[generation.evaluator]
backend="eager"
[integration]
serial_seconds=0.001
points=1024
shifts=8
max_rounds=1
workers=2
absolute_tolerance=0.0
relative_tolerance=0.0
"#,
    )
    .unwrap();
    let artifact = directory.path().join("mixed.fsd");
    let report = success(
        cli()
            .arg("run")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .args(["--generation-workers", "2"])
            .output()
            .unwrap(),
    );
    // The two factors cancel exactly: the original integral is 1/eps.
    // Decomposition nevertheless gives one exact chart (+1 at eps^0) and a
    // sampled chart (-1 there). This checks the new exact-setup child boundary.
    assert_eq!(report["estimate"]["orders"], serde_json::json!([-1, 0]));
    assert_eq!(
        report["contributions"]["exact_coefficients"],
        serde_json::json!([0.0, 1.0])
    );
    assert_eq!(report["snapshot"]["sectors"].as_array().unwrap().len(), 1);
    for (index, expected) in [1.0, 0.0].into_iter().enumerate() {
        let mean = report["estimate"]["mean"][index].as_f64().unwrap();
        let error = report["estimate"]["standard_error"][index]
            .as_f64()
            .unwrap();
        assert!((mean - expected).abs() < 8.0 * error + 1e-8);
    }
    assert_eq!(report["converged"], false);
    assert_eq!(report["stopping_reason"], "work limit");
    assert!(!artifact.with_extension("fsd.generation").exists());
}

#[test]
fn exact_only_complex_serial_card_and_resume_never_allocate_sampling_work() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("exact.toml");
    fs::write(
        &input,
        r#"
[direct]
domain="unit_cube"
parameters=[]
[[direct.terms]]
prefactor="2+3𝑖+(5-𝑖)*eps"
[generation]
serial=true
order=1
[generation.evaluator]
backend="eager"
[integration]
serial_seconds=60.0
workers=2
"#,
    )
    .unwrap();
    let artifact = directory.path().join("exact.fsd");
    let checkpoint = directory.path().join("exact.checkpoint.json");
    let report = success(
        cli()
            .arg("run")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .args(["--generation-workers", "2"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        report["estimate"]["mean"],
        serde_json::json!([2.0, 3.0, 5.0, -1.0])
    );
    assert_eq!(
        report["estimate"]["standard_error"],
        serde_json::json!([0.0, 0.0, 0.0, 0.0])
    );
    assert_eq!(report["snapshot"]["completed_points"], 0);
    assert!(report["snapshot"]["sectors"].as_array().unwrap().is_empty());
    assert_eq!(report["operational"]["evaluations"], 0);
    assert_eq!(report["snapshot"]["uncertainty"], "Exact");
    assert_eq!(report["converged"], true);
    let resumed = success(
        cli()
            .arg("integrate")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .args(["--resume", "--workers", "3", "--serial", "0.01"])
            .output()
            .unwrap(),
    );
    assert_eq!(resumed["estimate"], report["estimate"]);
    assert_eq!(resumed["operational"]["evaluations"], 0);
}
