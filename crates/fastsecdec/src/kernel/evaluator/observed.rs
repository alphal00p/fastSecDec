//! Optional operational counters around one numerical owner, including failures.
use crate::contour::functions::dynamic::diagnostics::{Accumulator, Configuration};
#[derive(Clone)]
pub(in crate::kernel) struct Observed<T> {
    pub evaluator: T,
    pub configuration: Configuration,
    pub diagnostics: Accumulator,
}
impl<T> Observed<T> {
    pub fn new(evaluator: T, configuration: Configuration) -> Self {
        Self {
            evaluator,
            configuration,
            diagnostics: Default::default(),
        }
    }
}
