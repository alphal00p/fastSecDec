//! Serializable status snapshots with terminal-independent display functions.
mod coefficient;
mod contour;
mod contour_runtime;
mod diagnostics;
mod generation;
mod geometry;
mod timings;
pub use coefficient::CoefficientExpansionSnapshot;
pub use contour::{
    ContourCheckCounters, ContourCheckpointProvenance, ContourEvaluationDiagnostics,
    ContourPilotProvenance, ContourRunReport,
};
pub use contour_runtime::ContourRuntimeDiagnostics;
pub use diagnostics::{DiagnosticsOverflow, EvaluationDiagnostics};
pub use geometry::GeometryReuseStatus;
use std::fmt;
pub use timings::GenerationTimings;

use serde::{Deserialize, Serialize};

use crate::integration::VectorEstimate;

/// Numerical components remain explicit so the full real/imaginary and
/// inter-order covariance is retained when an overall weight is complex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoefficientComponent {
    Real,
    Imag,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoppingReason {
    TargetReached,
    PlannedWorkComplete,
    WorkLimit,
    TimeLimit,
    Cancelled,
    NumericalFailure(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationMethod {
    DemocraticQmc,
    AdaptiveQmc,
    HavanaMc,
    HavanaDiscreteMc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationStage {
    Pilot,
    Production,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UncertaintyStatus {
    /// Every stochastic sector contributes enough complete independent replicas.
    Available,
    Exact,
    WaitingForCoverage,
    PilotOnly,
    /// Accepted coverage is retained, but native statistics could not be
    /// represented. This is neither missing work nor a zero estimate.
    StatisticalFailure {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscreteSectorAllocation {
    /// Frozen native probability for drawing this sector in a global batch.
    pub probability: f64,
    pub points_per_batch: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectorSnapshot {
    pub id: u64,
    pub dimension: usize,
    pub completed_points: u64,
    /// Fixed sector quota, or `None` when global samples choose sectors randomly.
    /// Historical serialized integer quotas continue to decode as `Some`.
    pub planned_points: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discrete_allocation: Option<DiscreteSectorAllocation>,
    pub complete_replicas: usize,
    pub planned_replicas: usize,
    pub worker_seconds: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegrationSnapshot {
    pub method: IntegrationMethod,
    pub stage: IntegrationStage,
    pub completed_points: u64,
    pub planned_points: u64,
    pub complete_sectors: usize,
    pub sectors: Vec<SectorSnapshot>,
    pub uncertainty: UncertaintyStatus,
    pub estimate: Option<VectorEstimate>,
    /// Sum of worker execution times; wall time belongs to the caller.
    pub worker_seconds: f64,
    /// The caller controls stopping and can annotate the final snapshot.
    pub stop_reason: Option<StoppingReason>,
    /// Optional caller-side kernel counters, independent of the integration
    /// library's generic evaluation closure and statistical observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_diagnostics: Option<EvaluationDiagnostics>,
}

impl fmt::Display for IntegrationSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:?} {:?}: {}/{} points; {}/{} sectors; uncertainty {:?}",
            self.method,
            self.stage,
            self.completed_points,
            self.planned_points,
            self.complete_sectors,
            self.sectors.len(),
            self.uncertainty
        )?;
        if let Some(estimate) = &self.estimate {
            for (((order, component), mean), error) in estimate
                .orders
                .iter()
                .zip(&estimate.components)
                .zip(&estimate.mean)
                .zip(&estimate.standard_error)
            {
                writeln!(
                    f,
                    "  {component:?} eps^{order}: {mean:.10e} +/- {error:.3e}"
                )?;
            }
        }
        if let Some(diagnostics) = &self.evaluation_diagnostics {
            writeln!(f, "  {diagnostics}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationStage {
    Input,
    Parametrization,
    Geometry,
    Mapping,
    FormulaPreparation,
    Symmetry,
    Subtraction,
    Expansion,
    CoefficientExpansion,
    Compilation,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerationSnapshot {
    pub stage: GenerationStage,
    pub completed: usize,
    pub total: Option<usize>,
    pub sectors: usize,
    pub kernels: usize,
    pub elapsed_seconds: f64,
    #[serde(default)]
    pub timings: GenerationTimings,
    /// Absent for physical-only generation. A named run retains the current or
    /// last completed representative's per-attempt observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coefficient_expansion: Option<CoefficientExpansionSnapshot>,
    /// Discovered numerical-dual formula requirements and completed unique
    /// builds. Absent before discovery and for older or symbolic-only runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula_preparation: Option<FormulaPreparationSnapshot>,
    pub detail: String,
}

/// Coordinator-owned counts for the distinct subtraction-formula phase.
/// Reused counts shared formula uses, not completed cache lookup operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormulaPreparationSnapshot {
    pub completed: usize,
    pub total: usize,
    pub sectors: usize,
    pub reused: usize,
}

impl fmt::Display for GenerationSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.stage, self.completed)?;
        if let Some(total) = self.total {
            write!(f, "/{total}")?;
        }
        write!(
            f,
            "; {} sectors; {} kernels; {:.2}s {}",
            self.sectors, self.kernels, self.elapsed_seconds, self.detail
        )
    }
}
