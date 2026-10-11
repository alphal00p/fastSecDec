//! Verified adapted opens and controlled relative blowup charts.
//!
//! These are overlapping algebraic charts with relative Jacobians and checked
//! local exceptional history. They are not a disjoint real integration atlas,
//! a global BM center-gluing algorithm or a complete endpoint certificate.
mod adapted;
mod general;
mod general_transform;
mod graph;
mod helpers;
mod saturation;
mod standard;
mod support;
pub use general::{
    CheckedRecursiveCenter, RecursiveAdaptation, RecursiveAdaptedOpen, RecursiveCenterOrigin,
    adapt_recursive_center,
};
pub use general_transform::{
    RelativeBlowupGeometry, RelativeMarkedTransform, RelativeRecursiveBlowup,
    RelativeRecursiveChart, blowup_recursive_center,
};
pub use saturation::SaturatedSupport;
pub use support::{StrictContactSupport, StrictSupportOpen, transport_contact_support};
mod history;
mod transform;
pub use adapted::{
    AdaptedCover, AdaptedOpen, AdaptedProduction, AdaptedProgress, EmptyAdaptedOpen,
    produce_adapted_cover,
};
pub use history::{BirthContext, HistoryCenter, HistoryChartStep, HistoryStep, ResolutionHistory};
pub use transform::{
    BlowupProduction, BlowupProgress, DivisorReceipt, MonomialBlowup, MonomialBlowupChart,
    produce_monomial_blowup,
};

#[cfg(test)]
mod tests;

mod coefficient;
pub use coefficient::{
    CarriedCompanionChart, CarriedCompanionSupport, SupportedGenerator, SupportedMarkedTransform,
    carry_companion_chart,
};

mod induced;
pub use induced::{
    CarriedMonomialCenter, EmbeddedChildCycle, InducedCenterProduction, InducedChildChart,
    induce_child_chart, produce_induced_monomial_center,
};

mod terminal;
pub use terminal::{
    LocalPrincipalization, PrincipalizationProduction, certify_local_principalization,
};

mod continuation;
pub use continuation::{
    CompletedLocalContinuation, ContinuationAdvance, ContinuationCompletion, ContinuationLimits,
    ContinuationNode, LocalCompanionContinuation,
};

#[cfg(test)]
mod relative_tests;
