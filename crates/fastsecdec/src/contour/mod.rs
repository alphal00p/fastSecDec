//! Parameter-space contour maps and caller-owned runtime policy.
mod definitions;
mod determinant;
mod diagnostics;
pub mod dynamic;
pub(crate) mod functions;
mod map;
mod metadata;
mod settings;

pub use definitions::{ContourDefinition, ContourDefinitions};
pub use diagnostics::{
    ContourDiagnosticRange, ContourDiagnosticsMode, ContourRuntimeReport, ContourRuntimeWork,
};
pub use map::FixedContourMap;
pub(crate) use map::{SmoothContourMap, continued_power};
pub use metadata::ContourMetadata;
pub use settings::{
    ContourMode, ContourSettings, ContourValidation, ContourValidationOptions, DynamicConstruction,
};

/// Reserved real runtime input. Its value never enters symbolic generation.
pub fn lambda_symbol() -> symbolica::atom::Symbol {
    symbolica::symbol!("fastsecdec::contour::lambda")
}

/// Identify native contour inputs separately from physical runtime parameters.
pub fn is_contour_parameter(symbol: symbolica::atom::Symbol) -> bool {
    use crate::kernel::ProgramRecipe;
    [ProgramRecipe::FixedV1, ProgramRecipe::DynamicPolynomialV1]
        .iter()
        .flat_map(|recipe| recipe.recipe_parameters())
        .any(|name| symbol.get_name() == *name)
}

#[cfg(test)]
mod tests;
