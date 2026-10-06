use serde::{Deserialize, Serialize};

use crate::generation::{
    CoefficientExpansionMethod, CoefficientExpansionStage, CoefficientRequestCounts,
};

/// Current representative's coefficient work, separate from physical
/// subtraction contributions. Counts reset at each native Series attempt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoefficientExpansionSnapshot {
    /// Zero-based representative currently being processed or most recently
    /// completed. The outer snapshot counts only completed representatives.
    pub sector: usize,
    pub requested_method: CoefficientExpansionMethod,
    /// Physical identifies the exact unregulated-endpoint fallback when the
    /// requested method is NativeNamed; it is not a retry after failure.
    pub effective_method: CoefficientExpansionMethod,
    pub stage: CoefficientExpansionStage,
    /// One-based native attempt; zero denotes admission before any attempt or
    /// exact physical fallback, which performs no named Series attempt.
    pub attempt: usize,
    /// Native relative width for the current attempt; zero has the same
    /// pre-attempt/fallback meaning. This is not an absolute Laurent cutoff.
    pub relative_width: i64,
    /// Endpoint-composition pieces in this attempt, recorded at Coverage and
    /// retained through Lowering/Complete. Earlier zeroes are placeholders;
    /// physical fallback has no named-composition count.
    pub formal_pieces: usize,
    /// Native counters in this attempt. Distinct requests and aliases are
    /// running counts only at Lowering and final counts at Complete; earlier
    /// zeroes do not mean those operations have completed with no work.
    pub requests: CoefficientRequestCounts,
}
