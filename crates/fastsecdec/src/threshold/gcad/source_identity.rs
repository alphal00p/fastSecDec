//! Canonical native identities: exact-only requests retain their established bytes.
use super::{GcadError, GcadRequest, Result};
use crate::generation::{self, identity::CanonicalAtom};
impl GcadRequest {
    pub fn source_identity(&self) -> Result<String> {
        let original =
            generation::source_identity(self.input(), &self.kinematics().runtime_parameters, &[])
                .map_err(|e| GcadError::Invalid(e.to_string()))?;
        let Some(represented) = self.represented_input() else {
            return Ok(original);
        };
        let exact = generation::source_identity(
            represented.exact(),
            &self.kinematics().runtime_parameters,
            &[],
        )
        .map_err(|e| GcadError::Invalid(e.to_string()))?;
        let mut h = blake3::Hasher::new();
        h.update(b"fastsecdec-threshold-represented-source-v1\0");
        serde_json::to_writer(&mut h, &(original, exact, represented.meaning()))
            .map_err(|e| GcadError::Invalid(e.to_string()))?;
        for row in represented.conversions() {
            serde_json::to_writer(
                &mut h,
                &(
                    &row.location,
                    CanonicalAtom(&row.original),
                    CanonicalAtom(&row.exact),
                    row.precision_bits,
                    row.binary_exponents,
                ),
            )
            .map_err(|e| GcadError::Invalid(e.to_string()))?;
        }
        Ok(h.finalize().to_hex().to_string())
    }
}
