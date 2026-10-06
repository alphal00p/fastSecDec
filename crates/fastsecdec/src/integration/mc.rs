//! Havana importance sampling with caller-owned batches and adaptation stages.
//!
//! Every production batch has the same sample count and a distinct jumped RNG
//! stream. The importance grid is frozen in production; pilot samples only
//! train it. Vector uncertainty uses independent batch means, retaining the
//! complete covariance without treating adaptive pilot observations as data.
mod config;
mod session;
mod worker;
pub(crate) use worker::training_envelope;

pub use config::HavanaSettings;
pub use session::HavanaSession;
pub use worker::{HavanaReturn, HavanaTask, HavanaWorker};
