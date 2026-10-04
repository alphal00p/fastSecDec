use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    // Exercise the program's safe banner bootstrap rather than relying on the
    // environment of the test runner.
    command.env_remove("SYMBOLICA_HIDE_BANNER");
    command.arg("--json").arg("--plain");
    command
}

fn card(path: &Path) {
    fs::write(
        path,
        r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
monomial_powers = ["1"]
[integration]
points = 1024
shifts = 4
"#,
    )
    .unwrap();
}

#[test]
fn every_shipped_run_card_loads_through_the_native_cli() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs");
    let mut cards = fs::read_dir(&examples)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect::<Vec<_>>();
    cards.sort();
    assert_eq!(cards.len(), 24, "update the explicit run-card inventory");
    let graphs = fs::read_dir(examples.parent().unwrap().join("graphs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "dot"))
        .count();
    assert_eq!(graphs, 17, "update the explicit native DOT inventory");
    for card in cards {
        let inspected = cli().arg("inspect").arg(&card).output().unwrap();
        assert!(
            inspected.status.success(),
            "{} failed: {} {}",
            card.display(),
            String::from_utf8_lossy(&inspected.stdout),
            String::from_utf8_lossy(&inspected.stderr)
        );
        let inspected: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
        assert!(
            inspected["name"].is_string(),
            "{} has no name",
            card.display()
        );
        assert!(
            inspected["terms"].is_number(),
            "{} has no term count",
            card.display()
        );
    }
}

#[test]
fn portable_generation_integration_resume_and_json_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    let checkpoint = dir.path().join("checkpoint.json");
    card(&input);
    let generated = cli()
        .current_dir(dir.path())
        .arg("--status-json")
        .arg("generate")
        .arg("input.toml")
        .arg("--output")
        .arg("integral.json")
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let compilation = String::from_utf8(generated.stderr)
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|snapshot| snapshot["stage"] == "Compilation")
        .collect::<Vec<_>>();
    assert_eq!(compilation.first().unwrap()["completed"], 0);
    assert_eq!(
        compilation.last().unwrap()["completed"],
        compilation.last().unwrap()["total"]
    );
    let generated: serde_json::Value = serde_json::from_slice(&generated.stdout).unwrap();
    assert_eq!(generated["orders"], serde_json::json!([0]));
    let timings = generated["generation_timings"].as_object().unwrap();
    let stage_seconds: f64 = timings
        .iter()
        .filter(|(name, _)| *name != "total_seconds")
        .map(|(_, value)| value.as_f64().unwrap())
        .sum();
    assert!(timings["compilation_seconds"].as_f64().unwrap() > 0.0);
    assert!(timings["total_seconds"].as_f64().unwrap() >= stage_seconds);
    let benchmarked = cli()
        .arg("benchmark")
        .arg(&artifact)
        .args(["--points", "16", "--repetitions", "2"])
        .output()
        .unwrap();
    assert!(
        benchmarked.status.success(),
        "{} {}",
        String::from_utf8_lossy(&benchmarked.stdout),
        String::from_utf8_lossy(&benchmarked.stderr)
    );
    let benchmarked: serde_json::Value = serde_json::from_slice(&benchmarked.stdout).unwrap();
    assert_eq!(benchmarked["stop"], "Complete");
    assert_eq!(
        benchmarked["sectors"][0]["measurements"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        benchmarked["sectors"][0]["measurements"]
            .as_array()
            .unwrap()
            .iter()
            .all(|sample| sample["evaluations"] == 16 && sample["complete"] == true)
    );
    assert!(benchmarked["loading_seconds"].as_f64().unwrap() > 0.0);
    assert_eq!(
        benchmarked["generation_timings"],
        generated["generation_timings"]
    );
    let run = cli()
        .arg("integrate")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let run: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert!((run["estimate"]["mean"][0].as_f64().unwrap() - 0.5).abs() < 0.002);
    assert!(run["loading_seconds"].as_f64().unwrap() > 0.0);
    assert_eq!(run["generation_timings"], generated["generation_timings"]);
    assert!(
        run["snapshot"]["evaluation_diagnostics"]["weighted_checks"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        run["snapshot"]["evaluation_diagnostics"]["additional_replays"]
            .as_u64()
            .unwrap()
            > 0
    );
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
    assert_eq!(saved["format_version"], 3);
    assert_eq!(saved["replay"]["states"][0]["verified"], true);
    // Observations are not mathematical identity and must not invalidate a
    // production checkpoint or the portable kernel's content certificate.
    let mut stored: serde_json::Value =
        serde_json::from_slice(&fs::read(&artifact).unwrap()).unwrap();
    stored["generation_timings"]["total_seconds"] = 999.0.into();
    fs::write(&artifact, serde_json::to_vec(&stored).unwrap()).unwrap();
    let resumed = cli()
        .arg("integrate")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
        .arg("--resume")
        .output()
        .unwrap();
    assert!(
        resumed.status.success(),
        "{}",
        String::from_utf8_lossy(&resumed.stdout)
    );
    let resumed: serde_json::Value = serde_json::from_slice(&resumed.stdout).unwrap();
    assert_eq!(resumed["generation_timings"]["total_seconds"], 999.0);
    assert_eq!(run["estimate"], resumed["estimate"]);
    assert_eq!(
        run["snapshot"]["completed_points"],
        resumed["snapshot"]["completed_points"]
    );
    assert_eq!(
        run["snapshot"]["evaluation_diagnostics"],
        resumed["snapshot"]["evaluation_diagnostics"]
    );
    let elsewhere = dir.path().join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    let cross_cwd = cli()
        .current_dir(&elsewhere)
        .arg("run")
        .arg(&input)
        .arg("--output")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
        .arg("--resume")
        .arg("--workers")
        .arg("2")
        .output()
        .unwrap();
    assert!(
        cross_cwd.status.success(),
        "{}",
        String::from_utf8_lossy(&cross_cwd.stdout)
    );
    let original = fs::read_to_string(&input).unwrap();
    fs::write(&input, original.replace("shifts = 4", "shifts = 8")).unwrap();
    let changed = cli()
        .arg("run")
        .arg(&input)
        .arg("--output")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
        .arg("--resume")
        .output()
        .unwrap();
    assert!(!changed.status.success());
    let changed: serde_json::Value = serde_json::from_slice(&changed.stdout).unwrap();
    assert!(
        changed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("changed since generation")
    );

    let failed = cli()
        .arg("inspect")
        .arg(dir.path().join("missing.toml"))
        .output()
        .unwrap();
    assert!(!failed.status.success());
    let failed: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert!(failed["error"]["message"].is_string());
}

#[cfg(unix)]
#[test]
fn plain_mode_sigint_saves_a_resumable_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    let checkpoint = dir.path().join("checkpoint.json");
    let saved_result = dir.path().join("result.json");
    card(&input);
    let mut child = cli()
        .arg("--status-json")
        .arg("run")
        .arg(&input)
        .arg("--output")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
        .arg("--save-result")
        .arg(&saved_result)
        .arg("--points")
        .arg("1048576")
        .arg("--shifts")
        .arg("64")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut status = BufReader::new(child.stderr.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert_ne!(
            status.read_line(&mut line).unwrap(),
            0,
            "process ended before integration: {line}"
        );
        if serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .is_some_and(|value| value.get("completed_points").is_some())
        {
            break;
        }
    }
    // `kill` is the shell builtin; the PID is an argument, never shell text.
    assert!(
        Command::new("bash")
            .args(["-c", "kill -INT \"$1\"", "fastsecdec-test"])
            .arg(child.id().to_string())
            .status()
            .unwrap()
            .success()
    );
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["stopping_reason"], "cancelled");
    assert_eq!(report["resume_status"], "checkpoint_saved");
    let state: serde_json::Value = serde_json::from_slice(&fs::read(checkpoint).unwrap()).unwrap();
    assert_eq!(state["format_version"], 3);
    assert_eq!(state["round_index"], 0);
    let saved = fastsecdec::results::read_result(&fs::read(saved_result).unwrap()).unwrap();
    assert_eq!(
        saved.stopping_reason,
        fastsecdec::status::StoppingReason::Cancelled
    );
    assert!(
        saved
            .reference(fastsecdec::results::ResultReferenceSelection::Estimate)
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(&saved.contributions).unwrap(),
        report["contributions"]
    );
}

#[test]
fn transcendental_laurent_artifact_loads_in_a_fresh_process() {
    let dir = tempfile::tempdir().unwrap();
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/runs/analytic_endpoint.toml");
    let artifact = dir.path().join("endpoint.json");
    let generated = cli()
        .arg("generate")
        .arg(input)
        .arg("--output")
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stdout)
    );
    let integrated = cli()
        .arg("integrate")
        .arg(&artifact)
        .arg("--points")
        .arg("1024")
        .arg("--shifts")
        .arg("4")
        .arg("--workers")
        .arg("2")
        .output()
        .unwrap();
    assert!(
        integrated.status.success(),
        "{}",
        String::from_utf8_lossy(&integrated.stdout)
    );
    let report: serde_json::Value = serde_json::from_slice(&integrated.stdout).unwrap();
    let means = report["estimate"]["mean"].as_array().unwrap();
    for (actual, expected) in means.iter().zip([1.0, -1.77036284546148, 3.19337345247]) {
        assert!((actual.as_f64().unwrap() - expected).abs() < 5e-6);
    }
    let boundaries = cli()
        .arg("check-boundaries")
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(
        boundaries.status.success(),
        "{}",
        String::from_utf8_lossy(&boundaries.stdout)
    );
    let report: serde_json::Value = serde_json::from_slice(&boundaries.stdout).unwrap();
    assert_eq!(report["diagnostics"]["failures"], 0);
    assert!(
        report["attempts"][0]["samples"]["probes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|probe| probe["precision_bits"].is_number())
    );
}
