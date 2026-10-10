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
mod companion;
mod components;
mod contact;
mod cover;
mod differential;
mod etale;
mod geometry;
mod iteration;
mod localization;
mod localized;
mod marked;
mod matrix;
mod native_basis;
mod producer;
mod transform;
pub use algebra::{Budget, Error, Ideal, Limits, Poly, Ring};
pub use blowup::{
    AdaptedCover, AdaptedOpen, AdaptedProduction, AdaptedProgress, BirthContext, BlowupProduction,
    BlowupProgress, DivisorReceipt, EmptyAdaptedOpen, HistoryChartStep, HistoryStep,
    MonomialBlowup, MonomialBlowupChart, ResolutionHistory, produce_adapted_cover,
    produce_monomial_blowup,
};
pub use chart::{Boundary, Chart, FrameCertificate, Guard, Map, VerifiedFrame};
pub use companion::{
    CartierDivision, FactorProduction, FactorProgress, OldBoundaryCoefficient,
    OldBoundaryProduction, OldBoundaryProgress, OldIncidence, QuotientMethod, RelativeOrderLayer,
    ResidualOrderProduction, ResidualOrderProgress, RestrictedResidualOrder,
    VerifiedCartierQuotient, WholeCartierFactorization, divide_cartier,
    factor_whole_cartier_equations, produce_old_boundary_coefficient,
    produce_restricted_residual_order,
};
pub use components::{
    BoundaryAlgebra, ComponentOpen, ComponentPattern, ComponentProduction, ComponentProgress,
    EliminationEvidence, RegularAlgebra, RegularOrigin, VerifiedAnnihilator,
    VerifiedComponentSplit, produce_component_split,
};
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
pub use iteration::{
    CarriedMonomialWitness, CompletedMonomialResolution, IterationAdvance, IterationCompletion,
    IterationLimits, MonomialFrontier, NodePath, PendingEvidence, ResolutionNode,
    carry_monomial_witness, exponent_rank,
};
pub use localization::{
    LocalizationProduction, LocalizationProgress, LocalizedHistory, VerifiedPrincipalOpen,
    localize_component_open,
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
