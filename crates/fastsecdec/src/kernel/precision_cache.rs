//! Bounded worker-local native evaluator storage. Cache history affects only
//! allocation and constant conversion, never the chosen precision or result.
use super::{KernelError, evaluator::MappingRequirements};
use std::sync::Arc;
use symbolica::{
    domains::{
        float::{Complex, Real},
        rational::Rational,
    },
    evaluate::{EvaluationDomain, ExpressionEvaluator},
};

const CAPACITY: usize = 4;

struct Entry<T> {
    bits: u32,
    evaluator: ExpressionEvaluator<T>,
    input: Vec<T>,
    output: Vec<T>,
}

pub(super) struct PrecisionCache<T> {
    entries: Vec<Entry<T>>,
    requirements: Arc<MappingRequirements>,
    pub(super) timing: super::EvaluatorTiming,
}

impl<T> PrecisionCache<T> {
    pub(super) fn new(requirements: Arc<MappingRequirements>) -> Self {
        Self {
            entries: Vec::new(),
            requirements,
            timing: Default::default(),
        }
    }

    pub(super) fn empty_clone(&self) -> Self {
        Self::new(self.requirements.clone())
    }
}

impl<T: EvaluationDomain + Real> PrecisionCache<T> {
    pub(super) fn evaluate(
        &mut self,
        exact: &ExpressionEvaluator<Complex<Rational>>,
        point: &[f64],
        bits: u32,
        coefficient: impl Fn(&Complex<Rational>) -> T,
        number: impl Fn(f64) -> T,
    ) -> Result<&[T], KernelError> {
        if let Some(index) = self.entries.iter().position(|entry| entry.bits == bits) {
            let entry = self.entries.remove(index);
            self.entries.push(entry);
        } else {
            let evaluator = self
                .requirements
                .map(exact, coefficient, bits)
                .map_err(KernelError::PrecisionEvaluation)?;
            if self.entries.len() == CAPACITY {
                self.entries.remove(0);
            }
            self.entries.push(Entry {
                bits,
                evaluator,
                input: vec![number(0.0); exact.get_input_len()],
                output: vec![number(0.0); exact.get_output_len()],
            });
        }
        let entry = self.entries.last_mut().unwrap();
        for (target, value) in entry.input.iter_mut().zip(point) {
            *target = number(*value);
        }
        let started = std::time::Instant::now();
        entry.evaluator.evaluate(&entry.input, &mut entry.output);
        self.timing.record(started);
        Ok(&entry.output)
    }
}
