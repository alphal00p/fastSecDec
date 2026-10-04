//! Exercise the typed diagnostic and cancellation contract across real CLI processes.
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use fastsecdec::diagnostics::{
    BoundaryAssessment, BoundaryScanProgress, BoundaryScanReport, DiagnosticProgress,
    DiagnosticStop,
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fastsecdec"));
    command.env("SYMBOLICA_HIDE_BANNER", "1").arg("--plain");
    command
}

fn generate(directory: &Path, card: &str) -> PathBuf {
    let input = directory.join("input.toml");
    let artifact = directory.join("integral.json");
    fs::write(&input, card).unwrap();
    let generated = cli()
        .arg("--json")
        .arg("generate")
        .arg(input)
        .arg("--output")
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{} {}",
        String::from_utf8_lossy(&generated.stdout),
        String::from_utf8_lossy(&generated.stderr)
    );
    artifact
}

#[test]
fn growth_flags_remain_diagnostic_and_retries_preserve_typed_history() {
    let directory = tempfile::tempdir().unwrap();
    let artifact = generate(
        directory.path(),
        r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
monomial_powers = ["0"]
[[direct.terms.factors]]
polynomial = "x+1/10000000"
exponent = "-3/4"
role = "singularity"
"#,
    );
    let flagged = cli()
        .arg("--json")
        .arg("check-boundaries")
        .arg(&artifact)
        .args(["--exponents", "4,6"])
        .output()
        .unwrap();
    assert!(
        flagged.status.success(),
        "a sampled growth flag is not an evaluation failure"
    );
    let flagged: BoundaryScanReport = serde_json::from_slice(&flagged.stdout).unwrap();
    assert_eq!(flagged.assessment, BoundaryAssessment::Flagged);
    assert_eq!(flagged.diagnostics.failures, 0);
    let retried = cli()
        .arg("--json")
        .arg("--status-json")
        .arg("check-boundaries")
        .arg(&artifact)
        .args([
            "--exponents",
            "4,6",
            "--growth-tolerance",
            "0.5",
            "--retry-scales",
            "0.01,0.0001",
        ])
        .output()
        .unwrap();
    assert!(retried.status.success());
    let report: BoundaryScanReport = serde_json::from_slice(&retried.stdout).unwrap();
    assert_eq!(report.assessment, BoundaryAssessment::WithinThreshold);
    assert_eq!(report.attempts.len(), 2);
    assert!(report.had_prior_flags);
    assert_eq!(report.attempts[1].scale, 0.01);
    assert_eq!(report.completed_probes, 8);
    let events = String::from_utf8(retried.stderr)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<BoundaryScanProgress>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, BoundaryScanProgress::Assessed { attempt: 1, .. }))
    );
    let plain = cli()
        .arg("check-boundaries")
        .arg(&artifact)
        .args(["--exponents", "4,6", "--retry-scales", "0.01"])
        .output()
        .unwrap();
    assert!(plain.status.success());
    let text = String::from_utf8(plain.stdout).unwrap();
    assert!(text.contains("Attempt") && text.contains("Flagged"));
    assert!(text.contains("not an integrability certificate"));
    assert!(
        !text.contains("\"attempts\""),
        "plain display must not dump nested JSON"
    );
}

#[test]
fn numerical_failure_emits_one_complete_json_document_and_nonzero_exit() {
    let directory = tempfile::tempdir().unwrap();
    let artifact = generate(
        directory.path(),
        r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
prefactor = "10^308"
monomial_powers = ["-1/2"]
"#,
    );
    let output = cli()
        .arg("--json")
        .arg("check-boundaries")
        .arg(artifact)
        .args(["--exponents", "3,6"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report: BoundaryScanReport = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report.diagnostics.failures > 0);
    assert!(
        report.attempts[0]
            .samples
            .probes
            .iter()
            .any(|probe| !probe.finite && probe.values.is_none())
    );
    assert_eq!(report.assessment, BoundaryAssessment::Inconclusive);
}

#[test]
fn sigint_preserves_partial_boundary_report_and_selected_sector_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let parameters = (0..20)
        .map(|i| format!("\"x{i}\""))
        .collect::<Vec<_>>()
        .join(",");
    let powers = std::iter::once("\"-3/4\"")
        .chain(std::iter::repeat_n("\"0\"", 19))
        .collect::<Vec<_>>()
        .join(",");
    let artifact = generate(
        directory.path(),
        &format!(
            r#"
[direct]
domain = "unit_cube"
parameters = [{parameters}]
[[direct.terms]]
monomial_powers = [{powers}]
"#
        ),
    );
    let mut child = cli()
        .arg("--json")
        .arg("--status-json")
        .arg("check-boundaries")
        .arg(artifact)
        .args([
            "--exponents",
            "3,6",
            "--max-codimension",
            "20",
            "--max-probes",
            "100000",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stderr.take().unwrap());
    loop {
        let mut line = String::new();
        assert_ne!(
            lines.read_line(&mut line).unwrap(),
            0,
            "process stopped before diagnostics"
        );
        if matches!(
            serde_json::from_str::<BoundaryScanProgress>(&line).unwrap(),
            BoundaryScanProgress::Sampling {
                progress: DiagnosticProgress::Boundary { completed: 0, .. },
                ..
            }
        ) {
            break;
        }
    }
    assert!(
        Command::new("bash")
            .args(["-c", "kill -INT \"$1\"", "fastsecdec-boundary-test"])
            .arg(child.id().to_string())
            .status()
            .unwrap()
            .success()
    );
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: BoundaryScanReport = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report.stop, DiagnosticStop::Cancelled);
    assert!(report.completed_probes < 100000);
    assert_eq!(report.attempts.len(), 1);
    assert_eq!(report.attempts[0].samples.sectors.len(), 1);
    assert!(!report.latest[0].sector.sampling_complete);
    assert!(matches!(
        report.assessment,
        BoundaryAssessment::Inconclusive | BoundaryAssessment::Flagged
    ));
}
