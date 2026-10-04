use fastsecdec::{
    integration::{
        ContributionReport, IntegrationObservation, QmcDesign, Tolerance, VectorEstimate,
    },
    status::{EvaluationDiagnostics, GenerationTimings, IntegrationSnapshot, StoppingReason},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct IntegrationReport {
    pub content_id: String,
    pub elapsed_seconds: f64,
    pub loading_seconds: f64,
    pub generation_timings: Option<GenerationTimings>,
    /// Only a full-integral target can establish unqualified convergence.
    pub converged: bool,
    pub scoped_target_reached: bool,
    pub scope: fastsecdec::results::ResultScope,
    pub stopping_reason: String,
    pub estimate: Option<VectorEstimate>,
    pub snapshot: IntegrationSnapshot,
    pub resume_status: ResumeStatus,
    /// Effective final-round design; adaptive sector allocations are authoritative.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qmc_design: Option<QmcDesign>,
    /// Native accepted-sector statistics, with authoritative total covariance.
    pub contributions: ContributionReport,
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

pub(super) struct ExecutionOutcome {
    pub scope: fastsecdec::results::ResultScope,
    pub cancelled: bool,
    pub failure: Option<String>,
    pub resume_status: ResumeStatus,
    pub qmc_design: Option<QmcDesign>,
}

pub(super) fn finish(
    artifact: &crate::artifact::Artifact,
    observation: IntegrationObservation,
    diagnostics: &EvaluationDiagnostics,
    tolerance: Tolerance,
    elapsed_seconds: f64,
    mut outcome: ExecutionOutcome,
) -> crate::CliResult<IntegrationReport> {
    let mut snapshot = with_diagnostics(observation.snapshot, diagnostics);
    if let fastsecdec::status::UncertaintyStatus::StatisticalFailure { reason } =
        &snapshot.uncertainty
    {
        outcome.failure.get_or_insert_with(|| reason.clone());
    }
    let scoped_target_reached = outcome.failure.is_none()
        && !outcome.cancelled
        && snapshot
            .estimate
            .as_ref()
            .map(|estimate| estimate.meets(tolerance))
            .transpose()?
            .unwrap_or(false);
    let (stop, stopping_reason) = if let Some(message) = outcome.failure {
        (
            StoppingReason::NumericalFailure(message.clone()),
            format!("numerical failure: {message}"),
        )
    } else if outcome.cancelled {
        (
            StoppingReason::Cancelled,
            if outcome.resume_status == ResumeStatus::PilotRestartRequired {
                "cancelled during MC pilot; restart the pilot to continue"
            } else {
                "cancelled"
            }
            .into(),
        )
    } else if scoped_target_reached {
        (StoppingReason::TargetReached, "accuracy reached".into())
    } else {
        (StoppingReason::WorkLimit, "work limit".into())
    };
    snapshot.stop_reason = Some(stop);
    Ok(IntegrationReport {
        content_id: artifact.content_id.clone(),
        elapsed_seconds,
        loading_seconds: artifact.loading_seconds,
        generation_timings: artifact.generation_timings.clone(),
        converged: scoped_target_reached && outcome.scope.is_full_integral(),
        scoped_target_reached,
        scope: outcome.scope,
        stopping_reason,
        estimate: snapshot.estimate.clone(),
        snapshot,
        resume_status: outcome.resume_status,
        qmc_design: outcome.qmc_design,
        contributions: observation.contributions,
    })
}

impl IntegrationReport {
    pub fn failed(&self) -> bool {
        matches!(
            self.snapshot.stop_reason,
            Some(StoppingReason::NumericalFailure(_))
        )
    }
}
