//! Native threshold geometry and its proof boundaries.
//!
//! Verified generic cells alone do not certify endpoint regularity or analytic
//! continuation. Each later generation stage must preserve and strengthen this
//! evidence before publishing an integration artifact.

pub mod gcad;
pub mod maps;
pub mod phase;
pub mod projective;
pub mod resolution;
