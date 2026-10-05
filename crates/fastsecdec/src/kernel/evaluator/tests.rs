use super::*;
use crate::kernel::{
    PrecisionPolicy, SectorKernel, cancellation::Cancellation, precision_cache::PrecisionCache,
    program::SectorProgram,
};
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo},
    domains::float::Float,
    function, symbol,
};

#[test]
fn unsupported_conditioning_callback_is_a_typed_construction_error() {
    let x = symbol!("conditioning_admission::x");
    let function = symbol!(
        "conditioning_admission::f",
        eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 1.0)
    );
    let exact = function!(function, Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    // Ordinary binary64 interpretation/JIT is supported. The separate native
    // error-tracking callback is intentionally absent.
    let mut hot = real(&exact).unwrap();
    let mut output = [0.0];
    hot.evaluate(&[0.25], &mut output);
    assert_eq!(output, [1.25]);
    let result = SectorKernel::from_program(
        SectorProgram {
            parameters: vec![x],
            exact,
            cancellation: Cancellation::new(0, Some(vec![vec![0]]), 1).unwrap(),
            exact_zero: vec![false],
            real_coefficients: vec![true],
        },
        &PrecisionPolicy::default(),
        false,
    );
    assert!(
        matches!(result, Err(KernelError::Compilation(ref reason)) if reason.contains("implementation"))
    );
}

#[test]
fn unsupported_rescue_callback_is_a_typed_precision_error() {
    let x = symbol!("precision_admission::x");
    let function = symbol!(
        "precision_admission::f",
        eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 1.0)
    );
    let exact = function!(function, Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    let mut cache = PrecisionCache::<Float>::default();
    let result = cache.evaluate(
        &exact,
        &[0.25],
        128,
        |coefficient| coefficient.re.to_multi_prec_float(128),
        |value| Float::with_val(128, value),
    );
    assert!(
        matches!(result, Err(KernelError::PrecisionEvaluation(ref reason)) if reason.contains("implementation"))
    );
}
