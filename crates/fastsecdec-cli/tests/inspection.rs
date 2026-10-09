#[path = "support/artifact.rs"]
mod artifact_data;
use std::{fs, process::Command};
fn cli() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    c.env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    c
}
fn success(output: std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn failure(output: std::process::Output) -> String {
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["error"]["message"].as_str().unwrap().to_owned()
}

#[test]
fn selected_deep_inspection_uses_only_its_record_and_keeps_global_ids() {
    use std::io::{Read, Seek, SeekFrom, Write};
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("local.toml");
    let artifact = directory.path().join("local.fsd");
    fs::write(
        &card,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["-1+eps","eps"]
[[direct.terms.factors]]
polynomial="x+y"
exponent="-1-eps"
[[direct.terms.factors]]
polynomial="1+2*x+3*y"
exponent="-1"
[generation]
order=0
[generation.evaluator]
backend="eager"
"#,
    )
    .unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(artifact.with_extension("fsd.json")).unwrap()).unwrap();
    let records = metadata["programs"]["catalogue"]["recipes"][0]["records"]
        .as_array()
        .unwrap();
    let sectors = records
        .iter()
        .filter(|r| r["sector"].as_u64().is_some())
        .collect::<Vec<_>>();
    assert!(sectors.len() >= 2);
    let id = sectors[1]["sector"].as_u64().unwrap();
    let selected = || {
        success(
            cli()
                .arg("inspect")
                .arg(&artifact)
                .args(["--deep", "--validate-artifact", "--sector", &id.to_string()])
                .output()
                .unwrap(),
        )
    };
    let before = selected();
    assert_eq!(before["loaded_sectors"], 1);
    assert_eq!(before["selected_sector"]["id"], id);
    assert_eq!(
        before["selected_sector"]["global_output_indices"],
        sectors[1]["output_indices"]
    );
    let mut data = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(artifact_data::data_path(&artifact))
        .unwrap();
    let offset = sectors[0]["offset"].as_u64().unwrap();
    data.seek(SeekFrom::Start(offset)).unwrap();
    let mut byte = [0];
    data.read_exact(&mut byte).unwrap();
    byte[0] ^= 1;
    data.seek(SeekFrom::Start(offset)).unwrap();
    data.write_all(&byte).unwrap();
    data.sync_all().unwrap();
    assert_eq!(
        selected(),
        before,
        "unselected payload must not be decoded or validated"
    );
    let error = failure(
        cli()
            .arg("inspect")
            .arg(&artifact)
            .args(["--deep", "--validate-artifact"])
            .output()
            .unwrap(),
    );
    assert!(!error.is_empty());
}
#[test]
fn default_and_selected_inspection_are_json_only_while_deep_and_expressions_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let base = dir.path().join("input.fsd");
    fs::write(&card,"[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1'\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&base)
            .output()
            .unwrap(),
    );
    let normal = success(cli().arg("inspect").arg(&base).output().unwrap());
    assert_eq!(normal["inspection_mode"], "metadata_only");
    assert_eq!(normal["binary_loaded"], false);
    assert_eq!(normal["binary_validated"], false);
    assert_eq!(normal["metadata_identity_validated"], false);
    let deep = success(
        cli()
            .arg("inspect")
            .arg(&base)
            .arg("--deep")
            .output()
            .unwrap(),
    );
    assert_eq!(deep["binary_loaded"], true);
    assert_eq!(deep["binary_validated"], false);
    assert_eq!(deep["metadata_identity_validated"], false);
    let checked = success(
        cli()
            .arg("inspect")
            .arg(&base)
            .args(["--deep", "--validate-artifact"])
            .output()
            .unwrap(),
    );
    assert_eq!(checked["binary_loaded"], true);
    assert_eq!(checked["binary_validated"], true);
    assert_eq!(checked["metadata_identity_validated"], true);
    assert_eq!(normal["content_id"], deep["content_id"]);
    assert_eq!(normal["orders"], deep["orders"]);
    assert_eq!(normal["evaluator_statistics"], deep["evaluator_statistics"]);
    let mut estimates = Vec::new();
    for validate in [false, true] {
        let checkpoint = dir.path().join(format!("integration-{validate}.json"));
        let mut command = cli();
        command
            .arg("integrate")
            .arg(&base)
            .args([
                "--method",
                "mc",
                "--points",
                "4",
                "--shifts",
                "2",
                "--workers",
                "1",
                "--max-rounds",
                "1",
            ])
            .arg("--checkpoint")
            .arg(&checkpoint);
        if validate {
            command.arg("--validate-artifact");
        }
        let result = success(command.output().unwrap());
        assert_eq!(result["estimate"]["production_complete"], true);
        estimates.push(result["estimate"].clone());
    }
    assert_eq!(estimates[0], estimates[1]);
    fs::remove_file(artifact_data::data_path(&base)).unwrap();
    let metadata_checked = success(
        cli()
            .arg("inspect")
            .arg(&base)
            .arg("--validate-artifact")
            .output()
            .unwrap(),
    );
    assert_eq!(metadata_checked["metadata_identity_validated"], true);
    assert_eq!(metadata_checked["binary_loaded"], false);
    assert_eq!(metadata_checked["binary_validated"], false);
    success(cli().arg("inspect").arg(&base).output().unwrap());
    let selected = success(
        cli()
            .arg("inspect")
            .arg(&base)
            .args(["--sector", "0"])
            .output()
            .unwrap(),
    );
    assert_eq!(selected["selected_sector"]["id"], 0);
    for flag in ["--deep", "--expressions"] {
        assert!(
            !cli()
                .arg("inspect")
                .arg(&base)
                .arg(flag)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(
        !cli()
            .arg("inspect")
            .arg(&base)
            .args(["--sector", "999"])
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn certification_is_explicit_and_validation_flags_do_not_change_inspection_scope() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let base = dir.path().join("input.fsd");
    fs::write(&card,"[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1'\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&base)
            .output()
            .unwrap(),
    );
    let metadata_path = base.with_extension("fsd.json");
    let original = fs::read_to_string(&metadata_path).unwrap();
    let metadata: serde_json::Value = serde_json::from_str(&original).unwrap();
    let changed = original.replacen(
        metadata["content_id"].as_str().unwrap(),
        "unchecked-recorded-identity",
        1,
    );
    fs::write(&metadata_path, changed).unwrap();
    let normal = success(cli().arg("inspect").arg(&base).output().unwrap());
    assert_eq!(normal["metadata_identity_validated"], false);
    let checked = cli()
        .arg("inspect")
        .arg(&base)
        .arg("--validate-artifact")
        .output()
        .unwrap();
    assert!(failure(checked).contains("content identity is invalid"));
    let checked = cli()
        .arg("integrate")
        .arg(&base)
        .arg("--validate-artifact")
        .output()
        .unwrap();
    assert!(failure(checked).contains("content identity is invalid"));
    let checked = cli()
        .arg("run")
        .arg(&card)
        .arg("--output")
        .arg(&base)
        .args(["--resume", "--validate-artifact"])
        .output()
        .unwrap();
    assert!(failure(checked).contains("content identity is invalid"));
    // This flag concerns saved artifacts and never triggers run-card preparation.
    let input = cli()
        .arg("inspect")
        .arg(dir.path().join("missing.toml"))
        .arg("--validate-artifact")
        .output()
        .unwrap();
    assert!(failure(input).contains("requires a generated artifact basename"));
}
