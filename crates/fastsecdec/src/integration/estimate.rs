use fastsecdec_qmc::QmcEstimate;
use numerica::domains::float::{DoubleFloat, RealLike};
use serde::{Deserialize, Serialize};

use super::{CoefficientComponent, IntegrationError, Result, Tolerance};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VectorEstimate {
    pub orders: Vec<i32>,
    pub components: Vec<CoefficientComponent>,
    pub mean: Vec<f64>,
    pub standard_error: Vec<f64>,
    /// Complete row-major covariance, including correlations between orders.
    pub covariance_of_mean: Vec<f64>,
    /// Only a complete production allocation may certify an accuracy stop.
    pub production_complete: bool,
}

impl VectorEstimate {
    /// Validate the native representation, not convergence or positive
    /// semidefiniteness. No covariance or uncertainty is reconstructed.
    pub fn validate(&self) -> Result<()> {
        let n = self.orders.len();
        if n == 0
            || self.components.len() != n
            || self.mean.len() != n
            || self.standard_error.len() != n
            || n.checked_mul(n) != Some(self.covariance_of_mean.len())
            || self.covariance_of_mean.iter().any(|v| !v.is_finite())
            || self.mean.iter().any(|v| !v.is_finite())
            || self
                .standard_error
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
            || self
                .orders
                .iter()
                .zip(&self.components)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != n
        {
            return Err(IntegrationError::Invalid(
                "inconsistent, duplicate or nonfinite estimate layout/values".into(),
            ));
        }
        Ok(())
    }

    pub fn meets(&self, tolerance: Tolerance) -> Result<bool> {
        tolerance.validate()?;
        let n = self.orders.len();
        if n == 0
            || self.components.len() != n
            || self.mean.len() != n
            || self.standard_error.len() != n
            || n.checked_mul(n) != Some(self.covariance_of_mean.len())
        {
            return Err(IntegrationError::Invalid(
                "estimate has inconsistent vector dimensions".into(),
            ));
        }
        Ok(self.production_complete
            && self
                .mean
                .iter()
                .zip(&self.standard_error)
                .all(|(mean, error)| {
                    mean.is_finite()
                        && error.is_finite()
                        && *error >= 0.0
                        && *error <= tolerance.absolute.max(tolerance.relative * mean.abs())
                }))
    }

    pub(crate) fn from_qmc(
        orders: &[i32],
        components: &[CoefficientComponent],
        value: QmcEstimate,
        complete: bool,
    ) -> Self {
        Self {
            orders: orders.to_vec(),
            components: components.to_vec(),
            mean: value.mean,
            standard_error: value.standard_error,
            covariance_of_mean: value.covariance_of_mean,
            production_complete: complete,
        }
    }
}

/// Reuse Numerica's compensated 106-bit arithmetic for cross-sector sums.
/// This is outside the hot point loop; no independent summation engine is needed.
pub(crate) fn precise_sum(values: impl IntoIterator<Item = f64>) -> Result<f64> {
    let value = values
        .into_iter()
        .fold(DoubleFloat::from(0.0), |sum, x| sum + DoubleFloat::from(x))
        .to_f64();
    if value.is_finite() {
        Ok(value)
    } else {
        Err(IntegrationError::NumericRange)
    }
}
