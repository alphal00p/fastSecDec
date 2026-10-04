use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--json", "--plain"]);
    command
}
fn success(output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn failure(output: Output) -> String {
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["error"]["message"].as_str().unwrap().into()
}
fn card(path: &Path) {
    // The two charts integrate to ln(2)/2 and (1-ln(2))/2, with no
    // symmetry merging. A separate zero-dimensional fixture tests exact offsets.
    fs::write(
        path,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["1","0"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1"
[integration]
points=1024
shifts=8
lattice="hkkn-alpha3"
absolute_tolerance=0.001
relative_tolerance=0.0
"#,
    )
    .unwrap();
}

#[test]
fn selected_execution_keeps_ids_scope_exact_offset_and_checkpoint_identity() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("artifact.json");
    card(&input);
    let generated = success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    assert_eq!(generated["sectors"], 2);
    let mut selected_values = Vec::new();
    for id in [0, 1] {
        let checkpoint = dir.path().join(format!("selected-{id}.checkpoint.json"));
        let saved = dir.path().join(format!("selected-{id}.result.json"));
        let output = cli()
            .arg("--status-json")
            .arg("integrate")
            .arg(&artifact)
            .args([
                "--sectors",
                &id.to_string(),
                "--exact-contributions",
                "exclude",
                "--checkpoint",
            ])
            .arg(&checkpoint)
            .arg("--save-result")
            .arg(&saved)
            .output()
            .unwrap();
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            if let Ok(status) = serde_json::from_str::<serde_json::Value>(line) {
                assert_eq!(
                    status["scope"]["SelectedSectors"]["sector_ids"],
                    serde_json::json!([id])
                );
            }
        }
        let report = success(output);
        assert_eq!(report["snapshot"]["completed_points"], 8192);
        assert_eq!(report["snapshot"]["sectors"].as_array().unwrap().len(), 1);
        assert_eq!(report["snapshot"]["sectors"][0]["id"], id);
        assert_eq!(report["converged"], false);
        assert_eq!(report["scoped_target_reached"], true);
        selected_values.push(report["estimate"]["mean"][0].as_f64().unwrap());
        let document = fastsecdec::results::read_result(&fs::read(&saved).unwrap()).unwrap();
        assert_eq!(document.manifest.sectors.len(), 2);
        assert!(matches!(
            document.scope,
            fastsecdec::results::ResultScope::SelectedSectors { .. }
        ));
        assert!(
            document
                .reference(fastsecdec::results::ResultReferenceSelection::Estimate)
                .is_err()
        );
        let checkpoint_json: serde_json::Value =
            serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
        assert_eq!(
            checkpoint_json["replay"]["states"][1 - id]["verified"],
            false
        );
        let resumed = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args([
                    "--sectors",
                    &id.to_string(),
                    "--exact-contributions",
                    "exclude",
                    "--checkpoint",
                ])
                .arg(&checkpoint)
                .args(["--resume", "--workers", "2"])
                .output()
                .unwrap(),
        );
        assert_eq!(resumed["estimate"], report["estimate"]);
        let mismatch = failure(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args([
                    "--sectors",
                    &id.to_string(),
                    "--exact-contributions",
                    "include",
                    "--checkpoint",
                ])
                .arg(&checkpoint)
                .arg("--resume")
                .output()
                .unwrap(),
        );
        assert!(mismatch.contains("settings differ"));
        let included = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args([
                    "--sectors",
                    &id.to_string(),
                    "--exact-contributions",
                    "include",
                    "--checkpoint",
                ])
                .arg(dir.path().join(format!("included-{id}.json")))
                .output()
                .unwrap(),
        );
        assert!(
            (included["estimate"]["mean"][0].as_f64().unwrap() - selected_values[id]).abs() < 1e-13,
            "included={included}; excluded={report}"
        );
    }
    for method in ["mc", "adaptive_mc"] {
        let checkpoint = dir.path().join(format!("selected-{method}.json"));
        let report = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args([
                    "--sectors",
                    "1",
                    "--exact-contributions",
                    "exclude",
                    "--method",
                    method,
                    "--points",
                    "4096",
                    "--checkpoint",
                ])
                .arg(&checkpoint)
                .output()
                .unwrap(),
        );
        assert_eq!(report["snapshot"]["completed_points"], 32768);
        assert_eq!(report["snapshot"]["sectors"].as_array().unwrap().len(), 1);
        assert_eq!(report["snapshot"]["sectors"][0]["id"], 1);
        let mean = report["estimate"]["mean"][0].as_f64().unwrap();
        let error = report["estimate"]["standard_error"][0].as_f64().unwrap();
        assert!((mean - selected_values[1]).abs() < 5.0 * error + 1e-4);
        let state: serde_json::Value =
            serde_json::from_slice(&fs::read(checkpoint).unwrap()).unwrap();
        assert_eq!(state["replay"]["states"][0]["verified"], false);
        assert_eq!(state["replay"]["states"][1]["verified"], true);
    }
    selected_values.sort_by(f64::total_cmp);
    assert!((selected_values[0] - (1.0 - 2.0_f64.ln()) / 2.0).abs() < 1e-4);
    assert!((selected_values[1] - 2.0_f64.ln() / 2.0).abs() < 1e-4);

    let selected_all = dir.path().join("selected-all.json");
    success(
        cli()
            .arg("integrate")
            .arg(&artifact)
            .args([
                "--sectors",
                "1,0",
                "--exact-contributions",
                "include",
                "--checkpoint",
            ])
            .arg(&selected_all)
            .output()
            .unwrap(),
    );
    assert!(
        failure(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .arg("--checkpoint")
                .arg(&selected_all)
                .arg("--resume")
                .output()
                .unwrap()
        )
        .contains("settings differ")
    );
    let reordered = success(
        cli()
            .arg("integrate")
            .arg(&artifact)
            .args([
                "--sectors",
                "0,1",
                "--exact-contributions",
                "include",
                "--checkpoint",
            ])
            .arg(&selected_all)
            .arg("--resume")
            .output()
            .unwrap(),
    );
    assert_eq!(
        reordered["scope"]["SelectedSectors"]["sector_ids"],
        serde_json::json!([0, 1])
    );
    assert_eq!(reordered["converged"], false);
    for ids in ["0,0", "9"] {
        assert!(
            failure(
                cli()
                    .arg("integrate")
                    .arg(&artifact)
                    .args(["--sectors", ids, "--exact-contributions", "include"])
                    .output()
                    .unwrap()
            )
            .contains("unique subset")
        );
    }
    let exact_card = dir.path().join("exact.toml");
    let exact_artifact = dir.path().join("exact.json");
    fs::write(
        &exact_card,
        "[direct]\ndomain='unit_cube'\nparameters=[]\n[[direct.terms]]\nprefactor='2'\n",
    )
    .unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&exact_card)
            .arg("--output")
            .arg(&exact_artifact)
            .output()
            .unwrap(),
    );
    for (policy, expected) in [("include", 2.0), ("exclude", 0.0)] {
        for method in ["qmc", "adaptive_qmc", "mc", "adaptive_mc"] {
            let report = success(
                cli()
                    .arg("integrate")
                    .arg(&exact_artifact)
                    .args([
                        "--sectors",
                        "none",
                        "--exact-contributions",
                        policy,
                        "--method",
                        method,
                        "--checkpoint",
                    ])
                    .arg(dir.path().join(format!("empty-{policy}-{method}.json")))
                    .output()
                    .unwrap(),
            );
            assert_eq!(report["estimate"]["mean"][0], expected);
            assert_eq!(report["snapshot"]["completed_points"], 0);
            assert_eq!(report["converged"], false);
            assert_eq!(report["scoped_target_reached"], true);
        }
    }
    let full_checkpoint = dir.path().join("full.json");
    let full = success(
        cli()
            .arg("integrate")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&full_checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(full["scope"], "FullIntegral");
    assert_eq!(full["converged"], true);
    let checkpoint: serde_json::Value =
        serde_json::from_slice(&fs::read(&full_checkpoint).unwrap()).unwrap();
    assert!(checkpoint["settings"].get("scope").is_none());
    // A scoped run card remains steerable without changing its input bytes.
    use std::io::Write;
    writeln!(
        fs::OpenOptions::new().append(true).open(&input).unwrap(),
        "\n[integration.scope.SelectedSectors]\nsector_ids=[1]\nexact_policy='ExcludeAll'"
    )
    .unwrap();
    let scoped_artifact = dir.path().join("scoped-card.json");
    let card_scope = success(
        cli()
            .arg("run")
            .arg(&input)
            .arg("--output")
            .arg(&scoped_artifact)
            .output()
            .unwrap(),
    );
    assert_eq!(
        card_scope["scope"]["SelectedSectors"]["sector_ids"],
        serde_json::json!([1])
    );
    let full_override_checkpoint = dir.path().join("full-override.json");
    let cleared = success(
        cli()
            .arg("integrate")
            .arg(&scoped_artifact)
            .args(["--full-integral", "--checkpoint"])
            .arg(&full_override_checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(cleared["scope"], "FullIntegral");
    assert!((cleared["estimate"]["mean"][0].as_f64().unwrap() - 0.5).abs() < 1e-4);
    let resumed = success(
        cli()
            .arg("integrate")
            .arg(&scoped_artifact)
            .args([
                "--full-integral",
                "--resume",
                "--workers",
                "2",
                "--checkpoint",
            ])
            .arg(&full_override_checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(resumed["estimate"], cleared["estimate"]);
    assert!(
        failure(
            cli()
                .arg("integrate")
                .arg(&scoped_artifact)
                .args(["--resume", "--checkpoint"])
                .arg(&full_override_checkpoint)
                .output()
                .unwrap()
        )
        .contains("settings differ")
    );
}

#[test]
fn inspect_uses_retained_native_metadata_and_missing_graph_has_structured_error() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("symmetric.toml");
    let artifact = dir.path().join("symmetric.json");
    fs::write(&input, "[direct]\ndomain='unit_cube'\nparameters=['x','y']\n[[direct.terms]]\nmonomial_powers=['0','0']\n[[direct.terms.factors]]\npolynomial='x+y'\nexponent='-1'\n").unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let inspect = success(cli().arg("inspect").arg(&artifact).output().unwrap());
    let stored: serde_json::Value = serde_json::from_slice(&fs::read(&artifact).unwrap()).unwrap();
    assert_eq!(
        inspect["generation_metadata"],
        stored["kernel"]["payload"]["metadata"]
    );
    assert_eq!(
        inspect["generation_metadata"]["charts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(inspect["sectors"], 1);
    assert_eq!(
        inspect["generation_metadata"]["domain"]["branch"],
        "NoThresholdReal"
    );
    for chart in inspect["generation_metadata"]["charts"].as_array().unwrap() {
        assert_eq!(chart["kernel_sector"], 0);
        assert!(
            !chart["geometry"]["factor_valuations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(!chart["measure_jacobian"].as_str().unwrap().is_empty());
    }
    let plain = Command::new(env!("CARGO_BIN_EXE_fastsecdec"))
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "inspect"])
        .arg(&artifact)
        .arg("--expressions")
        .output()
        .unwrap();
    assert!(plain.status.success());
    let text = String::from_utf8(plain.stdout).unwrap();
    assert!(text.contains("Positive real measure") && text.contains("Representative permutation"));
    let missing = dir.path().join("missing.toml");
    let model = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/models/scalar.json")
        .canonicalize()
        .unwrap();
    fs::write(
        &missing,
        format!(
            "[input]\ngraph='absent-native.dot'\nmodel={:?}\n",
            model.to_str().unwrap()
        ),
    )
    .unwrap();
    let message = failure(cli().arg("inspect").arg(&missing).output().unwrap());
    assert!(!message.is_empty());
}
