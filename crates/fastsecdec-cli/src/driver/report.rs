use fastsecdec::{
    integration::VectorEstimate,
    status::{EvaluationDiagnostics, GenerationTimings, IntegrationSnapshot, StoppingReason},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct IntegrationReport {
    pub content_id: String,
    pub elapsed_seconds: f64,
    pub loading_seconds: f64,
    pub generation_timings: Option<GenerationTimings>,
    pub converged: bool,
    pub stopping_reason: String,
    pub estimate: Option<VectorEstimate>,
    pub snapshot: IntegrationSnapshot,
    pub resume_status: ResumeStatus,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeStatus {
    CheckpointSaved,
    /// Havana does not export mutable pilot training state. Restart the pilot.
    PilotRestartRequired,
}

pub(super) fn with_diagnostics(
    mut snapshot: IntegrationSnapshot,
    diagnostics: &EvaluationDiagnostics,
) -> IntegrationSnapshot {
    snapshot.evaluation_diagnostics = Some(diagnostics.clone());
    snapshot
}

pub(super) fn stopped(
    mut snapshot: IntegrationSnapshot,
    cancelled: bool,
    converged: bool,
) -> IntegrationSnapshot {
    snapshot.stop_reason = Some(if cancelled {
        StoppingReason::Cancelled
    } else if converged {
        StoppingReason::TargetReached
    } else {
        StoppingReason::WorkLimit
    });
    snapshot
}
