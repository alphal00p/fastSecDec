//! CLI-owned scheduling, workers, stopping policy, and checkpoint files.
mod checkpoint;
mod execution;
mod refinement;
mod replay;
mod report;
#[cfg(test)]
mod tests;

pub use execution::integrate;
pub use report::{IntegrationReport, ResumeStatus};
