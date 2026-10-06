//! One caller-owned wave of complete native global batches. Worker callbacks
//! expose observational progress only; incomplete batches are never submitted.
use super::Slot;
use crate::{CliResult, display::IntegrationWorkerActivity, driver::replay::AcceptedReplay};
use fastsecdec::{
    integration::{
        IntegrationError,
        mc_discrete::{HavanaDiscreteReturn, HavanaDiscreteTask},
    },
    kernel::{KernelSet, ReplayState},
    status::EvaluationDiagnostics,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
};

pub(super) struct Completed {
    pub result: Result<HavanaDiscreteReturn, IntegrationError>,
    pub diagnostics: EvaluationDiagnostics,
    pub states: Option<Vec<(usize, ReplayState)>>,
    pub aborted_prefix: bool,
}

struct Progress {
    batch: u32,
    planned: u64,
    completed: AtomicU64,
    sector: AtomicU64,
    preparing: AtomicBool,
}
impl Progress {
    fn snapshot(&self, worker: usize) -> IntegrationWorkerActivity {
        let sector = self.sector.load(Ordering::Relaxed);
        IntegrationWorkerActivity {
            worker,
            batch: self.batch,
            completed_points: self.completed.load(Ordering::Relaxed),
            planned_points: self.planned,
            sector: (sector != u64::MAX).then_some(sector),
            preparing_context: self.preparing.load(Ordering::Relaxed),
        }
    }
}

struct StopOnDrop<'a>(&'a AtomicBool);
impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

pub(super) fn run(
    pool: &rayon::ThreadPool,
    slots: &mut [Slot],
    tasks: Vec<HavanaDiscreteTask>,
    kernels: &KernelSet,
    replay: &AcceptedReplay,
    mut poll: impl FnMut(Vec<IntegrationWorkerActivity>) -> CliResult<bool>,
) -> CliResult<Vec<Result<Completed, String>>> {
    if slots.len() < tasks.len() {
        return Err("discrete MC wave has more batches than worker slots".into());
    }
    let progress = tasks
        .iter()
        .map(|task| Progress {
            batch: task.batch(),
            planned: task.point_count() as u64,
            completed: AtomicU64::new(0),
            sector: AtomicU64::new(u64::MAX),
            preparing: AtomicBool::new(true),
        })
        .collect::<Vec<_>>();
    let stop = AtomicBool::new(false);
    pool.in_place_scope(|scope| {
        let _stop_on_exit = StopOnDrop(&stop);
        let (send, receive) = mpsc::channel();
        // Keep original wave/slot assignment and admission order: neither
        // completion timing nor status polling changes native RNG batches.
        let mut returned = (0..tasks.len()).map(|_| None).collect::<Vec<_>>();
        if poll(
            progress
                .iter()
                .enumerate()
                .map(|(id, p)| p.snapshot(id))
                .collect(),
        )? {
            return Ok(Vec::new());
        }
        for (index, (slot, task)) in slots.iter_mut().zip(tasks).enumerate() {
            let send = send.clone();
            let stop = &stop;
            let progress = &progress[index];
            scope.spawn(move |_| {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    evaluate(slot, task, kernels, replay, stop, progress)
                }))
                .map_err(|payload| {
                    payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                        .unwrap_or_else(|| "unknown panic payload".into())
                });
                // Notification survives a native panic, so the coordinator
                // cannot wait forever for a lost completion.
                let _ = send.send((index, result));
            });
        }
        let mut outstanding = returned.len();
        while outstanding != 0 {
            // Always poll, even while draining cancelled tasks. A repeated
            // interrupt remains available to the terminal lifecycle owner.
            if poll(
                progress
                    .iter()
                    .enumerate()
                    .map(|(id, p)| p.snapshot(id))
                    .collect(),
            )? {
                stop.store(true, Ordering::Relaxed);
            }
            match receive.recv_timeout(Duration::from_millis(50)) {
                Ok((index, result)) => {
                    if !result
                        .as_ref()
                        .is_ok_and(|completed| completed.result.is_ok() || completed.aborted_prefix)
                    {
                        stop.store(true, Ordering::Relaxed);
                    }
                    returned[index] = Some(result);
                    outstanding -= 1;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("discrete MC worker return channel disconnected".into());
                }
            }
        }
        Ok(returned
            .into_iter()
            .map(|value| value.expect("one completion per submitted batch"))
            .collect())
    })
}

fn evaluate(
    slot: &mut Slot,
    task: HavanaDiscreteTask,
    kernels: &KernelSet,
    replay: &AcceptedReplay,
    stop: &AtomicBool,
    progress: &Progress,
) -> Completed {
    let mut diagnostics = EvaluationDiagnostics::default();
    let mut aborted_prefix = false;
    let result = slot
        .worker
        .evaluate_weighted(task, |id, point, weight, output| {
            if stop.load(Ordering::Relaxed) {
                aborted_prefix = true;
                return Err("discrete MC batch stopped by caller".to_owned());
            }
            progress.sector.store(id, Ordering::Relaxed);
            if let std::collections::btree_map::Entry::Vacant(entry) = slot.contexts.entry(id) {
                progress.preparing.store(true, Ordering::Relaxed);
                entry.insert(
                    replay
                        .context(kernels, id)
                        .map_err(|error| error.to_string())?,
                );
            }
            progress.preparing.store(false, Ordering::Relaxed);
            if stop.load(Ordering::Relaxed) {
                aborted_prefix = true;
                return Err("discrete MC batch stopped by caller".to_owned());
            }
            super::super::evaluate_tracked(
                slot.contexts.get_mut(&id).expect("native selected sector"),
                point,
                weight,
                output,
                &mut diagnostics,
            )?;
            progress.completed.fetch_add(1, Ordering::Relaxed);
            Ok::<(), String>(())
        });
    let states = result.as_ref().ok().map(|_| {
        slot.contexts
            .iter()
            .map(|(id, context)| (*id as usize, context.state().clone()))
            .collect()
    });
    Completed {
        result,
        diagnostics,
        states,
        aborted_prefix,
    }
}
