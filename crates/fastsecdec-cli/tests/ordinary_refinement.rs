//! Ordinary drivers must steer the same native append APIs as library callers.
use serde_json::Value;
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
fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn ordinary_fixed_batches_and_point_growth_keep_distinct_designs_and_resume() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.toml");
    fs::write(
        &input,
        r#"
[direct]
domain="unit_cube"
parameters=["x"]
[[direct.terms]]
monomial_powers=["0"]
[[direct.terms.factors]]
polynomial="1+x"
exponent="-1"
[generation]
order=0
[generation.evaluator]
backend="eager"
[integration]
production_seconds=0.1
[integration.discrete_mc]
pilot_points=32
pilot_batches=2
pilot_iterations=1
bins=4
"#,
    )
    .unwrap();
    let artifact = directory.path().join("integral.fsd");
    success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    for method in ["qmc", "adaptive_qmc", "mc", "adaptive_mc", "discrete_mc"] {
        for double_points in [false, true] {
            let checkpoint = directory
                .path()
                .join(format!("{method}-{double_points}.json"));
            let args = [
                "--method",
                method,
                "--points",
                "1024",
                "--shifts",
                "2",
                "--seed",
                "871",
                "--max-rounds",
                "2",
                "--relative-tolerance",
                "0",
                "--absolute-tolerance",
                "0",
                "--double-points",
                if double_points { "true" } else { "false" },
            ];
            let result = success(
                cli()
                    .arg("integrate")
                    .arg(&artifact)
                    .args(args)
                    .args(["--workers", "2"])
                    .arg("--checkpoint")
                    .arg(&checkpoint)
                    .output()
                    .unwrap(),
            );
            assert!(!result["converged"].as_bool().unwrap());
            let saved: Value = serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
            assert_eq!(saved["round_index"], 1);
            let settings = if method == "discrete_mc" {
                &saved["session"]["settings"]["batch"]
            } else {
                &saved["session"]["settings"]
            };
            let qmc = matches!(method, "qmc" | "adaptive_qmc");
            let point_key = if qmc { "points" } else { "points_per_batch" };
            let batch_key = if qmc { "shifts" } else { "batches" };
            assert_eq!(settings[point_key], if double_points { 2048 } else { 1024 });
            assert_eq!(settings[batch_key], if double_points { 2 } else { 4 });
            let resumed = success(
                cli()
                    .arg("integrate")
                    .arg(&artifact)
                    .args(args)
                    .args(["--workers", "1", "--resume"])
                    .arg("--checkpoint")
                    .arg(&checkpoint)
                    .output()
                    .unwrap(),
            );
            assert_eq!(result["estimate"], resumed["estimate"]);
            assert_eq!(
                result["snapshot"]["completed_points"],
                resumed["snapshot"]["completed_points"]
            );
        }
    }
}
