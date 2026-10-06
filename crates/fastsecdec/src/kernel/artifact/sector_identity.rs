//! Additive representation identities; existing artifact formats stay intact.
use super::{native, parameter_names};
use crate::{
    kernel::{KernelError, KernelSet, PortableMetadata, PrecisionPolicy},
    status::CoefficientComponent,
};
use serde::Serialize;

#[cfg(test)]
mod tests;

#[derive(Serialize)]
struct SectorIdentity<'a> {
    program_codec: &'a str,
    compiler_policy: &'a str,
    parameters: Vec<String>,
    runtime_parameters: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    runtime_mass_constraints: Vec<(&'a str, String)>,
    program: &'a [u8],
    orders: &'a [i32],
    components: &'a [CoefficientComponent],
    precision: &'a PrecisionPolicy,
    cancellation_degree: usize,
    cancellation_terms: Option<&'a [Vec<usize>]>,
    // None explicitly distinguishes old artifacts without retained semantics.
    metadata: Option<PortableMetadata>,
}

impl KernelSet {
    /// Versioned content digest for a numerical sector and its retained semantics.
    ///
    /// Binds the immutable native program, ordered inputs/outputs, compiler and
    /// precision policies, cancellation metadata, domain and associated charts.
    /// Unrelated sectors, exact offsets and mutable evaluation state are excluded.
    /// The selected kernel's parent ordinal is localized; original chart ordinals
    /// remain, so chart renumbering may cause a safe cache miss.
    ///
    /// This identifies a representation, not mathematical equivalence or compiled
    /// machine code. A machine-code cache also needs architecture and backend
    /// build identity. Existing artifact, checkpoint and replay bindings remain
    /// unchanged. Legacy artifacts derive an identity from their validated current
    /// native program without changing their saved bytes or parent identity.
    pub fn sector_content_id(&self, index: usize) -> Result<String, KernelError> {
        let sector = self.sectors.get(index).ok_or_else(|| {
            KernelError::Artifact(format!("unknown numerical sector index {index}"))
        })?;
        let identity = SectorIdentity {
            program_codec: native::CODEC,
            compiler_policy: native::compiler_policy(),
            parameters: parameter_names(&sector.parameters),
            runtime_parameters: parameter_names(&sector.runtime_parameters),
            runtime_mass_constraints: self
                .runtime_mass_constraints
                .iter()
                .map(|constraint| {
                    (
                        constraint.name.as_str(),
                        symbolica::atom::AtomCore::to_canonical_string(&constraint.expression),
                    )
                })
                .collect(),
            program: &sector.program_bytes,
            orders: &self.orders,
            components: &self.components,
            precision: &sector.precision,
            cancellation_degree: sector.cancellation.degree(),
            cancellation_terms: sector.cancellation.terms(),
            metadata: self
                .metadata
                .as_ref()
                .map(|metadata| PortableMetadata::for_sector(metadata, index)),
        };
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-sector-content-v1\0");
        // Stream the existing canonical transport into the digest; never restore
        // coefficient Atoms or allocate a JSON copy of the native program bytes.
        serde_json::to_writer(&mut hash, &identity)?;
        Ok(format!("fsd-sector-v1:{}", hash.finalize().to_hex()))
    }
}
