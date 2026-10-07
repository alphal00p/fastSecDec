//! Responsive caller-owned ordinary Havana waves; only whole returns admit.
use super::McSlot;
use crate::{CliResult, display::IntegrationWorkerActivity, driver::replay::AcceptedReplay};
use fastsecdec::{
    integration::{
        IntegrationError, McLiveBatch,
        mc::{HavanaReturn, HavanaSession, HavanaTask},
    },
    kernel::{KernelSet, ReplayState},
    status::EvaluationDiagnostics,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

pub(super) struct Completed {
    pub sector: u64,
    pub batch: u32,
    pub result: Result<HavanaReturn, IntegrationError>,
    pub diagnostics: EvaluationDiagnostics,
    pub state: Option<ReplayState>,
    pub aborted: bool,
}
struct Stop<'a>(&'a AtomicBool);
impl Drop for Stop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    pool: &rayon::ThreadPool,
    slots: &mut [McSlot],
    tasks: Vec<HavanaTask>,
    session: &HavanaSession,
    kernels: &KernelSet,
    replay: &AcceptedReplay,
    frozen: &AcceptedReplay,
    mut poll: impl FnMut(
        Vec<IntegrationWorkerActivity>,
        &dyn Fn() -> Vec<McLiveBatch>,
        bool,
    ) -> CliResult<bool>,
) -> CliResult<Vec<Result<Completed, String>>> {
    if slots.len() < tasks.len() {
        return Err("ordinary MC wave exceeds worker slots".into());
    }
    let activity = tasks
        .iter()
        .enumerate()
        .map(|(worker, t)| {
            std::sync::Mutex::new(IntegrationWorkerActivity {
                worker,
                batch: t.batch(),
                completed_points: 0,
                planned_points: t.point_count() as u64,
                sector: Some(t.sector_id()),
                preparing_context: true,
            })
        })
        .collect::<Vec<_>>();
    let meters = slots.iter().map(|s| s.meter.clone()).collect::<Vec<_>>();
    for meter in &meters {
        meter.clear_live();
    }
    let collect_live = || meters.iter().filter_map(|meter| meter.live()).collect();
    let stop = AtomicBool::new(false);
    pool.in_place_scope(|scope| {
        let _guard = Stop(&stop);
        let (send, receive) = mpsc::channel();
        let mut returned = (0..tasks.len()).map(|_| None).collect::<Vec<_>>();
        for (index, (slot, task)) in slots.iter_mut().zip(tasks).enumerate() {
            let send = send.clone();
            let stop = &stop;
            let activity = &activity[index];
            scope.spawn(move |_| {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    evaluate(slot, task, session, kernels, replay, frozen, stop, activity)
                }))
                .map_err(|p| {
                    p.downcast_ref::<String>()
                        .cloned()
                        .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                        .unwrap_or_else(|| "unknown panic payload".into())
                });
                let _ = send.send((index, result));
            });
        }
        let mut outstanding = returned.len();
        while outstanding != 0 {
            if poll(
                activity
                    .iter()
                    .map(|a| a.lock().unwrap_or_else(|e| e.into_inner()).clone())
                    .collect(),
                &collect_live,
                false,
            )? {
                stop.store(true, Ordering::Relaxed);
            }
            match receive.recv_timeout(Duration::from_millis(50)) {
                Ok((index, result)) => {
                    if !result.as_ref().is_ok_and(|c| c.result.is_ok() || c.aborted) {
                        stop.store(true, Ordering::Relaxed);
                    }
                    returned[index] = Some(result);
                    outstanding -= 1;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("ordinary MC return channel disconnected".into());
                }
            }
        }
        poll(
            activity
                .iter()
                .map(|a| a.lock().unwrap_or_else(|e| e.into_inner()).clone())
                .collect(),
            &collect_live,
            true,
        )?;
        Ok(returned
            .into_iter()
            .map(|r| r.expect("each native job returns once"))
            .collect())
    })
}
#[allow(clippy::too_many_arguments)]
fn evaluate(
    slot: &mut McSlot,
    task: HavanaTask,
    session: &HavanaSession,
    kernels: &KernelSet,
    replay: &AcceptedReplay,
    frozen: &AcceptedReplay,
    stop: &AtomicBool,
    activity: &std::sync::Mutex<IntegrationWorkerActivity>,
) -> Completed {
    let id = task.sector_id();
    let batch = task.batch();
    let planned = task.point_count() as u64;
    let _span = slot.meter.task(Some(id));
    let mut diagnostics = EvaluationDiagnostics::default();
    let mut aborted = false;
    let mut last_publish = Instant::now();
    let publication_interval = slot.meter.publication_interval();
    let result = (|| {
        if stop.load(Ordering::Relaxed) {
            aborted = true;
            return Err(IntegrationError::Unavailable(
                "batch cancelled by caller".into(),
            ));
        }
        let prepare_started = Instant::now();
        if let std::collections::btree_map::Entry::Vacant(entry) = slot.contexts.entry(id) {
            entry.insert(
                replay
                    .context(kernels, id)
                    .map_err(|e| IntegrationError::Evaluation(e.to_string()))?,
            );
        }
        let context = slot.contexts.get_mut(&id).unwrap();
        context
            .merge_state(replay.state(id as usize))
            .map_err(|e| IntegrationError::Evaluation(e.to_string()))?;
        context
            .set_reference_state(frozen.state(id as usize))
            .map_err(|e| IntegrationError::Evaluation(e.to_string()))?;
        slot.meter
            .preparation(id, prepare_started.elapsed().as_secs_f64());
        if let std::collections::btree_map::Entry::Vacant(entry) = slot.workers.entry(id) {
            entry.insert(session.worker_context(id)?);
        }
        activity
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .preparing_context = false;
        slot.workers
            .get_mut(&id)
            .unwrap()
            .evaluate_weighted_observed(
                task,
                |point, weight, out| {
                    if stop.load(Ordering::Relaxed) {
                        aborted = true;
                        return Err("batch cancelled by caller".to_owned());
                    }
                    super::super::evaluate_observed(
                        context,
                        id,
                        point,
                        weight,
                        out,
                        &mut diagnostics,
                        &slot.meter,
                    )
                },
                |view| {
                    activity
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .completed_points = view.points();
                    if view.points() == planned || last_publish.elapsed() >= publication_interval {
                        slot.meter.publish(view.snapshot());
                        last_publish = Instant::now();
                    }
                },
            )
    })();
    if result.is_err() {
        slot.meter.clear_live();
    }
    let state = result
        .as_ref()
        .ok()
        .map(|_| slot.contexts[&id].state().clone());
    Completed {
        sector: id,
        batch,
        result,
        diagnostics,
        state,
        aborted,
    }
}
