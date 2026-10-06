//! Native Havana discrete importance sampling over sectors and their coordinates.
//!
//! A task is an independent global batch. Each sample selects a sector with the
//! native DiscreteGrid, then its ContinuousGrid. Production grids are frozen;
//! pilot observations never enter production means or full-vector covariance.
mod config;
mod session;
mod worker;
pub use config::HavanaDiscreteSettings;
pub use session::HavanaDiscreteSession;
pub use worker::{HavanaDiscreteReturn, HavanaDiscreteTask, HavanaDiscreteWorker};
