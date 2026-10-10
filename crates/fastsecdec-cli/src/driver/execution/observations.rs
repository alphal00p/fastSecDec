//! Caller-owned observational ledgers. No entry is an accepted native result.
use crate::CliResult;
use fastsecdec::{
    integration::{McLiveBatch, OperationalMetrics, SectorOperationalMetrics},
    status::EvaluationDiagnostics,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Default)]
pub(super) struct WorkerMeter(Arc<Mutex<State>>);
#[derive(Default)]
struct State {
    orders: Vec<i32>,
    publication_interval: Duration,
    metrics: OperationalMetrics,
    active: Option<(Instant, Option<u64>)>,
    live: Option<McLiveBatch>,
    activity: Option<crate::display::IntegrationWorkerActivity>,
}
pub(super) struct TaskSpan {
    meter: WorkerMeter,
}
impl Drop for TaskSpan {
    fn drop(&mut self) {
        let mut state = self.meter.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((start, sector)) = state.active.take() {
            let seconds = start.elapsed().as_secs_f64();
            state.metrics.worker_seconds += seconds;
            if let Some(id) = sector {
                row(&mut state.metrics, id).worker_seconds += seconds;
            }
        }
        state.activity = None;
    }
}
fn row(metrics: &mut OperationalMetrics, id: u64) -> &mut SectorOperationalMetrics {
    let index = metrics
        .sectors
        .iter()
        .position(|r| r.id == id)
        .unwrap_or_else(|| {
            metrics.sectors.push(SectorOperationalMetrics {
                id,
                ..Default::default()
            });
            metrics.sectors.len() - 1
        });
    &mut metrics.sectors[index]
}
impl WorkerMeter {
    fn new(orders: &[i32], publication_interval: Duration) -> Self {
        Self(Arc::new(Mutex::new(State {
            orders: orders.to_vec(),
            publication_interval,
            ..Default::default()
        })))
    }
    pub(super) fn publication_interval(&self) -> Duration {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .publication_interval
    }
    pub(super) fn preparation(&self, id: u64, seconds: f64) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        state.metrics.integrand_seconds += seconds;
        row(&mut state.metrics, id).integrand_seconds += seconds;
    }
    pub(super) fn activity(&self, batch: u32, planned: u64, sector: u64) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).activity =
            Some(crate::display::IntegrationWorkerActivity {
                worker: 0,
                batch,
                completed_points: 0,
                planned_points: planned,
                sector: Some(sector),
                preparing_context: false,
            });
    }
    pub(super) fn points_completed(&self, points: u64) {
        if let Some(activity) = &mut self.0.lock().unwrap_or_else(|e| e.into_inner()).activity {
            activity.completed_points += points;
        }
    }
    pub(super) fn task(&self, sector: Option<u64>) -> TaskSpan {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        assert!(state.active.is_none(), "one native task per worker slot");
        state.active = Some((Instant::now(), sector));
        state.live = None;
        TaskSpan {
            meter: self.clone(),
        }
    }
    pub(super) fn record_batch(
        &self,
        id: u64,
        integrand: f64,
        evaluator: f64,
        diagnostics: &EvaluationDiagnostics,
        outputs: &[f64],
    ) -> Result<(), String> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let State {
            metrics, orders, ..
        } = &mut *state;
        metrics
            .diagnostics
            .merge(diagnostics)
            .map_err(|e| e.to_string())?;
        metrics.integrand_seconds += integrand;
        metrics.evaluator_seconds += evaluator;
        metrics.evaluations = metrics.diagnostics.evaluations;
        let sector = row(metrics, id);
        sector
            .diagnostics
            .merge(diagnostics)
            .map_err(|e| e.to_string())?;
        sector.integrand_seconds += integrand;
        sector.evaluator_seconds += evaluator;
        sector.evaluations = sector.diagnostics.evaluations;
        for output in outputs.chunks(orders.len().max(1)) {
            let mut index = 0;
            while index < orders.len() {
                let order = orders[index];
                let mut magnitude = 0.0_f64;
                while index < orders.len() && orders[index] == order {
                    magnitude = magnitude.hypot(output[index]);
                    index += 1;
                }
                sector
                    .maximum_weighted_contribution
                    .entry(order)
                    .and_modify(|old| *old = old.max(magnitude))
                    .or_insert(magnitude);
            }
        }
        Ok(())
    }
    pub(super) fn publish(&self, batch: McLiveBatch) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).live = Some(batch);
    }
    pub(super) fn clear_live(&self) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).live = None;
    }
    pub(super) fn live(&self) -> Option<McLiveBatch> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .live
            .clone()
    }
    pub(super) fn add_sector_times(&self, values: impl Iterator<Item = (u64, f64)>) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        for (id, seconds) in values {
            row(&mut state.metrics, id).worker_seconds += seconds;
        }
    }
    fn snapshot(&self) -> OperationalMetrics {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let mut metrics = state.metrics.clone();
        if let Some((start, sector)) = state.active {
            let seconds = start.elapsed().as_secs_f64();
            metrics.worker_seconds += seconds;
            if let Some(id) = sector {
                row(&mut metrics, id).worker_seconds += seconds;
            }
        }
        metrics
    }
}

#[derive(Clone)]
pub(super) struct Operations {
    workers: Vec<WorkerMeter>,
    coordinator: Arc<Mutex<(f64, f64)>>,
    preparation_diagnostics: Arc<Mutex<EvaluationDiagnostics>>,
}
pub(super) struct CoordinatorSpan {
    start: Instant,
    preparation: bool,
    totals: Arc<Mutex<(f64, f64)>>,
}
impl Drop for CoordinatorSpan {
    fn drop(&mut self) {
        let seconds = self.start.elapsed().as_secs_f64();
        let mut totals = self.totals.lock().unwrap_or_else(|e| e.into_inner());
        if self.preparation {
            totals.0 += seconds;
        } else {
            totals.1 += seconds;
        }
    }
}
impl Operations {
    pub(super) fn activities(&self) -> Vec<crate::display::IntegrationWorkerActivity> {
        self.workers
            .iter()
            .enumerate()
            .filter_map(|(id, worker)| {
                let mut value = worker
                    .0
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .activity
                    .clone()?;
                value.worker = id;
                Some(value)
            })
            .collect()
    }
    pub(super) fn new(workers: usize, orders: &[i32], publication_interval: Duration) -> Self {
        Self {
            workers: (0..workers)
                .map(|_| WorkerMeter::new(orders, publication_interval))
                .collect(),
            coordinator: Default::default(),
            preparation_diagnostics: Default::default(),
        }
    }
    pub(super) fn record_preparation(&self, report: &EvaluationDiagnostics) -> CliResult<()> {
        self.preparation_diagnostics
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .merge(report)?;
        Ok(())
    }
    pub(super) fn coordinator(&self, preparation: bool) -> CoordinatorSpan {
        CoordinatorSpan {
            start: Instant::now(),
            preparation,
            totals: Arc::clone(&self.coordinator),
        }
    }
    pub(super) fn worker(&self, id: usize) -> WorkerMeter {
        self.workers[id].clone()
    }
    pub(super) fn snapshot(&self) -> CliResult<OperationalMetrics> {
        let mut result = OperationalMetrics {
            diagnostics: self
                .preparation_diagnostics
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
            ..Default::default()
        };
        (
            result.coordinator_integrand_seconds,
            result.coordinator_integrator_seconds,
        ) = *self.coordinator.lock().unwrap_or_else(|e| e.into_inner());
        for worker in &self.workers {
            let next = worker.snapshot();
            result.worker_seconds += next.worker_seconds;
            result.integrand_seconds += next.integrand_seconds;
            result.evaluator_seconds += next.evaluator_seconds;
            result.diagnostics.merge(&next.diagnostics)?;
            for next in next.sectors {
                let target = row(&mut result, next.id);
                target.worker_seconds += next.worker_seconds;
                target.integrand_seconds += next.integrand_seconds;
                target.evaluator_seconds += next.evaluator_seconds;
                target.diagnostics.merge(&next.diagnostics)?;
                target.evaluations = target.diagnostics.evaluations;
                for (order, maximum) in next.maximum_weighted_contribution {
                    target
                        .maximum_weighted_contribution
                        .entry(order)
                        .and_modify(|old| *old = old.max(maximum))
                        .or_insert(maximum);
                }
            }
        }
        result.evaluations = result.diagnostics.evaluations;
        result.sectors.sort_by_key(|r| r.id);
        Ok(result)
    }
}

#[derive(Default)]
pub(super) struct McLedger {
    batches: BTreeMap<(Option<u64>, u32), McLiveBatch>,
}
impl McLedger {
    pub(super) fn update(&mut self, values: impl Iterator<Item = McLiveBatch>) {
        for batch in values {
            self.batches.insert((batch.sector_id, batch.batch), batch);
        }
    }
    pub(super) fn discard(&mut self, sector: Option<u64>, batch: u32) {
        self.batches.remove(&(sector, batch));
    }
    pub(super) fn batches(&self) -> Vec<McLiveBatch> {
        self.batches.values().cloned().collect()
    }
}
