//! Optional operational observations, independent of contour mathematics and checks.
use crate::status::DiagnosticsOverflow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContourDiagnosticsMode {
    #[default]
    Disabled,
    Aggregate,
}

/// Approximate finite positive centres, not certified bounds. Missing values
/// include overflow, underflow and quantities below the display resolution.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourDiagnosticRange {
    pub count: u64,
    pub unavailable: u64,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}
impl ContourDiagnosticRange {
    pub(crate) fn record(&mut self, value: Option<f64>) -> Result<(), DiagnosticsOverflow> {
        if let Some(value) = value.filter(|value| value.is_finite() && *value > 0.) {
            self.count = self.count.checked_add(1).ok_or(DiagnosticsOverflow)?;
            self.minimum = Some(self.minimum.map_or(value, |old| old.min(value)));
            self.maximum = Some(self.maximum.map_or(value, |old| old.max(value)));
        } else {
            self.unavailable = self.unavailable.checked_add(1).ok_or(DiagnosticsOverflow)?;
        }
        Ok(())
    }
    fn merge(&mut self, other: &Self) -> Result<(), DiagnosticsOverflow> {
        self.count = self
            .count
            .checked_add(other.count)
            .ok_or(DiagnosticsOverflow)?;
        self.unavailable = self
            .unavailable
            .checked_add(other.unavailable)
            .ok_or(DiagnosticsOverflow)?;
        if let Some(value) = other.minimum {
            self.minimum = Some(self.minimum.map_or(value, |old| old.min(value)));
        }
        if let Some(value) = other.maximum {
            self.maximum = Some(self.maximum.map_or(value, |old| old.max(value)));
        }
        Ok(())
    }
}

/// Actual executed work, including discarded attempts and repeated native
/// callbacks. These counters must not be interpreted as accepted sample counts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourRuntimeWork {
    pub callback_calls: u64,
    pub callback_failures: u64,
    pub solver_calls: u64,
    pub solver_successes: u64,
    /// Partial iteration/evaluation counts of failed solves are unknown.
    pub solver_failures: u64,
    pub closed_form_calls: u64,
    pub solver_iterations: u64,
    pub solver_evaluations: u64,
    pub maximum_solver_iterations: u64,
    pub maximum_solver_evaluations: u64,
    pub solver_numerical_zero: u64,
    pub solver_bracket_width: u64,
    pub solver_newton_correction: u64,
    pub correction_evaluations: u64,
    pub maximum_bits: u32,
    pub strength: ContourDiagnosticRange,
    pub normalized_displacement: ContourDiagnosticRange,
    pub physical_displacement: ContourDiagnosticRange,
}
impl ContourRuntimeWork {
    /// Atomic checked merge; overflow is operational, never a numerical failure.
    pub fn merge(&mut self, other: &Self) -> Result<(), DiagnosticsOverflow> {
        let mut next = self.clone();
        macro_rules! add { ($($field:ident),* $(,)?) => {$({
            next.$field = next.$field.checked_add(other.$field).ok_or(DiagnosticsOverflow)?;
        })*}; }
        add!(
            callback_calls,
            callback_failures,
            solver_calls,
            solver_successes,
            solver_failures,
            closed_form_calls,
            solver_iterations,
            solver_evaluations,
            correction_evaluations,
            solver_numerical_zero,
            solver_bracket_width,
            solver_newton_correction
        );
        next.maximum_solver_iterations = next
            .maximum_solver_iterations
            .max(other.maximum_solver_iterations);
        next.maximum_solver_evaluations = next
            .maximum_solver_evaluations
            .max(other.maximum_solver_evaluations);
        next.maximum_bits = next.maximum_bits.max(other.maximum_bits);
        next.strength.merge(&other.strength)?;
        next.normalized_displacement
            .merge(&other.normalized_displacement)?;
        next.physical_displacement
            .merge(&other.physical_displacement)?;
        *self = next;
        Ok(())
    }
}

/// Phase-separated observations. Evaluation includes precision retries and
/// discarded batches; caller-owned integration reports distinguish adaptation
/// from production. Exact binding and independent pilots never count as samples.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContourRuntimeReport {
    pub evaluation: ContourRuntimeWork,
    pub conditioning: ContourRuntimeWork,
    pub preparation: ContourRuntimeWork,
    pub exact: ContourRuntimeWork,
    pub pilot: ContourRuntimeWork,
}
impl ContourRuntimeReport {
    pub fn merge(&mut self, other: &Self) -> Result<(), DiagnosticsOverflow> {
        let mut next = self.clone();
        next.evaluation.merge(&other.evaluation)?;
        next.conditioning.merge(&other.conditioning)?;
        next.preparation.merge(&other.preparation)?;
        next.exact.merge(&other.exact)?;
        next.pilot.merge(&other.pilot)?;
        *self = next;
        Ok(())
    }
}
