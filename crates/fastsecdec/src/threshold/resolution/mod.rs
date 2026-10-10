//! Local resolution certificates and constructive contact/monomial stages.
//!
//! Native Q-polynomial charts, supplied inverse frames and controlled blowups
//! are checked alongside affine quotient/etale relative derivations. Inclusive
//! derivative ideals construct algebraic maximum-order loci and finite contact
//! opens without inferring a boundary ledger. Verified relative boundary
//! geometry supports all monomial blowup charts and local exceptional histories.
//! The complete BM invariant/producer, global center gluing, real branch atlas,
//! orientation and closed-face endpoint certificates remain separate gates.
mod algebra;
mod blowup;
mod chart;
mod contact;
mod cover;
mod etale;
mod geometry;
mod localized;
mod marked;
mod matrix;
mod producer;
mod transform;
pub use algebra::{Budget, Error, Ideal, Limits, Poly, Ring};
pub use blowup::{
    AdaptedCover, AdaptedOpen, AdaptedProduction, AdaptedProgress, BirthContext, BlowupProduction,
    BlowupProgress, DivisorReceipt, EmptyAdaptedOpen, HistoryChartStep, MonomialBlowup,
    MonomialBlowupChart, ResolutionHistory, produce_adapted_cover, produce_monomial_blowup,
};
pub use chart::{Boundary, Chart, FrameCertificate, Guard, Map, VerifiedFrame};
pub use contact::{
    ContactJet, ContactProduction, ContactProgress, ContactQuotient, RingExtension, UnitClearing,
    construct_contact_quotient,
};
pub use cover::{OpenCoverCertificate, VerifiedOpenCover};
pub use etale::{EtaleCertificate, EtaleFrame};
pub use geometry::{
    BoundaryId, InitialDivisor, IntersectionCertificate, MonomialCandidate, MonomialProduction,
    MonomialProgress, MonomialWitness, NormalizationOutcome, NormalizationProgress,
    ProducedMonomialCenter, QuotientNormalizer, SncProduction, SncProgress,
    VerifiedQuotientNormalization, VerifiedRelativeSnc, produce_monomial_center,
    verify_initial_relative_snc,
};
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
