use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use fastsecdec::{
    reference::{
        CoefficientKey, ReferenceCoefficient, ReferenceProvenance, ReferenceResult,
        ReferenceUncertainty, ReferenceValidation, encode_reference,
    },
    status::CoefficientComponent,
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--json", "--plain"]);
    command
}

fn json(output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn target(path: &Path, value: f64, exact: bool, identity: Option<String>) {
    let mut reference = ReferenceResult::new(
        vec![ReferenceCoefficient {
            key: CoefficientKey {
                order: 0,
                component: CoefficientComponent::Real,
            },
            value,
            uncertainty: if exact {
                ReferenceUncertainty::Exact
            } else {
                ReferenceUncertainty::Unknown
            },
        }],
        ReferenceProvenance::new(
            "analytic unit-cube integral",
            "direct density integral on [0,1]",
        ),
    );
    if exact {
        reference.validation = ReferenceValidation::Checked {
            evidence: "The antiderivative of x is x^2/2.".into(),
        };
    }
    reference.kernel_content_id = identity;
    fs::write(path, encode_reference(&reference).unwrap()).unwrap();
}

const CARD: &str = r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
monomial_powers = ["1"]
[integration]
points = 1024
shifts = 4
"#;

const REFERENCE: &str = r#"
[reference]
path = "reference.json"
normalization_evidence = "Both expressions use the direct unit-cube measure."
kinematics_evidence = "There are no external kinematic parameters."
independence_evidence = "The reference is an analytic antiderivative, independent of the samples."
"#;

#[test]
fn reference_configuration_and_overrides_preserve_artifact_and_checkpoint_identity() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    let second_artifact = dir.path().join("compared.json");
    let checkpoint = dir.path().join("checkpoint.json");
    fs::write(&card, CARD).unwrap();
    let baseline = json(
        cli()
            .arg("run")
            .arg(&card)
            .arg("--output")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .output()
            .unwrap(),
    );
    assert!(baseline.get("reference").is_none());
    target(&dir.path().join("reference.json"), 0.5, true, None);
    fs::write(&card, format!("{CARD}{REFERENCE}")).unwrap();
    let elsewhere = dir.path().join("elsewhere");
    fs::create_dir(&elsewhere).unwrap();
    let compared = json(
        cli()
            .current_dir(&elsewhere)
            .arg("run")
            .arg(&card)
            .arg("--output")
            .arg(&artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .args(["--resume", "--workers", "2"])
            .output()
            .unwrap(),
    );
    assert_eq!(baseline["content_id"], compared["content_id"]);
    assert_eq!(baseline["estimate"], compared["estimate"]);
    assert_eq!(baseline["snapshot"], compared["snapshot"]);
    assert_eq!(compared["reference"]["status"], "compared");
    assert_eq!(
        compared["reference"]["comparison"]["eligibility"]["eligible"],
        true
    );
    assert_eq!(
        compared["reference"]["comparison"]["rows"][0]["reference"]["uncertainty"]["kind"],
        "exact"
    );
    assert_eq!(
        compared["reference"]["comparison"]["rows"][0]["pull"]["status"],
        "value"
    );
    let generated = json(
        cli()
            .arg("generate")
            .arg(&card)
            .arg("--output")
            .arg(&second_artifact)
            .output()
            .unwrap(),
    );
    assert_eq!(baseline["content_id"], generated["content_id"]);
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(&second_artifact).unwrap()).unwrap();
    assert!(Path::new(stored["reference"]["path"].as_str().unwrap()).is_absolute());
    let restored_default = json(
        cli()
            .arg("integrate")
            .arg(&second_artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .arg("--resume")
            .arg("--reference")
            .arg(dir.path().join("reference.json"))
            .output()
            .unwrap(),
    );
    assert_eq!(
        compared["reference"]["comparison"],
        restored_default["reference"]["comparison"]
    );
    target(&elsewhere.join("other.json"), 0.75, false, None);
    let overridden = json(
        cli()
            .current_dir(&elsewhere)
            .arg("integrate")
            .arg(&second_artifact)
            .arg("--checkpoint")
            .arg(&checkpoint)
            .args(["--resume", "--reference", "other.json"])
            .output()
            .unwrap(),
    );
    assert_eq!(baseline["estimate"], overridden["estimate"]);
    assert_eq!(baseline["snapshot"], overridden["snapshot"]);
    for field in ["normalization", "kinematics", "independence"] {
        assert_eq!(
            overridden["reference"]["comparison"]["context"][field]["status"],
            "unknown"
        );
    }
    assert_eq!(
        overridden["reference"]["comparison"]["rows"][0]["reference"]["value"],
        0.75
    );
    assert_eq!(
        overridden["reference"]["comparison"]["rows"][0]["pull"]["status"],
        "unavailable"
    );
    assert_eq!(
        overridden["reference"]["comparison"]["eligibility"]["eligible"],
        false
    );
    target(
        &elsewhere.join("wrong.json"),
        0.5,
        true,
        Some("wrong kernel".into()),
    );
    let wrong = cli()
        .current_dir(&elsewhere)
        .arg("integrate")
        .arg(&artifact)
        .args(["--reference", "wrong.json"])
        .output()
        .unwrap();
    assert!(!wrong.status.success());
    let wrong: serde_json::Value = serde_json::from_slice(&wrong.stdout).unwrap();
    assert!(
        wrong["error"]["message"]
            .as_str()
            .unwrap()
            .contains("differs from estimate identity")
    );
}

#[test]
fn invalid_reference_is_reported_before_graph_loading_and_historical_errors_stay_unknown() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    let artifact = dir.path().join("integral.json");
    fs::write(
        &card,
        "[input]\ngraph='missing.dot'\nmodel='missing.json'\n[reference]\npath='reference.json'\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("reference.json"),
        r#"{"format":"fastsecdec-reference","version":99,"reference":{}}"#,
    )
    .unwrap();
    let failed = cli()
        .arg("run")
        .arg(&card)
        .arg("--output")
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(!failed.status.success());
    let failed: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert!(
        failed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("unsupported native reference format/version")
    );
    assert!(!artifact.exists());
    fs::write(
        &card,
        format!("{CARD}\n[reference]\npath='reference.json'\n"),
    )
    .unwrap();
    fs::write(dir.path().join("reference.json"), r#"{
        "schema_version":1,"provenance":{"repository":"historical test","convention":"direct unit cube"},
        "orders":[0],"coefficients":[{"re":0.5,"im":0.0}],"standard_errors":null,
        "validation_status":"unverified rounded value"
    }"#).unwrap();
    let historical = json(
        cli()
            .arg("run")
            .arg(&card)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let comparison = &historical["reference"]["comparison"];
    assert_eq!(comparison["validation"]["status"], "unverified");
    assert_eq!(comparison["context"]["independence"]["status"], "unknown");
    assert_eq!(comparison["rows"].as_array().unwrap().len(), 2);
    assert_eq!(
        comparison["rows"][0]["reference"]["uncertainty"]["kind"],
        "unknown"
    );
    assert!(comparison["rows"][1]["estimate"].is_null());
    assert_eq!(comparison["rows"][1]["pull"]["value"], "MissingEstimate");
    assert_eq!(comparison["eligibility"]["eligible"], false);
    let plain = Command::new(env!("CARGO_BIN_EXE_fastsecdec"))
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .arg("--plain")
        .arg("integrate")
        .arg(&artifact)
        .arg("--resume")
        .output()
        .unwrap();
    assert!(plain.status.success());
    let text = String::from_utf8(plain.stdout).unwrap();
    assert!(text.contains("uncertainty unknown"));
    assert!(text.contains("estimate missing"));
}
