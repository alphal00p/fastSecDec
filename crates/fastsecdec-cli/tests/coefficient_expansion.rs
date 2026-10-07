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
    let physical = generate(&card, &directory.path().join("physical.fsd"), "60000");
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
    // Caller-dispatched generation measures the complete representative jobs
    // as one wall-time phase for either expression representation. It does not
    // sum nested worker subtraction/Laurent timers into overlapping totals.
    assert!(
        physical_report["generation_timings"]["coefficient_expansion_seconds"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert_eq!(
        physical_report["generation_timings"]["subtraction_seconds"],
        0.0
    );
    assert_eq!(
        physical_report["generation_timings"]["laurent_seconds"],
        0.0
    );
    assert_eq!(
        physical_report["generation"]["requested_coefficient_expansion"],
        "full_expression"
    );

    fs::write(&card, format!("{INPUT}\n[generation.coefficient_expansion]\nmethod='coefficient_series'\nmax_series_attempts=12\nmax_relative_width=128\nmax_unique_requests=10000\n")).unwrap();
    let mut counts = Vec::new();
    for interval in ["0", "60000"] {
        let output = generate(
            &card,
            &directory.path().join(format!("named-{interval}.fsd")),
            interval,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["orders"], physical_report["orders"]);
        assert_eq!(
            report["generation"]["requested_coefficient_expansion"],
            "coefficient_series"
        );
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
            // This is the master workload, not a fabricated single-sector
            // coefficient pass shared between concurrently running workers.
            assert!(row.get("coefficient_expansion").is_none());
            assert_eq!(row["completed"], row["workload"]["completed"]);
            assert_eq!(row["total"], row["workload"]["total"]);
            assert!(row["completed"].as_u64().unwrap() <= row["total"].as_u64().unwrap());
            assert!(!row["workload"]["workers"].as_array().unwrap().is_empty());
        }
        if interval == "0" {
            assert!(
                coefficients
                    .iter()
                    .any(|row| row["completed"] == row["total"])
            );
        }
        // Stage entry and the final result are forced. With a long observation
        // interval an intermediate workload-completion snapshot may coalesce.
        assert_eq!(
            rows.last().unwrap()["completed"],
            rows.last().unwrap()["total"]
        );
        counts.push(coefficients.len());
    }
    assert!(counts[1] < counts[0]);
}

#[test]
fn named_resource_error_remains_visible_without_completion_or_saved_artifact() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("input.toml");
    let artifact = directory.path().join("failed.fsd");
    fs::write(&card, format!("{INPUT}\n[generation.coefficient_expansion]\nmethod='coefficient_series'\nmax_series_attempts=0\n")).unwrap();
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
    assert!(!artifact.with_extension("fsd.json").exists());
    assert!(!artifact.with_extension("fsd.dat").exists());
}
