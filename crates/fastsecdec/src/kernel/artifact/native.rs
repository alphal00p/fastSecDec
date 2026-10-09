//! Version three binds ordered native IR and the full numerical/semantic policy.
use super::{atom, parameters, validate_orders};
use crate::{
    kernel::{
        CompilationSettings, KernelError, KernelLoadOptions, KernelSet, PrecisionPolicy,
        cancellation::Cancellation,
        metadata::PortableMetadata,
        program::{self, SectorProgram},
    },
    status::CoefficientComponent,
};
use serde::{Deserialize, Serialize};

pub(super) mod literal_zero;
#[cfg(test)]
mod tests;

// Historical compatible native evaluator wire-layout identifier, not the linked
// Symbolica source revision. Keep it stable while that serialization layout is
// unchanged; actual dependency provenance is recorded separately by Cargo/CLI.
#[cfg(feature = "native")]
pub(super) const CODEC: &str = "symbolica-3.0.1@98794d0d7337ba2b08e4c046dde584ad7fc1ce10:exact-evaluator-schema-v1:serde-bincode-2-standard:v1";
#[cfg(feature = "native")]
fn policy_prefix() -> String {
    format!(
        "symjit-version-code={}:O2",
        crate::kernel::symjit_version_code()
    )
}
#[cfg(feature = "portable")]
fn policy_prefix() -> String {
    "symbolica-3.0.1:interpreter:integer-malachite:float-astro".into()
}

pub(super) fn compiler_policy_with_settings(settings: CompilationSettings) -> String {
    let prefix = policy_prefix();
    #[cfg(feature = "native")]
    let prefix = if settings.backend.is_eager() {
        EAGER_POLICY_PREFIX.into()
    } else {
        prefix
    };
    if settings == CompilationSettings::legacy() {
        // Keep historical sector identities stable as well as the original envelope.
        #[cfg(feature = "portable")]
        return format!("{prefix}:horner-iterations=0");
        #[cfg(feature = "native")]
        format!("{prefix}:direct:horner-iterations=0")
    } else {
        format!(
            "{prefix}:evaluator-settings-v1:{}",
            serde_json::to_string(&settings).expect("scalar compilation settings serialize")
        )
    }
}

#[cfg(test)]
pub(super) fn compiler_policy() -> &'static str {
    static POLICY: std::sync::LazyLock<String> =
        std::sync::LazyLock::new(|| compiler_policy_with_settings(CompilationSettings::default()));
    POLICY.as_str()
}

fn parse_policy(policy: &str, prefix: &str) -> Option<CompilationSettings> {
    #[cfg(feature = "portable")]
    let legacy = format!("{prefix}:horner-iterations=0");
    #[cfg(feature = "native")]
    let legacy = format!("{prefix}:direct:horner-iterations=0");
    if policy == legacy {
        return Some(CompilationSettings::legacy());
    }
    let settings = policy.strip_prefix(&format!("{prefix}:evaluator-settings-v1:"))?;
    let settings: CompilationSettings = serde_json::from_str(settings).ok()?;
    settings.validate().ok()?;
    Some(settings)
}

pub(super) fn settings_from_policy(policy: &str) -> Option<CompilationSettings> {
    #[cfg(feature = "native")]
    if crate::kernel::symjit_version_code() == 22604
        && policy == "symjit-2.26.4:O2:direct:horner-iterations=0"
    {
        return Some(CompilationSettings::legacy());
    }
    #[cfg(feature = "native")]
    {
        if let Some(settings) = parse_policy(policy, EAGER_POLICY_PREFIX) {
            return settings.backend.is_eager().then_some(settings);
        }
        parse_policy(policy, &policy_prefix()).filter(|settings| !settings.backend.is_eager())
    }
    #[cfg(feature = "portable")]
    parse_policy(policy, &policy_prefix())
}

#[cfg(feature = "native")]
const EAGER_POLICY_PREFIX: &str = "symbolica-3.0.1:interpreter:integer-gmp:float-mpfr";

#[cfg(all(test, feature = "native"))]
fn compiler_policy_matches(policy: &str, version_code: usize) -> bool {
    parse_policy(policy, &format!("symjit-version-code={version_code}:O2")).is_some()
        || (version_code == 22604 && policy == "symjit-2.26.4:O2:direct:horner-iterations=0")
}

#[cfg(feature = "native")]
// Historical v3 hash domain; compiler_policy binds the actual backend separately.
const HASH_DOMAIN: &[u8] = b"fastsecdec-portable-kernel-v3:symbolica-3:symjit-2.26:f64";
#[cfg(feature = "portable")]
pub(super) const CODEC: &str = "symbolica-3.0.1@98794d0d7337ba2b08e4c046dde584ad7fc1ce10:exact-evaluator-schema-v1:serde-bincode-2-standard:v1:integer-malachite:float-astro";
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
#[cfg(test)]
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

#[cfg(test)]
#[derive(Serialize)]
struct PortableSectorRef<'a> {
    parameters: Vec<String>,
    program: &'a [u8],
    cancellation_degree: usize,
    cancellation_terms: Option<&'a [Vec<usize>]>,
}

#[cfg(test)]
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

#[cfg(test)]
fn encoded(payload: &impl Serialize) -> Result<(String, Vec<u8>), KernelError> {
    let id = content_id(payload)?;
    let bytes = serde_json::to_vec(&ArtifactRef {
        content_id: &id,
        payload,
    })?;
    Ok((id, bytes))
}

pub(super) fn component_layout(count: usize, complex: bool) -> Vec<CoefficientComponent> {
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

pub(super) fn load(
    bytes: &[u8],
    options: KernelLoadOptions,
    progress: &mut impl FnMut(&crate::kernel::CompilationProgress) -> std::ops::ControlFlow<()>,
) -> Result<KernelSet, KernelError> {
    // This historical format has no selected helper descriptor. In particular
    // it must not inherit an unrelated caller's dynamic preparation scope.
    let _preparing = crate::kernel::NativeProgramDescriptor::enter_optional(None);
    let artifact: Artifact = serde_json::from_slice(bytes)?;
    super::validate_content_id(&artifact.content_id)?;
    let payload = artifact.payload;
    let settings = settings_from_policy(&payload.compiler_policy);
    if payload.version != 3
        || payload.program_codec != CODEC
        || settings.is_none()
        || payload.metadata.is_none()
    {
        return Err(KernelError::Artifact(
            "unsupported native program codec or compiler policy".into(),
        ));
    }
    if options.validate && content_id(&payload)? != artifact.content_id {
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
    let mut encoded_programs = Vec::with_capacity(payload.sectors.len());
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
        if !use_complex && program::legacy_real_branch(&exact) {
            return Err(KernelError::Artifact(
                "historical real layout has an unproved branch domain; regenerate with the current compiler".into(),
            ));
        }
        if exact.get_input_len() != parameters.len()
            || exact.get_output_len() != payload.orders.len()
        {
            return Err(KernelError::Artifact(
                "native program input/output layout mismatch".into(),
            ));
        }
        programs.push(SectorProgram {
            parameters,
            runtime_parameters: Vec::new(),
            // Recover only facts proved by the decoded native instruction owner;
            // no serialized claim or sampled numerical zero suppresses rescue.
            exact_zero: literal_zero::outputs(&exact),
            exact,
            cancellation,
            real_coefficients: vec![false; payload.orders.len()],
        });
        encoded_programs.push(std::sync::Arc::<[u8]>::from(sector.program));
    }
    let coordinates = programs
        .iter()
        .map(|p| p.parameters.clone())
        .collect::<Vec<_>>();
    let metadata = payload
        .metadata
        .map(|m| m.into_native(&coordinates, options.validate))
        .transpose()?;
    let exact = payload
        .exact
        .into_iter()
        .map(atom)
        .collect::<Result<Vec<_>, _>>()?;
    if !use_complex
        && exact
            .iter()
            .any(|coefficient| !program::is_real_expression(coefficient, &[]))
    {
        return Err(KernelError::Artifact(
            "complex exact offset in real output layout".into(),
        ));
    }
    let mut kernels = KernelSet::from_programs_for_load_with_progress(
        payload.orders,
        programs,
        exact,
        payload.precision,
        metadata,
        use_complex,
        Vec::new(),
        settings.expect("compiler policy validated"),
        Some(encoded_programs),
        progress,
    )?;
    kernels.content_id = artifact.content_id;
    kernels.portable_artifact = Some(bytes.to_vec());
    Ok(kernels)
}
