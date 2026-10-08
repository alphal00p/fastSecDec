//! Bounded, caller-owned batches. Only independent primary evaluations move
//! ahead; weighted references and precision replays retain original row order.
use super::{KernelError, PrecisionClass, ReplayReport, StabilityMode, WeightedEvaluationContext};
use crate::kernel::{Backend, EvaluatorTiming};

/// A failed batch retains the reports for its successfully processed prefix.
/// `completed.len()` identifies the failing row (or zero for a shape error).
/// Later primary f64 rows may already have run in the native matrix call; their
/// actual work remains included in the context's cumulative evaluation metrics.
#[derive(Debug, thiserror::Error)]
#[error("batch evaluation failed after {count} points: {error}", count = .completed.len())]
pub struct BatchEvaluationError {
    pub completed: Vec<ReplayReport>,
    #[source]
    pub error: KernelError,
}

impl WeightedEvaluationContext {
    /// Evaluate a caller-bounded batch of one sector in point-major order.
    /// Coordinates have `weights.len() * dimension()` entries; output has
    /// `weights.len() * output_count()` entries. Runtime parameters are appended
    /// internally. Native f64 rows use the owner's real/complex matrix evaluator;
    /// eager and higher-precision rows reuse native scalar evaluators.
    ///
    /// Cutoff rows never enter a numerical evaluator. All successful reference
    /// updates and any precision replays run in original point order. Output
    /// rows at and after an error must be discarded. Primary batch timings in
    /// individual reports are amortized shares, preserving the measured total;
    /// `calls` still counts evaluated points, not matrix function invocations.
    pub fn evaluate_weighted_batch(
        &mut self,
        points: &[f64],
        weights: &[f64],
        output: &mut [f64],
    ) -> Result<Vec<ReplayReport>, BatchEvaluationError> {
        self.evaluate_weighted_batch_controlled(points, weights, output, || false)
    }

    /// The same batch operation with cooperative cancellation before native
    /// matrix work and between ordered points. Native evaluator calls themselves
    /// are indivisible. Cancellation returns `KernelError::Cancelled` and the
    /// processed prefix; callers must discard incomplete statistical work.
    pub fn evaluate_weighted_batch_controlled(
        &mut self,
        points: &[f64],
        weights: &[f64],
        output: &mut [f64],
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Vec<ReplayReport>, BatchEvaluationError> {
        let fail = |error| BatchEvaluationError {
            completed: Vec::new(),
            error,
        };
        if cancelled() {
            return Err(fail(KernelError::Cancelled));
        }
        let rows = weights.len();
        let dimension = self.dimension();
        let outputs = self.output_count();
        let expected = rows.checked_mul(dimension).ok_or_else(|| {
            fail(KernelError::Dimension {
                expected: usize::MAX,
                actual: points.len(),
            })
        })?;
        if expected != points.len() {
            return Err(fail(KernelError::Dimension {
                expected,
                actual: points.len(),
            }));
        }
        let expected = rows.checked_mul(outputs).ok_or_else(|| {
            fail(KernelError::OutputCount {
                expected: usize::MAX,
                actual: output.len(),
            })
        })?;
        if expected != output.len() {
            return Err(fail(KernelError::OutputCount {
                expected,
                actual: output.len(),
            }));
        }
        if rows == 0 {
            return Ok(Vec::new());
        }
        if !self.kernel.parameters_bound {
            // Match scalar validation precedence even when no matrix can run.
            let error = if !weights[0].is_finite() || weights[0] < 0.0 {
                KernelError::InvalidWeight
            } else if points[..dimension]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            {
                KernelError::InvalidPoint
            } else {
                KernelError::UnboundParameters
            };
            return Err(fail(error));
        }
        // Stop admission at the first invalid input, preserving scalar error
        // precedence and avoiding speculative work on invalid or later rows.
        let valid = weights
            .iter()
            .enumerate()
            .take_while(|(row, weight)| {
                weight.is_finite()
                    && **weight >= 0.0
                    && points[row * dimension..(row + 1) * dimension]
                        .iter()
                        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            })
            .count();
        let selected = (0..valid)
            .filter(|row| {
                self.kernel.stability.mode == StabilityMode::Validated
                    || self
                        .kernel
                        .routing
                        .class(&points[row * dimension..(row + 1) * dimension])
                        == PrecisionClass::F64
            })
            .collect::<Vec<_>>();
        let input_len = selected
            .len()
            .checked_mul(self.kernel.input.len())
            .ok_or_else(|| {
                fail(KernelError::Parameters(
                    "batch input matrix is too large".into(),
                ))
            })?;
        let mut input = Vec::with_capacity(input_len);
        for &row in &selected {
            input.extend_from_slice(&points[row * dimension..(row + 1) * dimension]);
            input.extend_from_slice(&self.kernel.input[dimension..]);
        }
        let native_outputs = self.kernel.native_output_count();
        let mut primary = vec![0.0; selected.len() * native_outputs];
        if cancelled() {
            return Err(fail(KernelError::Cancelled));
        }
        let timings = match &mut self.kernel.backend {
            Backend::Real(kernel) => {
                let timings = kernel.evaluator.evaluate_batch(
                    &input,
                    &mut primary,
                    selected.len(),
                    self.kernel.input.len(),
                    native_outputs,
                );
                for timing in &timings {
                    kernel.f64_timing.add(*timing);
                }
                timings
            }
            Backend::Complex(kernel) => {
                kernel.evaluate_primary_batch(&input, &mut primary, selected.len())
            }
        };
        let primary = if let Some(projection) = &self.kernel.projection {
            let mut scattered = vec![0.0; selected.len() * outputs];
            for (source, target) in primary
                .chunks_exact(native_outputs)
                .zip(scattered.chunks_exact_mut(outputs))
            {
                for (value, index) in source.iter().zip(&projection.indices) {
                    target[*index] = *value;
                }
            }
            scattered
        } else {
            primary
        };
        let mut completed = Vec::with_capacity(rows);
        let mut next_primary = 0;
        for row in 0..rows {
            if cancelled() {
                return Err(BatchEvaluationError {
                    completed,
                    error: KernelError::Cancelled,
                });
            }
            let (raw, timing) = if selected.get(next_primary) == Some(&row) {
                let index = next_primary;
                next_primary += 1;
                (
                    Some(&primary[index * outputs..(index + 1) * outputs]),
                    timings[index],
                )
            } else {
                (None, EvaluatorTiming::default())
            };
            let before = self.evaluation_metrics();
            match self.evaluate_weighted_inner(
                &points[row * dimension..(row + 1) * dimension],
                weights[row],
                &mut output[row * outputs..(row + 1) * outputs],
                raw,
            ) {
                Ok(mut report) => {
                    report.precision.timings = self.evaluation_metrics().since(before);
                    report.precision.timings.f64.add(timing);
                    completed.push(report);
                }
                Err(error) => return Err(BatchEvaluationError { completed, error }),
            }
        }
        Ok(completed)
    }
}
