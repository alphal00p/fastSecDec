//! Typed external-reference alignment over existing native integration results.
//! This module estimates no uncertainty and makes no normalization conversions.
mod compare;
mod display;
mod document;
mod legacy;
mod types;

pub use compare::compare;
pub use document::{encode_reference, read_reference};
pub use legacy::read_historical_target;
pub use types::*;

#[derive(Debug, thiserror::Error)]
pub enum ReferenceError {
    #[error("invalid reference comparison input: {0}")]
    Invalid(String),
    #[error("reference kernel identity {reference} differs from estimate identity {estimate}")]
    KernelIdentityMismatch { reference: String, estimate: String },
    #[error("nonfinite derived {quantity} for {key:?}")]
    NumericRange {
        key: CoefficientKey,
        quantity: &'static str,
    },
    #[error("invalid historical target JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ReferenceError>;
