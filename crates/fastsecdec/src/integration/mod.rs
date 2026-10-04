//! Caller-driven integration of complete sector coefficient vectors.
//!
//! Scheduling and worker pools belong to the caller. Lattices, random shifts,
//! partial statistics and grid adaptation reuse Numerica/Havana.
mod adaptive;
mod config;
mod estimate;
pub mod mc;
mod qmc;
mod worker;

pub use crate::status::CoefficientComponent;
pub use config::{
    IntegrationProblem, Periodization, QmcSettings, RuleSource, SectorSpec, Tolerance,
};
pub use estimate::VectorEstimate;
pub use qmc::{ProductionAllocation, QmcSession};
pub use worker::{QmcReturn, QmcTask, QmcWorker};

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("invalid integration configuration: {0}")]
    Invalid(String),
    #[error("invalid worker return: {0}")]
    InvalidReturn(String),
    #[error("integrand evaluation failed: {0}")]
    Evaluation(String),
    #[error("integration uncertainty is not available: {0}")]
    Unavailable(String),
    #[error(transparent)]
    Qmc(#[from] numerica::numerical_integration::qmc::QmcError),
    #[error("invalid integration checkpoint: {0}")]
    Checkpoint(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
