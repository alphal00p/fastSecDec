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
    evaluator: Option<Option<ExpressionEvaluator<T>>>,
    requirements: Arc<MappingRequirements>,
}

impl<T> Conditioning<T> {
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
    ) -> Option<&mut ExpressionEvaluator<T>> {
        self.evaluator
            .get_or_insert_with(|| self.requirements.map(exact, coefficient, 53).ok())
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
