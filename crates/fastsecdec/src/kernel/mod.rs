//! Native O2 or portable interpreted vector kernels. Worker ownership is explicit.
mod artifact;
#[cfg(feature = "native")]
mod backend_version;
#[cfg(feature = "native")]
pub use backend_version::symjit_version_code;
mod cancellation;
mod compilation;
pub use compilation::{CompilationCompletion, CompilationDispatch, CompilationJob};
mod complex;
mod evaluator;
mod metadata;
mod model_constraints;
pub use metadata::PortableMetadata;
pub use model_constraints::RuntimeMassConstraint;
mod precision;
mod precision_cache;
mod program;
mod statistics;
pub use statistics::{EvaluatorOperations, EvaluatorStatistics};
mod weighted;

#[cfg(all(test, feature = "native"))]
mod function_map_probe;
pub use precision::{PrecisionPolicy, PrecisionReport};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    domains::{
        float::{Complex, ErrorPropagatingFloat, Float, RealLike},
        rational::Rational,
    },
    evaluate::ExpressionEvaluator,
};
pub use weighted::{ReplayPolicy, ReplayReport, ReplayState, WeightedEvaluationContext};

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("kernel compilation cancelled")]
    Cancelled,
    #[error("invalid runtime parameters: {0}")]
    Parameters(String),
    #[error("runtime parameters must be bound before evaluation")]
    UnboundParameters,
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
    #[error("integration weight must be finite and nonnegative")]
    InvalidWeight,
    #[error("weighted replay failed: {0}")]
    Replay(String),
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
    runtime_parameters: Vec<Symbol>,
    parameters_bound: bool,
    input: Vec<f64>,
    exact_zero: Vec<bool>,
    program_bytes: std::sync::Arc<[u8]>,
    statistics: EvaluatorStatistics,
    backend: Backend,
}

#[cfg(feature = "native")]
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
    evaluator: evaluator::RealEvaluator,
    exact_evaluator: ExpressionEvaluator<Complex<Rational>>,
    conditioning: Option<ExpressionEvaluator<ErrorPropagatingFloat<f64>>>,
    check_input: Vec<ErrorPropagatingFloat<f64>>,
    check_output: Vec<ErrorPropagatingFloat<f64>>,
}

impl SectorKernel {
    /// Facts recorded from the actual complete-vector evaluator construction.
    /// Reading these records never constructs or executes an evaluator.
    pub fn statistics(&self) -> &EvaluatorStatistics {
        &self.statistics
    }
    pub fn dimension(&self) -> usize {
        self.parameters.len()
    }
    pub fn output_count(&self) -> usize {
        self.exact_zero.len()
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
        self.evaluate_scaled(point, output, 1.0)
    }

    fn evaluate_scaled(
        &mut self,
        point: &[f64],
        output: &mut [f64],
        weight: f64,
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
        if !self.parameters_bound {
            return Err(KernelError::UnboundParameters);
        }
        self.input[..point.len()].copy_from_slice(point);
        let point = self.input.as_slice();
        let backend = match &mut self.backend {
            Backend::Complex(kernel) => return kernel.evaluate_scaled(point, output, weight),
            Backend::Real(kernel) => kernel,
        };
        backend.evaluator.evaluate(point, output);
        // Native replay must scale before conversion: an amplifying weight can
        // make a raw zero/subnormal result materially inaccurate.
        let range_loss = weight > 1.0
            && output
                .iter()
                .zip(&self.exact_zero)
                .any(|(value, zero)| value.abs() < f64::MIN_POSITIVE && !zero);
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
            && let Some(conditioning) = &mut backend.conditioning
        {
            for (target, value) in backend.check_input.iter_mut().zip(point) {
                *target = ErrorPropagatingFloat::new(*value, 15.0);
            }
            conditioning.evaluate(&backend.check_input, &mut backend.check_output);
            let stable = backend
                .check_output
                .iter()
                .zip(output.iter())
                .zip(&self.exact_zero)
                .all(|((checked, compiled), zero)| {
                    if *zero {
                        return *compiled == 0.0;
                    }
                    let value = checked.to_f64() * weight;
                    let tolerance = self.precision.absolute_tolerance
                        + self.precision.relative_tolerance * value.abs();
                    value.is_finite()
                        && checked.get_absolute_error() * weight <= tolerance
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
        if nonfinite || boundary || range_loss {
            return precision::rescue(
                &backend.exact_evaluator,
                &mut backend.precision_cache,
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
        })
    }

    fn replay_scaled(
        &mut self,
        point: &[f64],
        output: &mut [f64],
        weight: f64,
        minimum_bits: u32,
    ) -> Result<PrecisionReport, KernelError> {
        if !self.parameters_bound {
            return Err(KernelError::UnboundParameters);
        }
        if point.len() != self.dimension() {
            return Err(KernelError::Dimension {
                expected: self.dimension(),
                actual: point.len(),
            });
        }
        self.input[..point.len()].copy_from_slice(point);
        let point = self.input.as_slice();
        let mut policy = self.precision.clone();
        policy.initial_bits = policy.initial_bits.max(minimum_bits);
        match &mut self.backend {
            Backend::Real(kernel) => precision::rescue(
                &kernel.exact_evaluator,
                &mut kernel.precision_cache,
                point,
                output,
                &self.cancellation,
                &policy,
                weight,
            ),
            Backend::Complex(kernel) => kernel.replay_scaled(point, output, weight, &policy),
        }
    }

    /// Clone native evaluator state and buffers for an independently owned worker.
    pub fn try_clone(&self) -> Result<Self, KernelError> {
        Ok(Self {
            cancellation: self.cancellation.clone(),
            precision: self.precision.clone(),
            parameters: self.parameters.clone(),
            runtime_parameters: self.runtime_parameters.clone(),
            parameters_bound: self.parameters_bound,
            input: self.input.clone(),
            exact_zero: self.exact_zero.clone(),
            program_bytes: self.program_bytes.clone(),
            statistics: self.statistics.clone(),
            backend: match &self.backend {
                Backend::Complex(kernel) => Backend::Complex(kernel.try_clone()?),
                Backend::Real(kernel) => Backend::Real(RealKernel {
                    precision_cache: kernel.precision_cache.empty_clone(),
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
    runtime_parameters: Vec<Symbol>,
    runtime_mass_constraints: Vec<RuntimeMassConstraint>,
    exact_kernel: Option<SectorKernel>,
    template_content_id: Option<String>,
    portable_artifact: Option<Vec<u8>>,
    metadata: Option<crate::generation::GenerationMetadata>,
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
    /// Ordered real scalar inputs appended after integration coordinates in the native evaluator.
    pub fn runtime_parameters(&self) -> &[Symbol] {
        &self.runtime_parameters
    }

    /// Immutable compiled-template identity, independent of the bound physical point.
    pub fn template_content_id(&self) -> &str {
        self.template_content_id
            .as_deref()
            .unwrap_or(&self.content_id)
    }

    pub fn parameters_bound(&self) -> bool {
        self.runtime_parameters.is_empty() || self.template_content_id.is_some()
    }

    /// Bind the complete physical point once before creating caller-owned workers.
    /// Rebinding leaves the compiled program untouched and derives a distinct numerical identity.
    pub fn bind_parameters(
        &mut self,
        values: &std::collections::BTreeMap<Symbol, f64>,
    ) -> Result<(), KernelError> {
        for symbol in values.keys() {
            if !self.runtime_parameters.contains(symbol) {
                return Err(KernelError::Parameters(format!(
                    "unknown parameter {}",
                    Atom::var(*symbol).to_canonical_string()
                )));
            }
        }
        let ordered = self
            .runtime_parameters
            .iter()
            .map(|symbol| {
                let name = Atom::var(*symbol).to_canonical_string();
                match values.get(symbol) {
                    Some(value) if value.is_finite() => Ok(*value),
                    Some(_) => Err(KernelError::Parameters(format!("{name} must be finite"))),
                    None => Err(KernelError::Parameters(format!("missing value for {name}"))),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.validate_runtime_masses_at(values)?;
        if self.runtime_parameters.is_empty() {
            return Ok(());
        }
        let exact_kernel = self.exact_kernel.as_mut().ok_or_else(|| {
            KernelError::Artifact("missing parameterized exact-offset evaluator".into())
        })?;
        exact_kernel.input.copy_from_slice(&ordered);
        exact_kernel.parameters_bound = true;
        let mut exact_coefficients = vec![0.0; exact_kernel.output_count()];
        exact_kernel.evaluate(&[], &mut exact_coefficients)?;
        for sector in &mut self.sectors {
            let dimension = sector.dimension();
            sector.input[dimension..].copy_from_slice(&ordered);
            sector.parameters_bound = true;
        }
        self.exact_coefficients = exact_coefficients;
        let original_id = self
            .template_content_id
            .get_or_insert_with(|| self.content_id.clone());
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-runtime-parameter-point-v1");
        hash.update(original_id.as_bytes());
        for value in ordered {
            hash.update(&value.to_bits().to_le_bytes());
        }
        self.content_id = hash.finalize().to_hex().to_string();
        Ok(())
    }
    /// Legacy version-one artifacts have no retained generation metadata.
    pub fn generation_metadata(&self) -> Option<&crate::generation::GenerationMetadata> {
        self.metadata.as_ref()
    }
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
