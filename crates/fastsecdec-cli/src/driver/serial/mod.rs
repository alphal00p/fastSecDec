//! Caller-owned resident integration processes. Native serial sessions own
//! statistical identities and admission, never these processes or files.
mod checkpoint;
mod coordinator;
mod worker;

pub(crate) use coordinator::integrate;
pub(crate) use worker::worker;

use fastsecdec::{
    integration::{
        IntegrationObservation, OperationalMetrics,
        serial::{ReplicaIdentity, SerialReturn, SerialSnapshot, SerialTask},
    },
    kernel::{ReplayPolicy, ReplayState, StabilitySettings},
    status::EvaluationDiagnostics,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Residency {
    pub worker: usize,
    pub pid: u32,
    pub sector: u64,
    pub residence_seconds: f64,
    pub loading_seconds: f64,
    pub completed_points: u64,
    pub planned_points: u64,
    pub preparing: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SerialRunSnapshot {
    pub native: SerialSnapshot,
    pub observation: IntegrationObservation,
    pub residents: Vec<Residency>,
    pub diagnostics: EvaluationDiagnostics,
    pub operational: OperationalMetrics,
    pub elapsed_seconds: f64,
    pub checkpoint_path: PathBuf,
    pub checkpoint_age_seconds: Option<f64>,
}
pub(crate) struct SerialOutcome {
    pub snapshot: SerialRunSnapshot,
    pub cancelled: bool,
    pub failure: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Job {
    task: SerialTask,
    data_path: PathBuf,
    catalogue_id: String,
    validate_artifact: bool,
    parameters: BTreeMap<String, f64>,
    policy: ReplayPolicy,
    stability: StabilitySettings,
    batch_size: usize,
    replay: Option<ReplayState>,
    return_path: PathBuf,
}
#[derive(Serialize, Deserialize)]
struct Returned {
    value: SerialReturn,
    replay: ReplayState,
    metrics: TaskMetrics,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct TaskMetrics {
    diagnostics: EvaluationDiagnostics,
    load_seconds: f64,
    worker_seconds: f64,
    integrand_seconds: f64,
    maxima: BTreeMap<i32, f64>,
}
#[derive(Debug, Serialize, Deserialize)]
enum Event {
    Loaded {
        identity: ReplicaIdentity,
        seconds: f64,
    },
    Progress {
        identity: ReplicaIdentity,
        completed: u64,
        planned: u64,
        sampling_seconds: f64,
        metrics: Box<TaskMetrics>,
    },
    Completed {
        identity: ReplicaIdentity,
        path: PathBuf,
    },
}
