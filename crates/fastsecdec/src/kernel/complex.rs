//! Native complex kernels preserve factored coefficient expressions. Numerical
//! output uses adjacent real/imaginary components for each Laurent coefficient.
use super::{
    KernelError, PrecisionPolicy, PrecisionReport, cancellation::Cancellation, evaluator, precision,
};
use symbolica::{
    atom::{Atom, AtomCore},
    domains::{
        float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, RealLike},
        rational::Rational,
    },
    evaluate::ExpressionEvaluator,
};

pub(super) struct ComplexKernel {
    precision_cache: super::precision_cache::PrecisionCache<Complex<Float>>,
    double_cache: super::precision_cache::PrecisionCache<Complex<DoubleFloat>>,
    f64_timing: super::EvaluatorTiming,
    conditioning_timing: super::EvaluatorTiming,
    exact: ExpressionEvaluator<Complex<Rational>>,
    evaluator: evaluator::ComplexEvaluator,
    conditioning: Option<ExpressionEvaluator<Complex<ErrorPropagatingFloat<f64>>>>,
    input: Vec<Complex<f64>>,
    integration_dimension: usize,
    output: Vec<Complex<f64>>,
    check_input: Vec<Complex<ErrorPropagatingFloat<f64>>>,
    check_output: Vec<Complex<ErrorPropagatingFloat<f64>>>,
    cancellation: Cancellation,
    precision: PrecisionPolicy,
    exact_zero: Vec<bool>,
    real_coefficients: Vec<bool>,
}

impl ComplexKernel {
    pub(super) fn evaluate_primary_batch(
        &mut self,
        input: &[f64],
        output: &mut [f64],
        rows: usize,
    ) -> Vec<super::EvaluatorTiming> {
        let input = input
            .iter()
            .map(|v| Complex::new(*v, 0.0))
            .collect::<Vec<_>>();
        let mut values = vec![Complex::new(0.0, 0.0); rows * self.output.len()];
        let timings = self.evaluator.evaluate_batch(
            &input,
            &mut values,
            rows,
            self.input.len(),
            self.output.len(),
        );
        for (out, value) in output.as_chunks_mut::<2>().0.iter_mut().zip(values) {
            out.copy_from_slice(&[value.re, value.im]);
        }
        for timing in &timings {
            self.f64_timing.add(*timing);
        }
        timings
    }

    pub(super) fn evaluation_metrics(&self) -> super::EvaluationTimings {
        super::EvaluationTimings {
            f64: self.f64_timing,
            double_float: self.double_cache.timing,
            arbitrary: self.precision_cache.timing,
            conditioning: self.conditioning_timing,
        }
    }

    pub(super) fn evaluate_at_class(
        &mut self,
        point: &[f64],
        output: &mut [f64],
        weight: f64,
        class: super::PrecisionClass,
    ) -> Result<(), KernelError> {
        use super::PrecisionClass;
        match class {
            PrecisionClass::F64 => {
                for (input, value) in self.input.iter_mut().zip(point) {
                    *input = Complex::new(*value, 0.0);
                }
                let started = std::time::Instant::now();
                self.evaluator.evaluate(&self.input, &mut self.output);
                self.f64_timing.record(started);
                for (target, value) in output.as_chunks_mut::<2>().0.iter_mut().zip(&self.output) {
                    target.copy_from_slice(&[value.re * weight, value.im * weight]);
                }
            }
            PrecisionClass::DoubleFloat => {
                let scale = DoubleFloat::from(weight);
                let values = self.double_cache.evaluate(
                    &self.exact,
                    point,
                    106,
                    |c| Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im)),
                    |v| Complex::new(DoubleFloat::from(v), DoubleFloat::from(0.0)),
                )?;
                for (target, value) in output.as_chunks_mut::<2>().0.iter_mut().zip(values) {
                    target.copy_from_slice(&[
                        (value.re * scale).to_f64(),
                        (value.im * scale).to_f64(),
                    ]);
                }
            }
            PrecisionClass::Arbitrary => {
                let bits = class.bits();
                let scale = Float::with_val(bits, weight);
                let values = self.precision_cache.evaluate(
                    &self.exact,
                    point,
                    bits,
                    |c| {
                        Complex::new(
                            c.re.to_multi_prec_float(bits),
                            c.im.to_multi_prec_float(bits),
                        )
                    },
                    |v| Complex::new(Float::with_val(bits, v), Float::with_val(bits, 0)),
                )?;
                for (target, value) in output.as_chunks_mut::<2>().0.iter_mut().zip(values) {
                    target.copy_from_slice(&[
                        (value.re.clone() * &scale).to_f64(),
                        (value.im.clone() * &scale).to_f64(),
                    ]);
                }
            }
            PrecisionClass::Unstable => output.fill(0.0),
        }
        Ok(())
    }
    pub(super) fn symjit_ir_bytes(&self) -> Option<usize> {
        self.evaluator.symjit_ir_bytes()
    }
    #[cfg(test)]
    pub(super) fn new(
        parameters: &[symbolica::atom::Symbol],
        coefficients: &[Atom],
        cancellation: Cancellation,
        precision: PrecisionPolicy,
    ) -> Result<Self, KernelError> {
        let variables = parameters
            .iter()
            .map(|symbol| Atom::var(*symbol))
            .collect::<Vec<_>>();
        let exact = Atom::evaluator_multiple(coefficients, &variables)
            .build()
            .map_err(|error| KernelError::Compilation(error.to_string()))?;
        Self::from_program(
            exact,
            parameters.len(),
            cancellation,
            precision,
            coefficients.iter().map(|value| value.is_zero()).collect(),
            coefficients
                .iter()
                .map(|value| super::program::is_real_coordinate_expression(value, parameters))
                .collect(),
            super::EvaluatorBackend::Auto,
        )
    }

    pub(super) fn from_program(
        exact: super::program::ExactProgram,
        integration_dimension: usize,
        cancellation: Cancellation,
        precision: PrecisionPolicy,
        exact_zero: Vec<bool>,
        real_coefficients: Vec<bool>,
        backend: super::EvaluatorBackend,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let inputs = exact.get_input_len();
        let outputs = exact.get_output_len();
        let evaluator = evaluator::complex(&exact, backend)?;
        let requirements =
            evaluator::MappingRequirements::new(&exact).map_err(KernelError::Compilation)?;
        let conditioning = requirements
            .map(
                &exact,
                |coefficient| {
                    Complex::new(
                        tracked(coefficient.re.to_f64()),
                        tracked(coefficient.im.to_f64()),
                    )
                },
                53,
            )
            .ok();
        Ok(Self {
            double_cache: super::precision_cache::PrecisionCache::new(requirements.clone()),
            f64_timing: Default::default(),
            conditioning_timing: Default::default(),
            precision_cache: super::precision_cache::PrecisionCache::new(requirements),
            exact,
            evaluator,
            conditioning,
            input: vec![Complex::new(0.0, 0.0); inputs],
            integration_dimension,
            output: vec![Complex::new(0.0, 0.0); outputs],
            check_input: vec![Complex::new(tracked(0.0), tracked(0.0)); inputs],
            check_output: vec![Complex::new(tracked(0.0), tracked(0.0)); outputs],
            cancellation,
            precision,
            exact_zero,
            // Only native symbolic proofs under the real input domain certify
            // an identically zero imaginary component. Real literals alone
            // do not establish a square-root or logarithm branch.
            real_coefficients,
        })
    }

    #[cfg(test)]
    pub(super) fn evaluate(
        &mut self,
        point: &[f64],
        output: &mut [f64],
    ) -> Result<PrecisionReport, KernelError> {
        self.evaluate_scaled(point, output, 1.0, None)
    }

    pub(super) fn evaluate_scaled(
        &mut self,
        point: &[f64],
        output: &mut [f64],
        weight: f64,
        primary: Option<&[f64]>,
    ) -> Result<PrecisionReport, KernelError> {
        if point.len() != self.input.len() {
            return Err(KernelError::Dimension {
                expected: self.input.len(),
                actual: point.len(),
            });
        }
        if output.len() != self.output.len() * 2 {
            return Err(KernelError::OutputCount {
                expected: self.output.len() * 2,
                actual: output.len(),
            });
        }
        if point.iter().any(|value| !value.is_finite())
            || point[..self.integration_dimension]
                .iter()
                .any(|value| !(0.0..=1.0).contains(value))
        {
            return Err(KernelError::InvalidPoint);
        }
        for (input, value) in self.input.iter_mut().zip(point) {
            *input = Complex::new(*value, 0.0);
        }
        if let Some(primary) = primary {
            for (out, value) in self.output.iter_mut().zip(primary.as_chunks::<2>().0) {
                *out = Complex::new(value[0], value[1]);
            }
        } else {
            let started = std::time::Instant::now();
            self.evaluator.evaluate(&self.input, &mut self.output);
            self.f64_timing.record(started);
        }
        for (target, value) in output.as_chunks_mut::<2>().0.iter_mut().zip(&self.output) {
            target.copy_from_slice(&[value.re, value.im]);
        }
        let range_loss = weight > 1.0
            && self
                .output
                .iter()
                .zip(&self.exact_zero)
                .zip(&self.real_coefficients)
                .any(|((value, zero), real)| {
                    !zero
                        && (value.re.abs() < f64::MIN_POSITIVE
                            || (!real && value.im.abs() < f64::MIN_POSITIVE))
                });
        for value in output.iter_mut() {
            *value *= weight;
        }
        let nonfinite = output.iter().any(|value| !value.is_finite());
        let boundary = self
            .cancellation
            .needs_check(point, self.precision.boundary_threshold);
        if boundary
            && !nonfinite
            && !range_loss
            && let Some(conditioning) = &mut self.conditioning
        {
            for (input, value) in self.check_input.iter_mut().zip(point) {
                *input = Complex::new(tracked(*value), tracked(0.0));
            }
            let started = std::time::Instant::now();
            conditioning.evaluate(&self.check_input, &mut self.check_output);
            self.conditioning_timing.record(started);
            // Relative accuracy applies to the complex coefficient's infinity
            // norm. An exactly zero imaginary component must not demand an
            // arbitrarily small relative error from native roundoff tracking.
            let stable = self
                .check_output
                .iter()
                .zip(&self.output)
                .all(|(checked, compiled)| {
                    let scale = checked.re.to_f64().abs().max(checked.im.to_f64().abs()) * weight;
                    let tolerance = self.precision.absolute_tolerance
                        + self.precision.relative_tolerance * scale;
                    [(checked.re, compiled.re), (checked.im, compiled.im)]
                        .iter()
                        .all(|(value, compiled)| {
                            value.to_f64().is_finite()
                                && value.get_absolute_error() * weight <= tolerance
                                && ((value.to_f64() - compiled) * weight).abs() <= tolerance
                        })
                });
            if stable {
                return Ok(PrecisionReport {
                    rescued: false,
                    checked: true,
                    bits: 53,
                    ..Default::default()
                });
            }
        }
        if nonfinite || boundary || range_loss {
            return precision::rescue_complex(
                &self.exact,
                &mut self.precision_cache,
                point,
                output,
                &self.cancellation,
                &self.precision,
                weight,
            );
        }
        Ok(PrecisionReport {
            rescued: false,
            checked: false,
            bits: 53,
            ..Default::default()
        })
    }

    pub(super) fn replay_scaled(
        &mut self,
        point: &[f64],
        output: &mut [f64],
        weight: f64,
        policy: &PrecisionPolicy,
    ) -> Result<PrecisionReport, KernelError> {
        precision::rescue_complex(
            &self.exact,
            &mut self.precision_cache,
            point,
            output,
            &self.cancellation,
            policy,
            weight,
        )
    }

    pub(super) fn try_clone(&self) -> Result<Self, KernelError> {
        Ok(Self {
            precision_cache: self.precision_cache.empty_clone(),
            double_cache: self.double_cache.empty_clone(),
            f64_timing: Default::default(),
            conditioning_timing: Default::default(),
            exact: self.exact.clone(),
            evaluator: self.evaluator.clone(),
            conditioning: self.conditioning.clone(),
            input: self.input.clone(),
            integration_dimension: self.integration_dimension,
            output: self.output.clone(),
            check_input: self.check_input.clone(),
            check_output: self.check_output.clone(),
            cancellation: self.cancellation.clone(),
            precision: self.precision.clone(),
            exact_zero: self.exact_zero.clone(),
            real_coefficients: self.real_coefficients.clone(),
        })
    }
}

fn tracked(value: f64) -> ErrorPropagatingFloat<f64> {
    if value == 0.0 {
        ErrorPropagatingFloat::new_with_accuracy(0.0, f64::INFINITY)
    } else {
        ErrorPropagatingFloat::new(value, 15.0)
    }
}

pub(super) fn exact(coefficients: &[Atom]) -> Result<Vec<f64>, KernelError> {
    let evaluator = Atom::evaluator_multiple(coefficients, &[] as &[Atom])
        .build()
        .map_err(|error| KernelError::Compilation(error.to_string()))?;
    let mut evaluator = evaluator::MappingRequirements::new(&evaluator)
        .and_then(|requirements| {
            requirements.map(
                &evaluator,
                |value| Complex::new(value.re.to_f64(), value.im.to_f64()),
                53,
            )
        })
        .map_err(KernelError::Compilation)?;
    let mut values = vec![Complex::new(0.0, 0.0); coefficients.len()];
    evaluator.evaluate(&[], &mut values);
    let flattened = values
        .iter()
        .flat_map(|value| [value.re, value.im])
        .collect::<Vec<_>>();
    if flattened.iter().any(|value| !value.is_finite()) {
        return Err(KernelError::NonFinite);
    }
    Ok(flattened)
}

#[cfg(test)]
mod tests {
    use super::*;
    use symbolica::{parse, symbol};

    fn weight() -> Atom {
        Atom::num(Complex::new(Rational::from(2), Rational::from(3)))
    }

    #[test]
    fn native_complex_outputs_and_constants_preserve_component_order() {
        let x = symbol!("complex_test::x");
        let expressions = [weight() * parse!("(1+complex_test::x)^2"), Atom::num(5)];
        let mut kernel = ComplexKernel::new(
            &[x],
            &expressions,
            Cancellation::new(0, None, 1).unwrap(),
            PrecisionPolicy::default(),
        )
        .unwrap();
        let mut output = [0.0; 4];
        kernel.evaluate(&[0.5], &mut output).unwrap();
        assert_eq!(output, [4.5, 6.75, 5.0, 0.0]);
        assert_eq!(
            exact(&[weight(), Atom::num(5)]).unwrap(),
            [2.0, 3.0, 5.0, 0.0]
        );
        assert!(kernel.evaluate(&[0.5], &mut output[..2]).is_err());
        assert!(kernel.evaluate(&[f64::NAN], &mut output).is_err());
    }

    #[test]
    fn native_complex_conditioning_avoids_unnecessary_rescue() {
        let mut kernel = ComplexKernel::new(
            &[symbol!("complex_test::x")],
            &[weight() * parse!("1+complex_test::x")],
            Cancellation::new(1, None, 1).unwrap(),
            PrecisionPolicy::default(),
        )
        .unwrap();
        let mut output = [0.0; 2];
        let report = kernel.evaluate(&[1e-4], &mut output).unwrap();
        assert!(report.checked);
        assert!(!report.rescued);
        assert!((output[0] - 2.0002).abs() < 1e-14);
        assert!((output[1] - 3.0003).abs() < 1e-14);
    }

    #[test]
    fn native_numeric_complex_rescue_runs_on_a_worker_without_atom_evaluation() {
        let kernel = ComplexKernel::new(
            &[symbol!("complex_test::x")],
            &[weight() * parse!("log(1+complex_test::x)/complex_test::x")],
            Cancellation::new(1, None, 1).unwrap(),
            PrecisionPolicy::default(),
        )
        .unwrap();
        let mut worker = kernel.try_clone().unwrap();
        let (output, report) = std::thread::spawn(move || {
            let mut output = [0.0; 2];
            let report = worker.evaluate(&[1e-80], &mut output).unwrap();
            (output, report)
        })
        .join()
        .unwrap();
        assert!(report.rescued);
        assert!(report.bits > 256);
        assert_eq!(output, [2.0, 3.0]);
    }
}
