//! Numerical evaluator calls only; allocation, mapping and caller work are excluded.
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvaluatorTiming {
    /// Number of evaluated points, including replays and batched rows.
    pub calls: u64,
    pub nanoseconds: u64,
    /// Actual native matrix invocations. Historical scalar-only records are zero.
    pub matrix_invocations: u64,
    /// Points included in matrix invocations; the remaining calls were scalar.
    pub matrix_points: u64,
}
impl EvaluatorTiming {
    pub(super) fn record(&mut self, started: Instant) {
        self.calls = self.calls.saturating_add(1);
        self.nanoseconds = self
            .nanoseconds
            .saturating_add(u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX));
    }
    pub fn since(self, previous: Self) -> Self {
        Self {
            calls: self.calls.saturating_sub(previous.calls),
            nanoseconds: self.nanoseconds.saturating_sub(previous.nanoseconds),
            matrix_invocations: self
                .matrix_invocations
                .saturating_sub(previous.matrix_invocations),
            matrix_points: self.matrix_points.saturating_sub(previous.matrix_points),
        }
    }
    pub(super) fn add(&mut self, other: Self) {
        self.calls = self.calls.saturating_add(other.calls);
        self.nanoseconds = self.nanoseconds.saturating_add(other.nanoseconds);
        self.matrix_invocations = self
            .matrix_invocations
            .saturating_add(other.matrix_invocations);
        self.matrix_points = self.matrix_points.saturating_add(other.matrix_points);
    }
    pub fn seconds(self) -> f64 {
        self.nanoseconds as f64 * 1e-9
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvaluationTimings {
    pub f64: EvaluatorTiming,
    pub double_float: EvaluatorTiming,
    pub arbitrary: EvaluatorTiming,
    pub conditioning: EvaluatorTiming,
}
impl EvaluationTimings {
    pub fn since(self, previous: Self) -> Self {
        Self {
            f64: self.f64.since(previous.f64),
            double_float: self.double_float.since(previous.double_float),
            arbitrary: self.arbitrary.since(previous.arbitrary),
            conditioning: self.conditioning.since(previous.conditioning),
        }
    }
    pub fn total_nanoseconds(self) -> u64 {
        self.f64
            .nanoseconds
            .saturating_add(self.double_float.nanoseconds)
            .saturating_add(self.arbitrary.nanoseconds)
            .saturating_add(self.conditioning.nanoseconds)
    }
}
