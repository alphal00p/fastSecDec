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
        };
        value.validate()?;
        Ok(value)
    }
}
