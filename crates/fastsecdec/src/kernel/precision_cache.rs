//! Bounded worker-local native evaluator storage. Cache history affects only
//! allocation and constant conversion, never the chosen precision or result.
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
}

impl<T> Default for PrecisionCache<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
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
    ) -> &[T] {
        if let Some(index) = self.entries.iter().position(|entry| entry.bits == bits) {
            let entry = self.entries.remove(index);
            self.entries.push(entry);
        } else {
            if self.entries.len() == CAPACITY {
                self.entries.remove(0);
            }
            self.entries.push(Entry {
                bits,
                evaluator: exact.clone().map_coeff_with_prec(&coefficient, bits),
                input: vec![number(0.0); exact.get_input_len()],
                output: vec![number(0.0); exact.get_output_len()],
            });
        }
        let entry = self.entries.last_mut().unwrap();
        for (target, value) in entry.input.iter_mut().zip(point) {
            *target = number(*value);
        }
        entry.evaluator.evaluate(&entry.input, &mut entry.output);
        &entry.output
    }
}
