//! Caller-owned replay of suspicious weighted samples. This growth heuristic
//! is a diagnostic safeguard, not a proof of accuracy or an integration rule.
use super::{KernelError, KernelSet, PrecisionReport, SectorKernel};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPolicy {
    /// Replay after this multiplicative growth over a previously accepted
    /// weighted maximum. Values greater than one avoid replaying constants.
    pub growth_factor: f64,
    /// Minimum starting precision of a forced two-precision native comparison.
    pub minimum_bits: u32,
}

impl Default for ReplayPolicy {
    fn default() -> Self {
        Self {
            growth_factor: 16.0,
            minimum_bits: 128,
        }
    }
}

impl ReplayPolicy {
    pub fn validate(&self) -> Result<(), KernelError> {
        if !self.growth_factor.is_finite() || self.growth_factor <= 1.0 || self.minimum_bits < 64 {
            return Err(KernelError::Replay("invalid weighted replay policy".into()));
        }
        Ok(())
    }
}

/// Serializable, identity-bound state. A merge is idempotent: maxima from the
/// same prior observations may safely be shared between independently owned
/// workers without double-counting numerical samples or diagnostic counters.
/// Merge only the state of successfully submitted complete work packages.
/// Replay choices depend on observation order; changing worker scheduling can
/// change rounding within the precision policy, so bitwise invariance is not
/// promised across different schedules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayState {
    version: u32,
    content_id: String,
    sector: usize,
    policy: ReplayPolicy,
    maximum_absolute_weighted: Vec<f64>,
    verified: bool,
}

impl ReplayState {
    pub fn maxima(&self) -> &[f64] {
        &self.maximum_absolute_weighted
    }
    pub fn verified(&self) -> bool {
        self.verified
    }
    pub fn policy(&self) -> &ReplayPolicy {
        &self.policy
    }

    fn validate(&self) -> Result<(), KernelError> {
        self.policy.validate()?;
        if self.version != 1
            || self.content_id.is_empty()
            || self.maximum_absolute_weighted.is_empty()
            || self.maximum_absolute_weighted.iter().any(|value| {
                !value.is_finite() || *value < 0.0 || (!self.verified && *value != 0.0)
            })
        {
            return Err(KernelError::Replay("invalid weighted replay state".into()));
        }
        Ok(())
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), KernelError> {
        self.validate()?;
        other.validate()?;
        if self.content_id != other.content_id
            || self.sector != other.sector
            || self.policy != other.policy
            || self.maximum_absolute_weighted.len() != other.maximum_absolute_weighted.len()
        {
            return Err(KernelError::Replay(
                "weighted replay identity, layout or policy mismatch".into(),
            ));
        }
        for (left, right) in self
            .maximum_absolute_weighted
            .iter_mut()
            .zip(&other.maximum_absolute_weighted)
        {
            *left = left.max(*right);
        }
        self.verified |= other.verified;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ReplayReport {
    pub precision: PrecisionReport,
    /// First observation or growth relative to previously accepted maxima.
    pub weighted_check: bool,
    /// An additional whole-vector precision evaluation actually took place.
    pub replayed: bool,
}

pub struct WeightedEvaluationContext {
    kernel: SectorKernel,
    state: ReplayState,
}

impl WeightedEvaluationContext {
    pub fn dimension(&self) -> usize {
        self.kernel.dimension()
    }
    pub fn output_count(&self) -> usize {
        self.kernel.output_count()
    }
    pub fn state(&self) -> &ReplayState {
        &self.state
    }
    pub fn merge_state(&mut self, state: &ReplayState) -> Result<(), KernelError> {
        self.state.merge(state)
    }

    /// Return coefficients with `weight` already applied exactly once. Use an
    /// explicitly weighted worker callback; do not multiply the result again.
    /// Native precision rescue applies the weight before conversion to f64.
    pub fn evaluate_weighted(
        &mut self,
        point: &[f64],
        weight: f64,
        output: &mut [f64],
    ) -> Result<ReplayReport, KernelError> {
        if !weight.is_finite() || weight < 0.0 {
            return Err(KernelError::InvalidWeight);
        }
        let mut precision = self.kernel.evaluate_scaled(point, output, weight)?;
        if weight == 0.0 {
            return Ok(ReplayReport {
                precision,
                weighted_check: false,
                replayed: false,
            });
        }
        let weighted_check = !self.state.verified
            || output
                .iter()
                .zip(&self.state.maximum_absolute_weighted)
                .any(|(value, maximum)| {
                    let magnitude = value.abs();
                    if *maximum == 0.0 {
                        magnitude > 0.0
                    } else {
                        magnitude >= *maximum * self.state.policy.growth_factor
                    }
                });
        let minimum = self
            .kernel
            .precision
            .initial_bits
            .max(self.state.policy.minimum_bits);
        let replayed =
            weighted_check && !(precision.rescued && precision.bits >= minimum.saturating_mul(2));
        if replayed {
            precision = self.kernel.replay_scaled(point, output, weight, minimum)?;
        }
        if output.iter().any(|value| !value.is_finite()) {
            return Err(KernelError::NonFinite);
        }
        // Only accepted, finite values influence future decisions.
        for (maximum, value) in self.state.maximum_absolute_weighted.iter_mut().zip(output) {
            *maximum = maximum.max(value.abs());
        }
        self.state.verified = true;
        Ok(ReplayReport {
            precision,
            weighted_check,
            replayed,
        })
    }
}

impl KernelSet {
    /// Create identity-bound checkpoint state without cloning evaluator IR.
    pub fn replay_state(
        &self,
        sector: usize,
        policy: ReplayPolicy,
    ) -> Result<ReplayState, KernelError> {
        policy.validate()?;
        let kernel = self
            .sectors
            .get(sector)
            .ok_or_else(|| KernelError::Replay("unknown sector".into()))?;
        if policy.minimum_bits > kernel.precision.max_bits / 2 {
            return Err(KernelError::Replay(
                "replay precision exceeds the kernel policy".into(),
            ));
        }
        Ok(ReplayState {
            version: 1,
            content_id: self.content_id.clone(),
            sector,
            policy,
            maximum_absolute_weighted: vec![0.0; kernel.output_count()],
            verified: false,
        })
    }

    pub fn validate_replay_state(
        &self,
        sector: usize,
        policy: &ReplayPolicy,
        state: &ReplayState,
    ) -> Result<(), KernelError> {
        self.replay_state(sector, policy.clone())?.merge(state)
    }

    pub fn evaluation_context(
        &self,
        sector: usize,
        policy: ReplayPolicy,
    ) -> Result<WeightedEvaluationContext, KernelError> {
        Ok(WeightedEvaluationContext {
            state: self.replay_state(sector, policy)?,
            kernel: self.sectors[sector].try_clone()?,
        })
    }

    pub fn restore_evaluation_context(
        &self,
        sector: usize,
        policy: ReplayPolicy,
        state: &ReplayState,
    ) -> Result<WeightedEvaluationContext, KernelError> {
        let mut context = self.evaluation_context(sector, policy)?;
        context.merge_state(state)?;
        Ok(context)
    }
}
