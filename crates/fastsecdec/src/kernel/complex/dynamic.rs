//! Dynamic numerical factory replacement and observational accounting.
use super::ComplexKernel;
use crate::kernel::{
    PrecisionClass,
    contour::dynamic::validation::{Coverage, Validation},
    evaluator::{ComplexEvaluator, Conditioning, MappingRequirements},
    precision_cache::PrecisionCache,
};
use std::sync::Arc;

impl ComplexKernel {
    pub(in crate::kernel) fn execution_backend(&self) -> crate::kernel::EvaluatorBackend {
        self.evaluator.execution_backend()
    }
    pub(in crate::kernel) fn has_dynamic_callbacks(&self) -> bool {
        self.evaluator.has_dynamic_callbacks()
    }
    pub(in crate::kernel) fn dynamic_validation(&self) -> Option<&Validation> {
        self.evaluator.validation()
    }
    pub(in crate::kernel) fn visit_dynamic_validations(&self, visit: &mut impl FnMut(&Validation)) {
        for validation in [
            self.evaluator.validation(),
            self.double_cache.validation.as_ref(),
            self.precision_cache.validation.as_ref(),
            self.conditioning.validation(),
        ]
        .into_iter()
        .flatten()
        {
            visit(validation);
        }
    }
    pub(in crate::kernel) fn visit_dynamic_validations_mut(
        &mut self,
        visit: &mut impl FnMut(&mut Validation),
    ) {
        for validation in [
            self.evaluator.validation_mut(),
            self.double_cache.validation.as_mut(),
            self.precision_cache.validation.as_mut(),
            self.conditioning.validation_mut(),
        ]
        .into_iter()
        .flatten()
        {
            visit(validation);
        }
    }
    pub(in crate::kernel) fn dynamic_coverage(&self, class: PrecisionClass) -> Option<&Coverage> {
        match class {
            PrecisionClass::F64 => self.evaluator.validation(),
            PrecisionClass::DoubleFloat => self.double_cache.validation.as_ref(),
            PrecisionClass::Arbitrary => self.precision_cache.validation.as_ref(),
            PrecisionClass::Unstable => None,
        }
        .and_then(Validation::coverage)
    }
    pub(in crate::kernel) fn apply_dynamic_mapping(
        &mut self,
        evaluator: ComplexEvaluator,
        requirements: Arc<MappingRequirements>,
    ) {
        self.evaluator = evaluator;
        let mut double_cache = PrecisionCache::new(requirements.clone());
        double_cache.timing = self.double_cache.timing;
        self.double_cache = double_cache;
        let mut precision_cache = PrecisionCache::new(requirements.clone());
        precision_cache.timing = self.precision_cache.timing;
        self.precision_cache = precision_cache;
        self.conditioning = Conditioning::new(requirements);
    }
}
