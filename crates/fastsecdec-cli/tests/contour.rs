//! Physical contour checks across real CLI process and artifact boundaries.
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

fn integration(artifact: &Path, serial: bool, policy: &str, checkpoint: &Path) -> Command {
    let mut command = cli();
    command
        .arg("integrate")
        .arg(artifact)
        .args([
            "--contour",
            "fixed",
            "--lambda",
            "0.2",
            "--contour-validation",
            policy,
            "--contour-pilot-points",
            "8",
            "--points",
            "1024",
            "--shifts",
            "4",
            "--max-rounds",
            "1",
            "--workers",
            "2",
            "--seed",
            "613",
        ])
        .arg("--checkpoint")
        .arg(checkpoint);
    if serial {
        command.args(["--serial", "0.001"]);
    }
    command
}

fn analytic_bubble(report: &Value) {
    let beta = (1f64 / 5.).sqrt();
    let real = 2. - beta * ((1. + beta) / (1. - beta)).ln();
    let imag = std::f64::consts::PI * beta;
    let estimate = &report["estimate"];
    for (index, (order, component)) in estimate["orders"]
        .as_array()
        .unwrap()
        .iter()
        .zip(estimate["components"].as_array().unwrap())
        .enumerate()
    {
        let expected = match (order.as_i64().unwrap(), component.as_str().unwrap()) {
            (-1, "Real") => 1.,
            (-1, "Imag") => 0.,
            (0, "Real") => real,
            (0, "Imag") => imag,
            other => panic!("unexpected coefficient {other:?}"),
        };
        let value = estimate["mean"][index].as_f64().unwrap();
        let error = estimate["standard_error"][index].as_f64().unwrap();
        assert!(
            (value - expected).abs() <= 8. * error + 2e-6,
            "coefficient {index}: {value} +/- {error}, expected {expected}"
        );
    }
}

#[test]
fn fixed_contour_universal_artifacts_policies_and_resume_preserve_sampling() {
    let directory = tempfile::tempdir().unwrap();
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/threshold_bubble.toml");
    for mode in ["symbolic", "numerical_dual"] {
        let mut card: toml::Value = toml::from_str(&fs::read_to_string(&source).unwrap()).unwrap();
        card["generation"]
            .as_table_mut()
            .unwrap()
            .insert("mode".into(), mode.into());
        // Exercise the generation switch, independently of the runtime settings.
        card["generation"].as_table_mut().unwrap().remove("contour");
        card["integration"]
            .as_table_mut()
            .unwrap()
            .insert("absolute_tolerance".into(), 0.0.into());
        card["integration"]["relative_tolerance"] = 0.0.into();
        let input = directory.path().join(format!("{mode}.toml"));
        fs::write(&input, toml::to_string(&card).unwrap()).unwrap();
        let mut mathematical_identity = None;
        for serial_generation in [false, true] {
            let base = directory
                .path()
                .join(format!("{mode}-{serial_generation}.fsd"));
            let mut generate = cli();
            generate
                .arg("generate")
                .arg(&input)
                .args(["--contour", "--workers", "2", "--output"])
                .arg(&base);
            if serial_generation {
                generate.arg("--serial");
            }
            let generated = success(generate.output().unwrap());
            if let Some(identity) = &mathematical_identity {
                assert_eq!(identity, &generated["content_id"]);
            } else {
                mathematical_identity = Some(generated["content_id"].clone());
            }
            let manifest_before = fs::read(base.with_extension("fsd.json")).unwrap();
            for serial_integration in [false, true] {
                let mut means: Option<Vec<Value>> = None;
                for policy in ["always", "pilot", "off"] {
                    let checkpoint = directory.path().join(format!(
                        "{mode}-{serial_generation}-{serial_integration}-{policy}.checkpoint.json"
                    ));
                    let report = success(
                        integration(&base, serial_integration, policy, &checkpoint)
                            .output()
                            .unwrap(),
                    );
                    analytic_bubble(&report);
                    assert_eq!(report["stopping_reason"], "work limit");
                    assert_eq!(report["contour"]["validation"]["policy"], policy);
                    let checks =
                        report["contour"]["invocation_checks"]["production"]["checked_arguments"]
                            .as_u64()
                            .unwrap();
                    assert_eq!(checks > 0, policy == "always");
                    if policy != "off" {
                        assert!(
                            report["contour"]["pilots"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|pilot| pilot["complete"] == true
                                    && pilot["sampled_points"].as_u64().unwrap() >= 8)
                        );
                    }
                    let actual = report["estimate"]["mean"].as_array().unwrap().clone();
                    if let Some(expected) = &means {
                        for (a, b) in actual.iter().zip(expected) {
                            assert!(
                                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-12,
                                "validation policy changed production points or values"
                            );
                        }
                    } else {
                        means = Some(actual);
                    }
                    if policy == "always" {
                        let resumed = success(
                            integration(&base, serial_integration, "off", &checkpoint)
                                .arg("--resume")
                                .output()
                                .unwrap(),
                        );
                        assert_eq!(resumed["estimate"], report["estimate"]);
                        assert_eq!(
                            resumed["contour"]["invocation_checks"]["production"]["checked_arguments"],
                            0
                        );
                        assert!(
                            !resumed["contour"]["pilots"].as_array().unwrap().is_empty(),
                            "policy-only continuation lost durable pilot evidence"
                        );
                    }
                }
            }
            assert_eq!(
                fs::read(base.with_extension("fsd.json")).unwrap(),
                manifest_before
            );
        }
    }
}

#[test]
fn run_forwards_fixed_contour_to_serial_subtraction_and_exact_offsets() {
    let directory = tempfile::tempdir().unwrap();
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/subtracted_pole.toml");
    let mut card: toml::Value = toml::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    card["generation"].as_table_mut().unwrap().remove("contour");
    card["integration"]
        .as_table_mut()
        .unwrap()
        .remove("contour");
    let input = directory.path().join("pole.toml");
    fs::write(&input, toml::to_string(&card).unwrap()).unwrap();
    let report = success(
        cli()
            .arg("run")
            .arg(input)
            .arg("--output")
            .arg(directory.path().join("pole.fsd"))
            .args([
                "--contour",
                "fixed",
                "--lambda",
                "0.2",
                "--contour-validation",
                "pilot",
                "--contour-pilot-points",
                "8",
                "--serial",
                "0.001",
                "--generation-workers",
                "2",
                "--workers",
                "2",
                "--points",
                "1024",
                "--shifts",
                "8",
                "--max-rounds",
                "1",
            ])
            .output()
            .unwrap(),
    );
    let estimate = &report["estimate"];
    for (index, (order, component)) in estimate["orders"]
        .as_array()
        .unwrap()
        .iter()
        .zip(estimate["components"].as_array().unwrap())
        .enumerate()
    {
        let expected = match (order.as_i64().unwrap(), component.as_str().unwrap()) {
            (-1, "Real") => -4.,
            (-1, "Imag") => 0.,
            (0, "Real") => -4. * 3f64.ln(),
            (0, "Imag") => 4. * std::f64::consts::PI,
            other => panic!("unexpected coefficient {other:?}"),
        };
        let value = estimate["mean"][index].as_f64().unwrap();
        let error = estimate["standard_error"][index].as_f64().unwrap();
        assert!((value - expected).abs() < 8. * error + 2e-6);
    }
    assert!(!report["contour"]["pilots"].as_array().unwrap().is_empty());
}

#[test]
fn diagnostic_commands_bind_contours_and_report_checks_without_production_statistics() {
    let directory = tempfile::tempdir().unwrap();
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/threshold_bubble.toml");
    let base = directory.path().join("diagnostics.fsd");
    success(
        cli()
            .arg("generate")
            .arg(input)
            .arg("--output")
            .arg(&base)
            .output()
            .unwrap(),
    );
    for policy in ["always", "pilot", "off"] {
        for name in ["benchmark", "check-boundaries"] {
            let mut command = cli();
            command.arg(name).arg(&base).args([
                "--contour",
                "fixed",
                "--lambda",
                "0.1",
                "--contour-validation",
                policy,
                "--contour-pilot-points",
                "8",
            ]);
            if name == "benchmark" {
                command.args(["--points", "16", "--repetitions", "1"]);
            } else {
                command.args([
                    "--exponents",
                    "3,6",
                    "--max-codimension",
                    "1",
                    "--max-probes",
                    "8",
                ]);
            }
            let report = success(command.output().unwrap());
            let contour = &report["contour"];
            assert_eq!(contour["settings"]["deformation"]["lambda"], 0.1);
            assert_eq!(contour["settings"]["validation"]["policy"], policy);
            assert_eq!(contour["pilot"].is_null(), policy == "off");
            if policy != "off" {
                assert_eq!(contour["pilot"]["complete"], true);
                assert!(contour["pilot"]["sampled_points"].as_u64().unwrap() >= 8);
            }
            let checked = contour["diagnostic_checks"]["checked_arguments"]
                .as_u64()
                .unwrap();
            assert_eq!(checked > 0, policy == "always");
            assert!(
                report.get("estimate").is_none(),
                "diagnostic points are not integration observations"
            );
            if name == "benchmark" {
                let sectors = report["sectors"].as_array().unwrap();
                for sector in sectors {
                    assert_eq!(sector["warmup_evaluations"], 16);
                    assert_eq!(sector["measurements"][0]["evaluations"], 16);
                    assert_eq!(sector["measurements"][0]["complete"], true);
                }
                assert_eq!(contour["checked_work"], "warmup_and_measurements");
                if policy == "always" {
                    assert_eq!(
                        checked,
                        32 * sectors.len() as u64,
                        "warmup checks were lost or counted twice"
                    );
                }
            } else {
                assert_eq!(contour["checked_work"], "boundary_probes");
                let native: fastsecdec::diagnostics::BoundaryScanReport =
                    serde_json::from_value(report.clone()).unwrap();
                assert_eq!(native.diagnostics.failures, 0);
                if policy == "always" {
                    assert_eq!(checked, report["completed_probes"].as_u64().unwrap());
                }
            }
        }
    }
    // This existing native graph retains a physical runtime scalar. The helper
    // must bind the artifact's recorded point even when no contour is requested.
    let ordinary = directory.path().join("ordinary.fsd");
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs/bubble.toml");
    success(
        cli()
            .arg("generate")
            .arg(input)
            .arg("--output")
            .arg(&ordinary)
            .output()
            .unwrap(),
    );
    let report = success(
        cli()
            .arg("benchmark")
            .arg(ordinary)
            .args(["--points", "16", "--repetitions", "1"])
            .output()
            .unwrap(),
    );
    assert_eq!(report["stop"], "Complete");
    assert!(report.get("contour").is_none());
}
