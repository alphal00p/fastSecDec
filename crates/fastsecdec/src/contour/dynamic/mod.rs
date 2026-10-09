//! Symbolic sufficient causal radii. This module does not solve their equations
//! or enable dynamic production integration; native runtime lowering is separate.
mod envelope;

pub use envelope::{
    CausalEnvelopeTerm, DynamicEnvelope, PositiveEnvelope, PositiveEnvelopeTerm, SmoothPositivePart,
};
use symbolica::atom::Symbol;

/// Reserved real mathematical inputs, independent of physical parameters.
pub fn lambda_cap_symbol() -> Symbol {
    symbolica::symbol!("fastsecdec::contour::dynamic::lambda_cap")
}
pub fn displacement_cap_symbol() -> Symbol {
    symbolica::symbol!("fastsecdec::contour::dynamic::displacement_cap")
}
pub fn safety_fraction_symbol() -> Symbol {
    symbolica::symbol!("fastsecdec::contour::dynamic::safety_fraction")
}
/// Dimensionless root coordinate; the positive root lies in (0, 1].
pub fn radius_fraction_symbol() -> Symbol {
    symbolica::symbol!("fastsecdec::contour::dynamic::radius_fraction")
}

#[cfg(test)]
mod map;
#[cfg(test)]
mod tests;
