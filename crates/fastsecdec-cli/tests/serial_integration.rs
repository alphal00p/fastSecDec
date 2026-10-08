//! Real resident processes exercise selective artifacts, statistics and recovery.
use serde_json::Value;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Output, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
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
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn generate(directory: &Path, serial: bool) -> std::path::PathBuf {
    let card = directory.join("integral.toml");
    fs::write(
        &card,
        r#"
[direct]
domain="unit_cube"
parameters=["x","y"]
[[direct.terms]]
prefactor="2+3𝑖"
monomial_powers=["0","0"]
[[direct.terms.factors]]
polynomial="x+2*y"
exponent="-1"
[generation]
order=0
[generation.evaluator]
backend="eager"
[integration]
points=1024
shifts=8
workers=2
max_rounds=1
absolute_tolerance=0.0
relative_tolerance=0.0
"#,
    )
    .unwrap();
    let base = directory.join(if serial { "serial.fsd" } else { "ordinary.fsd" });
    let mut command = cli();
    command
        .arg("generate")
        .arg(&card)
        .arg("--output")
        .arg(&base);
    if serial {
        command.args(["--serial", "--workers", "2"]);
    }
    let generated = success(command.output().unwrap());
    assert!(generated["sectors"].as_u64().unwrap() >= 2);
    base
}
fn integration(base: &Path, method: &str) -> Command {
    let mut command = cli();
    command
        .arg("integrate")
        .arg(base)
        .args(["--method", method]);
    command
}
fn check_analytic(value: &Value) {
    assert_eq!(value["stopping_reason"], "work limit");
    assert_eq!(value["estimate"]["production_complete"], true);
    let means = value["estimate"]["mean"].as_array().unwrap();
    let errors = value["estimate"]["standard_error"].as_array().unwrap();
    let components = value["estimate"]["components"].as_array().unwrap();
    assert_eq!(means.len(), 2);
    for ((mean, error), component) in means.iter().zip(errors).zip(components) {
        let factor = match component.as_str().unwrap() {
            "Real" => 2.,
            "Imag" => 3.,
            other => panic!("unexpected component {other}"),
        };
        let expected = factor * (1.5 * 3f64.ln() - 2f64.ln());
        assert!(
            (mean.as_f64().unwrap() - expected).abs() <= 8. * error.as_f64().unwrap() + 2e-6,
            "analytic control differs: {value}"
        );
    }
    assert!(value["operational"]["evaluations"].as_u64().unwrap() > 0);
}

#[test]
fn both_artifact_generation_modes_support_all_serial_integrators() {
    let directory = tempfile::tempdir().unwrap();
    for generation_serial in [false, true] {
        let base = generate(directory.path(), generation_serial);
        for method in ["qmc", "mc"] {
            check_analytic(&success(integration(&base, method).output().unwrap()));
        }
        for method in ["qmc", "adaptive_qmc", "mc", "adaptive_mc"] {
            // Exercise a checkpoint without a directory component, including
            // creation of its staging directory and durable publication.
            let checkpoint = format!("{generation_serial}-{method}.checkpoint");
            let result = success(
                integration(&base, method)
                    .current_dir(directory.path())
                    .args(["--serial", "0.001", "--checkpoint"])
                    .arg(&checkpoint)
                    .output()
                    .unwrap(),
            );
            check_analytic(&result);
            assert!(directory.path().join(&checkpoint).is_file());
            let resumed = success(
                integration(&base, method)
                    .current_dir(directory.path())
                    .args(["--serial", "0.2", "--checkpoint"])
                    .arg(&checkpoint)
                    .args([
                        "--resume",
                        "--workers",
                        "1",
                        "--evaluation-batch-size",
                        "19",
                    ])
                    .output()
                    .unwrap(),
            );
            assert_eq!(result["estimate"], resumed["estimate"]);
            assert_eq!(resumed["operational"]["evaluations"], 0);
        }
    }
}

#[test]
fn fixed_replica_growth_and_selective_complex_output_match_ordinary_integration() {
    let directory = tempfile::tempdir().unwrap();
    let base = generate(directory.path(), false);
    for method in ["qmc", "mc"] {
        let ordinary = success(integration(&base, method).output().unwrap());
        check_analytic(&ordinary);
        let serial = success(
            integration(&base, method)
                .args([
                    "--serial",
                    "0.001",
                    "--double-points",
                    "false",
                    "--max-rounds",
                    "2",
                ])
                .output()
                .unwrap(),
        );
        check_analytic(&serial);
        for ((a, b), (sa, sb)) in ordinary["estimate"]["mean"]
            .as_array()
            .unwrap()
            .iter()
            .zip(serial["estimate"]["mean"].as_array().unwrap())
            .zip(
                ordinary["estimate"]["standard_error"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(serial["estimate"]["standard_error"].as_array().unwrap()),
            )
        {
            let error = sa.as_f64().unwrap().hypot(sb.as_f64().unwrap());
            assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 8. * error + 2e-6);
        }
        for row in serial["snapshot"]["sectors"].as_array().unwrap() {
            assert_eq!(row["complete_replicas"], 16);
            assert_eq!(row["completed_points"], 16 * 1024);
        }
    }
}

#[cfg(unix)]
#[test]
fn interrupted_residents_resume_with_changed_workers_without_admitting_partial_replicas() {
    let directory = tempfile::tempdir().unwrap();
    let base = generate(directory.path(), false);
    let checkpoint = directory.path().join("cancelled.checkpoint");
    let stdout = directory.path().join("cancelled.json");
    let mut child = integration(&base, "qmc")
        .args([
            "--status-json",
            "--status-interval-ms",
            "0",
            "--serial",
            "60",
            "--points",
            "65536",
            "--checkpoint",
        ])
        .arg(&checkpoint)
        .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let (send, receive) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                let _ = send.send(value);
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut accepted = 0;
    let mut resident_pids = Vec::new();
    while Instant::now() < deadline {
        let Ok(value) = receive.recv_timeout(Duration::from_secs(1)) else {
            continue;
        };
        let Some(sectors) = value["serial"]["native"]["sectors"].as_array() else {
            continue;
        };
        accepted = sectors
            .iter()
            .map(|s| s["accepted_points"].as_u64().unwrap())
            .sum();
        if accepted > 0 {
            let residents = value["serial"]["residents"].as_array().unwrap();
            assert!(!residents.is_empty());
            assert!(
                residents
                    .iter()
                    .all(|resident| { resident["residence_seconds"].as_f64().unwrap() < 60. }),
                "completed lattices were not published before minimum residence"
            );
            resident_pids = residents
                .iter()
                .map(|resident| resident["pid"].as_u64().unwrap())
                .collect();
            break;
        }
    }
    if accepted == 0 {
        let _ = child.kill();
    }
    assert!(
        accepted > 0,
        "did not observe an accepted lattice before interruption"
    );
    assert!(
        Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            panic!("coordinator did not stop promptly")
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    reader.join().unwrap();
    for pid in resident_pids {
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "resident survived cancellation"
        );
    }
    let cancelled: Value = serde_json::from_slice(&fs::read(stdout).unwrap()).unwrap();
    assert_eq!(cancelled["stopping_reason"], "cancelled");
    let resumed = success(
        integration(&base, "qmc")
            .args([
                "--serial",
                "0.02",
                "--points",
                "65536",
                "--workers",
                "1",
                "--evaluation-batch-size",
                "31",
                "--checkpoint",
            ])
            .arg(&checkpoint)
            .arg("--resume")
            .output()
            .unwrap(),
    );
    check_analytic(&resumed);
    for row in resumed["snapshot"]["sectors"].as_array().unwrap() {
        assert_eq!(row["complete_replicas"], 8);
        assert_eq!(row["completed_points"], 8 * 65536);
    }
}
