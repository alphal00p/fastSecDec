//! Strict versioned envelopes around native symbolic/evaluator serialization.
//! Symbolica owns native program decoding and structural validation.
#[cfg(feature = "native")]
mod legacy;
mod native;
mod sector_identity;

use super::{KernelError, KernelSet, PrecisionPolicy};
use symbolica::atom::{Atom, AtomCore, AtomView, Symbol};

pub(super) fn atom(expression: String) -> Result<Atom, KernelError> {
    // Recover native callbacks before expressions are parsed in a cold host.
    let _ = symbolica::transcendental::gamma();
    Atom::parse(expression, "fastsecdec::artifact", Default::default())
        .map_err(KernelError::Artifact)
}

fn parameters(values: Vec<String>) -> Result<Vec<Symbol>, KernelError> {
    let parameters = values
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
    Ok(parameters)
}

fn validate_orders(orders: &[i32], exact_len: usize) -> Result<(), KernelError> {
    if orders.is_empty()
        || orders
            .windows(2)
            .any(|pair| pair[0].checked_add(1) != Some(pair[1]))
        || exact_len != orders.len()
    {
        return Err(KernelError::Artifact(
            "invalid Laurent output layout".into(),
        ));
    }
    Ok(())
}

fn parameter_names(parameters: &[Symbol]) -> Vec<String> {
    parameters
        .iter()
        .map(|p| Atom::var(*p).to_canonical_string())
        .collect()
}

impl crate::generation::GeneratedIntegral {
    /// Build and save portable native exact evaluator programs without JIT.
    /// This performs native expression-to-IR translation; it does not materialize
    /// aliased coefficients or compile host executable machine code.
    pub fn to_kernel_bytes(&self, precision: PrecisionPolicy) -> Result<Vec<u8>, KernelError> {
        native::generated(self, precision)
    }
}

impl KernelSet {
    pub(super) fn initialize_artifact(&mut self) -> Result<(), KernelError> {
        let (id, bytes) = native::compiled(self)?;
        self.content_id = id;
        self.portable_artifact = Some(bytes);
        Ok(())
    }

    /// Save immutable portable programs, metadata and numerical policy. Loaded
    /// version-one/two artifacts retain their original bytes and identities.
    /// Executable code and mutable evaluator work stacks are excluded.
    pub fn to_bytes(&self) -> Result<Vec<u8>, KernelError> {
        Ok(self.artifact_bytes()?.to_vec())
    }

    /// Borrow the retained portable artifact without copying its native programs.
    /// Loaded artifacts retain their exact original bytes, including formatting.
    pub fn artifact_bytes(&self) -> Result<&[u8], KernelError> {
        self.portable_artifact
            .as_deref()
            .ok_or_else(|| KernelError::Artifact("kernel artifact was not initialized".into()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KernelError> {
        // Dispatch does not replace either codec's strict owned schema.
        // Ignore the large program arrays here instead of allocating a second
        // full JSON representation before the selected codec reads them.
        #[derive(serde::Deserialize)]
        struct Envelope {
            payload: Version,
        }
        #[derive(serde::Deserialize)]
        struct Version {
            version: u32,
        }
        let envelope: Envelope = serde_json::from_slice(bytes)?;
        match envelope.payload.version {
            #[cfg(feature = "native")]
            1 | 2 => legacy::load(bytes),
            #[cfg(feature = "portable")]
            1 | 2 => Err(KernelError::Artifact(
                "legacy native artifacts require the native backend".into(),
            )),
            3 => native::load(bytes),
            _ => Err(KernelError::Artifact(
                "unsupported kernel artifact version".into(),
            )),
        }
    }
}
