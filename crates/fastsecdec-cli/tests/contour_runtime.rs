//! Actual worker reports are observational across every caller-owned sampler.
#[cfg(unix)]
#[path = "contour_runtime/cancellation.rs"]
mod cancellation;

use serde_json::Value;
use std::{
    fs,
    path::Path,
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
        "stdout:{}\nstderr:{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn generated(path: &Path) {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/threshold_bubble.toml");
    let mut card: toml::Value = toml::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    let integration = card["integration"].as_table_mut().unwrap();
    integration.remove("contour");
    integration.insert("absolute_tolerance".into(), 0.0.into());
    integration.insert("relative_tolerance".into(), 0.0.into());
    integration.insert("production_seconds".into(), 0.1.into());
    integration.insert(
        "discrete_mc".into(),
        toml::toml! { pilot_points = 64
            pilot_batches = 2
            pilot_iterations = 1
            bins = 8
        }
        .into(),
    );
    let input = path.with_extension("toml");
    fs::write(&input, toml::to_string(&card).unwrap()).unwrap();
    success(
        cli()
            .arg("generate")
            .arg(&input)
            .args([
                "--recipe",
                "dynamic-sign-aware-v1",
                "--workers",
                "1",
                "--output",
            ])
            .arg(path)
            .output()
            .unwrap(),
    );
}

fn integrate(
    path: &Path,
    checkpoint: &Path,
    method: &str,
    serial: bool,
    mode: &str,
    resume: bool,
) -> Value {
    integrate_shifts(path, checkpoint, method, serial, mode, resume, 4)
}

fn integrate_shifts(
    path: &Path,
    checkpoint: &Path,
    method: &str,
    serial: bool,
    mode: &str,
    resume: bool,
    shifts: u32,
) -> Value {
    let mut command = cli();
    command
        .arg("integrate")
        .arg(path)
        .args([
            "--method",
            method,
            "--contour",
            "dynamical=0.8",
            "--contour-validation",
            "pilot",
            "--contour-pilot-points",
            "2",
            "--contour-diagnostics",
            mode,
            "--points",
            "128",
            "--lattice",
            "hkkn-alpha3",
            "--shifts",
            &shifts.to_string(),
            "--workers",
            "1",
            "--seed",
            "711",
            "--max-rounds",
            "1",
        ])
        .arg("--checkpoint")
        .arg(checkpoint);
    if serial {
        command.args(["--serial", "0.001"]);
    }
    if resume {
        command.arg("--resume");
    }
    success(command.output().unwrap())
}

fn same_frozen_adaptive_allocation(artifact: &Path, checkpoint: &Path, directory: &Path) {
    use fastsecdec::integration::{IntegrationProblem, QmcSession};
    let mut saved: Value = serde_json::from_slice(&fs::read(checkpoint).unwrap()).unwrap();
    let problem: IntegrationProblem =
        serde_json::from_value(saved["session"]["problem"].clone()).unwrap();
    let mut session =
        QmcSession::restore(&serde_json::to_vec(&saved["session"]).unwrap(), &problem).unwrap();
    // Extend the already frozen native allocation. Do not run another timed
    // pilot: observation overhead may legitimately change its recommendation.
    session.extend_production_shifts(2).unwrap();
    saved["session"] = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    saved["settings"]["shifts"] = 8.into();
    let mut previous: Option<Value> = None;
    for mode in ["disabled", "aggregate"] {
        let path = directory.join(format!("frozen-{mode}.json"));
        fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
        let report = integrate_shifts(artifact, &path, "adaptive_qmc", false, mode, true, 8);
        if let Some(previous) = &previous {
            assert_eq!(report["estimate"], previous["estimate"]);
            assert_eq!(
                report["snapshot"]["completed_points"],
                previous["snapshot"]["completed_points"]
            );
        }
        previous = Some(report);
    }
}

fn calls(value: &Value, phase: &str, work: &str) -> u64 {
    value[phase][work]["callback_calls"].as_u64().unwrap()
}

#[test]
fn actual_worker_runtime_reports_preserve_sampling_and_checkpoint_compatibility() {
    let directory = tempfile::tempdir().unwrap();
    let artifact = directory.path().join("runtime.fsd");
    generated(&artifact);
    let bytes = fs::read(artifact.with_extension("fsd.json")).unwrap();
    for method in ["qmc", "adaptive_qmc", "mc", "adaptive_mc", "discrete_mc"] {
        for serial in [false, true] {
            if serial && method == "discrete_mc" {
                continue;
            }
            let observed_checkpoint = directory
                .path()
                .join(format!("{method}-{serial}-aggregate.json"));
            let plain_checkpoint = directory
                .path()
                .join(format!("{method}-{serial}-disabled.json"));
            let observed = integrate(
                &artifact,
                &observed_checkpoint,
                method,
                serial,
                "aggregate",
                false,
            );
            let plain = integrate(
                &artifact,
                &plain_checkpoint,
                method,
                serial,
                "disabled",
                false,
            );
            assert_eq!(observed["content_id"], plain["content_id"]);
            if method != "adaptive_qmc" || serial {
                assert_eq!(
                    observed["estimate"], plain["estimate"],
                    "mode changed {method}/{serial} statistics"
                );
                assert_eq!(
                    observed["snapshot"]["completed_points"],
                    plain["snapshot"]["completed_points"]
                );
            }
            let runtime = &observed["operational"]["diagnostics"]["contour_runtime"];
            assert!(
                calls(runtime, "production", "evaluation") > 0,
                "{method}/{serial}: {runtime}"
            );
            assert!(calls(runtime, "adaptation", "pilot") > 0);
            assert_eq!(calls(runtime, "production", "pilot"), 0);
            if method.starts_with("adaptive_") || method == "discrete_mc" {
                assert!(calls(runtime, "adaptation", "evaluation") > 0);
            }
            assert!(plain["operational"]["diagnostics"]["contour_runtime"].is_null());
            assert!(plain["snapshot"]["evaluation_diagnostics"]["contour_runtime"].is_null());
            let resumed = integrate(
                &artifact,
                &observed_checkpoint,
                method,
                serial,
                "disabled",
                true,
            );
            assert_eq!(resumed["estimate"], observed["estimate"]);
            assert_eq!(
                resumed["snapshot"]["evaluation_diagnostics"]["contour_runtime"],
                observed["snapshot"]["evaluation_diagnostics"]["contour_runtime"]
            );
            assert!(resumed["operational"]["diagnostics"]["contour_runtime"].is_null());
            let enabled = integrate(
                &artifact,
                &plain_checkpoint,
                method,
                serial,
                "aggregate",
                true,
            );
            assert_eq!(enabled["estimate"], plain["estimate"]);
            let runtime = &enabled["operational"]["diagnostics"]["contour_runtime"];
            // A completed checkpoint can still require binding/exact/pilot work,
            // but must not acquire another accepted production allocation.
            if !runtime.is_null() {
                assert_eq!(calls(runtime, "production", "evaluation"), 0);
            }
            if method == "adaptive_qmc" && !serial {
                same_frozen_adaptive_allocation(&artifact, &observed_checkpoint, directory.path());
            }
        }
    }
    assert_eq!(
        fs::read(artifact.with_extension("fsd.json")).unwrap(),
        bytes
    );
}

#[test]
fn misspelled_runtime_observation_mode_is_rejected_before_loading() {
    let result = cli()
        .args([
            "integrate",
            "missing.fsd",
            "--contour-diagnostics",
            "aggreagte",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("disabled or aggregate"));
}
