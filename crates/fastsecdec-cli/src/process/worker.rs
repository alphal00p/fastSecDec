use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use super::protocol::{self, Envelope};
use serde::{Serialize, de::DeserializeOwned};

/// A slot is released only after the OS has reaped the old process. The cap
/// therefore includes workers still shutting down, decoding or compiling.
pub(crate) struct ProcessPool {
    limit: usize,
    live: Arc<AtomicUsize>,
    residency_lock: Option<File>,
}

impl ProcessPool {
    pub(crate) fn new(limit: usize) -> io::Result<Self> {
        if limit == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "process worker count must be positive",
            ));
        }
        Ok(Self {
            limit,
            live: Arc::new(AtomicUsize::new(0)),
            residency_lock: None,
        })
    }

    /// Share the coordinator's OS lock with every child. After a coordinator
    /// crash, a replacement cannot acquire it until every old child has exited.
    /// The stable lock file must never be unlinked or explicitly unlocked.
    pub(crate) fn with_residency_lock(limit: usize, file: &File) -> io::Result<Self> {
        let mut pool = Self::new(limit)?;
        pool.residency_lock = Some(file.try_clone()?);
        Ok(pool)
    }

    pub(crate) fn live(&self) -> usize {
        self.live.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(crate) fn spawn<T: DeserializeOwned + Send + 'static>(
        &self,
        command: &mut Command,
        run_id: String,
        lease_id: u64,
    ) -> io::Result<ProcessWorker<T>> {
        self.spawn_inner(command, run_id, lease_id, None)
    }

    /// Native stdout is a separate diagnostic log: licenses, backend messages
    /// and user-requested compiler verbosity cannot corrupt control frames.
    pub(crate) fn spawn_native<T: DeserializeOwned + Send + 'static>(
        &self,
        command: &mut Command,
        run_id: String,
        lease_id: u64,
        log_path: &Path,
    ) -> io::Result<ProcessWorker<T>> {
        self.spawn_inner(command, run_id, lease_id, Some(log_path))
    }

    fn spawn_inner<T: DeserializeOwned + Send + 'static>(
        &self,
        command: &mut Command,
        run_id: String,
        lease_id: u64,
        log_path: Option<&Path>,
    ) -> io::Result<ProcessWorker<T>> {
        self.live
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < self.limit).then(|| n + 1)
            })
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "all process residency slots are occupied",
                )
            })?;
        let permit = Permit(Arc::clone(&self.live));
        let listener = if let Some(log) = log_path {
            let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
            listener.set_nonblocking(true)?;
            command
                .env(
                    "FASTSECDEC_WORKER_CONTROL",
                    listener.local_addr()?.to_string(),
                )
                .stdin(match &self.residency_lock {
                    Some(file) => Stdio::from(file.try_clone()?),
                    None => Stdio::null(),
                })
                .stdout(File::create(log)?);
            Some(listener)
        } else {
            command
                .env_remove("FASTSECDEC_WORKER_CONTROL")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            None
        };
        let mut child = command.stderr(Stdio::inherit()).spawn()?;
        type Channels = (
            Box<dyn Write + Send>,
            Box<dyn Read + Send>,
            Option<TcpStream>,
        );
        let channels = (|| -> io::Result<Channels> {
            if let Some(listener) = listener {
                let deadline = Instant::now() + Duration::from_secs(10);
                loop {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            stream.set_nodelay(true)?;
                            return Ok((
                                Box::new(stream.try_clone()?),
                                Box::new(stream.try_clone()?),
                                Some(stream),
                            ));
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                        Err(error) => return Err(error),
                    }
                    if let Some(status) = child.try_wait()? {
                        return Err(io::Error::other(format!(
                            "worker exited before control handshake: {status}"
                        )));
                    }
                    if Instant::now() >= deadline {
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "worker control handshake timed out",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            Ok((
                Box::new(child.stdin.take().expect("piped worker stdin")),
                Box::new(child.stdout.take().expect("piped worker stdout")),
                None,
            ))
        })();
        let (input, output, control) = match channels {
            Ok(channels) => channels,
            Err(error) => {
                let _ = child.kill();
                if child.wait().is_err() {
                    std::mem::forget(permit);
                }
                return Err(error);
            }
        };
        let input = BufWriter::new(input);
        let (sender, receiver) = mpsc::sync_channel(2);
        let reader = match std::thread::Builder::new()
            .name("fastsecdec-worker-control".into())
            .spawn(move || {
                let mut output = BufReader::new(output);
                loop {
                    match protocol::read::<Envelope<T>>(&mut output) {
                        Ok(Some(message)) => {
                            if sender.send(Ok(message)).is_err() {
                                return;
                            }
                        }
                        Ok(None) => return,
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            return;
                        }
                    }
                }
            }) {
            Ok(reader) => reader,
            Err(error) => {
                let _ = child.kill();
                if child.wait().is_err() {
                    std::mem::forget(permit);
                }
                return Err(error);
            }
        };
        Ok(ProcessWorker {
            child,
            input: Some(input),
            control,
            receiver: Some(receiver),
            reader: Some(reader),
            permit: Some(permit),
            run_id,
            lease_id,
            disconnected: false,
        })
    }
}

struct Permit(Arc<AtomicUsize>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) struct ProcessWorker<T> {
    child: Child,
    input: Option<BufWriter<Box<dyn Write + Send>>>,
    control: Option<TcpStream>,
    receiver: Option<Receiver<io::Result<Envelope<T>>>>,
    reader: Option<JoinHandle<()>>,
    permit: Option<Permit>,
    run_id: String,
    lease_id: u64,
    disconnected: bool,
}

impl<T> ProcessWorker<T> {
    pub(crate) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn output_closed(&self) -> bool {
        self.disconnected
    }

    pub(crate) fn send(&mut self, payload: impl Serialize) -> io::Result<()> {
        let input = self
            .input
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "worker input is closed"))?;
        protocol::write(
            input,
            &Envelope {
                run_id: self.run_id.clone(),
                lease_id: self.lease_id,
                payload,
            },
        )
    }

    /// Polling never blocks terminal refresh or the coordinator's cancellation.
    pub(crate) fn poll(&mut self) -> io::Result<Option<T>> {
        if self.disconnected {
            return Ok(None);
        }
        let Some(receiver) = &self.receiver else {
            return Ok(None);
        };
        match receiver.try_recv() {
            Ok(Ok(message)) => {
                if message.run_id != self.run_id || message.lease_id != self.lease_id {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "stale worker run or lease identity",
                    ));
                }
                Ok(Some(message.payload))
            }
            Ok(Err(error)) => Err(error),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                self.disconnected = true;
                Ok(None)
            }
        }
    }

    pub(crate) fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.permit.take();
        }
        Ok(status)
    }

    /// EOF tells the child watchdog to exit even if its native job is busy.
    pub(crate) fn close_input(&mut self) {
        self.input.take();
        if let Some(control) = &self.control {
            let _ = control.shutdown(Shutdown::Write);
        }
    }

    /// Used on eviction/cancellation before a replacement can obtain this slot.
    pub(crate) fn terminate(&mut self) -> io::Result<ExitStatus> {
        self.close_input();
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
        }
        let status = self.child.wait()?;
        self.permit.take();
        Ok(status)
    }
}

impl<T> Drop for ProcessWorker<T> {
    fn drop(&mut self) {
        self.close_input();
        if let Some(control) = &self.control {
            let _ = control.shutdown(Shutdown::Both);
        }
        // Unblock the bounded reader before joining it.
        self.receiver.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        // Keep a permit permanently occupied on the unlikely unrecoverable
        // wait failure: allowing a replacement would violate the memory bound.
        if self.child.wait().is_err() {
            if let Some(permit) = self.permit.take() {
                std::mem::forget(permit);
            }
        } else {
            self.permit.take();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
