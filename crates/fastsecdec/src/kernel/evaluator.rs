//! Compile-time availability with explicit runtime selection of native owners.
use super::{EvaluatorBackend, EvaluatorTiming, KernelError, program::ExactProgram};
use symbolica::domains::float::Complex;
mod batch;
mod conditioning;
mod mapping;
pub(super) use conditioning::Conditioning;
pub(super) use mapping::MappingRequirements;

macro_rules! evaluator {
    ($name:ident, $scalar:ty) => {
        #[derive(Clone)]
        pub(super) enum $name {
            #[cfg(feature = "native")]
            Symjit(symbolica::evaluate::JITCompiledEvaluator<$scalar>),
            Eager(symbolica::evaluate::ExpressionEvaluator<$scalar>),
        }
        impl $name {
            pub(super) fn evaluate(&mut self, input: &[$scalar], output: &mut [$scalar]) {
                match self {
                    #[cfg(feature = "native")]
                    Self::Symjit(evaluator) => evaluator.evaluate(input, output),
                    Self::Eager(evaluator) => evaluator.evaluate(input, output),
                }
            }
            pub(super) fn evaluate_batch(
                &mut self,
                input: &[$scalar],
                output: &mut [$scalar],
                rows: usize,
                inputs: usize,
                outputs: usize,
            ) -> Vec<EvaluatorTiming> {
                match self {
                    #[cfg(feature = "native")]
                    Self::Symjit(evaluator) => {
                        batch::evaluate_jit(evaluator, input, output, rows, inputs, outputs)
                    }
                    Self::Eager(evaluator) => {
                        batch::evaluate_eager(evaluator, input, output, rows, inputs, outputs)
                    }
                }
            }
            pub(super) fn symjit_ir_bytes(&self) -> Option<usize> {
                match self {
                    #[cfg(feature = "native")]
                    Self::Symjit(evaluator) => Some(evaluator.as_bytes().len()),
                    Self::Eager(_) => None,
                }
            }
        }
    };
}
evaluator!(RealEvaluator, f64);
evaluator!(ComplexEvaluator, Complex<f64>);

pub(super) fn real(
    exact: &ExactProgram,
    backend: EvaluatorBackend,
) -> Result<RealEvaluator, KernelError> {
    #[cfg(feature = "native")]
    if !backend.is_eager() {
        return exact
            .jit_compile::<f64>(settings())
            .map(RealEvaluator::Symjit)
            .map_err(KernelError::Compilation);
    }
    #[cfg(feature = "portable")]
    if backend == EvaluatorBackend::Symjit {
        return Err(KernelError::Compilation(
            "SymJIT is unavailable in a portable build".into(),
        ));
    }
    MappingRequirements::new(exact)
        .and_then(|requirements| requirements.map(exact, |value| value.re.to_f64(), 53))
        .map(RealEvaluator::Eager)
        .map_err(KernelError::Compilation)
}

pub(super) fn complex(
    exact: &ExactProgram,
    backend: EvaluatorBackend,
) -> Result<ComplexEvaluator, KernelError> {
    #[cfg(feature = "native")]
    if !backend.is_eager() {
        return exact
            .jit_compile::<Complex<f64>>(settings())
            .map(ComplexEvaluator::Symjit)
            .map_err(KernelError::Compilation);
    }
    #[cfg(feature = "portable")]
    if backend == EvaluatorBackend::Symjit {
        return Err(KernelError::Compilation(
            "SymJIT is unavailable in a portable build".into(),
        ));
    }
    MappingRequirements::new(exact)
        .and_then(|requirements| {
            requirements.map(
                exact,
                |value| Complex::new(value.re.to_f64(), value.im.to_f64()),
                53,
            )
        })
        .map(ComplexEvaluator::Eager)
        .map_err(KernelError::Compilation)
}

#[cfg(feature = "native")]
fn settings() -> symbolica::evaluate::JITCompilationSettings {
    symbolica::evaluate::JITCompilationSettings::default()
        .optimization_level(2)
        .direct_translation(true)
        .with_option("use_threads", "false")
}

#[cfg(test)]
mod eager_tests;
#[cfg(test)]
mod tests;
