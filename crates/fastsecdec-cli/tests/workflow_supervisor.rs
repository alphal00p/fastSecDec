//! Deadline enforcement also covers descendants with their own process group.
#![cfg(target_os = "linux")]
use std::{
    fs,
    os::unix::process::CommandExt,
    process::Command,
    time::{Duration, Instant},
};
#[path = "support/external_workflow.rs"]
mod external_workflow;

#[test]
fn expired_deadline_does_not_spawn_or_create_stage_files() {
    let directory = tempfile::tempdir().unwrap();
    let command = Command::new("must-not-be-executed");
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        external_workflow::run(
            command,
            directory.path(),
            "expired",
            Instant::now() - Duration::from_secs(1),
        )
    }));
    assert!(outcome.is_err());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
#[ignore = "private subprocess fixture, invoked only by deadline regression"]
fn owned_subprocess_fixture() {
    let Some(directory) = std::env::var_os("FASTSECDEC_SUPERVISOR_FIXTURE") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    if std::env::var_os("FASTSECDEC_SUPERVISOR_LEAF").is_none() {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "owned_subprocess_fixture",
                "--nocapture",
            ])
            .env("FASTSECDEC_SUPERVISOR_LEAF", "1")
            .process_group(0)
            .spawn()
            .unwrap();
        fs::write(directory.join("child.pid"), child.id().to_string()).unwrap();
        let _ = child.wait();
    } else {
        fs::write(directory.join("leaf.ready"), b"ready").unwrap();
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn deadline_terminates_child_in_an_independent_group() {
    let directory = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "owned_subprocess_fixture",
            "--nocapture",
        ])
        .env("FASTSECDEC_SUPERVISOR_FIXTURE", directory.path());
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        external_workflow::run(
            command,
            directory.path(),
            "timeout",
            Instant::now() + Duration::from_millis(800),
        )
    }));
    assert!(outcome.is_err());
    assert!(
        directory.path().join("leaf.ready").exists(),
        "fixture did not reach descendant stage"
    );
    let pid = fs::read_to_string(directory.path().join("child.pid")).unwrap();
    let until = Instant::now() + Duration::from_secs(1);
    loop {
        let state = fs::read_to_string(format!("/proc/{}/status", pid.trim()));
        if state
            .as_ref()
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
            || state.as_ref().is_ok_and(|s| {
                s.lines()
                    .any(|l| l.starts_with("State:") && l.split_whitespace().nth(1) == Some("Z"))
            })
        {
            break;
        }
        assert!(Instant::now() < until, "owned descendant survived cleanup");
        std::thread::sleep(Duration::from_millis(10));
    }
}
