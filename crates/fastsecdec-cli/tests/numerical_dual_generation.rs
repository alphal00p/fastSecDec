//! Saved observations and caller-owned formula preparation are independent of
//! the portable evaluator identity and worker count.
use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn success(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn formula_phase_is_saved_inspectable_and_worker_count_independent() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("input.toml");
    fs::write(
        &card,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["0","0"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1-eps"
[[direct.terms.factors]]
polynomial="1+2*x+3*y"
exponent="1"
role="polynomial"
[generation]
mode="numerical_dual"
order=1
"#,
    )
    .unwrap();
    let expected = json!({"completed":1,"total":1,"sectors":2,"reused":1});
    let mut artifacts = Vec::new();
    for workers in ["1", "4"] {
        let artifact = directory.path().join(format!("workers-{workers}.fsd"));
        let output = cli()
            .args(["--status-json", "--status-interval-ms", "0", "generate"])
            .arg(&card)
            .args(["--workers", workers, "--output"])
            .arg(&artifact)
            .output()
            .unwrap();
        let report = success(&output);
        assert_eq!(report["generation"]["formula_preparation"], expected);
        let timing = &report["generation_timings"];
        assert!(timing["formula_preparation_seconds"].as_f64().unwrap() > 0.0);
        assert!(timing["coefficient_expansion_seconds"].as_f64().unwrap() > 0.0);
        let phases = timing
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.as_str() != "total_seconds")
            .map(|(_, value)| value.as_f64().unwrap())
            .sum::<f64>();
        assert!(
            phases <= timing["total_seconds"].as_f64().unwrap(),
            "phase walls must not overlap"
        );
        let statuses = String::from_utf8_lossy(&output.stderr)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect::<Vec<_>>();
        let first_formula = statuses
            .iter()
            .position(|row| row["stage"] == "FormulaPreparation")
            .unwrap();
        let first_assembly = statuses
            .iter()
            .position(|row| row["stage"] == "CoefficientExpansion")
            .unwrap();
        assert!(first_formula < first_assembly);
        for row in statuses
            .iter()
            .filter(|row| row["stage"] == "FormulaPreparation")
        {
            assert_eq!(row["formula_preparation"]["total"], 1);
            assert_eq!(row["formula_preparation"]["sectors"], 2);
            assert!(row.get("coefficient_expansion").is_none());
            assert!(row["completed"].as_u64().unwrap() <= 1);
            if let Some(workload) = row.get("workload") {
                assert_eq!(workload["total"], 1);
            }
        }
        let saved: Value =
            serde_json::from_slice(&fs::read(artifact.with_extension("fsd.json")).unwrap())
                .unwrap();
        assert_eq!(saved["generation"], report["generation"]);
        assert_eq!(saved["generation_timings"], report["generation_timings"]);
        let inspected = success(&cli().arg("inspect").arg(&artifact).output().unwrap());
        assert_eq!(inspected["generation"], saved["generation"]);
        assert_eq!(inspected["generation_timings"], saved["generation_timings"]);
        artifacts.push((
            report["content_id"].clone(),
            fs::read(artifact.with_extension("fsd.dat")).unwrap(),
        ));
    }
    assert_eq!(artifacts[0], artifacts[1]);
}
