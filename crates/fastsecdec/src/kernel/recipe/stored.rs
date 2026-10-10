use super::*;

/// Native helpers keep their existing evaluator codec. No symbolic rebuilding
/// or Horner/CPE optimization occurs while restoring this transport value.
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedProgramDescriptor {
    version: u32,
    recipe: ProgramRecipe,
    charts: Vec<DynamicChartRecipe>,
    helpers: Vec<Vec<u8>>,
    exact_helpers: Vec<String>,
}
impl SavedProgramDescriptor {
    pub(crate) fn from_native(value: &NativeProgramDescriptor) -> Self {
        Self {
            version: 1,
            recipe: value.recipe,
            charts: value.charts.clone(),
            exact_helpers: value.exact_helpers.clone(),
            helpers: value
                .helpers
                .iter()
                .map(|helper| helper.bytes().to_vec())
                .collect(),
        }
    }

    pub(crate) fn restore(self) -> Result<NativeProgramDescriptor, KernelError> {
        self.restore_with_certificates(None)
    }

    pub(super) fn restore_with_certificates(
        self,
        certificates: Option<std::sync::Arc<[DynamicCheckProgram]>>,
    ) -> Result<NativeProgramDescriptor, KernelError> {
        if self.version != 1 {
            return Err(invalid("unsupported mathematical descriptor version"));
        }
        crate::contour::functions::dynamic::register();
        let helpers = self
            .helpers
            .iter()
            .map(|bytes| RootProgram::from_bytes(bytes).map_err(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let value = NativeProgramDescriptor {
            recipe: self.recipe,
            charts: self.charts,
            helpers,
            exact_helpers: self.exact_helpers,
            certificates,
        };
        value.validate()?;
        Ok(value)
    }
}

impl NativeProgramDescriptor {
    /// Generation staging retains the same native helper codec as v10 artifacts.
    /// Restoring this owner performs no algebra or evaluator optimization.
    pub(crate) fn to_staging_bytes(&self) -> Result<Vec<u8>, KernelError> {
        self.validate()?;
        if self.certificates.is_some() {
            return Err(invalid(
                "compiled certificate descriptors cannot be downgraded to source staging",
            ));
        }
        bincode::serde::encode_to_vec(
            SavedProgramDescriptor::from_native(self),
            bincode::config::standard(),
        )
        .map_err(invalid)
    }

    pub(crate) fn from_staging_bytes(bytes: &[u8]) -> Result<Self, KernelError> {
        let (saved, used): (SavedProgramDescriptor, usize) =
            bincode::serde::decode_from_slice(bytes, bincode::config::standard())
                .map_err(invalid)?;
        if used != bytes.len() {
            return Err(invalid("trailing bytes in staged native recipe descriptor"));
        }
        saved.restore()
    }
}
