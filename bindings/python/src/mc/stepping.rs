use fastsecdec::status::StoppingReason;
use pyo3::prelude::*;

use super::PyHavanaDiscreteSession;
use crate::{error, status::PyIntegrationSnapshot};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyHavanaDiscreteSession {
    /// Execute at most max_batches. False from observer pauses after an accepted
    /// global batch. Interrupted batches are retried, without partial statistics.
    #[pyo3(signature = (max_batches=1, *, evaluation_batch_size=256, observer=None))]
    fn step(
        &mut self,
        py: Python<'_>,
        max_batches: usize,
        evaluation_batch_size: usize,
        observer: Option<Py<PyAny>>,
    ) -> PyResult<PyIntegrationSnapshot> {
        if max_batches == 0 || evaluation_batch_size == 0 {
            return Err(error::native(
                py,
                "configuration",
                "max_batches and evaluation_batch_size must be positive",
            ));
        }
        self.stop_reason = None;
        for _ in 0..max_batches {
            if let Err(e) = py.check_signals() {
                if e.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) {
                    self.stop_reason = Some(StoppingReason::Cancelled);
                }
                return Err(e);
            }
            let Some(task) = self.session.next_work() else {
                break;
            };
            let mut signal_interrupted = false;
            let result = (|| {
                if self.worker.is_none() {
                    self.worker = Some(
                        self.session
                            .worker_context()
                            .map_err(|e| error::native(py, "integration", e))?,
                    );
                }
                let mut interrupted = None;
                let mut live = None;
                let value = self
                    .worker
                    .as_mut()
                    .expect("prepared native worker")
                    .evaluate_weighted_batch_observed(
                        task.clone(),
                        evaluation_batch_size,
                        |id, points, weights, output| {
                            let sector = usize::try_from(id).map_err(|e| e.to_string())?;
                            let context = self.contexts.get_mut(sector).ok_or_else(|| {
                                "native sector id outside kernel manifest".to_owned()
                            })?;
                            if context.is_none() {
                                *context = Some(
                                    self.kernels
                                        .restore_evaluation_context(
                                            sector,
                                            self.policy.clone(),
                                            &self.replay[sector],
                                        )
                                        .map_err(|e| e.to_string())?,
                                );
                            }
                            crate::execution::evaluate_batch(
                                py,
                                context.as_mut().expect("prepared weighted context"),
                                points,
                                weights,
                                output,
                                &mut self.diagnostics,
                                &mut interrupted,
                            )
                        },
                        |view| {
                            if view.points() == task.point_count() as u64 {
                                live = Some(view.snapshot());
                            }
                        },
                    );
                if let Some(e) = interrupted {
                    signal_interrupted = true;
                    return Err(e);
                }
                let value = value.map_err(|e| error::native(py, "integration", e))?;
                // Every selected sector's candidate replay is checked before the
                // native batch is submitted; accepted replay changes are atomic.
                let mut accepted = self.replay.clone();
                for (sector, context) in self.contexts.iter().enumerate() {
                    if let Some(context) = context {
                        accepted[sector]
                            .merge(context.state())
                            .map_err(|e| error::native(py, "integration", e))?;
                        self.kernels
                            .validate_replay_state(sector, &self.policy, &accepted[sector])
                            .map_err(|e| error::native(py, "integration", e))?;
                    }
                }
                self.session
                    .submit(value)
                    .map_err(|e| error::native(py, "integration", e))?;
                self.replay = accepted;
                if let Some(live) = live {
                    self.live_batches.push(live);
                }
                Ok(())
            })();
            if let Err(e) = result {
                self.discard_contexts();
                self.session
                    .retry(&task)
                    .map_err(|retry| error::native(py, "integration", retry))?;
                self.stop_reason = if e.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py)
                {
                    Some(StoppingReason::Cancelled)
                } else if signal_interrupted {
                    None
                } else {
                    Some(StoppingReason::NumericalFailure(e.to_string()))
                };
                return Err(e);
            }
            if let Some(observer) = &observer {
                let value = match observer.call1(py, (self.snapshot(py)?,)) {
                    Ok(value) => value,
                    Err(e) => {
                        if e.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) {
                            self.stop_reason = Some(StoppingReason::Cancelled);
                        }
                        return Err(e);
                    }
                };
                if !value.is_none(py) && !value.extract::<bool>(py)? {
                    self.stop_reason = Some(StoppingReason::Cancelled);
                    break;
                }
            }
        }
        if self.session.is_complete() && self.stop_reason.is_none() {
            self.stop_reason = Some(StoppingReason::PlannedWorkComplete);
        }
        self.snapshot(py)
    }
}
