//! Native HEPKit input. There is deliberately no second graph or DOT parser.

mod family;
mod graph;
mod numerator;
mod runtime_model;
mod validation;

pub use family::prepare_family_input;
pub use graph::GraphIntegral;
pub use numerator::{contract_numerator, default_algebra_settings};
pub use runtime_model::RuntimeModelBindings;
