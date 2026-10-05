//! Exact, domain-aware monomial sector decomposition.
//!
//! This crate owns geometry only. Numerica supplies exact arithmetic and linear
//! algebra (the same types reexported by Symbolica). The main FastSecDec library
//! owns the Symbolica expression-to-support bridge, integrand symmetry and all
//! physics conventions. Geometry therefore needs no expression engine or JIT.
#[cfg(all(feature = "native", feature = "portable"))]
compile_error!("select exactly one sector numeric backend: native or portable");
#[cfg(not(any(feature = "native", feature = "portable")))]
compile_error!("select a sector numeric backend: native (default) or portable");

mod arithmetic;
mod cache;
mod cone;
mod decompose;
mod map_validation;
mod stages;
mod support;
mod triangulate;
mod types;
mod work;

pub use cache::{GeometryCache, GeometryCacheOutcome, GeometryDispatch};
pub use decompose::decompose;
pub use support::PolynomialSupport;
pub use types::{
    Decomposition, DecompositionOptions, DecompositionPhase, DecompositionProgress,
    ParametricDomain, SectorError, SectorMap,
};
pub use work::{
    GeometryCompletion, GeometryJob, GeometryJobId, GeometryPlan, GeometryWorkError,
    PreparedGeometry,
};
