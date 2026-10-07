//! Compile-time adapter over Symbolica's existing evaluator backends.
use super::{KernelError, program::ExactProgram};
use symbolica::domains::float::Complex;
mod batch;
mod mapping;
pub(super) use batch::evaluate as evaluate_batch;
pub(super) use mapping::MappingRequirements;

#[cfg(feature = "native")]
pub(super) type RealEvaluator = symbolica::evaluate::JITCompiledEvaluator<f64>;
#[cfg(feature = "native")]
pub(super) type ComplexEvaluator = symbolica::evaluate::JITCompiledEvaluator<Complex<f64>>;
#[cfg(feature = "portable")]
pub(super) type RealEvaluator = symbolica::evaluate::ExpressionEvaluator<f64>;
#[cfg(feature = "portable")]
pub(super) type ComplexEvaluator = symbolica::evaluate::ExpressionEvaluator<Complex<f64>>;

pub(super) fn real(exact: &ExactProgram) -> Result<RealEvaluator, KernelError> {
    #[cfg(feature = "native")]
    let result = exact.jit_compile::<f64>(settings());
    #[cfg(feature = "portable")]
    let result = MappingRequirements::new(exact)
        .and_then(|requirements| requirements.map(exact, |value| value.re.to_f64(), 53));
    result.map_err(KernelError::Compilation)
}

pub(super) fn complex(exact: &ExactProgram) -> Result<ComplexEvaluator, KernelError> {
    #[cfg(feature = "native")]
    let result = exact.jit_compile::<Complex<f64>>(settings());
    #[cfg(feature = "portable")]
    let result = MappingRequirements::new(exact).and_then(|requirements| {
        requirements.map(
            exact,
            |value| Complex::new(value.re.to_f64(), value.im.to_f64()),
            53,
        )
    });
    result.map_err(KernelError::Compilation)
}

#[cfg(feature = "native")]
fn settings() -> symbolica::evaluate::JITCompilationSettings {
    symbolica::evaluate::JITCompilationSettings::default()
        .optimization_level(2)
        .direct_translation(true)
        // Runtime workers belong to the caller; ambient SymJIT configuration
        // must not create an additional pool inside each sector's matrix call.
        .with_option("use_threads", "false")
}

#[cfg(test)]
mod tests;
