//! Supplied local certificate checker draft; NOT a BM producer/resolver.
//!
//! Initial scope: native Q-polynomial ambient charts, supplied polynomial
//! relative inverse frames, principal opens, adapted coordinate centres and
//! standard blowup charts. Rational/algebraic inverse frames, global maxima,
//! a covering atlas and real orientation certificates remain separate gates.
mod algebra;
mod chart;
mod marked;
mod transform;
pub use algebra::{Budget, Error, Ideal, Limits, Poly, Ring};
pub use chart::{Boundary, Chart, FrameCertificate, Guard, Map, VerifiedFrame};
pub use marked::{
    CenterCertificate, CoefficientRestriction, CompanionArithmetic, MarkedIdeal, RecordedInvariant,
    VerifiedCenter,
};
pub use transform::{ControlledTransformCertificate, DivisorTransform, VerifiedTransform};

#[cfg(test)]
mod tests;
