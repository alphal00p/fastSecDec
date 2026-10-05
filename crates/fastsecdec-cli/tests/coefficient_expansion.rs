use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const INPUT: &str = r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
monomial_powers = ["-1+eps"]
[[direct.terms.factors]]
polynomial = "1+x"
exponent = "-2"
[generation]
order = 1
"#;

fn generate(card: &Path, artifact: &Path, interval: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fastsecdec"))
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args([
            "--json",
            "--plain",
            "--status-json",
            "--status-interval-ms",
            interval,
            "generate",
        ])
        .arg(card)
        .arg("--output")
        .arg(artifact)
        .output()
        .unwrap()
}

fn statuses(output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|value| value.get("stage").is_some())
        .collect()
}

#[test]
fn explicit_named_card_reports_exclusive_timing_and_survives_status_coalescing() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("input.toml");
    fs::write(&card, INPUT).unwrap();
    let physical = generate(&card, &directory.path().join("physical.json"), "60000");
    assert!(
        physical.status.success(),
        "{}",
        String::from_utf8_lossy(&physical.stderr)
    );
    let physical_report: serde_json::Value = serde_json::from_slice(&physical.stdout).unwrap();
    assert!(
        statuses(&physical)
            .iter()
            .all(|row| row.get("coefficient_expansion").is_none())
    );
    assert_eq!(
        physical_report["generation_timings"]["coefficient_expansion_seconds"],
        0.0
    );

    fs::write(&card, format!("{INPUT}\n[generation.coefficient_expansion]\nmethod='native_named'\nmax_series_attempts=12\nmax_relative_width=128\nmax_unique_requests=10000\n")).unwrap();
    let mut counts = Vec::new();
    for interval in ["0", "60000"] {
        let output = generate(
            &card,
            &directory.path().join(format!("named-{interval}.json")),
            interval,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["orders"], physical_report["orders"]);
        let timing = &report["generation_timings"];
        assert!(timing["coefficient_expansion_seconds"].as_f64().unwrap() > 0.0);
        assert_eq!(timing["subtraction_seconds"], 0.0);
        assert_eq!(timing["laurent_seconds"], 0.0);
        let rows = statuses(&output);
        assert_eq!(rows.first().unwrap()["stage"], "Input");
        assert_eq!(rows.last().unwrap()["stage"], "Complete");
        let coefficients = rows
            .iter()
            .filter(|row| row["stage"] == "CoefficientExpansion")
            .collect::<Vec<_>>();
        assert!(!coefficients.is_empty());
        for row in &coefficients {
            let progress = &row["coefficient_expansion"];
            assert_eq!(progress["requested_method"], "native_named");
            assert_eq!(progress["effective_method"], "native_named");
            let completed =
                progress["sector"].as_u64().unwrap() + u64::from(progress["stage"] == "complete");
            assert_eq!(row["completed"], completed);
        }
        assert!(
            coefficients
                .iter()
                .any(|row| row["coefficient_expansion"]["stage"] == "complete")
        );
        counts.push(coefficients.len());
    }
    assert!(counts[1] < counts[0]);
}

#[test]
fn named_resource_error_remains_visible_without_completion_or_saved_artifact() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("input.toml");
    let artifact = directory.path().join("failed.json");
    fs::write(&card, format!("{INPUT}\n[generation.coefficient_expansion]\nmethod='native_named'\nmax_series_attempts=0\n")).unwrap();
    let output = generate(&card, &artifact, "60000");
    assert!(!output.status.success());
    // --json preserves the CLI's final error document on stdout; stderr is the
    // independently coalesced status stream.
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid final error JSON: {error}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("coefficient-first series attempts")),
        "unexpected final error: {error}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !statuses(&output)
            .iter()
            .any(|row| row["stage"] == "Complete")
    );
    assert!(!artifact.exists());
}
