use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli() -> Command {
    let mut cli = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    cli.env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    cli
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

fn card(path: &Path, factor: &str, maximum: usize) {
    fs::write(
        path,
        format!(
            r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["-1+eps","0"]
[[direct.terms.factors]]
polynomial="{factor}"
exponent="-1"
[generation]
order=0
max_sectors={maximum}
[integration]
points=1024
shifts=16
workers=1
max_rounds=1
"#
        ),
    )
    .unwrap();
}

#[test]
fn parallel_geometry_preserves_artifact_and_complete_analytic_vector() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    card(&input, "x+y", 1000);
    let serial_path = dir.path().join("serial.json");
    let parallel_path = dir.path().join("parallel.json");
    let serial = success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&serial_path)
            .output()
            .unwrap(),
    );
    let output = cli()
        .arg("--status-json")
        .arg("generate")
        .arg(&input)
        .args(["--geometry-workers", "2", "--output"])
        .arg(&parallel_path)
        .output()
        .unwrap();
    let snapshots = String::from_utf8(output.stderr.clone())
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect::<Vec<_>>();
    for label in ["chart jobs returned", "cone jobs returned"] {
        assert!(snapshots.iter().any(|s| {
            s["detail"]
                .as_str()
                .is_some_and(|d| d.contains(label) && d.contains("native admission pending"))
        }));
    }
    let parallel = success(output);
    assert_eq!(serial["content_id"], parallel["content_id"]);
    assert_eq!(serial["orders"], serde_json::json!([-2, -1, 0]));
    assert_eq!(parallel["geometry_workers"], 2);
    let mut serial_artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(&serial_path).unwrap()).unwrap();
    let mut parallel_artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(&parallel_path).unwrap()).unwrap();
    serial_artifact
        .as_object_mut()
        .unwrap()
        .remove("generation_timings");
    parallel_artifact
        .as_object_mut()
        .unwrap()
        .remove("generation_timings");
    assert_eq!(
        serial_artifact, parallel_artifact,
        "ordered kernels, metadata and identities"
    );

    let run_path = dir.path().join("run.json");
    let checkpoint = dir.path().join("checkpoint.json");
    let result = success(
        cli()
            .arg("run")
            .arg(&input)
            .args(["--geometry-workers", "2", "--output"])
            .arg(&run_path)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(result["content_id"], serial["content_id"]);
    assert_eq!(result["estimate"]["orders"], serde_json::json!([-2, -1, 0]));
    // Integrate y exactly: log(1+x)-log(x). Its Mellin expansion is
    // 1/eps^2 + pi^2/12 + O(eps), independently of sector decomposition.
    for (i, expected) in [1.0, 0.0, std::f64::consts::PI.powi(2) / 12.0]
        .into_iter()
        .enumerate()
    {
        let mean = result["estimate"]["mean"][i].as_f64().unwrap();
        assert!(
            (mean - expected).abs() < 1e-4,
            "order {}: {mean} vs {expected}",
            i as i32 - 2
        );
    }
    assert_eq!(
        result["snapshot"]["completed_points"],
        result["snapshot"]["planned_points"]
    );
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
    assert!(saved["settings"].get("workers").is_none());
    assert_eq!(serial_artifact["provenance"]["integration"]["workers"], 1);
    assert!(saved["settings"].get("geometry_workers").is_none());
    let resumed = success(
        cli()
            .arg("run")
            .arg(&input)
            .args(["--geometry-workers", "3", "--resume", "--output"])
            .arg(&run_path)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(resumed["estimate"], result["estimate"]);
}

#[test]
fn parallel_geometry_keeps_domain_and_limit_failures_without_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("rejected.json");
    for (factor, limit) in [("1-x", 1000), ("x+y", 0)] {
        card(&input, factor, limit);
        let serial = cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap();
        let parallel = cli()
            .arg("generate")
            .arg(&input)
            .args(["--geometry-workers", "2", "--output"])
            .arg(&artifact)
            .output()
            .unwrap();
        assert!(!serial.status.success() && !parallel.status.success());
        let serial: serde_json::Value = serde_json::from_slice(&serial.stdout).unwrap();
        let parallel: serde_json::Value = serde_json::from_slice(&parallel.stdout).unwrap();
        assert_eq!(serial["error"], parallel["error"]);
        assert!(!artifact.exists());
    }
    let invalid = cli()
        .arg("generate")
        .arg(&input)
        .args(["--geometry-workers", "0"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
}
