use super::*;
use crate::{
    reference::{
        CoefficientKey, ReferenceCoefficient, ReferenceResult, ReferenceUncertainty, compare,
    },
    status::{IntegrationStage, StoppingReason},
};

impl SavedIntegrationResult {
    /// Compare retained original inputs, never saved rendered comparison rows.
    pub fn comparison(&self) -> Result<ResultComparison> {
        self.validate()?;
        let unavailable = if self.scope != ResultScope::FullIntegral {
            Some(ResultComparisonUnavailable::SelectedScope)
        } else if self.contributions.stage == IntegrationStage::Pilot {
            Some(ResultComparisonUnavailable::Pilot)
        } else {
            match self.stopping_reason {
                StoppingReason::Cancelled => Some(ResultComparisonUnavailable::Cancelled),
                StoppingReason::NumericalFailure(_) => {
                    Some(ResultComparisonUnavailable::NumericalFailure)
                }
                _ => None,
            }
        };
        if let Some(reason) = unavailable {
            return Ok(ResultComparison::Unavailable(reason));
        }
        let Some(stored) = &self.stored_reference else {
            return Ok(ResultComparison::Unavailable(
                ResultComparisonUnavailable::NoReference,
            ));
        };
        let Some(estimate) = &self.contributions.total else {
            return Ok(ResultComparison::Unavailable(
                ResultComparisonUnavailable::NoEstimate,
            ));
        };
        match compare(estimate, &stored.reference, &stored.context) {
            Ok(comparison) => Ok(ResultComparison::Compared(Box::new(comparison))),
            Err(crate::reference::ReferenceError::NumericRange { key, quantity }) => Ok(
                ResultComparison::Unavailable(ResultComparisonUnavailable::NumericRange {
                    key,
                    quantity: quantity.into(),
                }),
            ),
            Err(error) => Err(error.into()),
        }
    }

    /// Explicit selection; absent sources never cause an automatic fallback.
    /// Stored targets retain their evidence even when the computed run failed.
    pub fn reference(&self, selection: ResultReferenceSelection) -> Result<ReferenceResult> {
        self.validate()?;
        if selection == ResultReferenceSelection::StoredReference {
            return self
                .stored_reference
                .as_ref()
                .map(|s| s.reference.clone())
                .ok_or(ResultError::MissingReferenceSource);
        }
        if self.scope != ResultScope::FullIntegral {
            return Err(ResultError::SelectedScope);
        }
        if matches!(
            self.stopping_reason,
            StoppingReason::Cancelled | StoppingReason::NumericalFailure(_)
        ) || !self.production_complete()
        {
            return Err(ResultError::IneligibleEstimate(
                "failed, cancelled, pilot or incomplete production".into(),
            ));
        }
        let estimate = self
            .contributions
            .total
            .as_ref()
            .ok_or(ResultError::MissingReferenceSource)?;
        if !estimate.production_complete {
            return Err(ResultError::IneligibleEstimate(
                "total estimate is incomplete".into(),
            ));
        }
        Ok(ReferenceResult {
            coefficients: estimate
                .orders
                .iter()
                .zip(&estimate.components)
                .enumerate()
                .map(|(i, (&order, &component))| ReferenceCoefficient {
                    key: CoefficientKey { order, component },
                    value: estimate.mean[i],
                    uncertainty: ReferenceUncertainty::StandardError(estimate.standard_error[i]),
                })
                .collect(),
            provenance: self.provenance.clone(),
            validation: self.validation.clone(),
            kernel_content_id: Some(self.manifest.kernel_content_id.clone()),
        })
    }
}
