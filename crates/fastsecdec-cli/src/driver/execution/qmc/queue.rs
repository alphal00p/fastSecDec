//! Caller-owned refill loop. Only this coordinator reserves/submits native
//! packages and accepts replay state; jobs own at most one sector context each.
use super::slot::QmcSlot;
use crate::{CliResult, driver::replay::AcceptedReplay};
use fastsecdec::{
    integration::{IntegrationError, QmcReturn, QmcSession, QmcTask},
    kernel::{KernelSet, ReplayState},
    status::EvaluationDiagnostics,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
};

#[derive(Default)]
pub(super) struct Outcome {
    pub(super) cancelled: bool,
    pub(super) failure: Option<String>,
}

impl Outcome {
    fn stopped(&self) -> bool {
        self.cancelled || self.failure.is_some()
    }

    fn merge(&mut self, other: Self) {
        self.cancelled |= other.cancelled;
        if self.failure.is_none() {
            self.failure = other.failure;
        }
    }
}

pub(super) struct Phase<'a> {
    pub(super) pool: &'a rayon::ThreadPool,
    pub(super) kernels: &'a KernelSet,
    pub(super) session: &'a mut QmcSession,
    pub(super) workers: usize,
    pub(super) diagnostics: &'a mut EvaluationDiagnostics,
    pub(super) replay: &'a mut AcceptedReplay,
    pub(super) operations: &'a super::super::observations::Operations,
}

struct Completed {
    slot: QmcSlot,
    sector: usize,
    result: Result<QmcReturn, IntegrationError>,
    diagnostics: EvaluationDiagnostics,
    state: Option<ReplayState>,
    aborted_prefix: bool,
}

#[cfg(test)]
fn evaluate(slot: QmcSlot, task: QmcTask, stop: &AtomicBool) -> Completed {
    evaluate_batch(slot, task, stop, 1)
}

fn evaluate_batch(
    mut slot: QmcSlot,
    task: QmcTask,
    stop: &AtomicBool,
    batch_size: usize,
) -> Completed {
    let sector = task.sector_id() as usize;
    let _span = slot.meter.task(Some(sector as u64));
    slot.meter.activity(
        u32::try_from(task.work().start()).unwrap_or(u32::MAX),
        task.point_count(),
        sector as u64,
    );
    let active = slot.active.as_mut().unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    let mut aborted_prefix = false;
    let result =
        active
            .worker
            .evaluate_weighted_batch(task, batch_size, |points, weights, output| {
                if stop.load(Ordering::Relaxed) {
                    aborted_prefix = true;
                    return Err("QMC package stopped by caller".to_owned());
                }
                super::super::evaluate_batch_observed(
                    &mut active.context,
                    sector as u64,
                    points,
                    weights,
                    output,
                    &mut diagnostics,
                    &slot.meter,
                    stop,
                    &mut aborted_prefix,
                )?;
                slot.meter.points_completed(weights.len() as u64);
                Ok::<(), String>(())
            });
    let state = result.as_ref().ok().map(|_| active.context.state().clone());
    Completed {
        slot,
        sector,
        result,
        diagnostics,
        state,
        aborted_prefix,
    }
}

// Also stop jobs if a coordinator I/O error returns early from the scope body.
// Scoped Rayon still joins all jobs before borrowed kernel data can be dropped.
struct StopOnDrop<'a>(&'a AtomicBool);
impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

impl Phase<'_> {
    pub(super) fn run_batched(
        self,
        batch_size: usize,
        poll: impl FnMut(
            &QmcSession,
            &EvaluationDiagnostics,
            &AcceptedReplay,
            bool,
        ) -> CliResult<Outcome>,
    ) -> CliResult<Outcome> {
        self.run_with(poll, &|slot, task, stop| {
            evaluate_batch(slot, task, stop, batch_size)
        })
    }

    // The private execution seam lets tests force completion/failure order
    // without a second scheduler or a timing-dependent sleep assertion.
    fn run_with(
        self,
        mut poll: impl FnMut(
            &QmcSession,
            &EvaluationDiagnostics,
            &AcceptedReplay,
            bool,
        ) -> CliResult<Outcome>,
        execute: &(impl Fn(QmcSlot, QmcTask, &AtomicBool) -> Completed + Sync),
    ) -> CliResult<Outcome> {
        let stop = AtomicBool::new(false);
        let frozen = self.replay.clone();
        self.pool.in_place_scope(|scope| {
            let _stop_on_exit = StopOnDrop(&stop);
            let (send, receive) = mpsc::channel();
            let mut idle: Vec<_> = (0..self.workers)
                .map(|id| QmcSlot {
                    meter: self.operations.worker(id),
                    ..Default::default()
                })
                .collect();
            let mut in_flight = 0;
            let mut outcome = Outcome::default();
            loop {
                outcome.merge(poll(
                    self.session,
                    self.diagnostics,
                    self.replay,
                    outcome.stopped() || self.session.is_complete(),
                )?);
                if outcome.stopped() {
                    stop.store(true, Ordering::Relaxed);
                } else {
                    while let Some(mut slot) = idle.pop() {
                        let schedule_span = self.operations.coordinator(false);
                        let task = match self.session.next_work() {
                            Ok(Some(task)) => task,
                            Ok(None) => {
                                idle.push(slot);
                                break;
                            }
                            Err(error) => {
                                outcome.failure = Some(error.to_string());
                                break;
                            }
                        };
                        drop(schedule_span);
                        let preparation_span = self.operations.coordinator(true);
                        if let Err(error) =
                            slot.prepare(task.sector_id(), self.kernels, self.session, self.replay)
                        {
                            outcome.failure = Some(error.to_string());
                            break;
                        }
                        slot.active
                            .as_mut()
                            .unwrap()
                            .context
                            .set_reference_state(frozen.state(task.sector_id() as usize))?;
                        drop(preparation_span);
                        let send = send.clone();
                        let stop = &stop;
                        in_flight += 1;
                        scope.spawn(move |_| {
                            // Every job sends a notification even if its native
                            // evaluation panics. A lost job must not strand recv.
                            let result =
                                catch_unwind(AssertUnwindSafe(|| execute(slot, task, stop)))
                                    .map_err(|payload| {
                                        payload
                                            .downcast_ref::<String>()
                                            .cloned()
                                            .or_else(|| {
                                                payload
                                                    .downcast_ref::<&str>()
                                                    .map(|s| (*s).to_owned())
                                            })
                                            .unwrap_or_else(|| "unknown panic payload".to_owned())
                                    });
                            let _ = send.send(result);
                        });
                    }
                    if outcome.stopped() {
                        stop.store(true, Ordering::Relaxed);
                    }
                }
                if in_flight == 0 {
                    if outcome.stopped() || self.session.is_complete() {
                        return Ok(outcome);
                    }
                    return Err("QMC scheduler has no work before completion".into());
                }
                match receive.recv_timeout(Duration::from_millis(50)) {
                    Ok(returned) => {
                        let _span = self.operations.coordinator(false);
                        in_flight -= 1;
                        match returned {
                            Ok(completed) => {
                                self.diagnostics.merge(&completed.diagnostics)?;
                                if !completed.aborted_prefix
                                    && let Err(error) = super::super::submit_package(
                                        completed.result,
                                        completed.sector,
                                        completed.state,
                                        self.replay,
                                        |result| self.session.submit(result),
                                    )
                                {
                                    outcome.failure.get_or_insert_with(|| error.to_string());
                                }
                                idle.push(completed.slot);
                            }
                            Err(message) => {
                                outcome.failure.get_or_insert_with(|| {
                                    format!("QMC worker panicked: {message}")
                                });
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => {
                        return Err("QMC worker return channel disconnected".into());
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests;
