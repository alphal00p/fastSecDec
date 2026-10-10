//! Bounded supervisor shared by optional external interoperability tests.
use std::os::unix::process::CommandExt;
use std::{
    fs, io,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn signal(pid: i32, signal: i32) -> io::Result<()> {
    // These are exclusively process IDs started by this test or discovered
    // through their owned /proc child inventory. No external kill program.
    if unsafe { libc::kill(pid, signal) } == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

fn stop_tree(pid: u32, owned: &mut Vec<u32>) -> io::Result<()> {
    if owned.contains(&pid) {
        return Ok(());
    }
    if owned.len() >= 256 {
        return Err(io::Error::other("external workflow exceeds 256 processes"));
    }
    owned.push(pid);
    signal(pid as i32, libc::SIGSTOP)?;
    // Wait for SIGSTOP before inspecting children, so a live ancestor cannot
    // create a new worker after its inventory has been read.
    let until = Instant::now() + Duration::from_millis(100);
    loop {
        let status = fs::read_to_string(format!("/proc/{pid}/status"));
        match status {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
            Ok(status) => {
                if status.lines().any(|line| {
                    line.starts_with("State:")
                        && line
                            .split_whitespace()
                            .nth(1)
                            .is_some_and(|s| matches!(s, "T" | "t" | "Z"))
                }) {
                    break;
                }
            }
        }
        if Instant::now() >= until {
            return Err(io::Error::other("owned process did not stop"));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let tasks = match fs::read_dir(format!("/proc/{pid}/task")) {
        Ok(tasks) => tasks,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for task in tasks.take(256) {
        let children = match fs::read_to_string(task?.path().join("children")) {
            Ok(children) => children,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for child in children.split_whitespace() {
            let child = child.parse::<u32>().map_err(io::Error::other)?;
            stop_tree(child, owned)?;
        }
    }
    Ok(())
}

fn terminate_tree(pid: u32) -> io::Result<()> {
    let mut owned = Vec::new();
    let mut result = stop_tree(pid, &mut owned);
    // Kill every discovered descendant, including independently created groups.
    // Even if discovery reports an error, attempt all known cleanup and report it.
    for child in owned.into_iter().rev() {
        let killed = signal(child as i32, libc::SIGKILL);
        if result.is_ok() {
            result = killed;
        }
    }
    let group = signal(-(pid as i32), libc::SIGKILL);
    if result.is_ok() { group } else { result }
}

pub fn run(
    mut command: Command,
    directory: &Path,
    name: &str,
    deadline: Instant,
) -> serde_json::Value {
    assert!(
        Instant::now() < deadline,
        "{name}: common interoperability deadline already expired"
    );
    let stdout = directory.join(format!("{name}.stdout.json"));
    let stderr = directory.join(format!("{name}.stderr.log"));
    command
        .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr).unwrap()))
        .process_group(0);
    assert!(
        Instant::now() < deadline,
        "{name}: common interoperability deadline expired before spawn"
    );
    let mut child = command.spawn().unwrap();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let cleanup = terminate_tree(child.id());
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "{name} exceeded the common interoperability deadline; cleanup: {cleanup:?}; evidence: {}",
                directory.display()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(
        status.success(),
        "{name} failed; evidence: {}\n{}",
        directory.display(),
        fs::read_to_string(stderr).unwrap()
    );
    let bytes = fs::read(stdout).unwrap();
    // Public `solve --output` writes its proof without a stdout summary.
    // Callers must inspect that fresh proof and independently verify it.
    if bytes.iter().all(u8::is_ascii_whitespace) {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}
