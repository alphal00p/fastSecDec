//! Exact, domain-aware monomial sector decomposition.
//!
//! This crate owns geometry only. Numerica supplies exact arithmetic and linear
//! algebra (the same types reexported by Symbolica). The main FastSecDec library
//! owns the Symbolica expression-to-support bridge, integrand symmetry and all
//! physics conventions. Geometry therefore needs no expression engine or JIT.
mod arithmetic;
mod cone;
mod decompose;
mod support;
mod triangulate;
mod types;

pub use decompose::decompose;
pub use support::PolynomialSupport;
pub use types::{
    Decomposition, DecompositionOptions, DecompositionPhase, DecompositionProgress,
    ParametricDomain, SectorError, SectorMap,
};
