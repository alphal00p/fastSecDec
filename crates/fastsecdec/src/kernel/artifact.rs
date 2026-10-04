use super::{
    KernelError, KernelSet, PrecisionPolicy, SectorExpressions, cancellation::Cancellation,
};
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore, AtomView};

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
    metadata: Option<super::metadata::PortableMetadata>,
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

pub(super) fn atom(expression: String) -> Result<Atom, KernelError> {
    Atom::parse(expression, "fastsecdec::artifact", Default::default())
        .map_err(KernelError::Artifact)
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

impl crate::generation::GeneratedIntegral {
    /// Save portable kernel expressions before compilation. Loading with
    /// `KernelSet::from_bytes` validates and compiles the same O2 artifact.
    pub fn to_kernel_bytes(&self, precision: PrecisionPolicy) -> Result<Vec<u8>, KernelError> {
        precision.validate()?;
        let payload = Payload {
            version: 2,
            metadata: Some(super::metadata::PortableMetadata::from_native(
                self.metadata(),
            )),
            symjit_optimization: 2,
            orders: self.orders().to_vec(),
            exact: self
                .exact_coefficients()
                .iter()
                .map(AtomCore::to_canonical_string)
                .collect(),
            precision,
            sectors: self
                .sectors()
                .iter()
                .map(|sector| PortableSector {
                    parameters: sector
                        .parameters()
                        .iter()
                        .map(|p| Atom::var(*p).to_canonical_string())
                        .collect(),
                    coefficients: sector
                        .coefficients()
                        .iter()
                        .map(AtomCore::to_canonical_string)
                        .collect(),
                    cancellation_degree: sector.cancellation_degree(),
                    cancellation_terms: Some(sector.cancellation_terms().to_vec()),
                })
                .collect(),
        };
        Ok(serde_json::to_vec(&Artifact {
            content_id: content_id(&payload)?,
            payload,
        })?)
    }
}

impl KernelSet {
    fn payload(&self) -> Payload {
        Payload {
            version: if self.metadata.is_some() { 2 } else { 1 },
            metadata: self
                .metadata
                .as_ref()
                .map(super::metadata::PortableMetadata::from_native),
            symjit_optimization: 2,
            orders: self.coefficient_orders.clone(),
            exact: self
                .exact_expressions
                .iter()
                .map(AtomCore::to_canonical_string)
                .collect(),
            precision: self.precision.clone(),
            sectors: self
                .sectors
                .iter()
                .map(|sector| PortableSector {
                    parameters: sector
                        .parameters
                        .iter()
                        .map(|p| Atom::var(*p).to_canonical_string())
                        .collect(),
                    coefficients: sector
                        .coefficients
                        .iter()
                        .map(AtomCore::to_canonical_string)
                        .collect(),
                    cancellation_degree: sector.cancellation.degree(),
                    cancellation_terms: sector.cancellation.terms().map(<[Vec<usize>]>::to_vec),
                })
                .collect(),
        }
    }

    pub(super) fn compute_content_id(&self) -> Result<String, KernelError> {
        content_id(&self.payload())
    }

    /// Save expressions, metadata and numerical policy. Executable machine code
    /// is deliberately absent; loading instantiates portable O2 kernels locally.
    pub fn to_bytes(&self) -> Result<Vec<u8>, KernelError> {
        Ok(serde_json::to_vec(&Artifact {
            content_id: self.content_id.clone(),
            payload: self.payload(),
        })?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KernelError> {
        let artifact: Artifact = serde_json::from_slice(bytes)?;
        let payload = artifact.payload;
        if !matches!(payload.version, 1 | 2)
            || payload.symjit_optimization != 2
            || (payload.version == 2) != payload.metadata.is_some()
        {
            return Err(KernelError::Artifact(
                "unsupported artifact version or compiler policy".into(),
            ));
        }
        if content_id(&payload)? != artifact.content_id {
            return Err(KernelError::Artifact(
                "kernel content identity mismatch".into(),
            ));
        }
        if payload.orders.is_empty()
            || payload
                .orders
                .windows(2)
                .any(|pair| pair[0].checked_add(1) != Some(pair[1]))
            || payload.exact.len() != payload.orders.len()
        {
            return Err(KernelError::Artifact(
                "invalid Laurent output layout".into(),
            ));
        }
        let mut sectors = Vec::with_capacity(payload.sectors.len());
        for sector in payload.sectors {
            if sector.coefficients.len() != payload.orders.len() || sector.parameters.is_empty() {
                return Err(KernelError::Artifact("invalid sector output layout".into()));
            }
            let parameters = sector
                .parameters
                .into_iter()
                .map(|value| {
                    let value = atom(value)?;
                    let AtomView::Var(variable) = value.as_view() else {
                        return Err(KernelError::Artifact(
                            "sector parameter is not a symbol".into(),
                        ));
                    };
                    Ok(variable.get_symbol())
                })
                .collect::<Result<Vec<_>, _>>()?;
            if parameters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != parameters.len()
            {
                return Err(KernelError::Artifact("duplicate sector parameters".into()));
            }
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
                    .collect::<Result<Vec<_>, _>>()?,
                cancellation,
            });
        }
        let metadata = payload
            .metadata
            .map(|metadata| metadata.into_native(&sectors))
            .transpose()?;
        let restored = Self::from_expressions(
            payload.orders,
            sectors,
            payload
                .exact
                .into_iter()
                .map(atom)
                .collect::<Result<Vec<_>, _>>()?,
            payload.precision,
            metadata,
        )?;
        if restored.content_id != artifact.content_id {
            return Err(KernelError::Artifact(
                "kernel content identity mismatch".into(),
            ));
        }
        Ok(restored)
    }
}
