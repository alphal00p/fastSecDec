use serde::{Deserialize, Serialize};

use super::{DiagnosticError, Result};
use crate::status::EvaluationDiagnostics;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkOptions {
    pub points: usize,
    pub repetitions: usize,
    pub warmup: usize,
    /// Maximum evaluations between caller cancellation polls.
    pub batch_size: usize,
}

impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self {
            points: 100_000,
            repetitions: 5,
            warmup: 16,
            batch_size: 256,
        }
    }
}

impl BenchmarkOptions {
    pub(super) fn validate(&self) -> Result<()> {
        if self.points == 0 || self.repetitions == 0 || self.batch_size == 0 {
            return Err(DiagnosticError::Invalid(
                "points, repetitions and batch size must be positive",
            ));
        }
        if self
            .points
            .checked_mul(self.repetitions)
            .and_then(|n| n.checked_add(self.warmup))
            .is_none()
        {
            return Err(DiagnosticError::Invalid(
                "benchmark evaluation count overflows",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticStop {
    Complete,
    Cancelled,
    ProbeBudget,
    EvaluationFailure,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkMeasurement {
    pub repetition: usize,
    pub evaluations: usize,
    pub seconds: f64,
    pub complete: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelTiming {
    pub sector: usize,
    pub dimension: usize,
    pub outputs: usize,
    pub warmup_evaluations: usize,
    pub measurements: Vec<BenchmarkMeasurement>,
    /// Only complete repetitions contribute; even counts use the upper median.
    pub median_seconds: Option<f64>,
    pub evaluations_per_second: Option<f64>,
    pub diagnostics: EvaluationDiagnostics,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub options: BenchmarkOptions,
    pub sectors: Vec<KernelTiming>,
    pub planned_sectors: usize,
    pub stop: DiagnosticStop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BoundarySide {
    Lower,
    Upper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AxisEndpoint {
    pub axis: usize,
    pub side: BoundarySide,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryOptions {
    pub exponents: Vec<i32>,
    /// Enumerate every coordinate subset and side assignment up to this size.
    pub max_codimension: usize,
    /// Total point budget across all sectors and exponents.
    pub max_probes: usize,
    /// Coordinates not approaching a boundary stay at this interior point.
    pub interior: f64,
}

impl Default for BoundaryOptions {
    fn default() -> Self {
        Self {
            exponents: vec![3, 6, 9, 12, 15],
            max_codimension: 2,
            max_probes: 10_000,
            interior: 0.371,
        }
    }
}

impl BoundaryOptions {
    pub(super) fn distances(&self) -> Result<Vec<f64>> {
        if self.exponents.is_empty()
            || self.max_codimension == 0
            || self.max_probes == 0
            || !self.interior.is_finite()
            || !(0.0 < self.interior && self.interior < 1.0)
        {
            return Err(DiagnosticError::Invalid(
                "boundary exponents, codimension and budget must be nonempty/positive, with an interior anchor strictly inside the cube",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        self.exponents.iter().map(|exponent| {
            if *exponent <= 0 || !seen.insert(exponent) {
                return Err(DiagnosticError::Invalid("boundary exponents must be positive and distinct"));
            }
            let distance = 10f64.powi(-*exponent);
            let upper = 1.0 - distance;
            if !(0.0 < distance && distance < 1.0 && 0.0 < upper && upper < 1.0) {
                return Err(DiagnosticError::Invalid("boundary distances must remain representable strictly inside both ends of the cube"));
            }
            Ok(distance)
        }).collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryProbe {
    pub sector: usize,
    pub endpoints: Vec<AxisEndpoint>,
    pub exponent: i32,
    pub distance: f64,
    /// Actual binary64 coordinates, including rounded upper approaches.
    pub point: Vec<f64>,
    pub finite: bool,
    pub conditioning_checked: bool,
    pub rescued: bool,
    pub precision_bits: Option<u32>,
    pub max_absolute_value: Option<f64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryCoverage {
    pub max_codimension: usize,
    /// None denotes a count too large for u64; the point budget still applies.
    pub configured_probes: Option<u64>,
    pub all_face_pattern_probes: Option<u64>,
    pub planned_probes: usize,
    pub completed_probes: usize,
    pub truncated_by_budget: bool,
    /// Describes discrete face patterns, never exhaustive evaluation of a face.
    pub includes_all_face_patterns: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryReport {
    pub options: BoundaryOptions,
    pub probes: Vec<BoundaryProbe>,
    pub coverage: BoundaryCoverage,
    pub failures: usize,
    pub diagnostics: EvaluationDiagnostics,
    pub stop: DiagnosticStop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiagnosticProgress {
    Benchmark {
        sector: usize,
        /// None identifies warmup.
        repetition: Option<usize>,
        completed_evaluations: usize,
        planned_evaluations: usize,
        measurement: Option<BenchmarkMeasurement>,
    },
    Boundary {
        completed: usize,
        planned: usize,
        probe: Option<BoundaryProbe>,
    },
}

impl std::fmt::Display for BenchmarkReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Kernel benchmark: {:?}; {}/{} sectors",
            self.stop,
            self.sectors.len(),
            self.planned_sectors
        )?;
        for row in &self.sectors {
            let complete = row
                .measurements
                .iter()
                .filter(|sample| sample.complete)
                .count();
            write!(
                f,
                "  sector {}: {complete}/{} complete repetitions",
                row.sector, self.options.repetitions
            )?;
            if let Some(rate) = row.evaluations_per_second {
                write!(f, "; {rate:.3e} evaluations/s")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for BoundaryReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Boundary samples: {:?}; {}/{} probes, {} failures, codimension <= {}, budget truncated: {}",
            self.stop,
            self.coverage.completed_probes,
            self.coverage.planned_probes,
            self.failures,
            self.coverage.max_codimension,
            self.coverage.truncated_by_budget
        )
    }
}
