//! Provisional observations are deliberately separate from accepted covariance.
use super::CoefficientComponent;
use crate::status::IntegrationStage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveStatus {
    NotSampled,
    MeanOnly,
    Available,
    NumericalRange,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveEstimate {
    pub mean: Option<Vec<f64>>,
    pub standard_error: Option<Vec<f64>>,
    pub points: u64,
    pub replicas: usize,
    pub status: LiveStatus,
}
impl LiveEstimate {
    pub(crate) fn empty(points: u64, replicas: usize) -> Self {
        Self {
            mean: None,
            standard_error: None,
            points,
            replicas,
            status: LiveStatus::NotSampled,
        }
    }
    pub(crate) fn range(points: u64, replicas: usize) -> Self {
        Self {
            status: LiveStatus::NumericalRange,
            ..Self::empty(points, replicas)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveSector {
    pub id: u64,
    pub estimate: LiveEstimate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveSource {
    CurrentIteration,
    SinceResume,
    CompleteLattices,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LiveObservation {
    pub source: LiveSource,
    pub stage: IntegrationStage,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub sectors: Vec<LiveSector>,
    pub total: LiveEstimate,
}

/// Actual work during this invocation, including discarded prefixes and pilots.
/// Durations are summed worker elapsed spans, never operating-system CPU time.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OperationalMetrics {
    /// Coordinator evaluator/context preparation, excluding worker time.
    pub coordinator_integrand_seconds: f64,
    /// Active scheduling, admission, observation and checkpoint work; no waits.
    pub coordinator_integrator_seconds: f64,
    /// Aggregate elapsed spans of active worker tasks (including discarded work).
    pub worker_seconds: f64,
    /// Inclusive worker context preparation and weighted evaluation wrappers.
    pub integrand_seconds: f64,
    /// Native evaluator calls nested within `integrand_seconds`.
    pub evaluator_seconds: f64,
    pub evaluations: u64,
    pub diagnostics: crate::status::EvaluationDiagnostics,
    pub sectors: Vec<SectorOperationalMetrics>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SectorOperationalMetrics {
    pub id: u64,
    pub worker_seconds: f64,
    pub integrand_seconds: f64,
    pub evaluator_seconds: f64,
    pub evaluations: u64,
    pub diagnostics: crate::status::EvaluationDiagnostics,
    /// Largest final finite complex coefficient magnitude sampled this invocation.
    pub maximum_weighted_contribution: std::collections::BTreeMap<i32, f64>,
}
