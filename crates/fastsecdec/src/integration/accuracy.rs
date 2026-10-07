use super::{CoefficientComponent, IntegrationError, Result, Tolerance, VectorEstimate};
use serde::{Deserialize, Serialize};

/// Select stopping evidence without discarding any coefficient or covariance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "target", content = "order", rename_all = "snake_case")]
pub enum AccuracyTarget {
    #[default]
    AllComponents,
    LaurentOrder(i32),
}

impl AccuracyTarget {
    pub fn is_default(&self) -> bool {
        *self == Self::AllComponents
    }
    pub fn validate_layout(&self, orders: &[i32]) -> Result<()> {
        if let Self::LaurentOrder(order) = self
            && !orders.contains(order)
        {
            return Err(IntegrationError::Invalid(format!(
                "accuracy target epsilon^{order} is absent from the coefficient layout"
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccuracyAssessment {
    pub target: AccuracyTarget,
    pub reached: bool,
    /// Present for a selected coefficient; its Euclidean complex norm.
    pub magnitude: Option<f64>,
    /// RMS complex-vector error, sqrt(trace of the selected covariance block).
    pub standard_error: Option<f64>,
    /// Representable tolerance bound (saturated at f64::MAX on overflow).
    pub threshold: Option<f64>,
}

impl VectorEstimate {
    pub fn assess_accuracy(
        &self,
        target: AccuracyTarget,
        tolerance: Tolerance,
    ) -> Result<AccuracyAssessment> {
        self.validate()?;
        tolerance.validate()?;
        target.validate_layout(&self.orders)?;
        if target == AccuracyTarget::AllComponents {
            return Ok(AccuracyAssessment {
                target,
                reached: self.meets(tolerance)?,
                magnitude: None,
                standard_error: None,
                threshold: None,
            });
        }
        let AccuracyTarget::LaurentOrder(order) = target else {
            unreachable!()
        };
        let mut magnitude = 0.0_f64;
        let mut error = 0.0_f64;
        for (i, (&candidate, component)) in self.orders.iter().zip(&self.components).enumerate() {
            if candidate != order {
                continue;
            }
            match component {
                CoefficientComponent::Real | CoefficientComponent::Imag => {}
            }
            let variance = self.covariance_of_mean[i * self.orders.len() + i];
            if variance < 0.0 {
                return Err(IntegrationError::Invalid(
                    "negative coefficient variance".into(),
                ));
            }
            magnitude = magnitude.hypot(self.mean[i]);
            error = error.hypot(variance.sqrt());
        }
        if !magnitude.is_finite() || !error.is_finite() {
            return Err(IntegrationError::NumericRange);
        }
        let threshold = tolerance
            .absolute
            .max(tolerance.relative * magnitude)
            .min(f64::MAX);
        Ok(AccuracyAssessment {
            target,
            reached: self.production_complete && error <= threshold,
            magnitude: Some(magnitude),
            standard_error: Some(error),
            threshold: Some(threshold),
        })
    }
    pub fn meets_target(&self, target: AccuracyTarget, tolerance: Tolerance) -> Result<bool> {
        Ok(self.assess_accuracy(target, tolerance)?.reached)
    }
}
