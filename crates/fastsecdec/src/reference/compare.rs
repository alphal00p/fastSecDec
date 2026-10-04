use super::*;
use crate::integration::VectorEstimate;
use std::collections::{BTreeMap, BTreeSet};

/// Compare the union of supplied keys. Pulls combine independent scalar
/// standard errors; no diagonal or joint covariance model is constructed.
pub fn compare(
    estimate: &VectorEstimate,
    reference: &ReferenceResult,
    context: &ComparisonContext,
) -> Result<ReferenceComparison> {
    context.validate_reference(reference)?;
    estimate
        .validate()
        .map_err(|error| ReferenceError::Invalid(error.to_string()))?;
    let n = estimate.orders.len();
    let mut estimates = BTreeMap::new();
    for index in 0..n {
        let key = CoefficientKey {
            order: estimate.orders[index],
            component: estimate.components[index],
        };
        let value = EstimatedCoefficient {
            value: estimate.mean[index],
            standard_error: estimate.standard_error[index],
        };
        if !value.value.is_finite()
            || !value.standard_error.is_finite()
            || value.standard_error < 0.0
        {
            return Err(ReferenceError::Invalid(format!(
                "invalid estimate for {key:?}"
            )));
        }
        if estimates.insert(key, value).is_some() {
            return Err(ReferenceError::Invalid(format!(
                "duplicate estimate key {key:?}"
            )));
        }
    }
    let references: BTreeMap<_, _> = reference
        .coefficients
        .iter()
        .map(|value| (value.key, value))
        .collect();
    let mut reasons = BTreeSet::new();
    if !estimate.production_complete {
        reasons.insert(IneligibilityReason::IncompleteProduction);
    }
    if reference.validation == ReferenceValidation::Unverified {
        reasons.insert(IneligibilityReason::UnverifiedReference);
    }
    for (compatibility, unknown, mismatch) in [
        (
            &context.normalization,
            IneligibilityReason::NormalizationUnconfirmed,
            IneligibilityReason::NormalizationMismatch,
        ),
        (
            &context.kinematics,
            IneligibilityReason::KinematicsUnconfirmed,
            IneligibilityReason::KinematicsMismatch,
        ),
    ] {
        match compatibility {
            Compatibility::Unknown => {
                reasons.insert(unknown);
            }
            Compatibility::Mismatch { detail } => {
                if detail.trim().is_empty() {
                    return Err(ReferenceError::Invalid("empty mismatch explanation".into()));
                }
                reasons.insert(mismatch);
            }
            Compatibility::Confirmed { basis } if basis.trim().is_empty() => {
                return Err(ReferenceError::Invalid(
                    "empty compatibility evidence".into(),
                ));
            }
            Compatibility::Confirmed { .. } => {}
        }
    }
    let unavailable_independence = match &context.independence {
        Independence::Unknown => {
            reasons.insert(IneligibilityReason::IndependenceUnconfirmed);
            Some(UnavailablePull::IndependenceUnconfirmed)
        }
        Independence::Correlated { detail } => {
            if detail.trim().is_empty() {
                return Err(ReferenceError::Invalid(
                    "empty correlation explanation".into(),
                ));
            }
            reasons.insert(IneligibilityReason::EstimatesCorrelated);
            Some(UnavailablePull::EstimatesCorrelated)
        }
        Independence::Independent { basis } => {
            if basis.trim().is_empty() {
                return Err(ReferenceError::Invalid(
                    "empty independence evidence".into(),
                ));
            }
            None
        }
    };
    let keys: BTreeSet<_> = estimates.keys().chain(references.keys()).copied().collect();
    let mut rows = Vec::with_capacity(keys.len());
    for key in keys {
        let actual = estimates.get(&key).copied();
        let expected = references.get(&key).copied();
        let mut row = ComparisonRow {
            key,
            estimate: actual,
            reference: expected.cloned(),
            difference: None,
            relative_difference: None,
            combined_standard_error: None,
            pull: Pull::Unavailable(UnavailablePull::MissingReference),
        };
        match (actual, expected) {
            (None, _) => {
                reasons.insert(IneligibilityReason::MissingEstimate);
                row.pull = Pull::Unavailable(UnavailablePull::MissingEstimate);
            }
            (_, None) => {
                reasons.insert(IneligibilityReason::MissingReference);
            }
            (Some(actual), Some(expected)) => {
                let difference = finite(actual.value - expected.value, key, "difference")?;
                row.difference = Some(difference);
                if expected.value != 0.0 {
                    row.relative_difference = Some(finite(
                        difference / expected.value.abs(),
                        key,
                        "relative difference",
                    )?);
                }
                let reference_error = match expected.uncertainty {
                    ReferenceUncertainty::Exact => Some(0.0),
                    ReferenceUncertainty::StandardError(error) => Some(error),
                    ReferenceUncertainty::Unknown => None,
                };
                row.pull = match (reference_error, unavailable_independence) {
                    (None, _) => {
                        reasons.insert(IneligibilityReason::UnknownReferenceUncertainty);
                        Pull::Unavailable(UnavailablePull::UnknownReferenceUncertainty)
                    }
                    (Some(_), Some(reason)) => Pull::Unavailable(reason),
                    (Some(reference_error), None) => {
                        let sigma = finite(
                            actual.standard_error.hypot(reference_error),
                            key,
                            "combined standard error",
                        )?;
                        row.combined_standard_error = Some(sigma);
                        if sigma == 0.0 {
                            Pull::ZeroCombinedError {
                                equal: difference == 0.0,
                            }
                        } else {
                            Pull::Value(finite(difference / sigma, key, "pull")?)
                        }
                    }
                };
            }
        }
        rows.push(row);
    }
    Ok(ReferenceComparison {
        provenance: reference.provenance.clone(),
        validation: reference.validation.clone(),
        context: context.clone(),
        eligibility: ComparisonEligibility {
            eligible: reasons.is_empty(),
            reasons: reasons.into_iter().collect(),
        },
        rows,
    })
}

fn finite(value: f64, key: CoefficientKey, quantity: &'static str) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ReferenceError::NumericRange { key, quantity })
    }
}

pub(super) fn validate_reference(reference: &ReferenceResult) -> Result<()> {
    reference.provenance.validate()?;
    if reference
        .kernel_content_id
        .as_ref()
        .is_some_and(|identity| identity.is_empty())
    {
        return Err(ReferenceError::Invalid(
            "empty reference kernel identity".into(),
        ));
    }
    if let ReferenceValidation::Checked { evidence } = &reference.validation
        && evidence.trim().is_empty()
    {
        return Err(ReferenceError::Invalid(
            "checked reference requires evidence".into(),
        ));
    }
    let mut keys = BTreeSet::new();
    for coefficient in &reference.coefficients {
        if !coefficient.value.is_finite()
            || matches!(coefficient.uncertainty, ReferenceUncertainty::StandardError(error) if !error.is_finite() || error < 0.0)
            || !keys.insert(coefficient.key)
        {
            return Err(ReferenceError::Invalid(format!(
                "invalid or duplicate reference coefficient {:?}",
                coefficient.key
            )));
        }
    }
    Ok(())
}
