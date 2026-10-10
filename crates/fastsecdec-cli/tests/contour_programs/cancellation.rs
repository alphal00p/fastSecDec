use super::*;
use std::{
    io::{BufRead, BufReader},
    process::Stdio,
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn ordinary_family_cancellation_joins_workers_and_preserves_previous_publication() {
    let directory = tempfile::tempdir().unwrap();
    let artifact = bundle(directory.path(), false, "symbolic");
    let manifest = artifact.with_extension("fsd.json");
    let before = fs::read(&manifest).unwrap();
    let input = directory.path().join("replacement.toml");
    fs::write(
        &input,
        r#"
[direct]
domain="unit_cube"
parameters=["x", "y", "z", "w"]
[[direct.terms]]
monomial_powers=["0", "0", "0", "0"]
[[direct.terms.factors]]
polynomial="1+x+y+z+w+x*y*z*w"
exponent="-1-eps"
semantics="causal"
[generation]
order=2
[generation.evaluator]
backend="eager"
"#,
    )
    .unwrap();
    let stdout = directory.path().join("cancelled.json");
    let mut child = cli()
        .args(["--status-json", "--status-interval-ms", "0", "generate"])
        .arg(&input)
        .args(["--contour", "--workers", "2", "--output"])
        .arg(&artifact)
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
    let mut reached = false;
    while Instant::now() < deadline {
        if let Ok(status) = receive.recv_timeout(Duration::from_millis(50)) {
            reached = status["workload"]["workers"]
                .as_array()
                .is_some_and(|workers| workers.iter().any(|worker| worker["busy"] == true));
            if reached {
                break;
            }
        }
        if child.try_wait().unwrap().is_some() {
            break;
        }
    }
    if !reached {
        let _ = child.kill();
    }
    assert!(
        reached,
        "no executing native family job observed: {}",
        fs::read_to_string(&stdout).unwrap()
    );
    assert!(
        Command::new("bash")
            .args(["-c", "kill -INT \"$1\"", "fastsecdec-family-test"])
            .arg(child.id().to_string())
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("family cancellation did not join its workers promptly");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    reader.join().unwrap();
    assert!(!status.success());
    let report: Value = serde_json::from_slice(&fs::read(&stdout).unwrap()).unwrap();
    assert!(
        report["error"]["message"]
            .as_str()
            .unwrap()
            .contains("cancel"),
        "{report}"
    );
    assert_eq!(fs::read(manifest).unwrap(), before);
    let inspected = success(
        cli()
            .arg("inspect")
            .arg(artifact)
            .args(["--deep", "--validate-artifact"])
            .output()
            .unwrap(),
    );
    assert_eq!(inspected["selected_recipe"], "undeformed-v1");
}
