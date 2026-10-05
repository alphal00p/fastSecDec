//! Version three binds ordered native IR and the full numerical/semantic policy.
use super::{atom, parameter_names, parameters, validate_orders};
use crate::{
    kernel::{
        KernelError, KernelSet, PrecisionPolicy,
        cancellation::Cancellation,
        metadata::PortableMetadata,
        program::{self, SectorProgram},
    },
    status::CoefficientComponent,
};
use serde::{Deserialize, Serialize};
use symbolica::atom::AtomCore;

#[cfg(test)]
mod tests;

// Native evaluator serde is not a stable cross-revision interchange format.
// The local structural-validation patch preserves this upstream wire layout.
#[cfg(feature = "native")]
pub(super) const CODEC: &str = "symbolica-3.0.1@98794d0d7337ba2b08e4c046dde584ad7fc1ce10:exact-evaluator-schema-v1:serde-bincode-2-standard:v1";
#[cfg(feature = "native")]
pub(super) const COMPILER: &str = "symjit-2.26.4:O2:direct:horner-iterations=0";

#[cfg(feature = "native")]
const HASH_DOMAIN: &[u8] = b"fastsecdec-portable-kernel-v3:symbolica-3:symjit-2.26:f64";
#[cfg(feature = "portable")]
pub(super) const CODEC: &str = "symbolica-3.0.1@98794d0d7337ba2b08e4c046dde584ad7fc1ce10:exact-evaluator-schema-v1:serde-bincode-2-standard:v1:integer-malachite:float-astro";
#[cfg(feature = "portable")]
pub(super) const COMPILER: &str =
    "symbolica-3.0.1:interpreter:integer-malachite:float-astro:horner-iterations=0";
#[cfg(feature = "portable")]
const HASH_DOMAIN: &[u8] =
    b"fastsecdec-portable-kernel-v3:symbolica-3:interpreter:malachite:astro:f64";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    version: u32,
    program_codec: String,
    compiler_policy: String,
    orders: Vec<i32>,
    components: Vec<CoefficientComponent>,
    exact: Vec<String>,
    precision: PrecisionPolicy,
    sectors: Vec<PortableSector>,
    metadata: Option<PortableMetadata>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableSector {
    parameters: Vec<String>,
    program: Vec<u8>,
    cancellation_degree: usize,
    cancellation_terms: Option<Vec<Vec<usize>>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    content_id: String,
    payload: Payload,
}

// These views preserve the owned wire schema and its field order. Native program
// bytes and cancellation terms remain in their existing kernel owners.
#[derive(Serialize)]
struct PayloadRef<'a> {
    version: u32,
    program_codec: &'a str,
    compiler_policy: &'a str,
    orders: &'a [i32],
    components: &'a [CoefficientComponent],
    exact: Vec<String>,
    precision: &'a PrecisionPolicy,
    sectors: Vec<PortableSectorRef<'a>>,
    metadata: Option<PortableMetadata>,
}

#[derive(Serialize)]
struct PortableSectorRef<'a> {
    parameters: Vec<String>,
    program: &'a [u8],
    cancellation_degree: usize,
    cancellation_terms: Option<&'a [Vec<usize>]>,
}

#[derive(Serialize)]
struct ArtifactRef<'a, P> {
    content_id: &'a str,
    payload: &'a P,
}

fn content_id(payload: &impl Serialize) -> Result<String, KernelError> {
    let mut hash = blake3::Hasher::new();
    hash.update(HASH_DOMAIN);
    serde_json::to_writer(&mut hash, payload)?;
    Ok(hash.finalize().to_hex().to_string())
}

fn encoded(payload: &impl Serialize) -> Result<(String, Vec<u8>), KernelError> {
    let id = content_id(payload)?;
    let bytes = serde_json::to_vec(&ArtifactRef {
        content_id: &id,
        payload,
    })?;
    Ok((id, bytes))
}

fn component_layout(count: usize, complex: bool) -> Vec<CoefficientComponent> {
    (0..count)
        .flat_map(|_| {
            if complex {
                vec![CoefficientComponent::Real, CoefficientComponent::Imag]
            } else {
                vec![CoefficientComponent::Real]
            }
        })
        .collect()
}

pub(super) fn compiled(kernels: &KernelSet) -> Result<(String, Vec<u8>), KernelError> {
    encoded(&PayloadRef {
        version: 3,
        program_codec: CODEC,
        compiler_policy: COMPILER,
        orders: &kernels.coefficient_orders,
        components: &kernels.components,
        exact: kernels
            .exact_expressions
            .iter()
            .map(AtomCore::to_canonical_string)
            .collect(),
        precision: &kernels.precision,
        sectors: kernels
            .sectors
            .iter()
            .map(|sector| PortableSectorRef {
                parameters: parameter_names(&sector.parameters),
                program: &sector.program_bytes,
                cancellation_degree: sector.cancellation.degree(),
                cancellation_terms: sector.cancellation.terms(),
            })
            .collect(),
        metadata: kernels.metadata.as_ref().map(PortableMetadata::from_native),
    })
}

pub(super) fn generated(
    value: &crate::generation::GeneratedIntegral,
    precision: PrecisionPolicy,
) -> Result<Vec<u8>, KernelError> {
    precision.validate()?;
    let complex = value
        .sectors()
        .iter()
        .flat_map(|sector| sector.aliased_coefficients())
        .any(|c| !program::is_real(c))
        || value
            .exact_coefficients()
            .iter()
            .any(crate::kernel::has_complex_coefficients);
    let sectors = value
        .sectors()
        .iter()
        .map(|sector| {
            let program = program::build(
                sector.parameters().to_vec(),
                sector.aliased_coefficients(),
                Cancellation::new(
                    sector.cancellation_degree(),
                    Some(sector.cancellation_terms().to_vec()),
                    sector.dimension(),
                )?,
            )?;
            Ok(PortableSector {
                parameters: parameter_names(&program.parameters),
                program: program::encode(&program.exact)?,
                cancellation_degree: program.cancellation.degree(),
                cancellation_terms: program.cancellation.terms().map(<[Vec<usize>]>::to_vec),
            })
        })
        .collect::<Result<_, KernelError>>()?;
    Ok(encoded(&Payload {
        version: 3,
        program_codec: CODEC.into(),
        compiler_policy: COMPILER.into(),
        orders: value.orders().to_vec(),
        components: component_layout(value.orders().len(), complex),
        exact: value
            .exact_coefficients()
            .iter()
            .map(AtomCore::to_canonical_string)
            .collect(),
        precision,
        sectors,
        metadata: Some(PortableMetadata::from_native(value.metadata())),
    })?
    .1)
}

pub(super) fn load(bytes: &[u8]) -> Result<KernelSet, KernelError> {
    let artifact: Artifact = serde_json::from_slice(bytes)?;
    let payload = artifact.payload;
    if payload.version != 3
        || payload.program_codec != CODEC
        || payload.compiler_policy != COMPILER
        || payload.metadata.is_none()
    {
        return Err(KernelError::Artifact(
            "unsupported native program codec or compiler policy".into(),
        ));
    }
    if content_id(&payload)? != artifact.content_id {
        return Err(KernelError::Artifact(
            "kernel content identity mismatch".into(),
        ));
    }
    validate_orders(&payload.orders, payload.exact.len())?;
    payload.precision.validate()?;
    let use_complex = if payload.components == component_layout(payload.orders.len(), false) {
        false
    } else if payload.components == component_layout(payload.orders.len(), true) {
        true
    } else {
        return Err(KernelError::Artifact(
            "invalid native program component layout".into(),
        ));
    };
    let mut programs = Vec::with_capacity(payload.sectors.len());
    for sector in payload.sectors {
        let parameters = parameters(sector.parameters)?;
        if parameters.is_empty() {
            return Err(KernelError::Artifact(
                "native sector has no integration parameters".into(),
            ));
        }
        let cancellation = Cancellation::new(
            sector.cancellation_degree,
            sector.cancellation_terms,
            parameters.len(),
        )?;
        let exact = program::decode(&sector.program)?;
        if exact.get_input_len() != parameters.len()
            || exact.get_output_len() != payload.orders.len()
        {
            return Err(KernelError::Artifact(
                "native program input/output layout mismatch".into(),
            ));
        }
        programs.push(SectorProgram {
            parameters,
            exact,
            cancellation,
            // No serialized fact can suppress a precision check. Native decoded
            // programs use conservative facts unless their owner proves more.
            exact_zero: vec![false; payload.orders.len()],
            real_coefficients: vec![false; payload.orders.len()],
        });
    }
    let coordinates = programs
        .iter()
        .map(|p| p.parameters.clone())
        .collect::<Vec<_>>();
    let metadata = payload
        .metadata
        .map(|m| m.into_native(&coordinates))
        .transpose()?;
    let exact = payload
        .exact
        .into_iter()
        .map(atom)
        .collect::<Result<Vec<_>, _>>()?;
    if !use_complex && exact.iter().any(crate::kernel::has_complex_coefficients) {
        return Err(KernelError::Artifact(
            "complex exact offset in real output layout".into(),
        ));
    }
    let mut kernels = KernelSet::from_programs_for_load(
        payload.orders,
        programs,
        exact,
        payload.precision,
        metadata,
        use_complex,
    )?;
    kernels.content_id = artifact.content_id;
    kernels.portable_artifact = Some(bytes.to_vec());
    Ok(kernels)
}
