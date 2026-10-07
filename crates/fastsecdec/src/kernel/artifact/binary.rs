//! Native context-aware binserde caches. JSON presentation has no program bytes.
use super::{native, validate_orders};
use crate::{
    kernel::{
        KernelError, KernelSet, PrecisionPolicy, RuntimeMassConstraint,
        cancellation::Cancellation,
        metadata::PortableMetadata,
        program::{self, SectorProgram},
    },
    status::CoefficientComponent,
};
use bincode::{Decode, Encode};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    state::{State, StateMap},
};

pub(super) const PREFIX: &[u8] = b"FastSecDec\0binserde";
pub(super) const MAGIC: &[u8] = b"FastSecDec\0binserde\x07";
const MAGIC_V6: &[u8] = b"FastSecDec\0binserde\x06";
const MAGIC_V5: &[u8] = b"FastSecDec\0binserde\x05";
const CODEC: &str = "symbolica-3:context-binserde-atoms-v1:serde-binserde-evaluators-v1";
#[derive(Encode, Decode)]
struct Envelope {
    content_id: String,
    digest: [u8; 32],
    state: Vec<u8>,
    payload: Vec<u8>,
}
#[derive(Encode, Decode)]
#[bincode(decode_context = "StateMap")]
struct Payload {
    codec: String,
    compiler_policy: String,
    orders: Vec<i32>,
    #[bincode(with_serde)]
    components: Vec<CoefficientComponent>,
    exact: Vec<Atom>,
    runtime_parameters: Vec<Symbol>,
    runtime_mass_constraints: Vec<MassConstraint>,
    #[bincode(with_serde)]
    precision: PrecisionPolicy,
    sectors: Vec<Sector>,
    metadata: Option<PortableMetadata>,
}
#[derive(Decode)]
#[bincode(decode_context = "StateMap")]
struct PayloadV6 {
    codec: String,
    compiler_policy: String,
    orders: Vec<i32>,
    #[bincode(with_serde)]
    components: Vec<CoefficientComponent>,
    exact: Vec<Atom>,
    runtime_parameters: Vec<Symbol>,
    runtime_mass_constraints: Vec<MassConstraint>,
    #[bincode(with_serde)]
    precision: PrecisionPolicy,
    sectors: Vec<LegacySector>,
    metadata: Option<PortableMetadata>,
}
impl From<PayloadV6> for Payload {
    fn from(value: PayloadV6) -> Self {
        Self {
            codec: value.codec,
            compiler_policy: value.compiler_policy,
            orders: value.orders,
            components: value.components,
            exact: value.exact,
            runtime_parameters: value.runtime_parameters,
            runtime_mass_constraints: value.runtime_mass_constraints,
            precision: value.precision,
            sectors: value.sectors.into_iter().map(Into::into).collect(),
            metadata: value.metadata,
        }
    }
}
#[derive(Encode, Decode)]
#[bincode(decode_context = "StateMap")]
struct MassConstraint {
    name: String,
    expression: Atom,
}
/// Version five predates runtime mass constraints. Keep its exact owned layout
/// for previously validated templates; no decoder guesses missing fields.
#[derive(Decode)]
#[bincode(decode_context = "StateMap")]
struct PayloadV5 {
    codec: String,
    compiler_policy: String,
    orders: Vec<i32>,
    #[bincode(with_serde)]
    components: Vec<CoefficientComponent>,
    exact: Vec<Atom>,
    runtime_parameters: Vec<Symbol>,
    #[bincode(with_serde)]
    precision: PrecisionPolicy,
    sectors: Vec<LegacySector>,
    metadata: Option<PortableMetadata>,
}
impl From<PayloadV5> for Payload {
    fn from(value: PayloadV5) -> Self {
        Self {
            codec: value.codec,
            compiler_policy: value.compiler_policy,
            orders: value.orders,
            components: value.components,
            exact: value.exact,
            runtime_parameters: value.runtime_parameters,
            runtime_mass_constraints: Vec::new(),
            precision: value.precision,
            sectors: value.sectors.into_iter().map(Into::into).collect(),
            metadata: value.metadata,
        }
    }
}
#[derive(Encode, Decode)]
#[bincode(decode_context = "StateMap")]
struct Sector {
    parameters: Vec<Symbol>,
    // Reuse the native evaluator's established serde/bincode codec. Numerica
    // 3.0.1 GMP Integer::Large native Encode drops a negative sign; its serde
    // codec preserves it. Explicit Atom fields still use native StateMap Decode.
    program: Vec<u8>,
    cancellation_degree: usize,
    cancellation_terms: Option<Vec<Vec<usize>>>,
    #[bincode(with_serde)]
    endpoint_profiles: Option<Vec<crate::generation::EndpointProfileRow>>,
}
#[derive(Decode)]
#[bincode(decode_context = "StateMap")]
struct LegacySector {
    parameters: Vec<Symbol>,
    // Reuse the native evaluator's established serde/bincode codec. Numerica
    // 3.0.1 GMP Integer::Large native Encode drops a negative sign; its serde
    // codec preserves it. Explicit Atom fields still use native StateMap Decode.
    program: Vec<u8>,
    cancellation_degree: usize,
    cancellation_terms: Option<Vec<Vec<usize>>>,
}
impl From<LegacySector> for Sector {
    fn from(value: LegacySector) -> Self {
        Self {
            parameters: value.parameters,
            program: value.program,
            cancellation_degree: value.cancellation_degree,
            cancellation_terms: value.cancellation_terms,
            endpoint_profiles: None,
        }
    }
}
fn failure(error: impl std::fmt::Display) -> KernelError {
    KernelError::Artifact(format!("native binserde cache: {error}"))
}
fn digest(magic: &[u8], state: &[u8], payload: &[u8]) -> blake3::Hash {
    let mut hash = blake3::Hasher::new();
    hash.update(magic);
    hash.update(&(state.len() as u64).to_le_bytes());
    hash.update(state);
    hash.update(payload);
    hash.finalize()
}
fn semantic_id(payload: &Payload, version: u8) -> Result<String, KernelError> {
    #[derive(serde::Serialize)]
    struct SemanticSector<'a> {
        parameters: Vec<String>,
        program: &'a [u8],
        cancellation_degree: usize,
        cancellation_terms: &'a Option<Vec<Vec<usize>>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        endpoint_profiles: &'a Option<Vec<crate::generation::EndpointProfileRow>>,
    }
    #[derive(serde::Serialize)]
    struct SemanticPayload<'a> {
        codec: &'a str,
        compiler_policy: &'a str,
        orders: &'a [i32],
        components: &'a [CoefficientComponent],
        exact: Vec<String>,
        runtime_parameters: Vec<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        runtime_mass_constraints: Vec<(&'a str, String)>,
        precision: &'a PrecisionPolicy,
        sectors: Vec<SemanticSector<'a>>,
        metadata: &'a Option<PortableMetadata>,
    }
    // Native evaluator serde uses portable symbol definitions, while raw Atom
    // binserde uses process-local IDs. Scientific identity must not include the
    // latter IDs or unrelated entries in Symbolica's shared context.
    let semantic = SemanticPayload {
        codec: &payload.codec,
        compiler_policy: &payload.compiler_policy,
        orders: &payload.orders,
        components: &payload.components,
        exact: payload
            .exact
            .iter()
            .map(AtomCore::to_canonical_string)
            .collect(),
        runtime_parameters: super::parameter_names(&payload.runtime_parameters),
        runtime_mass_constraints: payload
            .runtime_mass_constraints
            .iter()
            .map(|constraint| {
                (
                    constraint.name.as_str(),
                    constraint.expression.to_canonical_string(),
                )
            })
            .collect(),
        precision: &payload.precision,
        sectors: payload
            .sectors
            .iter()
            .map(|s| {
                Ok(SemanticSector {
                    parameters: super::parameter_names(&s.parameters),
                    program: &s.program,
                    cancellation_degree: s.cancellation_degree,
                    cancellation_terms: &s.cancellation_terms,
                    endpoint_profiles: &s.endpoint_profiles,
                })
            })
            .collect::<Result<_, KernelError>>()?,
        metadata: &payload.metadata,
    };
    let mut hash = blake3::Hasher::new();
    hash.update(match version {
        5 => b"fastsecdec-native-semantic-v5",
        6 => b"fastsecdec-native-semantic-v6",
        _ => b"fastsecdec-native-semantic-v7",
    });
    serde_json::to_writer(&mut hash, &semantic)?;
    Ok(hash.finalize().to_hex().to_string())
}
fn encode(payload: Payload) -> Result<(String, Vec<u8>), KernelError> {
    let content_id = semantic_id(&payload, 7)?;
    let mut symbols = Atom::Zero.get_all_symbols(true);
    let mut collect = |atom: &Atom| {
        symbols.extend(atom.get_all_symbols(true));
    };
    for atom in &payload.exact {
        collect(atom);
    }
    for constraint in &payload.runtime_mass_constraints {
        collect(&constraint.expression);
    }
    if let Some(metadata) = &payload.metadata {
        metadata.visit_atoms(&mut collect);
    }
    symbols.extend(payload.runtime_parameters.iter().copied());
    symbols.extend(
        payload
            .sectors
            .iter()
            .flat_map(|s| s.parameters.iter().copied()),
    );
    let mut state = Vec::new();
    State::export_partial(&mut state, symbols).map_err(failure)?;
    let payload = bincode::encode_to_vec(payload, bincode::config::standard()).map_err(failure)?;
    let digest = digest(MAGIC, &state, &payload);
    let envelope = Envelope {
        content_id: content_id.clone(),
        digest: *digest.as_bytes(),
        state,
        payload,
    };
    let mut bytes = MAGIC.to_vec();
    bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).map_err(failure)?);
    Ok((content_id, bytes))
}
pub(super) fn compiled(kernels: &KernelSet) -> Result<(String, Vec<u8>), KernelError> {
    encode(Payload {
        codec: CODEC.into(),
        compiler_policy: native::compiler_policy().into(),
        orders: kernels.coefficient_orders.clone(),
        components: kernels.components.clone(),
        exact: kernels.exact_expressions.clone(),
        runtime_parameters: kernels.runtime_parameters.clone(),
        runtime_mass_constraints: kernels
            .runtime_mass_constraints
            .iter()
            .map(|constraint| MassConstraint {
                name: constraint.name.clone(),
                expression: constraint.expression.clone(),
            })
            .collect(),
        precision: kernels.precision.clone(),
        sectors: kernels
            .sectors
            .iter()
            .map(|sector| {
                Ok(Sector {
                    parameters: sector.parameters.clone(),
                    program: sector.program_bytes.to_vec(),
                    cancellation_degree: sector.cancellation.degree(),
                    cancellation_terms: sector.cancellation.terms().map(<[Vec<usize>]>::to_vec),
                    endpoint_profiles: sector.cancellation.endpoint_profiles().map(<[_]>::to_vec),
                })
            })
            .collect::<Result<_, KernelError>>()?,
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
        .flat_map(|s| s.aliased_coefficients())
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
                )?
                .with_endpoint_profiles(sector.endpoint_profiles().to_vec())?,
            )?;
            Ok(Sector {
                parameters: program.parameters,
                program: program::encode(&program.exact)?,
                cancellation_degree: program.cancellation.degree(),
                cancellation_terms: program.cancellation.terms().map(<[Vec<usize>]>::to_vec),
                endpoint_profiles: program.cancellation.endpoint_profiles().map(<[_]>::to_vec),
            })
        })
        .collect::<Result<_, KernelError>>()?;
    Ok(encode(Payload {
        codec: CODEC.into(),
        compiler_policy: native::compiler_policy().into(),
        orders: value.orders().to_vec(),
        components: native::component_layout(value.orders().len(), complex),
        exact: value.exact_coefficients().to_vec(),
        runtime_parameters: Vec::new(),
        runtime_mass_constraints: Vec::new(),
        precision,
        sectors,
        metadata: Some(PortableMetadata::from_native(value.metadata())),
    })?
    .1)
}
pub(super) fn load(bytes: &[u8]) -> Result<KernelSet, KernelError> {
    let (wire, magic, version) = if let Some(wire) = bytes.strip_prefix(MAGIC) {
        (wire, MAGIC, 7)
    } else if let Some(wire) = bytes.strip_prefix(MAGIC_V6) {
        (wire, MAGIC_V6, 6)
    } else if let Some(wire) = bytes.strip_prefix(MAGIC_V5) {
        (wire, MAGIC_V5, 5)
    } else {
        return Err(failure("unsupported header"));
    };
    let (envelope, used): (Envelope, usize) =
        bincode::decode_from_slice(wire, bincode::config::standard()).map_err(failure)?;
    if used != wire.len() {
        return Err(failure("trailing envelope bytes"));
    }
    let id = digest(magic, &envelope.state, &envelope.payload);
    if id.as_bytes() != &envelope.digest {
        return Err(failure("content identity mismatch"));
    }
    let _ = symbolica::transcendental::gamma();
    let mut state_source = envelope.state.as_slice();
    let context = State::import(&mut state_source, None).map_err(failure)?;
    if !state_source.is_empty() {
        return Err(failure("trailing symbol context bytes"));
    }
    let (payload, used): (Payload, usize) = if version == 5 {
        let (payload, used): (PayloadV5, usize) = bincode::decode_from_slice_with_context(
            &envelope.payload,
            bincode::config::standard(),
            context,
        )
        .map_err(failure)?;
        (payload.into(), used)
    } else if version == 6 {
        let (payload, used): (PayloadV6, usize) = bincode::decode_from_slice_with_context(
            &envelope.payload,
            bincode::config::standard(),
            context,
        )
        .map_err(failure)?;
        (payload.into(), used)
    } else {
        bincode::decode_from_slice_with_context(
            &envelope.payload,
            bincode::config::standard(),
            context,
        )
        .map_err(failure)?
    };
    if used != envelope.payload.len() {
        return Err(failure("trailing payload bytes"));
    }
    if semantic_id(&payload, version)? != envelope.content_id {
        return Err(failure("semantic content identity mismatch"));
    }
    if payload.codec != CODEC || payload.compiler_policy != native::compiler_policy() {
        return Err(failure("unsupported codec or compiler policy"));
    }
    validate_orders(&payload.orders, payload.exact.len())?;
    payload.precision.validate()?;
    let use_complex = if payload.components == native::component_layout(payload.orders.len(), false)
    {
        false
    } else if payload.components == native::component_layout(payload.orders.len(), true) {
        true
    } else {
        return Err(failure("invalid component layout"));
    };
    let unique = payload
        .runtime_parameters
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    if unique.len() != payload.runtime_parameters.len() {
        return Err(failure("duplicate runtime parameter"));
    }
    let mut programs = Vec::with_capacity(payload.sectors.len());
    for sector in payload.sectors {
        let coordinates = sector
            .parameters
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        if coordinates.is_empty()
            || coordinates.len() != sector.parameters.len()
            || coordinates.iter().any(|p| unique.contains(p))
        {
            return Err(failure("invalid sector coordinates"));
        }
        let program = program::decode(&sector.program)?;
        if program.get_input_len() != sector.parameters.len() + payload.runtime_parameters.len()
            || program.get_output_len() != payload.orders.len()
        {
            return Err(failure("native program input/output layout mismatch"));
        }
        let mut cancellation = Cancellation::new(
            sector.cancellation_degree,
            sector.cancellation_terms,
            sector.parameters.len(),
        )?;
        if let Some(profiles) = sector.endpoint_profiles {
            cancellation = cancellation.with_endpoint_profiles(profiles)?;
        }
        programs.push(SectorProgram {
            parameters: sector.parameters,
            runtime_parameters: payload.runtime_parameters.clone(),
            exact_zero: native::literal_zero::outputs(&program),
            exact: program,
            cancellation,
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
    if !use_complex
        && payload
            .exact
            .iter()
            .any(crate::kernel::has_complex_coefficients)
    {
        return Err(failure("complex exact offset in real output layout"));
    }
    let mut kernels = KernelSet::from_programs_for_load(
        payload.orders,
        programs,
        payload.exact,
        payload.precision,
        metadata,
        use_complex,
        payload.runtime_parameters,
    )?;
    kernels.runtime_mass_constraints = payload
        .runtime_mass_constraints
        .into_iter()
        .map(|constraint| RuntimeMassConstraint {
            name: constraint.name,
            expression: constraint.expression,
        })
        .collect();
    kernels.validate_runtime_mass_constraints()?;
    kernels.content_id = envelope.content_id;
    kernels.portable_artifact = Some(bytes.to_vec());
    Ok(kernels)
}
