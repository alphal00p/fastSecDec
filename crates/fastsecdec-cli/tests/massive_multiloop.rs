//! Complete CLI diagnostics with frozen independent numerical comparisons.
//! All sampling, precision controls and checkpoints belong to the production CLI.
#[path = "support/artifact.rs"]
mod artifact_data;
#[path = "support/multiloop_reference.rs"]
mod reference;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use fastsecdec::{
    integration::VectorEstimate,
    reference::{Pull, ReferenceComparison},
    status::{CoefficientComponent, GenerationTimings, IntegrationSnapshot},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct GenerationReport {
    content_id: String,
    sectors: usize,
    orders: Vec<i32>,
    generation_timings: GenerationTimings,
}

#[derive(Deserialize, Serialize)]
struct IntegrationReport {
    content_id: String,
    elapsed_seconds: f64,
    loading_seconds: f64,
    converged: bool,
    stopping_reason: String,
    estimate: Option<VectorEstimate>,
    snapshot: IntegrationSnapshot,
}

#[derive(Serialize)]
struct CaseReport {
    name: &'static str,
    loops: usize,
    propagators: usize,
    runtime_parameters: Vec<String>,
    generation: Option<GenerationReport>,
    integration: Option<IntegrationReport>,
    reference_comparison: Option<ReferenceComparison>,
    errors: Vec<String>,
}

fn invoke(
    directory: &Path,
    name: &str,
    phase: &str,
    args: &[&std::ffi::OsStr],
) -> Result<Vec<u8>, String> {
    let stdout = directory.join(format!("{name}.{phase}.json"));
    let stderr = directory.join(format!("{name}.{phase}.status.jsonl"));
    let mut child = Command::new(env!("CARGO_BIN_EXE_fastsecdec"))
        .env("SYMBOLICA_HIDE_BANNER", "1")
        .args(["--json", "--plain", "--status-json"])
        .args(args)
        .stdout(Stdio::from(
            File::create(&stdout).map_err(|error| error.to_string())?,
        ))
        .stderr(Stdio::from(
            File::create(&stderr).map_err(|error| error.to_string())?,
        ))
        .spawn()
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    // Each isolated generation/integration stage has a wall-clock watchdog.
    // A timeout is a diagnostic failure, never a fabricated zero coefficient.
    let limit = Duration::from_secs(180);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            let bytes = fs::read(&stdout).map_err(|error| error.to_string())?;
            if status.success() {
                return Ok(bytes);
            }
            return Err(format!(
                "{phase} exited {status}: {} (status log {})",
                String::from_utf8_lossy(&bytes),
                stderr.display()
            ));
        }
        if started.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{phase} exceeded {} seconds; retained logs in {}",
                limit.as_secs(),
                directory.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn write_summary(directory: &Path, cases: &[CaseReport]) {
    let summary = serde_json::json!({
        "purpose":"full native pipeline with bounded independent numerical comparisons; convergence certification pending",
        "independently_certified":false,
        "settings":{"points":1024,"shifts":8,"seed":20261004,"workers":2,"periodization":"korobov3","max_rounds":1},
        "cases":cases,
    });
    fs::write(
        directory.join("summary.json"),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
}

fn check_estimate(report: &IntegrationReport, propagators: usize) -> Result<(), String> {
    let estimate = report
        .estimate
        .as_ref()
        .ok_or("missing complete-vector estimate")?;
    if estimate.orders != [0] || estimate.components != [CoefficientComponent::Real] {
        return Err(format!(
            "expected finite real Euclidean coefficient, got {:?}/{:?}",
            estimate.orders, estimate.components
        ));
    }
    if !estimate.production_complete
        || report.snapshot.completed_points != report.snapshot.planned_points
    {
        return Err("incomplete production allocation".into());
    }
    if estimate.mean.len() != 1
        || estimate.standard_error.len() != 1
        || estimate.covariance_of_mean.len() != 1
    {
        return Err("inconsistent complete-vector dimensions".into());
    }
    let sign = if propagators.is_multiple_of(2) {
        1.0
    } else {
        -1.0
    };
    if !estimate.mean[0].is_finite()
        || estimate.mean[0] * sign <= 0.0
        || !estimate.standard_error[0].is_finite()
        || estimate.standard_error[0] < 0.0
        || !estimate.covariance_of_mean[0].is_finite()
        || estimate.covariance_of_mean[0] < 0.0
    {
        return Err(
            "nonfinite output, negative variance, or unexpected Euclidean normalization sign"
                .into(),
        );
    }
    let diagnostics = report
        .snapshot
        .evaluation_diagnostics
        .as_ref()
        .ok_or("missing precision diagnostics")?;
    if diagnostics.failures != 0 || diagnostics.evaluations != report.snapshot.completed_points {
        return Err("evaluation diagnostics do not match complete production coverage".into());
    }
    Ok(())
}

#[test]
#[ignore = "bounded six-example CLI generation/integration diagnostics; run explicitly in an exclusive Symbolica runtime slot"]
fn six_massive_multiloop_cards_generate_and_integrate_complete_vectors() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let directory = std::env::var_os("FASTSECDEC_MULTILOOP_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| repository.join("output/diagnostics/massive-multiloop"));
    fs::create_dir_all(&directory).unwrap();
    let mut cases = Vec::new();
    // Finish native generation for every case before beginning numerical work.
    for (name, loops, propagators) in reference::CASES {
        println!("Generating {name} (L={loops}, N={propagators})");
        let card = repository.join(format!("examples/runs/{name}.toml"));
        let artifact = directory.join(format!("{name}.fsd"));
        let mut report = CaseReport {
            name,
            loops,
            propagators,
            runtime_parameters: Vec::new(),
            generation: None,
            integration: None,
            reference_comparison: None,
            errors: Vec::new(),
        };
        match invoke(
            &directory,
            name,
            "generate",
            &[
                "generate".as_ref(),
                card.as_os_str(),
                "--output".as_ref(),
                artifact.as_os_str(),
            ],
        )
        .and_then(|bytes| {
            serde_json::from_slice::<GenerationReport>(&bytes).map_err(|error| error.to_string())
        }) {
            Ok(generation) => {
                // Generation is a portable template. The run card's historical
                // point is bound only by the later integration invocation.
                let metadata_path = artifact.with_extension("fsd.json");
                let data_path = artifact_data::data_path(&artifact);
                let metadata: serde_json::Value =
                    serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
                assert!(!artifact.exists());
                assert!(data_path.is_file());
                report.runtime_parameters =
                    serde_json::from_value(metadata["kernel"]["runtime_parameters"].clone())
                        .unwrap();
                assert!(
                    report
                        .runtime_parameters
                        .iter()
                        .any(|name| name == "model::mt")
                );
                assert!(
                    report.runtime_parameters.len() > 1,
                    "kinematics must remain runtime inputs"
                );
                if generation.orders != [0] || generation.sectors == 0 {
                    report.errors.push(format!(
                        "unexpected generated orders {:?} or empty numerical support",
                        generation.orders
                    ));
                }
                println!(
                    "{name}: {} native kernels, {:.3}s generation",
                    generation.sectors, generation.generation_timings.total_seconds
                );
                report.generation = Some(generation);
            }
            Err(error) => {
                eprintln!("{name}: {error}");
                report.errors.push(error);
            }
        }
        cases.push(report);
        write_summary(&directory, &cases);
    }
    for index in 0..cases.len() {
        let case = &mut cases[index];
        if case.generation.is_none() {
            continue;
        }
        let name = case.name;
        println!("Integrating {name}: 1024 points × 8 shifts, complete vector");
        let artifact = directory.join(format!("{name}.fsd"));
        let checkpoint = directory.join(format!("{name}.checkpoint.json"));
        match invoke(
            &directory,
            name,
            "integrate",
            &[
                "integrate".as_ref(),
                artifact.as_os_str(),
                "--checkpoint".as_ref(),
                checkpoint.as_os_str(),
                "--points".as_ref(),
                "1024".as_ref(),
                "--shifts".as_ref(),
                "8".as_ref(),
                "--seed".as_ref(),
                "20261004".as_ref(),
                "--workers".as_ref(),
                "2".as_ref(),
            ],
        )
        .and_then(|bytes| {
            serde_json::from_slice::<IntegrationReport>(&bytes).map_err(|error| error.to_string())
        }) {
            Ok(integration) => {
                if integration.content_id != case.generation.as_ref().unwrap().content_id {
                    case.errors
                        .push("generation/integration content identities differ".into());
                }
                if let Err(error) = check_estimate(&integration, case.propagators) {
                    case.errors.push(error);
                }
                if let Some(estimate) = &integration.estimate {
                    println!(
                        "{name}: {:?} ± {:?}; {:.3}s integration (diagnostic, uncertified)",
                        estimate.mean, estimate.standard_error, integration.elapsed_seconds
                    );
                    let comparison = reference::compare_reference(
                        &repository,
                        name,
                        &integration.content_id,
                        estimate,
                    );
                    if !comparison.eligibility.eligible
                        || comparison.rows.len() != 1
                        || !matches!(comparison.rows[0].pull, Pull::Value(value) if value.abs() < 5.0)
                    {
                        case.errors.push(format!(
                            "initial independent comparison failed: {comparison:?}"
                        ));
                    }
                    println!("{name}: {comparison}");
                    case.reference_comparison = Some(comparison);
                }
                case.integration = Some(integration);
            }
            Err(error) => {
                eprintln!("{name}: {error}");
                case.errors.push(error);
            }
        }
        write_summary(&directory, &cases);
    }
    let failed = cases
        .iter()
        .filter(|case| !case.errors.is_empty())
        .map(|case| (case.name, &case.errors))
        .collect::<Vec<_>>();
    assert!(
        failed.is_empty(),
        "multiloop diagnostics failed: {failed:?}; see {}",
        directory.display()
    );
}
