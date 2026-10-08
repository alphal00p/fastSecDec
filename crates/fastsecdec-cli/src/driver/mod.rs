//! CLI-owned scheduling, workers, stopping policy, and checkpoint files.
mod checkpoint;
mod execution;
mod refinement;
mod replay;
pub(crate) mod report;
#[cfg(test)]
mod selection_tests;
pub(crate) mod serial;
#[cfg(test)]
mod tests;

pub use execution::integrate;
pub use report::{IntegrationReport, ResumeStatus};
