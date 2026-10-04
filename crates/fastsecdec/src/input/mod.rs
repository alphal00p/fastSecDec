//! Native HEPKit input. There is deliberately no second graph or DOT parser.

mod graph;
mod numerator;
mod validation;

pub use graph::GraphIntegral;
pub use numerator::{contract_numerator, default_algebra_settings};
