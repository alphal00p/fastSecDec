//! Original expression-only codecs retain their payload ordering and identities.
use super::{atom, parameters, validate_orders};
use crate::kernel::{
    KernelError, KernelSet, PrecisionPolicy, SectorExpressions, cancellation::Cancellation,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    version: u32,
    symjit_optimization: u8,
    orders: Vec<i32>,
    exact: Vec<String>,
    precision: PrecisionPolicy,
    sectors: Vec<PortableSector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<crate::kernel::metadata::PortableMetadata>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableSector {
    parameters: Vec<String>,
    coefficients: Vec<String>,
    cancellation_degree: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancellation_terms: Option<Vec<Vec<usize>>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    content_id: String,
    payload: Payload,
}

fn content_id(payload: &Payload) -> Result<String, KernelError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(if payload.version == 1 {
        b"fastsecdec-portable-kernel-v1:symbolica-3:symjit-2.26:f64"
    } else {
        b"fastsecdec-portable-kernel-v2:symbolica-3:symjit-2.26:f64"
    });
    hasher.update(&serde_json::to_vec(payload)?);
    Ok(hasher.finalize().to_hex().to_string())
}

pub(super) fn load(bytes: &[u8]) -> Result<KernelSet, KernelError> {
    let artifact: Artifact = serde_json::from_slice(bytes)?;
    let payload = artifact.payload;
    if !matches!(payload.version, 1 | 2)
        || payload.symjit_optimization != 2
        || (payload.version == 2) != payload.metadata.is_some()
    {
        return Err(KernelError::Artifact(
            "unsupported legacy artifact version or compiler policy".into(),
        ));
    }
    if content_id(&payload)? != artifact.content_id {
        return Err(KernelError::Artifact(
            "kernel content identity mismatch".into(),
        ));
    }
    validate_orders(&payload.orders, payload.exact.len())?;
    payload.precision.validate()?;
    let mut sectors = Vec::with_capacity(payload.sectors.len());
    for sector in payload.sectors {
        if sector.coefficients.len() != payload.orders.len() || sector.parameters.is_empty() {
            return Err(KernelError::Artifact("invalid sector output layout".into()));
        }
        let parameters = parameters(sector.parameters)?;
        let cancellation = Cancellation::new(
            sector.cancellation_degree,
            sector.cancellation_terms,
            parameters.len(),
        )?;
        sectors.push(SectorExpressions {
            parameters,
            coefficients: sector
                .coefficients
                .into_iter()
                .map(atom)
                .collect::<Result<_, _>>()?,
            cancellation,
        });
    }
    let coordinates = sectors
        .iter()
        .map(|sector| sector.parameters.clone())
        .collect::<Vec<_>>();
    let metadata = payload
        .metadata
        .map(|value| value.into_native(&coordinates))
        .transpose()?;
    let mut restored = KernelSet::from_expressions_for_load(
        payload.orders,
        sectors,
        payload
            .exact
            .into_iter()
            .map(atom)
            .collect::<Result<_, _>>()?,
        payload.precision,
        metadata,
    )?;
    // Historical checkpoints continue to name their original scientific bytes.
    restored.content_id = artifact.content_id;
    restored.portable_artifact = Some(bytes.to_vec());
    Ok(restored)
}
