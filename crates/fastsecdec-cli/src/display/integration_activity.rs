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
