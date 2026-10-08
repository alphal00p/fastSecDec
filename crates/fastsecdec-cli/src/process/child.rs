//! Child control loop. An EOF watchdog stops native work if its caller dies.
use super::protocol::{self, Envelope};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::mpsc,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum WorkerCommand<T> {
    Execute(T),
    Shutdown,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum WorkerEvent<T> {
    Ready { pid: u32, build: String },
    Update(T),
    Finished,
    Failed { message: String },
}

pub(crate) fn build_identity() -> String {
    let mut hash = blake3::Hasher::new();
    for part in [
        env!("CARGO_PKG_VERSION"),
        env!("FASTSECDEC_SOURCE_IDENTITY"),
        env!("FASTSECDEC_SYMBOLICA_REVISION"),
        env!("FASTSECDEC_SYMBOLICA_STATE"),
        env!("FASTSECDEC_NUMERICA_REVISION"),
        env!("FASTSECDEC_NUMERICA_STATE"),
        env!("FASTSECDEC_FEYNKIT_REVISION"),
        env!("FASTSECDEC_FEYNKIT_STATE"),
    ] {
        hash.update(&(part.len() as u64).to_le_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().to_hex().to_string()
}

pub(crate) fn serve<I: DeserializeOwned + Send + 'static, O: Serialize>(
    run_id: String,
    lease_id: u64,
    mut execute: impl FnMut(I, &mut dyn FnMut(O) -> io::Result<()>) -> io::Result<()>,
) -> io::Result<()> {
    let (input, output): (Box<dyn Read + Send>, Box<dyn Write>) =
        if let Some(address) = std::env::var_os("FASTSECDEC_WORKER_CONTROL") {
            let address: SocketAddr = address
                .to_string_lossy()
                .parse()
                .map_err(io::Error::other)?;
            if !address.ip().is_loopback() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "worker control must be local",
                ));
            }
            let stream = TcpStream::connect(address)?;
            stream.set_nodelay(true)?;
            (Box::new(stream.try_clone()?), Box::new(stream))
        } else {
            (Box::new(io::stdin()), Box::new(io::stdout()))
        };
    let (send, receive) = mpsc::sync_channel(1);
    let expected_run = run_id.clone();
    std::thread::Builder::new()
        .name("fastsecdec-parent-watchdog".into())
        .spawn(move || {
            let mut input = BufReader::new(input);
            loop {
                let request = protocol::read::<Envelope<WorkerCommand<I>>>(&mut input);
                match request {
                    Ok(Some(request))
                        if request.run_id == expected_run && request.lease_id == lease_id =>
                    {
                        if send.try_send(request.payload).is_err() {
                            eprintln!(
                                "fastsecdec worker: control queue exceeded its bounded capacity"
                            );
                            std::process::exit(1);
                        }
                    }
                    Ok(None) => std::process::exit(1),
                    Ok(Some(_)) => {
                        eprintln!("fastsecdec worker: stale coordinator run or lease");
                        std::process::exit(1);
                    }
                    Err(error) => {
                        eprintln!("fastsecdec worker control: {error}");
                        std::process::exit(1);
                    }
                }
            }
        })?;
    let mut output = BufWriter::new(output);
    let mut emit = |payload| {
        protocol::write(
            &mut output,
            &Envelope {
                run_id: run_id.clone(),
                lease_id,
                payload,
            },
        )
    };
    emit(WorkerEvent::Ready {
        pid: std::process::id(),
        build: build_identity(),
    })?;
    while let Ok(command) = receive.recv() {
        match command {
            WorkerCommand::Execute(job) => {
                if let Err(error) = execute(job, &mut |event| emit(WorkerEvent::Update(event))) {
                    let _ = emit(WorkerEvent::Failed {
                        message: error.to_string(),
                    });
                    return Err(error);
                }
                emit(WorkerEvent::Finished)?;
            }
            WorkerCommand::Shutdown => return Ok(()),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "coordinator control stopped",
    ))
}
