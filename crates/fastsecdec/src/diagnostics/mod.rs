//! Caller-cancellable kernel diagnostics with explicit, bounded sample coverage.
//!
//! Boundary samples diagnose numerical evaluation; they are not a certificate
//! of positivity or integrability. Scheduling, presentation and storage belong
//! to the caller. No worker threads are created here.

mod benchmark;
mod boundary;
mod growth;
mod types;

pub use benchmark::benchmark;
pub use boundary::boundaries;
pub use growth::*;
pub use types::*;

#[derive(Debug, thiserror::Error)]
pub enum DiagnosticError {
    #[error("invalid diagnostic request: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    CounterOverflow(#[from] crate::status::DiagnosticsOverflow),
}

type Result<T> = std::result::Result<T, DiagnosticError>;
