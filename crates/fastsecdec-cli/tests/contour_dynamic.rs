//! Dynamic singleton recipes across actual CLI generation, restore and workers.
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

fn card(path: &Path, mode: &str) {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/threshold_bubble.toml");
    let mut card: toml::Value = toml::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    let generation = card["generation"].as_table_mut().unwrap();
    // Deliberately retain contour=true: explicit native recipe selection wins.
    generation.insert("mode".into(), mode.into());
    let integration = card["integration"].as_table_mut().unwrap();
    integration.remove("contour");
    integration.insert("absolute_tolerance".into(), 0.0.into());
    integration.insert("relative_tolerance".into(), 0.0.into());
    fs::write(path, toml::to_string(&card).unwrap()).unwrap();
}

fn runtime(command: &mut Command, construction: &str, policy: &str) {
    runtime_with(command, construction, policy, "0.8", ["1", "1"], "8");
}

fn runtime_with(
    command: &mut Command,
    construction: &str,
    policy: &str,
    safety: &str,
    caps: [&str; 2],
    pilot: &str,
) {
    command.args([
        "--contour",
        &format!("dynamical={safety}"),
        "--lambda-cap",
        caps[0],
        "--displacement-cap",
        caps[1],
        "--contour-construction",
        construction,
        "--contour-validation",
        policy,
        "--contour-pilot-points",
        pilot,
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
    ]);
}

fn analytic_bubble(report: &Value, construction: &str, policy: &str) {
    let beta = (1f64 / 5.).sqrt();
    let real = 2. - beta * ((1. + beta) / (1. - beta)).ln();
    let imag = std::f64::consts::PI * beta;
    let estimate = &report["estimate"];
    assert_eq!(estimate["orders"], serde_json::json!([-1, -1, 0, 0]));
    assert_eq!(
        estimate["components"],
        serde_json::json!(["Real", "Imag", "Real", "Imag"])
    );
    assert_eq!(estimate["mean"].as_array().unwrap().len(), 4);
    assert_eq!(estimate["standard_error"].as_array().unwrap().len(), 4);
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
    assert_eq!(report["contour"]["deformation"]["mode"], "dynamical");
    assert_eq!(
        report["contour"]["deformation"]["construction"],
        construction
    );
    assert_eq!(report["contour"]["validation"]["policy"], policy);
    let checks = report["contour"]["invocation_checks"]["production"]["checked_arguments"]
        .as_u64()
        .unwrap();
    assert_eq!(checks > 0, policy == "always");
    if policy != "off" {
        assert!(report["contour"]["pilots"].as_array().unwrap().iter().any(
            |pilot| pilot["complete"] == true && pilot["sampled_points"].as_u64().unwrap() >= 8
        ));
    }
}

#[test]
fn dynamic_singletons_restore_both_recipes_and_preserve_policy_sampling() {
    let directory = tempfile::tempdir().unwrap();
    for (recipe, construction) in [
        ("dynamic-polynomial-v1", "polynomial"),
        ("dynamic-sign-aware-v1", "sign_aware"),
    ] {
        for (mode, serial_generation) in [("symbolic", false), ("numerical_dual", true)] {
            let input = directory.path().join(format!("{recipe}-{mode}.toml"));
            card(&input, mode);
            let artifact = input.with_extension("fsd");
            let mut generate = cli();
            generate
                .arg("generate")
                .arg(&input)
                .args(["--recipe", recipe, "--workers", "2", "--output"])
                .arg(&artifact);
            if serial_generation {
                generate.arg("--serial");
            }
            success(generate.output().unwrap());
            let before = fs::read(artifact.with_extension("fsd.json")).unwrap();
            for serial_integration in [false, true] {
                let mut means: Option<Value> = None;
                for policy in ["always", "pilot", "off"] {
                    let checkpoint = directory.path().join(format!(
                        "{recipe}-{mode}-{serial_integration}-{policy}.checkpoint"
                    ));
                    let mut integrate = cli();
                    integrate.arg("integrate").arg(&artifact);
                    runtime(&mut integrate, construction, policy);
                    integrate.arg("--checkpoint").arg(&checkpoint);
                    if serial_integration {
                        integrate.args(["--serial", "0.001"]);
                    }
                    let report = success(integrate.output().unwrap());
                    analytic_bubble(&report, construction, policy);
                    if let Some(expected) = &means {
                        for (a, b) in report["estimate"]["mean"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .zip(expected.as_array().unwrap())
                        {
                            assert!(
                                (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-12,
                                "policy changed production points or values"
                            );
                        }
                    } else {
                        means = Some(report["estimate"]["mean"].clone());
                    }
                    if policy == "always" {
                        let mut resume = cli();
                        resume.arg("integrate").arg(&artifact);
                        // Validation strength/count are observational; changing
                        // either must preserve accepted production history.
                        runtime_with(&mut resume, construction, "off", "0.8", ["1", "1"], "17");
                        resume.arg("--checkpoint").arg(&checkpoint).arg("--resume");
                        if serial_integration {
                            resume.args(["--serial", "0.001"]);
                        }
                        let resumed = success(resume.output().unwrap());
                        assert_eq!(resumed["estimate"], report["estimate"]);
                        assert_eq!(
                            resumed["contour"]["invocation_checks"]["production"]["checked_arguments"],
                            0
                        );
                        assert!(!resumed["contour"]["pilots"].as_array().unwrap().is_empty());

                        let checkpoint_before = fs::read(&checkpoint).unwrap();
                        for (safety, caps) in [
                            ("0.7", ["1", "1"]),
                            ("0.8", ["0.9", "1"]),
                            ("0.8", ["1", "0.9"]),
                        ] {
                            let mut changed = cli();
                            changed.arg("integrate").arg(&artifact);
                            runtime_with(&mut changed, construction, "off", safety, caps, "17");
                            changed.arg("--checkpoint").arg(&checkpoint).arg("--resume");
                            if serial_integration {
                                changed.args(["--serial", "0.001"]);
                            }
                            let output = changed.output().unwrap();
                            assert!(!output.status.success());
                            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
                            assert!(
                                report["error"]["message"]
                                    .as_str()
                                    .unwrap()
                                    .contains("settings differ"),
                                "{report}"
                            );
                            assert_eq!(fs::read(&checkpoint).unwrap(), checkpoint_before);
                        }
                        let other = if construction == "polynomial" {
                            "sign_aware"
                        } else {
                            "polynomial"
                        };
                        let mut changed = cli();
                        changed.arg("integrate").arg(&artifact);
                        runtime(&mut changed, other, "off");
                        changed.arg("--checkpoint").arg(&checkpoint).arg("--resume");
                        if serial_integration {
                            changed.args(["--serial", "0.001"]);
                        }
                        let output = changed.output().unwrap();
                        assert!(!output.status.success());
                        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
                        assert!(
                            report["error"]["message"]
                                .as_str()
                                .unwrap()
                                .contains("recipe"),
                            "{report}"
                        );
                        assert_eq!(fs::read(&checkpoint).unwrap(), checkpoint_before);
                    }
                }
            }
            assert_eq!(
                fs::read(artifact.with_extension("fsd.json")).unwrap(),
                before
            );
        }
    }
}

#[test]
fn dynamic_run_forwards_requested_recipe_to_existing_worker_paths() {
    let directory = tempfile::tempdir().unwrap();
    for (index, (construction, mode, serial_generation, serial_integration)) in [
        ("polynomial", "symbolic", false, false),
        ("sign_aware", "numerical_dual", false, true),
        ("polynomial", "numerical_dual", true, false),
        ("sign_aware", "symbolic", true, true),
    ]
    .into_iter()
    .enumerate()
    {
        let input = directory.path().join(format!("run-{index}.toml"));
        card(&input, mode);
        if serial_generation {
            let mut content: toml::Value =
                toml::from_str(&fs::read_to_string(&input).unwrap()).unwrap();
            content["generation"]
                .as_table_mut()
                .unwrap()
                .insert("serial".into(), true.into());
            fs::write(&input, toml::to_string(&content).unwrap()).unwrap();
        }
        let mut run = cli();
        run.arg("run")
            .arg(&input)
            .args(["--geometry-workers", "2", "--output"])
            .arg(input.with_extension("fsd"));
        runtime(&mut run, construction, "pilot");
        if serial_integration {
            run.args(["--serial", "0.001"]);
        }
        analytic_bubble(&success(run.output().unwrap()), construction, "pilot");
        let manifest: Value =
            serde_json::from_slice(&fs::read(input.with_extension("fsd.json")).unwrap()).unwrap();
        assert_eq!(manifest["programs"]["default_recipe"], "undeformed-v1");
        assert_eq!(
            manifest["programs"]["catalogue"]["recipes"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        // The selected resident served Run without changing the saved default.
        let inspected = success(
            cli()
                .arg("inspect")
                .arg(input.with_extension("fsd"))
                .output()
                .unwrap(),
        );
        assert_eq!(inspected["selected_recipe"], "undeformed-v1");
    }
}
