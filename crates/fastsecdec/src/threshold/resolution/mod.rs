//! Local resolution certificates and an ordinary relative contact stage.
//!
//! Native Q-polynomial charts, supplied inverse frames and controlled blowups
//! are checked alongside affine quotient/etale relative derivations. Inclusive
//! derivative ideals construct algebraic maximum-order loci and finite contact
//! opens without inferring a boundary ledger. The complete BM producer,
//! exceptional-divisor invariant/history, real branch atlas, orientation and
//! closed-face endpoint certificates remain separate gates.
mod algebra;
mod chart;
mod contact;
mod cover;
mod etale;
mod localized;
mod marked;
mod producer;
mod transform;
pub use algebra::{Budget, Error, Ideal, Limits, Poly, Ring};
pub use chart::{Boundary, Chart, FrameCertificate, Guard, Map, VerifiedFrame};
pub use contact::{
    ContactJet, ContactProduction, ContactProgress, ContactQuotient, RingExtension, UnitClearing,
    construct_contact_quotient,
};
pub use cover::{OpenCoverCertificate, VerifiedOpenCover};
pub use etale::{EtaleCertificate, EtaleFrame};
pub use localized::LocalizedAlgebra;
pub use marked::{
    CenterCertificate, CoefficientRestriction, CompanionArithmetic, MarkedIdeal, RecordedInvariant,
    VerifiedCenter,
};
pub use producer::{
    ContactCandidate, DerivativeStage, OrderProduction, ProducedContactCover, ProductionProgress,
    produce_ordinary_contact_cover,
};
pub use transform::{ControlledTransformCertificate, DivisorTransform, VerifiedTransform};

#[cfg(test)]
mod etale_tests;
#[cfg(test)]
mod tests;
