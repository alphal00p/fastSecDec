//! Saved numerical observations, independent of graphs and compiled evaluators.
//! Validation checks representation and declared scope, not scientific truth.
mod display;
mod document;
mod manifest;
mod reference;
mod types;
mod validation;

pub use display::SavedResultView;
pub use document::{encode_result, read_result};
pub use types::*;

#[derive(Debug, thiserror::Error)]
pub enum ResultError {
    #[error("invalid saved integration result: {0}")]
    Invalid(String),
    #[error("selected-sector results cannot become full-integral references")]
    SelectedScope,
    #[error("the selected result reference source is absent")]
    MissingReferenceSource,
    #[error("computed estimate is ineligible for reference extraction: {0}")]
    IneligibleEstimate(String),
    #[error("unknown coefficient key {0:?}")]
    UnknownCoefficient(crate::reference::CoefficientKey),
    #[error(transparent)]
    Integration(#[from] crate::integration::IntegrationError),
    #[error(transparent)]
    Reference(#[from] crate::reference::ReferenceError),
    #[error("invalid saved-result JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ResultError>;
