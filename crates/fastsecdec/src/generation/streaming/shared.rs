//! Common geometry and immutable residual preparation; recipe execution stays
//! in independent caller-owned jobs and retains its own exact proof context.
mod discover;
mod prepare;
mod source;

pub use discover::discover_prepared;
pub use prepare::{prepare_recipes_with_runtime, prepare_recipes_with_runtime_and_dispatch};
pub use source::prepare_chart_source;
