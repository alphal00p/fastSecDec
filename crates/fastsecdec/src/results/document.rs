use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    format: String,
    version: u32,
    result: SavedIntegrationResult,
}

/// Strict versioned numerical transport; no artifact or symbolic initialization.
pub fn encode_result(result: &SavedIntegrationResult) -> Result<Vec<u8>> {
    result.validate()?;
    Ok(serde_json::to_vec(&Document {
        format: "fastsecdec-integration-result".into(),
        version: 1,
        result: result.clone(),
    })?)
}

pub fn read_result(bytes: &[u8]) -> Result<SavedIntegrationResult> {
    let document: Document = serde_json::from_slice(bytes)?;
    if document.format != "fastsecdec-integration-result" || document.version != 1 {
        return Err(ResultError::Invalid(
            "expected fastsecdec-integration-result version 1".into(),
        ));
    }
    document.result.validate()?;
    Ok(document.result)
}
