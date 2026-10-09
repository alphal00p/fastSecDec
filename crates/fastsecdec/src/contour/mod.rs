//! Parameter-space contour maps and caller-owned runtime policy.
mod determinant;
pub mod dynamic;
pub(crate) mod functions;
mod map;
mod metadata;
mod settings;

pub use map::FixedContourMap;
pub(crate) use map::SmoothContourMap;
pub use metadata::ContourMetadata;
pub use settings::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions};

/// Reserved real runtime input. Its value never enters symbolic generation.
pub fn lambda_symbol() -> symbolica::atom::Symbol {
    symbolica::symbol!("fastsecdec::contour::lambda")
}

#[cfg(test)]
mod tests;
