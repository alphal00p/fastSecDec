//! Sampled physical-component growth; never an integrability certificate.
mod analysis;
mod display;
mod scan;
mod types;

pub use analysis::analyze_boundary_growth;
pub use scan::scan_boundaries;
pub use types::*;
