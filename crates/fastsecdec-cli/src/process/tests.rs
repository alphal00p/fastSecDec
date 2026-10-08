use super::{ProcessPool, protocol};
use std::{
    io::Cursor,
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn framed_control_rejects_truncation_corruption_and_large_frames() {
    let message = protocol::Envelope {
        run_id: "run".into(),
        lease_id: 7,
        payload: vec![1u64, 2, 3],
    };
    let mut bytes = Vec::new();
    protocol::write(&mut bytes, &message).unwrap();
    type Message = protocol::Envelope<Vec<u64>>;
    let restored: Message = protocol::read(&mut Cursor::new(&bytes)).unwrap().unwrap();
    assert_eq!(restored.payload, message.payload);
    for end in 1..bytes.len() {
        assert!(protocol::read::<Message>(&mut Cursor::new(&bytes[..end])).is_err());
    }
    bytes[0] ^= 1;
    assert!(protocol::read::<Message>(&mut Cursor::new(&bytes)).is_err());
    bytes[0] ^= 1;
    bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(protocol::read::<Message>(&mut Cursor::new(&bytes)).is_err());
    assert!(protocol::write(&mut Vec::new(), &"x".repeat(protocol::MAX_MESSAGE_BYTES)).is_err());
}

#[cfg(unix)]
#[test]
fn residency_limit_is_released_only_after_worker_exit() {
    let pool = ProcessPool::new(1).unwrap();
    let mut command = Command::new("cat");
    let mut worker = pool.spawn::<String>(&mut command, "run".into(), 2).unwrap();
    assert_eq!(pool.live(), 1);
    assert!(
        pool.spawn::<String>(&mut Command::new("cat"), "run".into(), 3)
            .is_err()
    );
    worker.send("work").unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(reply) = worker.poll().unwrap() {
            assert_eq!(reply, "work");
            break;
        }
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(5));
    }
    worker.terminate().unwrap();
    assert_eq!(pool.live(), 0);
    let replacement = pool
        .spawn::<String>(&mut Command::new("cat"), "run".into(), 3)
        .unwrap();
    assert_ne!(worker.pid(), replacement.pid());
    drop(replacement);
    assert_eq!(pool.live(), 0);
}

#[cfg(unix)]
#[test]
fn failed_spawn_returns_its_residency_slot() {
    let pool = ProcessPool::new(1).unwrap();
    assert!(
        pool.spawn::<String>(
            &mut Command::new("/nonexistent/fastsecdec-worker"),
            "run".into(),
            1
        )
        .is_err()
    );
    assert_eq!(pool.live(), 0);
}

/// Executed only as a child by the socket/OS-lock regression below. The test
/// harness and native stdout noise must be harmless to the control transport.
#[test]
#[ignore = "subprocess fixture"]
fn socket_child_fixture() {
    if std::env::var_os("FASTSECDEC_WORKER_CONTROL").is_none() {
        return;
    }
    super::child::serve(
        "socket-test".into(),
        3,
        |_: (), emit: &mut dyn FnMut(String) -> std::io::Result<()>| {
            println!("native library stdout is not an IPC frame");
            emit("completed".into())
        },
    )
    .unwrap();
}

#[test]
fn socket_control_survives_stdout_and_inherits_residency_lock() {
    use super::child::{WorkerCommand, WorkerEvent};
    let directory = tempfile::tempdir().unwrap();
    let lock_path = directory.path().join("run.lock");
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .unwrap();
    lock.try_lock().unwrap();
    let pool = ProcessPool::with_residency_lock(1, &lock).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args([
        "--exact",
        "process::tests::socket_child_fixture",
        "--ignored",
        "--nocapture",
    ]);
    let mut worker = pool
        .spawn_native::<WorkerEvent<String>>(
            &mut command,
            "socket-test".into(),
            3,
            &directory.path().join("worker.log"),
        )
        .unwrap();
    drop(command); // Command itself retains the configured inherited stdin file.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut received = false;
    loop {
        if let Some(event) = worker.poll().unwrap() {
            match event {
                WorkerEvent::Ready { .. } => worker.send(WorkerCommand::Execute(())).unwrap(),
                WorkerEvent::Update(value) => {
                    assert_eq!(value, "completed");
                    received = true;
                }
                WorkerEvent::Finished => break,
                WorkerEvent::Failed { message } => panic!("{message}"),
            }
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(received);
    // Simulate loss of the coordinator's own file descriptors. The child is
    // idle, still holding inherited stdin, until an explicit shutdown or EOF.
    drop(lock);
    drop(pool);
    let replacement = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    assert!(
        replacement.try_lock().is_err(),
        "old child must fence a new coordinator"
    );
    worker.send(WorkerCommand::<()>::Shutdown).unwrap();
    while worker.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    replacement.try_lock().unwrap();
    assert!(
        std::fs::read_to_string(directory.path().join("worker.log"))
            .unwrap()
            .contains("native library stdout")
    );
}

/// Keep the accepted socket idle and split both headers and payloads across
/// writes. A nonblocking connection would lose the frame at either boundary.
#[test]
#[ignore = "subprocess fixture"]
fn fragmented_socket_child_fixture() {
    use std::{io::Write, net::TcpStream};
    let Some(address) = std::env::var_os("FASTSECDEC_WORKER_CONTROL") else {
        return;
    };
    let mut stream = TcpStream::connect(address.to_str().unwrap()).unwrap();
    stream.set_nodelay(true).unwrap();
    for payload in ["first", "second"] {
        let mut frame = Vec::new();
        protocol::write(
            &mut frame,
            &protocol::Envelope {
                run_id: "fragmented-socket".into(),
                lease_id: 5,
                payload,
            },
        )
        .unwrap();
        let mut start = 0;
        for end in [1, 8, 10, 12, frame.len() - 1, frame.len()] {
            std::thread::sleep(Duration::from_millis(20));
            stream.write_all(&frame[start..end]).unwrap();
            start = end;
        }
    }
}

#[test]
fn socket_control_waits_for_delayed_fragmented_frames() {
    let directory = tempfile::tempdir().unwrap();
    let pool = ProcessPool::new(1).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args([
        "--exact",
        "process::tests::fragmented_socket_child_fixture",
        "--ignored",
        "--nocapture",
    ]);
    let mut worker = pool
        .spawn_native::<String>(
            &mut command,
            "fragmented-socket".into(),
            5,
            &directory.path().join("worker.log"),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut messages = Vec::new();
    while !worker.output_closed() {
        if let Some(message) = worker.poll().unwrap() {
            messages.push(message);
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(messages, ["first", "second"]);
    while worker.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(worker.try_wait().unwrap().unwrap().success());
    assert_eq!(pool.live(), 0);
}
