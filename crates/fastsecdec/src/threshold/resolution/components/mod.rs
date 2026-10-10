//! Checked algebraic component splitting on native regular quotient owners.
//! These open covers carry no real integration-atlas or endpoint authority.
mod elimination;
mod idempotent;
mod owner;
pub use elimination::{EliminationEvidence, VerifiedAnnihilator};
pub use idempotent::{
    ComponentOpen, ComponentPattern, ComponentProduction, ComponentProgress,
    VerifiedComponentSplit, produce_component_split,
};
pub use owner::{BoundaryAlgebra, RegularAlgebra, RegularOrigin};
#[cfg(test)]
mod tests;
