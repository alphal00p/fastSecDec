//! One numerical owner and its independent optional certificate workspace.
//! No observer or ball state exists in the unchecked evaluator variants.
use crate::kernel::contour::dynamic::validation::{Specification, Validation};
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::kernel) struct Checked<T> {
    pub evaluator: T,
    pub validation: Validation,
    pub point: Vec<f64>,
}
impl<T> Checked<T> {
    pub fn new(evaluator: T, specification: Arc<Specification>) -> Self {
        Self {
            evaluator,
            validation: Validation::new(specification),
            point: Vec::new(),
        }
    }
}
