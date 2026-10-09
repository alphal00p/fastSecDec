//! Contour preflight respects mathematical scope and caller cancellation.
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
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn integrate(base: &Path, checkpoint: &Path, serial: bool) -> Command {
    let mut command = cli();
    command
        .arg("integrate")
        .arg(base)
        .args([
            "--contour",
            "fixed",
            "--lambda",
            "0.05",
            "--contour-validation",
            "always",
            "--contour-pilot-points",
            "8",
            "--points",
            "1024",
            "--shifts",
            "4",
            "--workers",
            "1",
            "--max-rounds",
            "1",
            "--checkpoint",
        ])
        .arg(checkpoint);
    if serial {
        command.args(["--serial", "0.001"]);
    }
    command
}

#[test]
fn selected_sector_preflight_excludes_an_unresolved_chart() {
    let directory = tempfile::tempdir().unwrap();
    let card = directory.path().join("scope.toml");
    let base = directory.path().join("scope.fsd");
    // Reuse the native scope control: one primary chart contains the stationary
    // F=0 at its centre, the other is regular throughout its integration range.
    fs::write(
        &card,
        r#"
[direct]
domain="projective_simplex"
parameters=["x","y"]
[[direct.terms]]
monomial_powers=["0","0"]
[[direct.terms.factors]]
polynomial="(x-2*y)^2"
exponent="-1"
semantics="causal"
[generation]
contour=true
[generation.evaluator]
backend="eager"
[integration]
absolute_tolerance=0.0
relative_tolerance=0.0
"#,
    )
    .unwrap();
    let generated = success(
        cli()
            .arg("generate")
            .arg(card)
            .arg("--output")
            .arg(&base)
            .output()
            .unwrap(),
    );
    assert_eq!(generated["sectors"], 2);
    for serial in [false, true] {
        let mut good = 0;
        let mut unresolved = 0;
        for id in 0..2 {
            let checkpoint = directory
                .path()
                .join(format!("{serial}-{id}.checkpoint.json"));
            let output = integrate(&base, &checkpoint, serial)
                .args([
                    "--sectors",
                    &id.to_string(),
                    "--exact-contributions",
                    "exclude",
                ])
                .output()
                .unwrap();
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            if output.status.success() {
                good += 1;
                assert_eq!(report["snapshot"]["sectors"].as_array().unwrap().len(), 1);
                assert_eq!(report["snapshot"]["sectors"][0]["id"], id);
                assert!(report["snapshot"]["completed_points"].as_u64().unwrap() > 0);
                let proofs = report["contour"]["pilots"].as_array().unwrap();
                assert_eq!(proofs.len(), 1);
                assert_eq!(proofs[0]["required_charts"].as_array().unwrap().len(), 1);
                assert_eq!(proofs[0]["required_charts"], proofs[0]["validated_charts"]);
            } else {
                unresolved += 1;
                let message = report["error"]["message"]
                    .as_str()
                    .or_else(|| report["stopping_reason"].as_str())
                    .unwrap();
                assert!(message.contains("unresolved"), "{report}");
                if let Some(completed) = report["snapshot"]["completed_points"].as_u64() {
                    assert_eq!(completed, 0);
                }
            }
        }
        assert_eq!(
            (good, unresolved),
            (1, 1),
            "scope admission changed in serial={serial}"
        );
    }
}

#[cfg(unix)]
#[test]
fn cancellation_during_preflight_admits_no_production_work_and_reaps_residents() {
    use std::{
        io::{BufRead, BufReader},
        process::Stdio,
        sync::mpsc,
        time::{Duration, Instant},
    };
    let directory = tempfile::tempdir().unwrap();
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/contour/threshold_bubble.toml");
    let base = directory.path().join("bubble.fsd");
    success(
        cli()
            .arg("generate")
            .arg(input)
            .arg("--output")
            .arg(&base)
            .output()
            .unwrap(),
    );
    for serial in [false, true] {
        let checkpoint = directory
            .path()
            .join(format!("cancel-{serial}.checkpoint.json"));
        let stdout = directory.path().join(format!("cancel-{serial}.json"));
        let mut command = cli();
        command
            .args(["--status-json", "--status-interval-ms", "5", "integrate"])
            .arg(&base)
            .args([
                "--contour",
                "fixed",
                "--lambda",
                "0.2",
                "--contour-validation",
                "always",
                "--contour-pilot-points",
                "10000000",
                "--workers",
                "1",
                "--points",
                "1024",
                "--shifts",
                "4",
                "--max-rounds",
                "1",
                "--checkpoint",
            ])
            .arg(&checkpoint)
            .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
            .stderr(Stdio::piped());
        if serial {
            command.args(["--serial", "0.001"]);
        }
        let mut child = command.spawn().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    let _ = sender.send(value);
                }
            }
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut ready = false;
        let mut residents = Vec::new();
        while Instant::now() < deadline {
            let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) else {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                continue;
            };
            if serial {
                if let Some(slots) = event["serial"]["residents"].as_array() {
                    ready = slots.iter().any(|slot| {
                        let pilot = &slot["contour_pilot"];
                        matches!((pilot["completed"].as_u64(), pilot["total"].as_u64()),
                            (Some(done), Some(total)) if done > 0 && done < total)
                    });
                    if ready {
                        residents = slots
                            .iter()
                            .filter_map(|slot| slot["pid"].as_u64())
                            .collect();
                    }
                }
            } else {
                ready = event["phase"] == "contour_validation"
                    && matches!((event["completed"].as_u64(), event["total"].as_u64()),
                        (Some(done), Some(total)) if done > 0 && done < total);
            }
            if ready {
                break;
            }
        }
        if !ready {
            let _ = child.kill();
        }
        assert!(
            ready,
            "no unfinished preflight observed in serial={serial}: {}",
            fs::read_to_string(&stdout).unwrap()
        );
        assert!(
            Command::new("bash")
                .args(["-c", "kill -INT \"$1\"", "fastsecdec-contour-test"])
                .arg(child.id().to_string())
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
                panic!("preflight cancellation did not stop promptly");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        reader.join().unwrap();
        for pid in residents {
            assert!(
                !Path::new(&format!("/proc/{pid}")).exists(),
                "resident survived cancellation"
            );
        }
        let report: Value = serde_json::from_slice(&fs::read(&stdout).unwrap()).unwrap();
        if serial {
            assert_eq!(report["stopping_reason"], "cancelled");
            assert_eq!(report["snapshot"]["completed_points"], 0);
            assert!(
                report["contour"]["pilots"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|p| p["sampled_points"] == 0)
            );
        } else {
            assert!(
                report["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("preflight cancelled")
            );
            assert!(
                !checkpoint.exists(),
                "preflight created a numerical checkpoint"
            );
        }
    }
}
