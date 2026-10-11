//! Distance-selected native evaluations of one complete Laurent vector.
use super::{
    Backend, EvaluationTimings, KernelError, PrecisionClass, PrecisionReport, SectorKernel,
};
use symbolica::domains::float::{DoubleFloat, Float, RealLike};

impl SectorKernel {
    pub fn evaluation_metrics(&self) -> EvaluationTimings {
        match &self.backend {
            Backend::Complex(kernel) => kernel.evaluation_metrics(),
            Backend::Real(kernel) => {
                let mut conditioning = kernel.conditioning_timing;
                if let Some(cache) = &kernel.root_precision {
                    conditioning.add(cache.timing);
                }
                EvaluationTimings {
                    f64: kernel.f64_timing,
                    double_float: kernel.double_cache.timing,
                    arbitrary: kernel.precision_cache.timing,
                    conditioning,
                }
            }
        }
    }

    pub(super) fn evaluate_distance_class(
        &mut self,
        output: &mut [f64],
        weight: f64,
        mut class: PrecisionClass,
    ) -> Result<PrecisionReport, KernelError> {
        if !self.algebraic.is_empty() {
            return Err(KernelError::Stability(
                "algebraic-root kernels require validated stability without cutoff zeros".into(),
            ));
        }
        if self.projection.is_some() {
            return self.project_output(output, None, |kernel, values, _| {
                kernel.evaluate_distance_class(values, weight, class)
            });
        }
        loop {
            if class == PrecisionClass::Unstable {
                output.fill(0.0);
            } else {
                match &mut self.backend {
                    Backend::Complex(kernel) => {
                        kernel.evaluate_at_class(&self.input, output, weight, class)?
                    }
                    Backend::Real(kernel) => match class {
                        PrecisionClass::F64 => {
                            let started = std::time::Instant::now();
                            kernel.evaluator.evaluate(&self.input, output);
                            kernel.f64_timing.record(started);
                            for value in output.iter_mut() {
                                *value *= weight;
                            }
                        }
                        PrecisionClass::DoubleFloat => {
                            let scale = DoubleFloat::from(weight);
                            let values = kernel.double_cache.evaluate(
                                &kernel.exact_evaluator,
                                &self.input,
                                106,
                                |c| DoubleFloat::from(&c.re),
                                DoubleFloat::from,
                            )?;
                            for (out, value) in output.iter_mut().zip(values) {
                                *out = (*value * scale).to_f64();
                            }
                        }
                        PrecisionClass::Arbitrary => {
                            let bits = class.bits();
                            let scale = Float::with_val(bits, weight);
                            let values = kernel.precision_cache.evaluate(
                                &kernel.exact_evaluator,
                                &self.input,
                                bits,
                                |c| c.re.to_multi_prec_float(bits),
                                |v| Float::with_val(bits, v),
                            )?;
                            for (out, value) in output.iter_mut().zip(values) {
                                *out = (value.clone() * &scale).to_f64();
                            }
                        }
                        PrecisionClass::Unstable => unreachable!(),
                    },
                }
            }
            if output.iter().all(|value| value.is_finite()) {
                return Ok(PrecisionReport {
                    rescued: matches!(
                        class,
                        PrecisionClass::DoubleFloat | PrecisionClass::Arbitrary
                    ),
                    checked: false,
                    bits: class.bits(),
                    class,
                    timings: Default::default(),
                });
            }
            class = match class {
                PrecisionClass::F64 => PrecisionClass::DoubleFloat,
                PrecisionClass::DoubleFloat => PrecisionClass::Arbitrary,
                _ => return Err(KernelError::NonFinite),
            };
        }
    }
}
