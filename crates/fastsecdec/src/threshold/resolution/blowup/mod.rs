//! Verified adapted opens and controlled monomial blowup charts.
//!
//! These are overlapping algebraic charts with relative Jacobians and checked
//! local exceptional history. They are not a disjoint real integration atlas,
//! a global BM center-gluing algorithm or a complete endpoint certificate.
mod adapted;
mod helpers;
mod history;
mod transform;
pub use adapted::{
    AdaptedCover, AdaptedOpen, AdaptedProduction, AdaptedProgress, EmptyAdaptedOpen,
    produce_adapted_cover,
};
pub use history::{BirthContext, HistoryChartStep, ResolutionHistory};
pub use transform::{
    BlowupProduction, BlowupProgress, DivisorReceipt, MonomialBlowup, MonomialBlowupChart,
    produce_monomial_blowup,
};

#[cfg(test)]
mod tests;
