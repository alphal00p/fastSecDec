use super::*;
use crate::kernel::{
    Backend, PrecisionPolicy, SectorKernel, cancellation::Cancellation,
    precision_cache::PrecisionCache, program::SectorProgram,
};
use symbolica::{
    atom::{Atom, AtomCore, EvaluationInfo},
    domains::float::{ErrorPropagatingFloat, Float, RealLike},
    evaluate::{FunctionMap, FunctionRegistrationOptions, InliningPolicy},
    function, parse, symbol,
};

#[test]
fn unsupported_conditioning_preserves_ordinary_values_and_rejects_unsupported_rescue() {
    let x = symbol!("conditioning_admission::x");
    let function = symbol!(
        "conditioning_admission::f",
        eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 1.0)
    );
    let exact = function!(function, Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap();
    let mut kernel = SectorKernel::from_program(
        SectorProgram {
            parameters: vec![x],
            exact,
            cancellation: Cancellation::new(1, Some(vec![vec![1]]), 1).unwrap(),
            exact_zero: vec![false],
            real_coefficients: vec![true],
        },
        &PrecisionPolicy::default(),
        false,
    )
    .unwrap();
    let Backend::Real(backend) = &kernel.backend else {
        panic!()
    };
    assert!(backend.conditioning.is_none());
    let mut output = [0.0];
    let report = kernel.evaluate_scaled(&[0.25], &mut output, 1.0).unwrap();
    assert_eq!(output, [1.25]);
    assert!(!report.checked);
    // Missing conditioning cannot turn a flagged boundary into unchecked output.
    assert!(matches!(
        kernel.evaluate(&[1e-4], &mut output),
        Err(KernelError::PrecisionEvaluation(reason)) if reason.contains("implementation")
    ));
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
    let mut cache = PrecisionCache::<Float>::new(MappingRequirements::new(&exact).unwrap());
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

#[test]
fn fixed_native_polygamma_uses_real_and_complex_rescue_without_domain_fallback() {
    let _ = symbolica::transcendental::gamma();
    let x = symbol!("gamma_admission::x");
    // polygamma(1,3) retains a native tagged, fixed-argument callback.
    let coefficient = parse!("polygamma(1,3)*(1+gamma_admission::x)");
    let constant = std::f64::consts::PI.powi(2) / 6.0 - 1.25;
    for complex in [false, true] {
        let coefficient = if complex {
            coefficient.clone() * parse!("2+3𝑖")
        } else {
            coefficient.clone()
        };
        let exact = coefficient.evaluator(&[Atom::var(x)]).build().unwrap();
        assert!(!exact.export_instructions().constant_functions.is_empty());
        let requirements = MappingRequirements::new(&exact).unwrap();
        assert!(
            requirements
                .map(
                    &exact,
                    |v| ErrorPropagatingFloat::new(v.re.to_f64(), 15.0),
                    53
                )
                .is_err()
        );
        let mut kernel = SectorKernel::from_program(
            SectorProgram {
                parameters: vec![x],
                exact,
                cancellation: Cancellation::new(1, None, 1).unwrap(),
                exact_zero: vec![false],
                real_coefficients: vec![!complex],
            },
            &PrecisionPolicy::default(),
            complex,
        )
        .unwrap();
        // Worker cloning must retain callback admission without symbolic work.
        let mut worker = kernel.try_clone().unwrap();
        let mut output = vec![0.0; if complex { 2 } else { 1 }];
        let report = worker.evaluate_scaled(&[1e-4], &mut output, 2.0).unwrap();
        assert!(report.checked && report.rescued && report.bits >= 256);
        let expected = constant * 1.0001 * 2.0;
        if complex {
            assert!((output[0] - 2.0 * expected).abs() < 1e-14);
            assert!((output[1] - 3.0 * expected).abs() < 1e-14);
        } else {
            assert!((output[0] - expected).abs() < 1e-14);
        }
        // Original and independent worker retain the same numerical policy.
        let mut original = vec![0.0; output.len()];
        assert_eq!(
            kernel.evaluate_scaled(&[1e-4], &mut original, 2.0).unwrap(),
            report
        );
        assert_eq!(original, output);
    }
}

#[test]
fn upstream_mapping_rejects_fixed_nonreal_conversion_and_constant_failure() {
    let _ = symbolica::transcendental::gamma();
    let exact = parse!("gamma(1+𝑖)")
        .evaluator(&[] as &[Atom])
        .build()
        .unwrap();
    let requirements = MappingRequirements::new(&exact).unwrap();
    assert!(requirements.map(&exact, |v| v.re.to_f64(), 53).is_err());
    let mut complex = requirements
        .map(&exact, |v| Complex::new(v.re.to_f64(), v.im.to_f64()), 53)
        .unwrap();
    let mut output = [Complex::new(0.0, 0.0)];
    complex.evaluate(&[], &mut output);
    assert!(output[0].re.is_finite() && output[0].im.is_finite());
    assert_ne!(output[0].im, 0.0);
    let mut kernel = crate::kernel::complex::ComplexKernel::new(
        &[symbol!("gamma_complex_admission::x")],
        &[parse!("gamma(1+𝑖)*(1+gamma_complex_admission::x)")],
        Cancellation::new(1, None, 1).unwrap(),
        PrecisionPolicy::default(),
    )
    .unwrap();
    let mut rescued = [0.0; 2];
    let report = kernel.evaluate(&[1e-4], &mut rescued).unwrap();
    assert!(report.rescued && report.bits >= 256);
    assert!((rescued[0] - output[0].re * 1.0001).abs() < 1e-14);
    assert!((rescued[1] - output[0].im * 1.0001).abs() < 1e-14);

    let failing = symbol!(
        "mapping_admission::constant",
        eval = EvaluationInfo::constant(|_, _| Err("deliberate constant failure".into()))
    );
    let exact = function!(failing)
        .evaluator(&[] as &[Atom])
        .build()
        .unwrap();
    let result = MappingRequirements::new(&exact)
        .unwrap()
        .map(&exact, |v| v.re.to_f64(), 53);
    assert!(matches!(result, Err(reason) if reason.contains("deliberate constant failure")));
}

#[test]
fn retained_function_bodies_and_native_builtins_do_not_require_external_callbacks() {
    let x = symbol!("mapping_body::x");
    let arg = symbol!("mapping_body::arg");
    let f = symbol!("mapping_body::f");
    let g = symbol!("mapping_body::g");
    let mut functions = FunctionMap::new();
    functions
        .add_function_with_options(
            f,
            vec![arg],
            parse!("exp(mapping_body::arg)"),
            FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
        )
        .unwrap();
    functions
        .add_function_with_options(
            g,
            vec![arg],
            function!(f, Atom::var(arg)) + 1,
            FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
        )
        .unwrap();
    let exact = function!(g, Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .function_map(functions)
        .build()
        .unwrap();
    assert!(
        !exact.export_instructions().sub_evaluators[0]
            .instructions
            .sub_evaluators
            .is_empty()
    );
    let mut mapped = MappingRequirements::new(&exact)
        .unwrap()
        .map(&exact, |v| v.re.to_multi_prec_float(128), 128)
        .unwrap();
    let mut output = [Float::with_val(128, 0)];
    mapped.evaluate(&[Float::with_val(128, 0)], &mut output);
    assert_eq!(output[0].to_f64(), 2.0);

    let external = symbol!(
        "mapping_body::external",
        eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 1.0)
    );
    let mut functions = FunctionMap::new();
    functions
        .add_function_with_options(
            f,
            vec![arg],
            function!(external, Atom::var(arg)),
            FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
        )
        .unwrap();
    let exact = function!(f, Atom::var(x))
        .evaluator(&[Atom::var(x)])
        .function_map(functions)
        .build()
        .unwrap();
    let result = MappingRequirements::new(&exact).unwrap().map(
        &exact,
        |v| v.re.to_multi_prec_float(128),
        128,
    );
    assert!(matches!(result, Err(reason) if reason.contains("implementation")));
}
