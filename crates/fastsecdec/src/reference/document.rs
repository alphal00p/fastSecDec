use super::{ReferenceError, ReferenceResult, Result, read_historical_target};
use serde::{Deserialize, Serialize};

const FORMAT: &str = "fastsecdec-reference";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format: String,
    version: u32,
    reference: ReferenceResult,
}

/// Encode the native version-one transport envelope. Native library callers
/// can instead pass a `ReferenceResult` directly without any serialization.
pub fn encode_reference(reference: &ReferenceResult) -> Result<Vec<u8>> {
    reference.validate()?;
    Ok(serde_json::to_vec(&Document {
        format: FORMAT.into(),
        version: 1,
        reference: reference.clone(),
    })?)
}

/// Read either a native version-one envelope or the historical schema-one
/// target format. Mixed discriminators and unversioned objects are rejected;
/// importing a historical target never upgrades its validation or uncertainty.
pub fn read_reference(bytes: &[u8]) -> Result<ReferenceResult> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let object = value.as_object().ok_or_else(|| {
        ReferenceError::Invalid("reference document must be a versioned JSON object".into())
    })?;
    let native = ["format", "version", "reference"]
        .iter()
        .any(|field| object.contains_key(*field));
    let historical = [
        "schema_version",
        "orders",
        "coefficients",
        "standard_errors",
    ]
    .iter()
    .any(|field| object.contains_key(*field));
    if native && historical {
        return Err(ReferenceError::Invalid(
            "ambiguous reference document mixes native and historical fields".into(),
        ));
    }
    if native {
        if object.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT)
            || object.get("version").and_then(serde_json::Value::as_u64) != Some(1)
        {
            return Err(ReferenceError::Invalid("unsupported native reference format/version; expected fastsecdec-reference version 1".into()));
        }
        let document: Document = serde_json::from_slice(bytes)?;
        document.reference.validate()?;
        return Ok(document.reference);
    }
    if historical {
        return read_historical_target(bytes);
    }
    Err(ReferenceError::Invalid(
        "expected native format/version envelope or historical schema_version 1 target".into(),
    ))
}
