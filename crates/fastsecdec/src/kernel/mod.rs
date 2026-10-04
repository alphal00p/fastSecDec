//! Portable SymJIT O2 vector kernels. Worker ownership is explicit.
mod artifact;
mod cancellation;
mod complex;
mod precision;
mod precision_cache;
use crate::generation::GeneratedIntegral;
pub use precision::{PrecisionPolicy, PrecisionReport};
use std::{collections::HashMap, ops::ControlFlow, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::{
        float::{Complex, ErrorPropagatingFloat, Float, RealLike},
        rational::Rational,
    },
    evaluate::{ExpressionEvaluator, JITCompilationSettings, JITCompiledEvaluator},
};

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("kernel compilation cancelled")]
    Cancelled,
    #[error("invalid kernel artifact: {0}")]
    Artifact(String),
    #[error("invalid precision rescue policy")]
    PrecisionPolicy,
    #[error("precision rescue failed: {0}")]
    PrecisionEvaluation(String),
    #[error("precision rescue did not converge within {bits} bits")]
    PrecisionExhausted { bits: u32 },
    #[error("kernel construction failed: {0}")]
    Compilation(String),
    #[error("expected input dimension {expected}, received {actual}")]
    Dimension { expected: usize, actual: usize },
    #[error("expected {expected} output coefficients, received {actual}")]
    OutputCount { expected: usize, actual: usize },
    #[error("kernel input must lie in the finite closed unit cube")]
    InvalidPoint,
    #[error("kernel produced a non-finite value at the supplied point")]
    NonFinite,
    #[error("portable evaluator serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Clone, Copy, Debug)]
pub struct CompilationProgress {
    pub completed: usize,
    pub total: usize,
    pub elapsed_seconds: f64,
}

pub struct SectorKernel {
    cancellation: cancellation::Cancellation,
    precision: PrecisionPolicy,
    parameters: Vec<Symbol>,
    coefficients: Vec<Atom>,
    backend: Backend,
}

struct SectorExpressions {
    parameters: Vec<Symbol>,
    coefficients: Vec<Atom>,
    cancellation: cancellation::Cancellation,
}

enum Backend {
    Real(RealKernel),
    Complex(complex::ComplexKernel),
}

struct RealKernel {
    precision_cache: precision_cache::PrecisionCache<Float>,
    evaluator: JITCompiledEvaluator<f64>,
    exact_evaluator: ExpressionEvaluator<Complex<Rational>>,
    conditioning: ExpressionEvaluator<ErrorPropagatingFloat<f64>>,
    check_input: Vec<ErrorPropagatingFloat<f64>>,
    check_output: Vec<ErrorPropagatingFloat<f64>>,
}

impl SectorKernel {
    pub fn dimension(&self) -> usize {
        self.parameters.len()
    }
    pub fn output_count(&self) -> usize {
        self.coefficients.len()
            * if matches!(self.backend, Backend::Complex(_)) {
                2
            } else {
                1
            }
    }

    pub fn evaluate(&mut self, point: &[f64], output: &mut [f64]) -> Result<(), KernelError> {
        self.evaluate_with_diagnostics(point, output).map(|_| ())
    }

    pub fn evaluate_with_diagnostics(
        &mut self,
        point: &[f64],
        output: &mut [f64],
    ) -> Result<PrecisionReport, KernelError> {
        if point.len() != self.dimension() {
            return Err(KernelError::Dimension {
                expected: self.dimension(),
                actual: point.len(),
            });
        }
        if output.len() != self.output_count() {
            return Err(KernelError::OutputCount {
                expected: self.output_count(),
                actual: output.len(),
            });
        }
        if point
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(KernelError::InvalidPoint);
        }
        let backend = match &mut self.backend {
            Backend::Complex(kernel) => return kernel.evaluate(point, output),
            Backend::Real(kernel) => kernel,
        };
        backend.evaluator.evaluate(point, output);
        let nonfinite = output.iter().any(|value| !value.is_finite());
        let boundary = self
            .cancellation
            .needs_check(point, self.precision.boundary_threshold);
        if boundary && !nonfinite {
            for (target, value) in backend.check_input.iter_mut().zip(point) {
                *target = ErrorPropagatingFloat::new(*value, 15.0);
            }
            backend
                .conditioning
                .evaluate(&backend.check_input, &mut backend.check_output);
            let stable = backend
                .check_output
                .iter()
                .zip(output.iter())
                .zip(&self.coefficients)
                .all(|((checked, compiled), expression)| {
                    if expression.is_zero() {
                        return *compiled == 0.0;
                    }
                    let value = checked.to_f64();
                    let tolerance = self.precision.absolute_tolerance
                        + self.precision.relative_tolerance * value.abs();
                    value.is_finite()
                        && checked.get_absolute_error() <= tolerance
                        && (value - compiled).abs() <= tolerance
                });
            if stable {
                return Ok(PrecisionReport {
                    rescued: false,
                    checked: true,
                    bits: 53,
                });
            }
        }
        if nonfinite || boundary {
            return precision::rescue(
                &backend.exact_evaluator,
                &mut backend.precision_cache,
                point,
                output,
                &self.cancellation,
                &self.precision,
            );
        }
        Ok(PrecisionReport {
            rescued: false,
            checked: false,
            bits: 53,
        })
    }

    /// Clone native evaluator state and buffers for an independently owned worker.
    pub fn try_clone(&self) -> Result<Self, KernelError> {
        Ok(Self {
            cancellation: self.cancellation.clone(),
            precision: self.precision.clone(),
            parameters: self.parameters.clone(),
            coefficients: self.coefficients.clone(),
            backend: match &self.backend {
                Backend::Complex(kernel) => Backend::Complex(kernel.try_clone()?),
                Backend::Real(kernel) => Backend::Real(RealKernel {
                    precision_cache: Default::default(),
                    evaluator: kernel.evaluator.clone(),
                    exact_evaluator: kernel.exact_evaluator.clone(),
                    conditioning: kernel.conditioning.clone(),
                    check_input: kernel.check_input.clone(),
                    check_output: kernel.check_output.clone(),
                }),
            },
        })
    }
}

pub struct KernelSet {
    coefficient_orders: Vec<i32>,
    components: Vec<crate::status::CoefficientComponent>,
    precision: PrecisionPolicy,
    exact_expressions: Vec<Atom>,
    content_id: String,
    orders: Vec<i32>,
    sectors: Vec<SectorKernel>,
    exact_coefficients: Vec<f64>,
}

impl KernelSet {
    pub fn components(&self) -> &[crate::status::CoefficientComponent] {
        &self.components
    }
    pub fn content_id(&self) -> &str {
        &self.content_id
    }
    pub fn orders(&self) -> &[i32] {
        &self.orders
    }
    pub fn sectors(&self) -> &[SectorKernel] {
        &self.sectors
    }
    pub fn sectors_mut(&mut self) -> &mut [SectorKernel] {
        &mut self.sectors
    }
    pub fn exact_coefficients(&self) -> &[f64] {
        &self.exact_coefficients
    }
}

impl GeneratedIntegral {
    pub fn compile(&self) -> Result<KernelSet, KernelError> {
        self.compile_with_precision(PrecisionPolicy::default())
    }

    pub fn compile_with_precision(
        &self,
        precision: PrecisionPolicy,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_and_progress(precision, |_| ControlFlow::Continue(()))
    }

    pub fn compile_with_progress(
        &self,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        self.compile_with_precision_and_progress(PrecisionPolicy::default(), progress)
    }

    pub fn compile_with_precision_and_progress(
        &self,
        precision: PrecisionPolicy,
        progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<KernelSet, KernelError> {
        KernelSet::from_expressions_with_progress(
            self.orders().to_vec(),
            self.sectors()
                .iter()
                .map(|sector| {
                    Ok(SectorExpressions {
                        parameters: sector.parameters().to_vec(),
                        coefficients: sector.coefficients().to_vec(),
                        cancellation: cancellation::Cancellation::new(
                            sector.cancellation_degree(),
                            Some(sector.cancellation_terms().to_vec()),
                            sector.dimension(),
                        )?,
                    })
                })
                .collect::<Result<_, KernelError>>()?,
            self.exact_coefficients().to_vec(),
            precision,
            progress,
        )
    }
}

impl KernelSet {
    fn from_expressions(
        orders: Vec<i32>,
        expressions: Vec<SectorExpressions>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
    ) -> Result<Self, KernelError> {
        Self::from_expressions_with_progress(
            orders,
            expressions,
            exact_expressions,
            precision,
            |_| ControlFlow::Continue(()),
        )
    }

    fn from_expressions_with_progress(
        orders: Vec<i32>,
        expressions: Vec<SectorExpressions>,
        exact_expressions: Vec<Atom>,
        precision: PrecisionPolicy,
        mut progress: impl FnMut(&CompilationProgress) -> ControlFlow<()>,
    ) -> Result<Self, KernelError> {
        precision.validate()?;
        let started = Instant::now();
        let total = expressions.len();
        let mut emit = |completed| {
            if progress(&CompilationProgress {
                completed,
                total,
                elapsed_seconds: started.elapsed().as_secs_f64(),
            })
            .is_break()
            {
                Err(KernelError::Cancelled)
            } else {
                Ok(())
            }
        };
        emit(0)?;
        let use_complex = expressions
            .iter()
            .flat_map(|sector| &sector.coefficients)
            .chain(&exact_expressions)
            .any(has_complex_coefficients);
        let mut sectors = Vec::with_capacity(expressions.len());
        for SectorExpressions {
            parameters,
            coefficients,
            cancellation,
        } in expressions
        {
            let backend = if use_complex {
                Backend::Complex(complex::ComplexKernel::new(
                    &parameters,
                    &coefficients,
                    cancellation.clone(),
                    precision.clone(),
                )?)
            } else {
                let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
                let evaluator = Atom::evaluator_multiple(&coefficients, &variables)
                    .build()
                    .map_err(|error| KernelError::Compilation(error.to_string()))?;
                let conditioning = evaluator.clone().map_coeff(&|coefficient| {
                    ErrorPropagatingFloat::new(coefficient.re.to_f64(), 15.0)
                });
                let exact_evaluator = evaluator.clone();
                let evaluator = evaluator
                    .jit_compile::<f64>(JITCompilationSettings::default().optimization_level(2))
                    .map_err(KernelError::Compilation)?;
                Backend::Real(RealKernel {
                    precision_cache: Default::default(),
                    conditioning,
                    check_input: vec![ErrorPropagatingFloat::new(0.0, 15.0); parameters.len()],
                    check_output: vec![ErrorPropagatingFloat::new(0.0, 15.0); coefficients.len()],
                    evaluator,
                    exact_evaluator,
                })
            };
            sectors.push(SectorKernel {
                cancellation,
                precision: precision.clone(),
                parameters,
                coefficients,
                backend,
            });
            emit(sectors.len())?;
        }
        let constants = HashMap::<Atom, f64>::new();
        let exact_coefficients = if use_complex {
            complex::exact(&exact_expressions)?
        } else {
            exact_expressions
                .iter()
                .map(|coefficient| {
                    coefficient
                        .evaluate(&constants)
                        .map_err(|error| KernelError::Compilation(error.to_string()))
                })
                .collect::<Result<Vec<f64>, _>>()?
        };
        if exact_coefficients.iter().any(|value| !value.is_finite()) {
            return Err(KernelError::NonFinite);
        }
        let mut result = KernelSet {
            components: orders
                .iter()
                .flat_map(|_| {
                    if use_complex {
                        vec![
                            crate::status::CoefficientComponent::Real,
                            crate::status::CoefficientComponent::Imag,
                        ]
                    } else {
                        vec![crate::status::CoefficientComponent::Real]
                    }
                })
                .collect(),
            content_id: String::new(),
            precision,
            exact_expressions,
            coefficient_orders: orders.clone(),
            orders: orders
                .into_iter()
                .flat_map(|order| {
                    if use_complex {
                        vec![order, order]
                    } else {
                        vec![order]
                    }
                })
                .collect(),
            sectors,
            exact_coefficients,
        };
        result.content_id = result.compute_content_id()?;
        Ok(result)
    }
}

fn has_complex_coefficients(expression: &Atom) -> bool {
    let mut found = false;
    expression.visitor(&mut |term| {
        if let AtomView::Num(number) = term {
            found |= !number.get_coeff_view().is_real();
        }
        !found
    });
    found
}
