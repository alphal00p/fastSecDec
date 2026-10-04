use serde::{Deserialize, Serialize};

use crate::status::{CoefficientComponent, EvaluationDiagnostics};

use super::super::{
    AxisEndpoint, BoundaryOptions, BoundaryReport, DiagnosticError, DiagnosticProgress,
    DiagnosticStop, Result,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryGrowthOptions {
    /// Diagnostic threshold per simultaneously approached coordinate.
    /// Exceeding this threshold does not prove mathematical divergence.
    pub max_power_per_axis: f64,
    pub numerical_slack: f64,
}

impl Default for BoundaryGrowthOptions {
    fn default() -> Self {
        Self {
            max_power_per_axis: 0.5,
            numerical_slack: 1e-6,
        }
    }
}

impl BoundaryGrowthOptions {
    pub(super) fn threshold(&self, codimension: usize) -> Result<f64> {
        let value = self.max_power_per_axis * codimension as f64 + self.numerical_slack;
        if !self.max_power_per_axis.is_finite()
            || self.max_power_per_axis < 0.0
            || !self.numerical_slack.is_finite()
            || self.numerical_slack < 0.0
            || !value.is_finite()
        {
            return Err(DiagnosticError::Invalid(
                "growth tolerance and slack must be finite and nonnegative, with a finite codimension threshold",
            ));
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BoundaryScanOptions {
    /// `max_probes` is shared by the initial scan and every retry.
    pub sampling: BoundaryOptions,
    pub growth: BoundaryGrowthOptions,
    /// Strictly decreasing scales in (0,1), relative to the original distances.
    /// The empty default performs one attempt.
    pub retry_scales: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryGrowthUnavailable {
    MissingProbe,
    MissingValues,
    EvaluationFailed,
    InvalidDistance,
    NumericalRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum BoundaryGrowthEstimate {
    /// Effective positive growth, clipped to zero for decreasing magnitudes.
    Power(f64),
    BothZero,
    DecreasesToZero,
    /// A zero crossing cannot establish a finite power law from two samples.
    EmergesFromZero,
    Unavailable(BoundaryGrowthUnavailable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryAssessment {
    WithinThreshold,
    /// Sampled rapid growth relative to the requested policy; not divergence.
    Flagged,
    Inconclusive,
    NotApplicable,
}

impl BoundaryAssessment {
    pub(super) fn needs_retry(self) -> bool {
        matches!(self, Self::Flagged | Self::Inconclusive)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryGrowthPair {
    pub sector: usize,
    pub endpoints: Vec<AxisEndpoint>,
    /// Index in the report's parallel `orders`/`components` layout.
    pub component_index: usize,
    pub farther_probe: Option<usize>,
    pub nearer_probe: Option<usize>,
    pub farther_magnitude: Option<f64>,
    pub nearer_magnitude: Option<f64>,
    /// Mean of actual log endpoint-distance ratios over approached axes.
    pub mean_log_distance_ratio: Option<f64>,
    pub estimate: BoundaryGrowthEstimate,
    pub threshold: f64,
    pub assessment: BoundaryAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundarySectorAssessment {
    pub sector: usize,
    pub sampling_complete: bool,
    pub expected_pairs: Option<u64>,
    pub observed_pairs: usize,
    pub flagged_pairs: usize,
    pub inconclusive_pairs: usize,
    pub assessment: BoundaryAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryGrowthReport {
    pub options: BoundaryGrowthOptions,
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub pairs: Vec<BoundaryGrowthPair>,
    pub sectors: Vec<BoundarySectorAssessment>,
    pub assessment: BoundaryAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryScanAttempt {
    pub index: usize,
    pub scale: f64,
    pub selected_sectors: Vec<usize>,
    pub samples: BoundaryReport,
    pub growth: BoundaryGrowthReport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryLatestAssessment {
    pub attempt: usize,
    pub sector: BoundarySectorAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryScanReport {
    pub options: BoundaryScanOptions,
    /// Earlier flagged attempts are never overwritten by later observations.
    pub attempts: Vec<BoundaryScanAttempt>,
    pub latest: Vec<BoundaryLatestAssessment>,
    pub had_prior_flags: bool,
    pub completed_probes: usize,
    pub diagnostics: EvaluationDiagnostics,
    /// Execution state is independent of the diagnostic growth assessment.
    pub stop: DiagnosticStop,
    pub assessment: BoundaryAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BoundaryScanProgress {
    Sampling {
        attempt: usize,
        scale: f64,
        progress: DiagnosticProgress,
    },
    Assessed {
        attempt: usize,
        scale: f64,
        report: BoundaryGrowthReport,
    },
}

impl std::fmt::Display for BoundaryAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WithinThreshold => "sampled growth is within the requested threshold",
            Self::Flagged => "rapid sampled growth flagged",
            Self::Inconclusive => "growth assessment is inconclusive",
            Self::NotApplicable => "no stochastic boundary growth to assess",
        })
    }
}
