use std::{fs, path::Path, process::Command};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--plain", "--json"]);
    command
}

fn fixture(path: &Path) {
    fs::write(
        path.join("model.json"),
        include_str!("../../../examples/models/scalar.json"),
    )
    .unwrap();
    fs::write(path.join("parameters.json"), r#"{"mt":[1,0]}"#).unwrap();
    fs::write(
        path.join("graph.dot"),
        r#"digraph repeated {
        graph [num="5", overall_factor="2", projector="3"];
        a -> b [id=0, particle="phi", lmb_id=0];
        b -> a [id=1, particle="phi"];
    }"#,
    )
    .unwrap();
}

#[test]
fn modes_reach_both_family_paths_and_are_recorded_in_artifacts() {
    let directory = tempfile::tempdir().unwrap();
    fixture(directory.path());
    for (index, mode, prepared) in [
        (0, None, false),
        (1, Some("dots"), false),
        (2, Some("full"), true),
        (3, Some("none"), false),
    ] {
        let input = directory.path().join(format!("{index}.toml"));
        let output = directory.path().join(format!("{index}.fsd"));
        let mode_line = mode
            .map(|m| format!("contraction_mode='{m}'"))
            .unwrap_or_default();
        let preparation = if prepared {
            "[generation.family_preparation.SingleUnitTerm]\nmax_states=32"
        } else {
            ""
        };
        fs::write(
            &input,
            format!(
                r#"
[input]
graph="graph.dot"
model="model.json"
parameter_card="parameters.json"
[generation]
{mode_line}
{preparation}
"#
            ),
        )
        .unwrap();
        let result = cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .args(["--workers", "1"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(output.with_extension("fsd.json")).unwrap()).unwrap();
        assert_eq!(
            metadata["generation"]["contraction_mode"],
            mode.unwrap_or("minimal")
        );
    }
}

#[test]
fn invalid_or_inapplicable_modes_fail_before_generation() {
    let directory = tempfile::tempdir().unwrap();
    for (mode, expected) in [
        ("unknown", "unknown variant"),
        ("dots", "contraction_mode requires native graph input"),
    ] {
        let card = directory.path().join("run.toml");
        fs::write(
            &card,
            format!(
                r#"
[direct]
domain="unit_cube"
parameters=["x"]
[[direct.terms]]
monomial_powers=["0"]
[generation]
contraction_mode="{mode}"
"#
            ),
        )
        .unwrap();
        let output = directory.path().join("unused.fsd");
        let result = cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!result.status.success());
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert!(
            report["error"]["message"]
                .as_str()
                .unwrap()
                .contains(expected),
            "{report}"
        );
        assert!(!output.with_extension("fsd.json").exists());
        assert!(!output.with_extension("fsd.dat").exists());
    }
}
