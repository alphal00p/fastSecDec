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

fn card(path: &Path, integration: &str, dimension: usize) {
    let parameters = (0..dimension)
        .map(|index| format!("\"x{index}\""))
        .collect::<Vec<_>>()
        .join(",");
    let powers = (0..dimension)
        .map(|_| "\"1\"")
        .collect::<Vec<_>>()
        .join(",");
    fs::write(path,format!("[direct]\ndomain=\"unit_cube\"\nparameters=[{parameters}]\n[[direct.terms]]\nmonomial_powers=[{powers}]\n[integration]\n{integration}\n")).unwrap();
}

#[test]
fn explicit_catalogues_integrate_one_artifact_and_checkpoint_distinct_sampling() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    card(&input, "points=1024\nshifts=4", 1);
    let generated = success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let original = fs::read(&artifact).unwrap();
    for (name, native) in [
        ("kuo33002", "Kuo33002"),
        ("kuo38005", "Kuo38005"),
        ("kuo39101", "Kuo39101"),
        ("hkkn-alpha3", "HkknAlpha3"),
    ] {
        let checkpoint = dir.path().join(format!("{name}.json"));
        let result = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args(["--lattice", name, "--checkpoint"])
                .arg(&checkpoint)
                .output()
                .unwrap(),
        );
        assert_eq!(result["content_id"], generated["content_id"]);
        assert!((result["estimate"]["mean"][0].as_f64().unwrap() - 0.5).abs() < 1e-5);
        assert_eq!(result["snapshot"]["completed_points"], 4096);
        assert_eq!(result["qmc_design"]["allocations"][0]["points"], 1024);
        assert_eq!(result["qmc_design"]["allocations"][0]["shifts"], 4);
        let saved: serde_json::Value =
            serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
        if name == "kuo33002" {
            assert!(saved["settings"].get("lattice").is_none());
            assert_eq!(saved["session"]["settings"]["rule"], "Kuo");
            assert_eq!(result["qmc_design"]["settings"]["rule"], "Kuo");
        } else {
            assert_eq!(saved["settings"]["lattice"], name);
            assert_eq!(saved["session"]["settings"]["rule"]["Published"], native);
            assert_eq!(
                result["qmc_design"]["settings"]["rule"]["Published"],
                native
            );
        }
        let resumed = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .args(["--lattice", name, "--checkpoint"])
                .arg(&checkpoint)
                .args(["--resume", "--workers", "2"])
                .output()
                .unwrap(),
        );
        assert_eq!(resumed["estimate"], result["estimate"]);
        assert_eq!(resumed["qmc_design"], result["qmc_design"]);
        assert_eq!(
            resumed["snapshot"]["completed_points"],
            result["snapshot"]["completed_points"]
        );
    }
    assert_eq!(
        fs::read(&artifact).unwrap(),
        original,
        "integration overrides must not regenerate the symbolic artifact"
    );
    let incompatible = cli()
        .arg("integrate")
        .arg(&artifact)
        .args(["--lattice", "kuo39101", "--checkpoint"])
        .arg(dir.path().join("hkkn-alpha3.json"))
        .arg("--resume")
        .output()
        .unwrap();
    assert!(!incompatible.status.success());
    let error: serde_json::Value = serde_json::from_slice(&incompatible.stdout).unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("settings differ")
    );
}

#[test]
fn native_catalogue_bounds_apply_to_toml_without_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    let checkpoint = dir.path().join("checkpoint.json");
    card(&input, "points=2\nshifts=4\nlattice=\"hkkn-alpha3\"", 1);
    let result = success(
        cli()
            .arg("run")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .output()
            .unwrap(),
    );
    assert_eq!(result["snapshot"]["completed_points"], 8);
    let rejected = cli()
        .arg("integrate")
        .arg(&artifact)
        .args(["--lattice", "kuo33002", "--checkpoint"])
        .arg(dir.path().join("rejected.json"))
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    let error: serde_json::Value = serde_json::from_slice(&rejected.stdout).unwrap();
    assert!(error["error"]["message"].as_str().unwrap().contains("1024"));

    let wide = dir.path().join("wide.toml");
    card(&wide, "points=1024\nshifts=4\nlattice=\"hkkn-alpha3\"", 11);
    let rejected = cli()
        .arg("run")
        .arg(&wide)
        .arg("--output")
        .arg(dir.path().join("wide.json"))
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    let error: serde_json::Value = serde_json::from_slice(&rejected.stdout).unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("dimension")
    );
}
