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
    assert_eq!(normal["binary_validated"], false);
    let deep = success(
        cli()
            .arg("inspect")
            .arg(&base)
            .arg("--deep")
            .output()
            .unwrap(),
    );
    assert_eq!(deep["binary_validated"], true);
    assert_eq!(normal["content_id"], deep["content_id"]);
    assert_eq!(normal["orders"], deep["orders"]);
    assert_eq!(normal["evaluator_statistics"], deep["evaluator_statistics"]);
    fs::remove_file(base.with_extension("fsd.dat")).unwrap();
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
