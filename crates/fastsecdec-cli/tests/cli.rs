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
fn portable_generation_integration_resume_and_json_errors() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    let checkpoint = dir.path().join("checkpoint.json");
    card(&input);
    let generated = cli()
        .current_dir(dir.path())
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
    let generated: serde_json::Value = serde_json::from_slice(&generated.stdout).unwrap();
    assert_eq!(generated["orders"], serde_json::json!([0]));
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
    fs::write(&input, format!("{original}\n# changed source\n")).unwrap();
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
    card(&input);
    let mut child = cli()
        .arg("--status-json")
        .arg("run")
        .arg(&input)
        .arg("--output")
        .arg(&artifact)
        .arg("--checkpoint")
        .arg(&checkpoint)
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
    assert_eq!(state["format_version"], 2);
    assert_eq!(state["round_index"], 0);
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
    assert_eq!(report["failures"], 0);
    assert!(
        report["probes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|probe| probe["precision_bits"].is_number())
    );
}
