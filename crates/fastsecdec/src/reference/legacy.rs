//! Reader for the supplied historical target files; no trust is inferred.
use super::*;
use crate::status::CoefficientComponent;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    schema_version: u32,
    provenance: Provenance,
    orders: Vec<i32>,
    coefficients: Vec<ComplexValue>,
    standard_errors: Option<Vec<ComplexValue>>,
    validation_status: String,
}

#[derive(Deserialize)]
struct Provenance {
    repository: String,
    convention: String,
    revision: Option<String>,
    source_file: Option<String>,
    recorded_utc: Option<String>,
    engine: Option<String>,
    #[serde(flatten)]
    attributes: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComplexValue {
    re: f64,
    im: f64,
}

/// Read historical schema version one. Null errors remain unknown; numeric
/// zero errors remain reported standard errors, never declared exact values.
/// Every imported reference remains unverified regardless of its status prose.
pub fn read_historical_target(bytes: &[u8]) -> Result<ReferenceResult> {
    let target: Target = serde_json::from_slice(bytes)?;
    if target.schema_version != 1
        || target.orders.len() != target.coefficients.len()
        || target
            .standard_errors
            .as_ref()
            .is_some_and(|values| values.len() != target.orders.len())
    {
        return Err(ReferenceError::Invalid(
            "unsupported historical schema or inconsistent coefficient lengths".into(),
        ));
    }
    let mut coefficients = Vec::with_capacity(target.orders.len().saturating_mul(2));
    for (index, (&order, value)) in target.orders.iter().zip(&target.coefficients).enumerate() {
        let errors = target.standard_errors.as_ref().map(|errors| &errors[index]);
        for (component, value, error) in [
            (
                CoefficientComponent::Real,
                value.re,
                errors.map(|error| error.re),
            ),
            (
                CoefficientComponent::Imag,
                value.im,
                errors.map(|error| error.im),
            ),
        ] {
            coefficients.push(ReferenceCoefficient {
                key: CoefficientKey { order, component },
                value,
                uncertainty: error.map_or(
                    ReferenceUncertainty::Unknown,
                    ReferenceUncertainty::StandardError,
                ),
            });
        }
    }
    let p = target.provenance;
    let reference = ReferenceResult::new(
        coefficients,
        ReferenceProvenance {
            source: p.repository,
            convention: p.convention,
            revision: p.revision,
            location: p.source_file,
            recorded_utc: p.recorded_utc,
            engine: p.engine,
            notes: vec![target.validation_status],
            attributes: p.attributes,
        },
    );
    super::compare::validate_reference(&reference)?;
    Ok(reference)
}
