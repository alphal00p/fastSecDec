//! Essential numerical failure admission, separate from optional causal checks.
use super::*;
use crate::{
    contour::functions::dynamic::{self, ProgramScope, RootProgram},
    kernel::{
        CompilationSettings, KernelSet, PrecisionPolicy, ReplayPolicy, StabilitySettings,
        cancellation::Cancellation, program::SectorProgram,
    },
};
use std::sync::LazyLock;
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo, Symbol},
    domains::float::{DoubleFloat, ErrorPropagatingFloat, Float, FloatLike},
    function, symbol,
};

static MASK: LazyLock<Symbol> = LazyLock::new(|| {
    symbol!(
        "dynamic_failure_fence::opaque_consumer",
        eval = EvaluationInfo::new()
            .register(|_: &[f64]| 1.)
            .register(|args: &[DoubleFloat]| args[0].one())
            .register(|args: &[Float]| args[0].one())
            .register(|args: &[ErrorPropagatingFloat<f64>]| args[0].one())
            .register(|args: &[ErrorPropagatingFloat<Float>]| args[0].one())
            .register(|_: &[Complex<f64>]| Complex::new(1., 0.))
            .register(|args: &[Complex<DoubleFloat>]| args[0].one())
            .register(|args: &[Complex<Float>]| args[0].one())
            .register(|args: &[Complex<ErrorPropagatingFloat<f64>>]| args[0].one())
            .register(|args: &[Complex<ErrorPropagatingFloat<Float>>]| args[0].one())
    )
});

fn fixture() -> (RootProgram, Symbol, ExactProgram) {
    let helper = RootProgram::build(1).unwrap();
    let x = symbol!("dynamic_failure_fence::x");
    // exp(800*x) is finite in MP and overflows at x=1 in f64/DD. The
    // downstream opaque callback deliberately masks the root's NaN as one.
    let strength = dynamic::strength(
        &helper,
        &[(Atom::num(800) * Atom::var(x)).exp()],
        &Atom::num((4, 5)),
        &Atom::one(),
    )
    .unwrap();
    let exact = Atom::evaluator_multiple(
        &[
            function!(*MASK, strength.clone()),
            Atom::num(2) * function!(*MASK, strength),
        ],
        &[Atom::var(x)],
    )
    .build()
    .unwrap();
    (helper, x, exact)
}

#[test]
fn masked_dynamic_failure_invalidates_whole_primary_batch_and_restores_outer_state() {
    let (helper, _, exact) = fixture();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let requirements = MappingRequirements::new(&exact).unwrap();
    assert!(requirements.has_dynamic_callbacks());
    let mut raw = requirements.map(&exact, |c| c.re.to_f64(), 53).unwrap();
    let mut output = [0.; 2];
    let (_, error) = dynamic::isolated_attempt(|| raw.evaluate(&[1.], &mut output));
    assert_eq!(
        output,
        [1., 2.],
        "the native consumer really masks the failure"
    );
    assert!(error.unwrap().contains("finite real centres"));

    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
        let mut evaluator = real(&exact, backend).unwrap();
        dynamic::failure("outer evaluation".into());
        let mut output = [0.; 6];
        evaluator.evaluate_batch(&[0.1, 1., 0.2], &mut output, 3, 1, 2);
        assert!(output.iter().all(|value| value.is_nan()));
        assert_eq!(dynamic::take_failure().as_deref(), Some("outer evaluation"));
        evaluator.evaluate_batch(&[0.1, 0.2, 0.3], &mut output, 3, 1, 2);
        assert_eq!(output, [1., 2., 1., 2., 1., 2.]);
        assert!(dynamic::take_failure().is_none());
        let mut evaluator = complex(&exact, backend).unwrap();
        let mut output = [Complex::new(0., 0.); 4];
        evaluator.evaluate_batch(
            &[Complex::new(0.1, 0.), Complex::new(1., 0.)],
            &mut output,
            2,
            1,
            2,
        );
        assert!(
            output
                .iter()
                .all(|value| value.re.is_nan() && value.im.is_nan())
        );
    }
}

#[test]
fn masked_dynamic_batch_failure_recovers_whole_vectors_in_native_precision() {
    for complex in [false, true] {
        for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
            let (helper, x, exact) = fixture();
            let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
            // Exercise the real execution and replay lanes with a raw native
            // program, without opening public dynamic recipe admission.
            let mut kernels = KernelSet::from_programs_for_load(
                vec![-1, 0],
                vec![SectorProgram {
                    parameters: vec![x],
                    runtime_parameters: vec![],
                    exact,
                    cancellation: Cancellation::new(0, None, 1).unwrap(),
                    exact_zero: vec![false; 2],
                    real_coefficients: vec![true; 2],
                }],
                vec![Atom::Zero; 2],
                PrecisionPolicy::default(),
                None,
                complex,
                vec![],
                CompilationSettings {
                    backend,
                    ..Default::default()
                },
            )
            .unwrap();
            kernels
                .set_stability_settings(&StabilitySettings::validated())
                .unwrap();
            let mut context = kernels
                .evaluation_context(0, ReplayPolicy::default())
                .unwrap();
            let width = context.output_count();
            let mut output = vec![0.; 3 * width];
            let reports = context
                .evaluate_weighted_batch(&[0.1, 1., 0.2], &[1.; 3], &mut output)
                .unwrap();
            assert!(reports.iter().all(|report| report.precision.rescued));
            let expected: &[f64] = if complex {
                &[1., 0., 2., 0.]
            } else {
                &[1., 2.]
            };
            assert!(output.chunks_exact(width).all(|row| row == expected));
            let reports = context
                .evaluate_weighted_batch(&[0.1, 0.2, 0.3], &[1.; 3], &mut output)
                .unwrap();
            assert!(reports.iter().all(|report| !report.precision.rescued));
            assert!(output.chunks_exact(width).all(|row| row == expected));
            assert!(dynamic::take_failure().is_none());
        }
    }
}

#[test]
fn ordinary_native_program_does_not_enter_dynamic_failure_state() {
    let x = symbol!("dynamic_failure_fence::ordinary_x");
    let exact = (Atom::var(x) + 1)
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    assert!(
        !MappingRequirements::new(&exact)
            .unwrap()
            .has_dynamic_callbacks()
    );
    let mut evaluator = real(&exact, EvaluatorBackend::Eager).unwrap();
    assert!(matches!(evaluator, RealEvaluator::Eager(_)));
    dynamic::failure("outer ordinary call".into());
    let mut output = [0.];
    evaluator.evaluate(&[0.5], &mut output);
    assert_eq!(output, [1.5]);
    assert_eq!(
        dynamic::take_failure().as_deref(),
        Some("outer ordinary call")
    );
}

#[test]
fn masked_constant_callback_failure_is_an_explicit_mapping_error() {
    let x = symbol!("dynamic_failure_fence::constant_x");
    let constant = crate::contour::functions::smooth_positive::positive_part(
        &(-Atom::num(10).pow(308)),
        &Atom::num(10).pow(-150),
    );
    let exact = (function!(*MASK, constant) + Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    assert!(!exact.export_instructions().constant_functions.is_empty());
    let requirements = MappingRequirements::new(&exact).unwrap();
    assert!(requirements.has_dynamic_callbacks());
    for backend in [EvaluatorBackend::Eager, EvaluatorBackend::Auto] {
        dynamic::failure("outer preparation".into());
        assert!(matches!(
            real(&exact, backend),
            Err(KernelError::Compilation(reason)) if reason.contains("dynamic callback preparation failed")
        ));
        assert_eq!(
            dynamic::take_failure().as_deref(),
            Some("outer preparation")
        );
        assert!(matches!(
            complex(&exact, backend),
            Err(KernelError::Compilation(reason)) if reason.contains("dynamic callback preparation failed")
        ));
    }
    // The underlying mathematical constant remains valid in native MP. This
    // is an explicit f64 preparation limitation, never a fabricated zero.
    let mut high = requirements
        .map(&exact, |c| c.re.to_multi_prec_float(192), 192)
        .unwrap();
    let mut output = [Float::with_val(192, 0)];
    high.evaluate(&[Float::with_val(192, 0.5)], &mut output);
    assert_eq!(output[0], Float::with_val(192, 1.5));
    assert!(dynamic::take_failure().is_none());
}

mod diagnostics;
