//! Optional native primary evaluator. Exact instructions remain authoritative.
use crate::kernel::{EvaluatorBackend, KernelError};
use bincode::{Decode, Encode};

// This identifies the owner's native Encode/Decode tuple containing external
// descriptors, saved Application and JITCompilationSettings. It is deliberately
// independent of the historical exact-evaluator serde codec.
const CODEC: &str = "symbolica-3:jit-bincode-descriptors-application-settings-v1";

#[derive(Encode, Decode)]
pub(in crate::kernel) struct SavedPrimary {
    codec: String,
    owner_version: String,
    backend_version: usize,
    complex: bool,
    inputs: usize,
    outputs: usize,
    exact_digest: [u8; 32],
    digest: [u8; 32],
    bytes: Vec<u8>,
}

impl SavedPrimary {
    #[cfg(test)]
    pub(in crate::kernel) fn with_incompatible_owner(mut self) -> Self {
        self.owner_version.push_str("-other-revision");
        self.digest = self.checksum();
        self
    }
    pub(in crate::kernel) fn new(
        bytes: Vec<u8>,
        exact: &[u8],
        complex: bool,
        inputs: usize,
        outputs: usize,
    ) -> Self {
        let mut result = Self {
            codec: CODEC.into(),
            owner_version: symbolica::license::LicenseManager::get_version().into(),
            #[cfg(feature = "native")]
            backend_version: crate::kernel::symjit_version_code(),
            #[cfg(feature = "portable")]
            backend_version: 0,
            complex,
            inputs,
            outputs,
            exact_digest: *blake3::hash(exact).as_bytes(),
            digest: [0; 32],
            bytes,
        };
        result.digest = result.checksum();
        result
    }

    fn checksum(&self) -> [u8; 32] {
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-native-primary-cache-v1\0");
        bincode::encode_into_std_write(
            (
                &self.codec,
                &self.owner_version,
                self.backend_version,
                self.complex,
                self.inputs,
                self.outputs,
                self.exact_digest,
                &self.bytes,
            ),
            &mut hash,
            bincode::config::standard(),
        )
        .expect("hash writer is infallible");
        *hash.finalize().as_bytes()
    }

    /// Byte-integrity checks follow the caller's validation policy. Shape and
    /// backend admission remain mandatory; compatibility misses may reuse only
    /// the already admitted exact program, never relax its owner policy.
    pub(in crate::kernel) fn admit(
        &self,
        exact: &[u8],
        complex: bool,
        inputs: usize,
        outputs: usize,
        backend: EvaluatorBackend,
        validate: bool,
    ) -> Result<bool, KernelError> {
        if validate && self.digest != self.checksum() {
            return Err(KernelError::Artifact(
                "native primary cache digest differs".into(),
            ));
        }
        if (validate && self.exact_digest != *blake3::hash(exact).as_bytes())
            || (self.complex, self.inputs, self.outputs) != (complex, inputs, outputs)
        {
            return Err(KernelError::Artifact(
                "native primary cache differs from its exact program/layout".into(),
            ));
        }
        #[cfg(feature = "native")]
        return Ok(!backend.is_eager()
            && self.codec == CODEC
            && self.owner_version == symbolica::license::LicenseManager::get_version()
            && self.backend_version == crate::kernel::symjit_version_code());
        #[cfg(feature = "portable")]
        {
            let _ = backend;
            Ok(false)
        }
    }

    pub(in crate::kernel) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(feature = "native")]
    pub(super) fn restore<T>(bytes: &[u8]) -> Result<T, String>
    where
        T: bincode::Decode<()>,
    {
        let (value, consumed) = bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|error| format!("native primary evaluator decoding: {error}"))?;
        if consumed != bytes.len() {
            return Err("trailing native primary evaluator bytes".into());
        }
        Ok(value)
    }
}
#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn payload_hashes_are_optional_but_shape_and_revision_admission_are_not() {
        let exact = b"native exact identity";
        let backend = EvaluatorBackend::Symjit;
        let mut saved = SavedPrimary::new(vec![1, 2, 3], exact, true, 3, 2);
        assert!(saved.admit(exact, true, 3, 2, backend, true).unwrap());
        saved.digest[0] ^= 1;
        assert!(saved.admit(exact, true, 3, 2, backend, false).unwrap());
        assert!(saved.admit(exact, true, 3, 2, backend, true).is_err());
        saved.digest = saved.checksum();
        assert!(
            saved
                .admit(b"different exact bytes", true, 3, 2, backend, false)
                .unwrap()
        );
        assert!(
            saved
                .admit(b"different exact bytes", true, 3, 2, backend, true)
                .is_err()
        );
        for validate in [false, true] {
            assert!(saved.admit(exact, false, 3, 2, backend, validate).is_err());
            assert!(saved.admit(exact, true, 4, 2, backend, validate).is_err());
            assert!(saved.admit(exact, true, 3, 1, backend, validate).is_err());
        }
        saved.owner_version.push_str("-other-revision");
        saved.digest = saved.checksum();
        for validate in [false, true] {
            assert!(!saved.admit(exact, true, 3, 2, backend, validate).unwrap());
        }
    }
}
