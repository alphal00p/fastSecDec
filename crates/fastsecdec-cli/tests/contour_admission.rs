//! Unsupported mathematical requests fail before generation or publication.
use std::{fs, process::Command};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

#[test]
fn invalid_contour_settings_fail_before_generation_or_publication() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("missing-graph.toml");
    // Native graph inputs deliberately do not exist: setting admission must
    // happen before any model read, algebra, worker launch or publication.
    fs::write(
        &input,
        "[input]\ngraph='missing.dot'\nmodel='missing.json'\n[generation]\ncontour=true\n",
    )
    .unwrap();
    let output = directory.path().join("rejected.fsd");
    for (arguments, expected) in [
        (vec!["--contour", "dynamical=1"], "safety fraction"),
        (
            vec!["--contour", "dynamical=0.8", "--lambda-cap", "0"],
            "caps must be finite and positive",
        ),
        (
            vec!["--contour", "off", "--contour-construction", "polynomial"],
            "require a dynamical contour prescription",
        ),
        (vec!["--contour", "fixed"], "requires --lambda"),
    ] {
        for serial in [false, true] {
            let mut command = cli();
            command
                .arg("run")
                .arg(&input)
                .arg("--output")
                .arg(&output)
                .args(&arguments);
            if serial {
                command.args(["--serial", "0.001"]);
            }
            let result = command.output().unwrap();
            assert!(!result.status.success());
            let result: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            let error = result["error"]["message"].as_str().unwrap();
            assert!(error.contains(expected), "{error}");
            assert!(!output.with_extension("fsd.json").exists());
            assert!(!directory.path().join("rejected.fsd.generation").exists());
        }
    }
}
