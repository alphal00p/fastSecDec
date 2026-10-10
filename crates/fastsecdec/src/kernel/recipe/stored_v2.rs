//! The descriptor is nested in native wire v11. Published v10/v1 layouts stay
//! byte-for-byte distinct and never infer missing certificate fields.
use super::{
    DynamicCheckProgram, KernelError, NativeProgramDescriptor, SavedProgramDescriptor, invalid,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedProgramDescriptorV2 {
    version: u32,
    legacy: SavedProgramDescriptor,
    certificates: Vec<DynamicCheckProgram>,
}
impl SavedProgramDescriptorV2 {
    pub(crate) fn from_native(descriptor: &NativeProgramDescriptor) -> Result<Self, KernelError> {
        descriptor.validate()?;
        let certificates = descriptor
            .certificates()
            .ok_or_else(|| invalid("v11 descriptor lacks explicit certificate storage"))?;
        Ok(Self {
            version: 2,
            legacy: SavedProgramDescriptor::from_native(descriptor),
            certificates: certificates.to_vec(),
        })
    }

    pub(crate) fn restore(self) -> Result<NativeProgramDescriptor, KernelError> {
        if self.version != 2 {
            return Err(invalid(
                "unsupported dynamic certificate descriptor version",
            ));
        }
        // Certificate-only helpers may not appear in a retained chart or exact
        // expression. Restore their unchanged native owner before validating
        // complete v11 references, rather than validating the v10 subset first.
        self.legacy
            .restore_with_certificates(Some(self.certificates.into()))
    }
}
