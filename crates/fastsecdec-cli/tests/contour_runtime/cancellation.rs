use super::*;
use std::{
    io::{BufRead, BufReader},
    process::Stdio,
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn interrupted_serial_reservation_preserves_its_last_report_once() {
    let directory = tempfile::tempdir().unwrap();
    let artifact = directory.path().join("interrupted.fsd");
    generated(&artifact);
    let checkpoint = directory.path().join("interrupted.checkpoint");
    let output = directory.path().join("interrupted.json");
    let mut child = cli()
        .arg("integrate")
        .arg(&artifact)
        .args([
            "--method",
            "qmc",
            "--contour",
            "dynamical=0.8",
            "--contour-validation",
            "off",
            "--contour-diagnostics",
            "aggregate",
            "--points",
            "1048576",
            "--shifts",
            "2",
            "--workers",
            "1",
            "--serial",
            "60",
            "--status-json",
            "--status-interval-ms",
            "0",
            "--evaluation-batch-size",
            "128",
            "--max-rounds",
            "1",
            "--checkpoint",
        ])
        .arg(&checkpoint)
        .stdout(Stdio::from(fs::File::create(&output).unwrap()))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let (send, receive) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut last_live = None;
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            let serial = &value["serial"];
            if serial["residents"]
                .as_array()
                .is_some_and(|v| !v.is_empty())
            {
                let runtime = &serial["operational"]["diagnostics"]["contour_runtime"];
                if runtime["production"]["evaluation"]["callback_calls"]
                    .as_u64()
                    .is_some_and(|n| n > 0)
                {
                    last_live = Some(runtime.clone());
                    let _ = send.send(serial.clone());
                }
            }
        }
        last_live
    });
    let progress = receive.recv_timeout(Duration::from_secs(30));
    if progress.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let progress = progress.expect("no partial native worker report before deadline");
    assert!(progress["residents"].as_array().unwrap().iter().all(|r| {
        r["completed_points"].as_u64().unwrap() < r["planned_points"].as_u64().unwrap()
    }));
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
            let _ = child.kill();
            let _ = child.wait();
            panic!("serial coordinator did not stop after interruption");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let last_live = reader.join().unwrap().unwrap();
    let report: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert_eq!(report["stopping_reason"], "cancelled");
    let runtime = &report["operational"]["diagnostics"]["contour_runtime"];
    assert_eq!(
        runtime, &last_live,
        "the last cumulative task report was admitted twice"
    );
    assert!(calls(runtime, "production", "evaluation") > 0);
    assert!(
        report["snapshot"]["sectors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["completed_points"].as_u64() == Some(0))
    );
    assert_eq!(
        &report["snapshot"]["evaluation_diagnostics"]["contour_runtime"],
        runtime
    );
    let saved: Value = serde_json::from_slice(&fs::read(checkpoint).unwrap()).unwrap();
    assert_eq!(
        &saved["checkpoint"]["diagnostics"]["contour_runtime"],
        runtime
    );
}
