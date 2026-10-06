//! In-flight observations are separate from accepted native statistics.
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct IntegrationWorkerActivity {
    pub worker: usize,
    pub batch: u32,
    pub completed_points: u64,
    pub planned_points: u64,
    pub sector: Option<u64>,
    pub preparing_context: bool,
}

pub(super) fn summary(workers: &[IntegrationWorkerActivity]) -> String {
    if workers.is_empty() {
        return String::new();
    }
    let completed: u64 = workers.iter().map(|worker| worker.completed_points).sum();
    let planned: u64 = workers.iter().map(|worker| worker.planned_points).sum();
    let preparing = workers
        .iter()
        .filter(|worker| worker.preparing_context)
        .count();
    format!(
        "{} batches in flight · {completed}/{planned} points (not yet accepted) · {preparing} preparing evaluators",
        workers.len()
    )
}
