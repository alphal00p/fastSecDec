use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use fastsecdec::parametric::{
    FamilyPreparationPolicy, FamilyPreparationReport, FamilyPreparationStatus,
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

fn fixture(directory: &Path) {
    fs::write(
        directory.join("model.json"),
        include_str!("../../../examples/models/scalar.json"),
    )
    .unwrap();
    fs::write(directory.join("parameters.json"), r#"{"mt":[1,0]}"#).unwrap();
    fs::write(
        directory.join("graph.dot"),
        r#"digraph repeated {
        graph [num="5", overall_factor="2", projector="3"];
        a -> b [id=0, particle="phi", lmb_id=0];
        b -> a [id=1, particle="phi"];
    }"#,
    )
    .unwrap();
}

fn card(path: &Path, policy: &str, named: bool) {
    let coefficient = if named {
        "[generation.coefficient_expansion]\nmethod='coefficient_series'"
    } else {
        ""
    };
    fs::write(
        path,
        format!(
            r#"
[input]
graph="graph.dot"
model="model.json"
parameter_card="parameters.json"
[integral]
dimension="4-2*eps"
measure_multiplier="7"
[generation]
order=1
{policy}
{coefficient}
[integration]
points=2048
shifts=8
seed=419
workers=1
max_rounds=1
[integration.parameters]
"model::mt"=1.0
"#
        ),
    )
    .unwrap();
}

const PREPARED: &str = "[generation.family_preparation.SingleUnitTerm]\nmax_states=32";

fn assert_complex_laurent_layout(value: &serde_json::Value) {
    assert_eq!(value["orders"], serde_json::json!([-1, -1, 0, 0, 1, 1]));
    assert_eq!(
        value["components"],
        serde_json::json!(["Real", "Imag", "Real", "Imag", "Real", "Imag"])
    );
}

#[test]
fn prepared_graph_cold_artifact_preserves_full_weighted_laurent_vector_and_sources() {
    let directory = tempfile::tempdir().unwrap();
    fixture(directory.path());
    let mut originals = Vec::new();
    let mut means = Vec::new();
    for (name, policy, named) in [("original", "", false), ("prepared", PREPARED, true)] {
        let input = directory.path().join(format!("{name}.toml"));
        let artifact = directory.path().join(format!("{name}.fsd"));
        card(&input, policy, named);
        let inspected = success(cli().arg("inspect").arg(&input).output().unwrap());
        assert_eq!(inspected["parameters"], 2);
        if named {
            assert_eq!(inspected["active_parameters"], 1);
        } else {
            assert!(inspected.get("family_preparation").is_none());
            assert!(inspected.get("active_parameters").is_none());
        }
        success(
            cli()
                .arg("generate")
                .arg(&input)
                .arg("--output")
                .arg(&artifact)
                .output()
                .unwrap(),
        );
        // Each following command is a fresh process loading the ordinary artifact.
        let cold = success(cli().arg("inspect").arg(&artifact).output().unwrap());
        assert_complex_laurent_layout(&cold);
        assert_eq!(cold["parameters_bound"], false);
        assert!(cold["exact_coefficients"].is_null());
        let provenance = &cold["provenance"];
        if named {
            let report: FamilyPreparationReport =
                serde_json::from_value(provenance["family_preparation"].clone()).unwrap();
            assert_eq!(
                report.policy,
                FamilyPreparationPolicy::SingleUnitTerm { max_states: 32 }
            );
            assert_eq!(report.status, FamilyPreparationStatus::Projected);
            assert_eq!(report.original_powers, [1, 1]);
            assert_eq!(report.active_original_indices, [1]);
            assert_eq!(report.active_powers, [2]);
            assert_eq!(
                inspected["family_preparation"],
                provenance["family_preparation"]
            );
            let charts = cold["generation_metadata"]["charts"].as_array().unwrap();
            assert!(!charts.is_empty());
            for chart in charts {
                assert_eq!(
                    chart["source_parameters"],
                    serde_json::json!(["fastsecdec::{}::x1"])
                );
                assert!(chart["target_parameters"].as_array().unwrap().is_empty());
            }
        } else {
            assert!(provenance.get("family_preparation").is_none());
        }
        let sources = provenance["sources"].as_array().unwrap();
        assert_eq!(sources.len(), 4);
        for source in sources.iter().skip(1) {
            let bytes = fs::read(directory.path().join(source["path"].as_str().unwrap())).unwrap();
            assert_eq!(source["blake3"], blake3::hash(&bytes).to_hex().as_str());
        }
        originals.push(sources[1..].to_vec());
        let run = success(
            cli()
                .arg("integrate")
                .arg(&artifact)
                .arg("--checkpoint")
                .arg(directory.path().join(format!("{name}.checkpoint.json")))
                .output()
                .unwrap(),
        );
        assert_complex_laurent_layout(&run["estimate"]);
        let estimate: fastsecdec::integration::VectorEstimate =
            serde_json::from_value(run["estimate"].clone()).unwrap();
        estimate.validate().unwrap();
        assert_eq!(estimate.covariance_of_mean.len(), 36);
        let mean = run["estimate"]["mean"].as_array().unwrap();
        assert_eq!(mean.len(), 6);
        // Independent normalized massive vacuum result: 5*2*3*7*Gamma(eps).
        // This checks both pole and finite/higher coefficients, not chart densities.
        let gamma = std::f64::consts::EULER_GAMMA;
        let expected = [
            210.0,
            0.0,
            -210.0 * gamma,
            0.0,
            210.0 * (gamma * gamma / 2.0 + std::f64::consts::PI.powi(2) / 12.0),
            0.0,
        ];
        for (value, expected) in mean.iter().zip(expected) {
            assert!(
                (value.as_f64().unwrap() - expected).abs() < 0.005,
                "{name}: {mean:?} vs {expected}"
            );
        }
        means.push(mean.clone());
        assert_eq!(
            run["snapshot"]["completed_points"],
            run["snapshot"]["planned_points"]
        );
    }
    assert_eq!(originals[0], originals[1]);
    for (original, prepared) in means[0].iter().zip(&means[1]) {
        assert!((original.as_f64().unwrap() - prepared.as_f64().unwrap()).abs() < 0.005);
    }
}

#[test]
fn original_and_native_fallback_remain_explicit_and_invalid_uses_save_nothing() {
    let directory = tempfile::tempdir().unwrap();
    fixture(directory.path());
    let input = directory.path().join("input.toml");
    let artifact = directory.path().join("integral.fsd");
    card(&input, "family_preparation='Original'", false);
    let original = success(cli().arg("inspect").arg(&input).output().unwrap());
    assert!(original.get("family_preparation").is_none());
    card(
        &input,
        "[generation.family_preparation.SingleUnitTerm]\nmax_states=1",
        false,
    );
    success(
        cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap(),
    );
    let fallback = success(cli().arg("inspect").arg(&artifact).output().unwrap());
    let report: FamilyPreparationReport =
        serde_json::from_value(fallback["provenance"]["family_preparation"].clone()).unwrap();
    assert_eq!(
        report.status,
        FamilyPreparationStatus::Original(
            fastsecdec::parametric::FamilyPreparationFallback::StateLimit
        )
    );
    assert_eq!(report.active_original_indices, [0, 1]);
    assert_eq!(report.active_powers, [1, 1]);
    assert_complex_laurent_layout(&fallback);
    // Preparation is part of complete artifact identity, not observational data.
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(artifact.with_extension("fsd.json")).unwrap()).unwrap();
    saved["provenance"]["family_preparation"]["active_powers"][0] = 9.into();
    fs::write(
        artifact.with_extension("fsd.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    assert!(
        !cli()
            .arg("inspect")
            .arg(&artifact)
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::remove_file(artifact.with_extension("fsd.json")).unwrap();
    fs::remove_file(artifact.with_extension("fsd.dat")).unwrap();
    for (source, error) in [
        (
            "[direct]\ndomain='unit_cube'\nparameters=['x']\nterms=[]\n[generation.family_preparation.SingleUnitTerm]\nmax_states=32",
            "requires native graph input",
        ),
        (
            "[input]\ngraph='graph.dot'\nmodel='model.json'\nparameter_card='parameters.json'\n[generation.family_preparation.SingleUnitTerm]\nmax_states=0",
            "state bound must be positive",
        ),
    ] {
        fs::write(&input, source).unwrap();
        let rejected = cli()
            .arg("generate")
            .arg(&input)
            .arg("--output")
            .arg(&artifact)
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let failure: serde_json::Value = serde_json::from_slice(&rejected.stdout).unwrap();
        assert!(
            failure["error"]["message"]
                .as_str()
                .unwrap()
                .contains(error),
            "{failure}"
        );
        assert!(!artifact.with_extension("fsd.json").exists());
        assert!(!artifact.with_extension("fsd.dat").exists());
    }
}
