//! Bounded process dispatch. A scientific worker is reaped before slot reuse.
use super::{
    jobs::{Request, Response},
    journal::Journal,
};
use crate::{
    CliResult,
    display::Dashboard,
    generate::dispatch::{Progress, Worker},
    process::{
        ProcessPool, ProcessWorker,
        child::{WorkerCommand, WorkerEvent, build_identity},
    },
};
use fastsecdec::status::{GenerationSnapshot, GenerationStage};
use std::{
    collections::VecDeque,
    fs::File,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

struct Active {
    key: String,
    request: Request,
    process: ProcessWorker<WorkerEvent<GenerationSnapshot>>,
    ready: bool,
    finished: bool,
    started: Instant,
}

pub(super) struct Runner {
    pool: ProcessPool,
    run_id: String,
    next_lease: u64,
    workers: usize,
    started: Instant,
    source_charts: usize,
    formula_uses: usize,
}
impl Runner {
    pub fn new(workers: usize, run_id: String, residency_lock: &File) -> CliResult<Self> {
        Ok(Self {
            pool: ProcessPool::with_residency_lock(workers, residency_lock)?,
            run_id,
            next_lease: 0,
            workers,
            started: Instant::now(),
            source_charts: 0,
            formula_uses: 0,
        })
    }
    pub fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
    pub fn formula_uses(&mut self, count: usize) {
        self.formula_uses = count;
    }
    pub fn run(
        &mut self,
        journal: &mut Journal,
        jobs: Vec<(String, Request)>,
        stage: GenerationStage,
        label: &str,
        dashboard: &mut Dashboard,
    ) -> CliResult<Vec<Response>> {
        let total = jobs.len();
        let mut pending = VecDeque::new();
        let mut results = Vec::with_capacity(total);
        for (key, request) in jobs {
            let (_, completed) = journal.request(&key, request.clone())?;
            if let Some(response) = completed {
                results.push(response);
            } else {
                pending.push_back((key, request));
            }
        }
        let mut active = (0..self.workers)
            .map(|_| None)
            .collect::<Vec<Option<Active>>>();
        let mut status = GenerationSnapshot {
            stage,
            completed: results.len(),
            total: Some(total),
            sectors: self.source_charts,
            kernels: 0,
            elapsed_seconds: self.elapsed(),
            timings: Default::default(),
            coefficient_expansion: None,
            formula_preparation: None,
            detail: label.into(),
        };
        let mut worker_view = Progress {
            completed: results.len(),
            total,
            elapsed_seconds: 0.,
            workers: (0..self.workers)
                .map(|index| Worker {
                    index,
                    activity: "ready".into(),
                    busy: false,
                    completed: 0,
                    active_seconds: 0.,
                    busy_seconds: 0.,
                    started: None,
                })
                .collect(),
        };
        let result = (|| -> CliResult<Vec<Response>> {
            loop {
                if dashboard.cancelled() {
                    return Err(
                        "generation cancelled; durable sector receipts are retained for --resume"
                            .into(),
                    );
                }
                for (slot, active_slot) in active.iter_mut().enumerate() {
                    if active_slot.is_none()
                        && let Some((key, request)) = pending.pop_front()
                    {
                        let lease = self.next_lease;
                        self.next_lease = self
                            .next_lease
                            .checked_add(1)
                            .ok_or("generation worker lease counter exhausted")?;
                        let mut command = Command::new(crate::process::executable()?);
                        #[cfg(not(test))]
                        command
                            .arg("__generate-worker")
                            .arg("--run-id")
                            .arg(&self.run_id)
                            .arg("--lease-id")
                            .arg(lease.to_string());
                        #[cfg(test)]
                        command
                            .args([
                                "--exact",
                                "generate::serial::family_tests::native_child_entry",
                                "--ignored",
                                "--nocapture",
                            ])
                            .env("FASTSECDEC_GENERATION_TEST_RUN", &self.run_id)
                            .env("FASTSECDEC_GENERATION_TEST_LEASE", lease.to_string());
                        let log = journal.root.join(format!("worker-{lease}.log"));
                        let process = self.pool.spawn_native(
                            &mut command,
                            self.run_id.clone(),
                            lease,
                            &log,
                        )?;
                        worker_view.workers[slot].activity =
                            format!("{key} · starting process {}", process.pid());
                        worker_view.workers[slot].busy = true;
                        *active_slot = Some(Active {
                            key,
                            request,
                            process,
                            ready: false,
                            finished: false,
                            started: Instant::now(),
                        });
                    }
                    let Some(worker) = active_slot.as_mut() else {
                        continue;
                    };
                    for _ in 0..64 {
                        match worker.process.poll()? {
                            Some(WorkerEvent::Ready { pid, build }) => {
                                if worker.ready
                                    || pid != worker.process.pid()
                                    || build != build_identity()
                                {
                                    return Err(
                                        "generation worker handshake identity mismatch".into()
                                    );
                                }
                                worker.ready = true;
                                worker
                                    .process
                                    .send(WorkerCommand::Execute(journal.job_path(&worker.key)))?;
                            }
                            Some(WorkerEvent::Update(update)) => {
                                if !worker.ready || worker.finished {
                                    return Err(
                                        "generation worker emitted progress outside its lease"
                                            .into(),
                                    );
                                }
                                worker_view.workers[slot].activity =
                                    format!("{} · {}", worker.key, update.detail);
                                status.coefficient_expansion = update.coefficient_expansion;
                                status.formula_preparation = update.formula_preparation;
                            }
                            Some(WorkerEvent::Finished) => {
                                if !worker.ready || worker.finished {
                                    return Err(
                                        "duplicate or unissued generation completion".into()
                                    );
                                }
                                worker.finished = true;
                                worker.process.send(WorkerCommand::<PathBuf>::Shutdown)?;
                            }
                            Some(WorkerEvent::Failed { message }) => {
                                return Err(format!(
                                    "generation job {} failed: {message}",
                                    worker.key
                                )
                                .into());
                            }
                            None => break,
                        }
                    }
                    if let Some(exit) = worker.process.try_wait()? {
                        if !exit.success() || (!worker.finished && worker.process.output_closed()) {
                            return Err(format!(
                                "generation job {} exited without a successful receipt ({exit})",
                                worker.key
                            )
                            .into());
                        }
                        if !worker.finished {
                            continue;
                        }
                        let response = journal.accept(&worker.key, &worker.request)?;
                        worker_view.workers[slot].busy = false;
                        worker_view.workers[slot].completed += 1;
                        worker_view.workers[slot].busy_seconds +=
                            worker.started.elapsed().as_secs_f64();
                        worker_view.workers[slot].activity = "completed and released".into();
                        results.push(response);
                        *active_slot = None;
                    } else {
                        worker_view.workers[slot].active_seconds =
                            worker.started.elapsed().as_secs_f64();
                    }
                }
                for result in &results {
                    match result {
                        Response::Prepared(prepared) => {
                            self.source_charts = prepared.native.charts.len();
                        }
                        Response::PreparedPrograms(prepared) => {
                            self.source_charts = prepared
                                .native
                                .recipes
                                .first()
                                .map_or(0, |recipe| recipe.charts.len());
                        }
                        _ => {}
                    }
                }
                status.sectors = self.source_charts;
                status.kernels = results
                    .iter()
                    .filter_map(|r| match r {
                        Response::Compiled(c) => {
                            Some(c.receipts.iter().filter(|r| r.dimension.is_some()).count())
                        }
                        _ => None,
                    })
                    .sum();
                status.completed = results.len();
                if stage == GenerationStage::FormulaPreparation {
                    status.formula_preparation =
                        Some(fastsecdec::status::FormulaPreparationSnapshot {
                            completed: results.len(),
                            total,
                            sectors: self.formula_uses,
                            reused: self.formula_uses.saturating_sub(total),
                        });
                }
                status.elapsed_seconds = self.elapsed();
                status.detail = format!(
                    "{label} · {} completed · {} processes resident · {} queued",
                    results.len(),
                    self.pool.live(),
                    pending.len()
                );
                worker_view.completed = results.len();
                worker_view.elapsed_seconds = self.elapsed();
                dashboard.worker_processes(
                    &active
                        .iter()
                        .filter_map(|w| w.as_ref().map(|w| w.process.pid()))
                        .collect::<Vec<_>>(),
                );
                dashboard.generation_workers(&worker_view);
                dashboard.generation(&status)?;
                if pending.is_empty() && active.iter().all(Option::is_none) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Ok(results)
        })();
        drop(active);
        dashboard.worker_processes(&[]);
        dashboard.generation_coordinator();
        result
    }
}
