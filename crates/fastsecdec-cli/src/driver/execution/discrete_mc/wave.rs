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
    time::{Duration, Instant},
};

pub(super) struct Completed {
    pub batch: u32,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    pool: &rayon::ThreadPool,
    slots: &mut [Slot],
    tasks: Vec<HavanaDiscreteTask>,
    kernels: &KernelSet,
    replay: &AcceptedReplay,
    frozen_replay: &AcceptedReplay,
    mut poll: impl FnMut(
        Vec<IntegrationWorkerActivity>,
        &dyn Fn() -> Vec<fastsecdec::integration::McLiveBatch>,
        bool,
    ) -> CliResult<bool>,
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
    let meters = slots
        .iter()
        .map(|slot| slot.meter.clone())
        .collect::<Vec<_>>();
    for meter in &meters {
        meter.clear_live();
    }
    // Native moment vectors are cloned only when the coordinator requests an
    // observation, or once when this wave finishes before slots are reused.
    let collect_live = || meters.iter().filter_map(|meter| meter.live()).collect();
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
            &collect_live,
            false,
        )? {
            return Ok(Vec::new());
        }
        for (index, (slot, task)) in slots.iter_mut().zip(tasks).enumerate() {
            let send = send.clone();
            let stop = &stop;
            let progress = &progress[index];
            scope.spawn(move |_| {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    evaluate(slot, task, kernels, replay, frozen_replay, stop, progress)
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
                &collect_live,
                false,
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
        poll(
            progress
                .iter()
                .enumerate()
                .map(|(id, p)| p.snapshot(id))
                .collect(),
            &collect_live,
            true,
        )?;
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
    frozen_replay: &AcceptedReplay,
    stop: &AtomicBool,
    progress: &Progress,
) -> Completed {
    let _span = slot.meter.task(None);
    let batch = task.batch();
    let planned = task.point_count() as u64;
    let mut last_publish = Instant::now();
    let publication_interval = slot.meter.publication_interval();
    let mut diagnostics = EvaluationDiagnostics::default();
    let mut aborted_prefix = false;
    let result = slot.worker.evaluate_weighted_observed(
        task,
        |id, point, weight, output| {
            if stop.load(Ordering::Relaxed) {
                aborted_prefix = true;
                return Err("discrete MC batch stopped by caller".to_owned());
            }
            progress.sector.store(id, Ordering::Relaxed);
            if let std::collections::btree_map::Entry::Vacant(entry) = slot.contexts.entry(id) {
                let prepare_started = Instant::now();
                progress.preparing.store(true, Ordering::Relaxed);
                let mut context = replay
                    .context(kernels, id)
                    .map_err(|error| error.to_string())?;
                context
                    .set_reference_state(frozen_replay.state(id as usize))
                    .map_err(|e| e.to_string())?;
                entry.insert(context);
                slot.meter
                    .preparation(id, prepare_started.elapsed().as_secs_f64());
            }
            progress.preparing.store(false, Ordering::Relaxed);
            if stop.load(Ordering::Relaxed) {
                aborted_prefix = true;
                return Err("discrete MC batch stopped by caller".to_owned());
            }
            super::super::evaluate_observed(
                slot.contexts.get_mut(&id).expect("native selected sector"),
                id,
                point,
                weight,
                output,
                &mut diagnostics,
                &slot.meter,
            )?;
            progress.completed.fetch_add(1, Ordering::Relaxed);
            Ok::<(), String>(())
        },
        |view| {
            if let Some(timing) = view.last_point_timing() {
                slot.meter.add_sector_times(std::iter::once(timing));
            }
            if view.points() == planned || last_publish.elapsed() >= publication_interval {
                slot.meter.publish(view.snapshot());
                last_publish = Instant::now();
            }
        },
    );
    if result.is_err() {
        slot.meter.clear_live();
    }
    let states = result.as_ref().ok().map(|_| {
        slot.contexts
            .iter()
            .map(|(id, context)| (*id as usize, context.state().clone()))
            .collect()
    });
    Completed {
        batch,
        result,
        diagnostics,
        states,
        aborted_prefix,
    }
}
