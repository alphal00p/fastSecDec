//! Size of the actual shared evaluator compiled for a complete sector vector.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluatorOperations {
    pub additions: usize,
    pub multiplications: usize,
    pub inversions: usize,
    pub function_calls: usize,
}
impl From<symbolica::evaluate::OperationCount> for EvaluatorOperations {
    fn from(value: symbolica::evaluate::OperationCount) -> Self {
        Self {
            additions: value.additions,
            multiplications: value.multiplications,
            inversions: value.inversions,
            function_calls: value.function_calls,
        }
    }
}

/// Counts apply to the complete shared native program, never separately to a
/// Laurent coefficient. Operations are Symbolica's exact evaluator operations
/// before SymJIT real/complex lowering and optimization, not machine operations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluatorStatistics {
    /// Surviving first-image-partial inputs supplied by contour-only dual
    /// arithmetic after symbolic endpoint reduction. Known only at build:
    /// Some(0) is native cancellation; None is unknown/restored or inapplicable.
    /// This observation is not part of the saved executable or its identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbolic_endpoint_contour_partials: Option<usize>,
    pub version: u32,
    pub backend: String,
    pub arithmetic: String,
    pub inputs: usize,
    /// Shared evaluator outputs; complex outputs later split into components.
    pub outputs: usize,
    pub exact_program_bytes: usize,
    pub operations: EvaluatorOperations,
    /// Compressed serialized SymJIT application, not executable machine-code
    /// bytes. None for the portable interpreter; measured on this host's build.
    pub symjit_ir_bytes: Option<usize>,
}
