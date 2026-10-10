//! Worker-owned, lazy native roundoff tracking for the validated policy.
use super::{ExactProgram, MappingRequirements};
use std::sync::Arc;
use symbolica::{
    domains::{float::Complex, rational::Rational},
    evaluate::{EvaluationDomain, ExpressionEvaluator},
};

#[derive(Clone)]
pub(in crate::kernel) struct Conditioning<T> {
    // None: not requested; Some(None): the native numeric domain is unsupported.
    evaluator: Option<Option<ConditioningEvaluator<T>>>,
    requirements: Arc<MappingRequirements>,
}

#[derive(Clone)]
pub(in crate::kernel) struct ConditioningEvaluator<T> {
    evaluator: ExpressionEvaluator<T>,
    dynamic: bool,
    validation: Option<crate::kernel::contour::dynamic::validation::Validation>,
    last_error: Option<String>,
}
impl<T: symbolica::domains::float::Real> ConditioningEvaluator<T> {
    /// Return whether essential callback evaluation failed, independently of
    /// whether a later instruction made the tracked output finite again.
    pub(in crate::kernel) fn evaluate_at(
        &mut self,
        input: &[T],
        output: &mut [T],
        point: &[f64],
    ) -> bool {
        self.last_error = None;
        if let Some(validation) = &mut self.validation {
            let valid = validation.evaluate(point, || self.evaluator.evaluate(input, output));
            self.last_error = validation.last_error.clone();
            !valid
        } else if self.dynamic {
            let (_, failure) = crate::contour::functions::dynamic::isolated_attempt(|| {
                self.evaluator.evaluate(input, output)
            });
            self.last_error = failure;
            self.last_error.is_some()
        } else {
            self.evaluator.evaluate(input, output);
            false
        }
    }
}

impl<T> Conditioning<T> {
    pub(in crate::kernel) fn clear_dynamic_attempt(&mut self) {
        if let Some(Some(evaluator)) = &mut self.evaluator {
            evaluator.last_error = None;
            if let Some(validation) = &mut evaluator.validation {
                validation.clear_attempt();
            }
        }
    }
    pub(in crate::kernel) fn last_dynamic_error(&self) -> Option<&str> {
        self.evaluator.as_ref()?.as_ref()?.last_error.as_deref()
    }
    pub(in crate::kernel) fn validation(
        &self,
    ) -> Option<&crate::kernel::contour::dynamic::validation::Validation> {
        self.evaluator.as_ref()?.as_ref()?.validation.as_ref()
    }
    pub(in crate::kernel) fn validation_mut(
        &mut self,
    ) -> Option<&mut crate::kernel::contour::dynamic::validation::Validation> {
        self.evaluator.as_mut()?.as_mut()?.validation.as_mut()
    }
    pub(in crate::kernel) fn new(requirements: Arc<MappingRequirements>) -> Self {
        Self {
            evaluator: None,
            requirements,
        }
    }

    #[cfg(test)]
    pub(in crate::kernel) fn attempted(&self) -> bool {
        self.evaluator.is_some()
    }
}

impl<T: EvaluationDomain> Conditioning<T> {
    pub(in crate::kernel) fn get_or_map(
        &mut self,
        exact: &ExactProgram,
        coefficient: impl Fn(&Complex<Rational>) -> T,
    ) -> Option<&mut ConditioningEvaluator<T>> {
        self.evaluator
            .get_or_insert_with(|| {
                self.requirements
                    .map(exact, coefficient, 53)
                    .ok()
                    .map(|evaluator| ConditioningEvaluator {
                        evaluator,
                        dynamic: self.requirements.has_dynamic_callbacks(),
                        validation: self.requirements.validation(),
                        last_error: None,
                    })
            })
            .as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use symbolica::{atom::AtomCore, domains::float::ErrorPropagatingFloat, parse, symbol};

    #[test]
    fn native_mapping_is_attempted_once_per_worker_owner() {
        let x = symbol!("lazy_conditioning_map::x");
        let exact = parse!("2+lazy_conditioning_map::x")
            .evaluator(&[symbolica::atom::Atom::var(x)])
            .build()
            .unwrap();
        let mut conditioning = Conditioning::new(MappingRequirements::new(&exact).unwrap());
        let calls = Cell::new(0);
        let map = |value: &Complex<Rational>| {
            calls.set(calls.get() + 1);
            ErrorPropagatingFloat::new(value.re.to_f64(), 15.0)
        };
        assert!(!conditioning.attempted());
        assert!(conditioning.get_or_map(&exact, map).is_some());
        let mapped = calls.get();
        assert!(mapped > 0);
        assert!(conditioning.get_or_map(&exact, map).is_some());
        assert_eq!(calls.get(), mapped);
        assert!(conditioning.clone().get_or_map(&exact, map).is_some());
        assert_eq!(calls.get(), mapped);
    }
}
