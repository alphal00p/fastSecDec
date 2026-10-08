//! Recyclable setup work: native global state never survives into serial sampling.
use crate::{
    CliResult,
    artifact::Artifact,
    config::IntegrationInput,
    display::Dashboard,
    process::{
        ProcessPool,
        child::{self, WorkerCommand, WorkerEvent},
    },
};
use fastsecdec::{
    integration::SectorSpec,
    kernel::{KernelLoadOptions, indexed::IndexedReader},
    results::KernelResultManifest,
    status::GenerationSnapshot,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Serialize, Deserialize)]
enum Request {
    Exact {
        artifact: PathBuf,
        expected: String,
        settings: Box<IntegrationInput>,
        validate: bool,
        result: PathBuf,
    },
    Generate {
        input: PathBuf,
        output: PathBuf,
        workers: usize,
        reference: Option<PathBuf>,
    },
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Progress {
    pub snapshot: GenerationSnapshot,
    pub workers: Option<crate::generate::dispatch::Progress>,
}

pub(crate) fn exact(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
    options: KernelLoadOptions,
    dashboard: &mut Dashboard,
) -> CliResult<KernelResultManifest> {
    let staging = tempfile::tempdir()?;
    let result = staging.path().join("manifest.json");
    run(
        Request::Exact {
            artifact: path.into(),
            expected: artifact.kernel_content_id.clone(),
            settings: Box::new(settings.clone()),
            validate: options.validate,
            result: result.clone(),
        },
        dashboard,
    )?;
    let manifest: KernelResultManifest = serde_json::from_reader(File::open(result)?)?;
    manifest.validate()?;
    Ok(manifest)
}
pub(crate) fn generate(
    input: &Path,
    output: &Path,
    workers: usize,
    reference: Option<&Path>,
    dashboard: &mut Dashboard,
) -> CliResult<()> {
    run(
        Request::Generate {
            input: input.into(),
            output: output.into(),
            workers,
            reference: reference.map(Path::to_owned),
        },
        dashboard,
    )
}

fn run(request: Request, dashboard: &mut Dashboard) -> CliResult<()> {
    let staging = tempfile::tempdir()?;
    let path = staging.path().join("request.json");
    crate::artifact::atomic_write(&path, &serde_json::to_vec(&request)?)?;
    let run = format!("setup-{}-{:?}", std::process::id(), Instant::now());
    let pool = ProcessPool::new(1)?;
    let mut command = Command::new(crate::process::executable()?);
    command.args(["__setup-worker", "--run-id", &run, "--lease-id", "0"]);
    let mut worker = pool.spawn_native::<WorkerEvent<Progress>>(
        &mut command,
        run,
        0,
        &staging.path().join("worker.log"),
    )?;
    dashboard.worker_processes(&[worker.pid()]);
    let outcome = (|| {
        let mut ready = false;
        let mut finished = false;
        loop {
            if dashboard.cancelled() {
                return Err("setup cancelled".into());
            }
            while let Some(event) = worker.poll()? {
                match event {
                    WorkerEvent::Ready { pid, build } => {
                        if ready || pid != worker.pid() || build != child::build_identity() {
                            return Err("setup worker identity differs".into());
                        }
                        ready = true;
                        worker.send(WorkerCommand::Execute(path.clone()))?;
                    }
                    WorkerEvent::Update(progress) => {
                        if !ready || finished {
                            return Err("unissued setup progress".into());
                        }
                        if let Some(workers) = progress.workers {
                            dashboard.generation_workers(&workers);
                        }
                        dashboard.generation(&progress.snapshot)?;
                    }
                    WorkerEvent::Finished => {
                        if !ready || finished {
                            return Err("duplicate setup completion".into());
                        }
                        finished = true;
                        worker.send(WorkerCommand::<PathBuf>::Shutdown)?;
                    }
                    WorkerEvent::Failed { message } => return Err(message.into()),
                }
            }
            if let Some(exit) = worker.try_wait()? {
                if !exit.success() {
                    return Err(format!("setup worker exited: {exit}").into());
                }
                if finished {
                    return Ok(());
                }
                if worker.output_closed() {
                    return Err("setup worker exited without a receipt".into());
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    // Reap before either returning its compact result or starting sampling.
    worker.terminate()?;
    dashboard.worker_processes(&[]);
    outcome
}

pub(crate) fn worker(run: String, lease: u64) -> CliResult<()> {
    child::serve::<PathBuf, Progress>(run, lease, |path, emit| {
        execute(path, emit).map_err(|error| std::io::Error::other(error.to_string()))
    })?;
    Ok(())
}
fn execute(path: PathBuf, emit: &mut dyn FnMut(Progress) -> std::io::Result<()>) -> CliResult<()> {
    match serde_json::from_reader(File::open(path)?)? {
        Request::Exact {
            artifact: path,
            expected,
            settings,
            validate,
            result,
        } => {
            let artifact =
                Artifact::load_metadata_with_options(&path, KernelLoadOptions { validate })?;
            let catalogue = artifact
                .catalogue()
                .ok_or("setup requires an indexed artifact")?;
            if catalogue.content_id != expected {
                return Err("artifact changed during exact setup".into());
            }
            let mut reader = IndexedReader::from_reader(
                File::open(artifact.data_path(&path)?)?,
                KernelLoadOptions { validate },
            )?;
            if reader.catalogue().content_id != expected {
                return Err("indexed data changed during exact setup".into());
            }
            let mut exact = reader.load_exact()?;
            crate::bind_parameters(&mut exact, &settings)?;
            let manifest = KernelResultManifest {
                kernel_content_id: exact.content_id().into(),
                orders: catalogue.orders.clone(),
                components: catalogue.components.clone(),
                exact_coefficients: exact.exact_coefficients().to_vec(),
                sectors: catalogue
                    .records
                    .iter()
                    .filter_map(|record| {
                        record.sector.map(|id| SectorSpec {
                            id: id as u64,
                            dimension: record
                                .receipt
                                .dimension
                                .expect("validated numerical record"),
                        })
                    })
                    .collect(),
            };
            manifest.validate()?;
            crate::artifact::atomic_write(&result, &serde_json::to_vec(&manifest)?)?;
            Ok(())
        }
        Request::Generate {
            input,
            output,
            workers,
            reference,
        } => {
            let (send, receive) = std::sync::mpsc::sync_channel(2);
            std::thread::scope(|scope| {
                let job = scope.spawn(move || -> Result<(), String> {
                    let work = || -> CliResult<()> {
                        let mut dashboard = Dashboard::new(false, false)?;
                        dashboard.forward_generation(send);
                        let reference = crate::reference::from_card(&input, reference.as_deref())?;
                        crate::generate::generate_with_workers(
                            &input,
                            &output,
                            &mut dashboard,
                            reference.as_ref(),
                            workers,
                        )?;
                        Ok(())
                    };
                    work().map_err(|error| error.to_string())
                });
                let mut error = None;
                for progress in receive {
                    if let Err(failure) = emit(progress) {
                        error = Some(failure);
                        break;
                    }
                }
                // A failed control receiver disconnects the bounded producer.
                let generated = job.join().map_err(|_| "isolated generation panicked")?;
                if let Some(error) = error {
                    return Err(error.into());
                }
                generated.map_err(Into::into)
            })
        }
    }
}
