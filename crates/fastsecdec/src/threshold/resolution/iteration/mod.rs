//! Caller-stepped local monomial cosupport resolution.
//!
//! Every branch retains a checked transform and strictly decreasing monomial
//! rank. Completion is algebraic: a real integration atlas, the general BM
//! invariant, and durable worker checkpoints remain separate obligations.
mod carry;
mod frontier;
pub use carry::{CarriedMonomialWitness, carry_monomial_witness, exponent_rank};
pub use frontier::{
    CompletedMonomialResolution, IterationAdvance, IterationCompletion, IterationLimits,
    MonomialFrontier, NodePath, PendingEvidence, ResolutionNode,
};

#[cfg(test)]
mod tests;
