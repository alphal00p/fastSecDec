//! Caller-driven integration of complete sector coefficient vectors.
//!
//! Scheduling and worker pools belong to the caller. Lattices, random shifts,
//! partial statistics and grid adaptation reuse Numerica/Havana.
mod adaptive;
mod config;
mod contributions;
mod estimate;
pub mod mc;
pub mod mc_discrete;
mod observation;
mod qmc;
mod worker;

pub use crate::status::CoefficientComponent;
pub use config::{
    IntegrationProblem, Periodization, QmcSettings, RuleSource, SectorSpec, Tolerance,
};
pub use contributions::{ContributionReport, ReplicaRelation, SectorContribution};
pub use estimate::VectorEstimate;
pub use numerica::numerical_integration::qmc::PublishedLattice;
pub use observation::IntegrationObservation;
pub use qmc::{ProductionAllocation, QmcDesign, QmcSession};
pub use worker::{QmcReturn, QmcTask, QmcWorker};

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("invalid integration configuration: {0}")]
    Invalid(String),
    #[error("invalid worker return: {0}")]
    InvalidReturn(String),
    #[error("integrand evaluation failed: {0}")]
    Evaluation(String),
    #[error(
        "weighted MC pilot envelope {weighted_envelope} cannot be represented by the native Havana training API at sampled weight {sampled_weight}; production accepts final weighted coefficients without inverse scaling"
    )]
    UnrepresentablePilotEnvelope {
        weighted_envelope: f64,
        sampled_weight: f64,
    },
    #[error("integration uncertainty is not available: {0}")]
    Unavailable(String),
    #[error("native numerical accumulation exceeded representable range")]
    NumericRange,
    #[error(transparent)]
    Qmc(#[from] numerica::numerical_integration::qmc::QmcError),
    #[error("invalid integration checkpoint: {0}")]
    Checkpoint(#[from] serde_json::Error),
}

impl IntegrationError {
    pub(crate) fn is_statistical_range(&self) -> bool {
        matches!(
            self,
            Self::NumericRange
                | Self::Qmc(
                    numerica::numerical_integration::qmc::QmcError::NumericOverflow
                        | numerica::numerical_integration::qmc::QmcError::NonFiniteValue
                )
        )
    }
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
