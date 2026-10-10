//! Selective recipe loading through the real CLI, including disposable workers.
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

// Exercise native family publication through both production CLI schedulers.
fn bundle(directory: &Path, serial: bool, mode: &str) -> std::path::PathBuf {
    let input = directory.join("input.toml");
    fs::write(
        &input,
        format!(
            r#"
[direct]
domain="unit_cube"
parameters=["x"]
[[direct.terms]]
monomial_powers=["0"]
[[direct.terms.factors]]
polynomial="1+x"
exponent="-1"
semantics="causal"
[generation]
mode="{mode}"
[generation.evaluator]
backend="eager"
[integration]
absolute_tolerance=0.0
relative_tolerance=0.0
"#
        ),
    )
    .unwrap();
    let path = directory.join("family.fsd");
    let mut command = cli();
    command
        .arg("generate")
        .arg(&input)
        .args(["--contour", "--workers", "2", "--output"])
        .arg(&path);
    if serial {
        command.arg("--serial");
    }
    let generated = success(command.output().unwrap());
    assert_eq!(generated["workers"], 2);
    assert_eq!(generated["default_recipe"], "undeformed-v1");
    assert_eq!(generated["selected_recipe"], "undeformed-v1");
    assert_eq!(generated["available_recipes"].as_array().unwrap().len(), 4);
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.with_extension("fsd.json")).unwrap()).unwrap();
    assert_eq!(manifest["programs"]["default_recipe"], "undeformed-v1");
    assert_eq!(
        manifest["programs"]["catalogue"]["recipes"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let inspected = success(cli().arg("inspect").arg(&path).output().unwrap());
    assert_eq!(inspected["selected_recipe"], "undeformed-v1");
    if serial {
        success(
            cli()
                .arg("generate")
                .arg(&input)
                .args(["--contour", "--workers", "1", "--resume", "--output"])
                .arg(&path)
                .output()
                .unwrap(),
        );
        let before = fs::read(path.with_extension("fsd.json")).unwrap();
        assert!(
            !cli()
                .arg("generate")
                .arg(&input)
                .args(["--recipe", "fixed-v1", "--resume", "--output"])
                .arg(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert_eq!(before, fs::read(path.with_extension("fsd.json")).unwrap());
    }
    path
}

#[test]
fn explicit_recipes_work_in_resident_and_serial_cli_with_full_covariance() {
    for (generation_mode, serial_generation) in [("symbolic", false), ("numerical_dual", true)] {
        let directory = tempfile::tempdir().unwrap();
        let path = bundle(directory.path(), serial_generation, generation_mode);
        for (recipe, mode, construction) in [
            ("undeformed-v1", "off", None),
            ("fixed-v1", "fixed", None),
            ("dynamic-polynomial-v1", "dynamical=0.8", Some("polynomial")),
            ("dynamic-sign-aware-v1", "dynamical=0.8", Some("sign_aware")),
        ] {
            let inspected = success(
                cli()
                    .arg("inspect")
                    .arg(&path)
                    .args(["--recipe", recipe, "--validate-artifact"])
                    .output()
                    .unwrap(),
            );
            assert_eq!(inspected["selected_recipe"], recipe);
            assert_eq!(inspected["binary_loaded"], false);
            let deep = success(
                cli()
                    .arg("inspect")
                    .arg(&path)
                    .args([
                        "--recipe",
                        recipe,
                        "--sector",
                        "0",
                        "--deep",
                        "--validate-artifact",
                    ])
                    .output()
                    .unwrap(),
            );
            assert_eq!(deep["selected_recipe"], recipe);
            assert_eq!(deep["loaded_sectors"], 1);
            let all = success(
                cli()
                    .arg("inspect")
                    .arg(&path)
                    .args(["--recipe", recipe, "--deep", "--validate-artifact"])
                    .output()
                    .unwrap(),
            );
            for view in [&deep, &all] {
                assert_eq!(view["selected_recipe"], recipe);
                assert_eq!(view["available_recipes"], inspected["available_recipes"]);
                assert_eq!(
                    view["selected_catalogue_content_id"],
                    inspected["selected_catalogue_content_id"]
                );
            }
            assert!(all["selected_catalogue_content_id"].is_string());
            for serial in [false, true] {
                let checkpoint = directory
                    .path()
                    .join(format!("{recipe}-{serial}.checkpoint.json"));
                let saved = directory
                    .path()
                    .join(format!("{recipe}-{serial}.result.json"));
                let mut command = cli();
                command
                    .arg("integrate")
                    .arg(&path)
                    .args([
                        "--contour",
                        mode,
                        "--contour-validation",
                        "always",
                        "--contour-pilot-points",
                        "8",
                        "--points",
                        "1024",
                        "--shifts",
                        "4",
                        "--workers",
                        "2",
                        "--max-rounds",
                        "1",
                        "--seed",
                        "77237",
                        "--validate-artifact",
                    ])
                    .arg("--checkpoint")
                    .arg(&checkpoint)
                    .arg("--save-result")
                    .arg(&saved);
                if mode == "fixed" {
                    command.args(["--lambda", "0.2"]);
                }
                if let Some(construction) = construction {
                    command.args(["--contour-construction", construction]);
                }
                if serial {
                    command.args(["--serial", "0.001"]);
                }
                let report = success(command.output().unwrap());
                let estimate = &report["estimate"];
                let native: fastsecdec::integration::VectorEstimate =
                    serde_json::from_value(estimate.clone()).unwrap();
                native.validate().unwrap();
                // Numerical-dual causal templates may retain a zero imaginary
                // slot even for the undeformed recipe. Test the complete native
                // selected layout, rather than guessing it from the mode.
                assert_eq!(estimate["orders"], all["orders"]);
                assert_eq!(estimate["components"], all["components"]);
                assert!(native.orders.iter().all(|order| *order == 0));
                let expected_width = all["orders"].as_array().unwrap().len();
                assert_eq!(native.mean.len(), expected_width);
                assert_eq!(
                    native.covariance_of_mean.len(),
                    expected_width * expected_width
                );
                let components = estimate["components"].as_array().unwrap();
                for (index, component) in components.iter().enumerate() {
                    let expected = if component == "Real" { 2f64.ln() } else { 0. };
                    let mean = estimate["mean"][index].as_f64().unwrap();
                    let error = estimate["standard_error"][index].as_f64().unwrap();
                    assert!(
                        (mean - expected).abs() < 8. * error + 1e-6,
                        "{recipe} serial={serial}: {estimate}"
                    );
                }
                let saved = fastsecdec::results::read_result(&fs::read(saved).unwrap()).unwrap();
                assert_eq!(saved.provenance.attributes["selected_recipe"], recipe);
                let incompatible = if mode == "fixed" { "off" } else { "fixed" };
                let mut resume = cli();
                resume
                    .arg("integrate")
                    .arg(&path)
                    .args([
                        "--contour",
                        incompatible,
                        "--resume",
                        "--max-rounds",
                        "1",
                        "--points",
                        "1024",
                        "--shifts",
                        "4",
                        "--seed",
                        "77237",
                    ])
                    .arg("--checkpoint")
                    .arg(&checkpoint);
                if incompatible == "fixed" {
                    resume.args(["--lambda", "0.2"]);
                }
                if serial {
                    resume.args(["--serial", "0.001"]);
                }
                assert!(
                    !resume.output().unwrap().status.success(),
                    "cross-recipe checkpoint was accepted"
                );
                if let Some(construction) = construction {
                    // Both dynamic recipes exist: refusal must come from the
                    // completed checkpoint identity, not missing capability.
                    let other = if construction == "polynomial" {
                        "sign_aware"
                    } else {
                        "polynomial"
                    };
                    let before = fs::read(&checkpoint).unwrap();
                    let mut changed = cli();
                    changed
                        .arg("integrate")
                        .arg(&path)
                        .args([
                            "--contour",
                            "dynamical=0.8",
                            "--contour-construction",
                            other,
                            "--contour-validation",
                            "off",
                            "--points",
                            "1024",
                            "--shifts",
                            "4",
                            "--seed",
                            "77237",
                            "--max-rounds",
                            "1",
                            "--resume",
                        ])
                        .arg("--checkpoint")
                        .arg(&checkpoint);
                    if serial {
                        changed.args(["--serial", "0.001"]);
                    }
                    let output = changed.output().unwrap();
                    assert!(!output.status.success());
                    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
                    assert!(
                        report["error"]["message"]
                            .as_str()
                            .unwrap()
                            .contains("differ"),
                        "{report}"
                    );
                    assert_eq!(fs::read(&checkpoint).unwrap(), before);
                }
            }
        }
    }
}

#[cfg(unix)]
#[path = "contour_programs/cancellation.rs"]
mod cancellation;
