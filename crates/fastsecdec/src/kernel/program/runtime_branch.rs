//! Branch-sensitive controls against native complex evaluation and exact integrals.
use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    kernel::{CompilationSession, EvaluatorBackend, KernelSet, ReplayPolicy, SectorKernel},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::evaluate::{FunctionMap, FunctionRegistrationOptions, InliningPolicy};
use symbolica::parse;

fn eager() -> CompilationSettings {
    CompilationSettings {
        backend: EvaluatorBackend::Eager,
        ..Default::default()
    }
}

fn assert_close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(actual.is_finite());
        assert!((actual - expected).abs() < 2e-12 * expected.abs().max(1.0));
    }
}

#[test]
fn runtime_branch_sqrt_and_log_survive_compilation_modes_batches_and_reload() {
    let x = symbol!("runtime_branch_public::x");
    let p = symbol!("runtime_branch_public::p");
    let eps = symbol!("runtime_branch_public::eps");
    for coefficient in [
        parse!("runtime_branch_public::p^(1/2)"),
        parse!("log(runtime_branch_public::p)"),
        parse!("abs((-runtime_branch_public::p)^(1/2))"),
    ] {
        let input = ParametricIntegrand::new(
            vec![x],
            eps,
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                coefficient.clone(),
                vec![Atom::one()],
                vec![],
            )],
        )
        .unwrap();
        let generated = Arc::new(
            generate(&input, &GenerationOptions::default(), |_| {
                ControlFlow::Continue(())
            })
            .unwrap(),
        );
        let direct = generated
            .compile_with_settings_parameters_and_progress(
                Default::default(),
                &[p],
                eager(),
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        let mut session =
            CompilationSession::new(generated.clone(), vec![p], Default::default(), eager())
                .unwrap();
        while !session.is_complete() {
            session.step(1, |_| ControlFlow::Continue(())).unwrap();
        }
        let stepped = session.take_result().unwrap();
        let dispatched = generated
            .compile_with_settings_parameters_and_dispatch(
                Default::default(),
                &[p],
                eager(),
                &mut |jobs| jobs.map(|job| job.run()).collect(),
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        assert_eq!(
            direct.artifact_bytes().unwrap(),
            stepped.artifact_bytes().unwrap()
        );
        assert_eq!(
            direct.artifact_bytes().unwrap(),
            dispatched.artifact_bytes().unwrap()
        );
        assert_eq!(direct.orders(), &[0, 0]);
        let mut reference = coefficient
            .evaluator(&[Atom::var(p)])
            .build()
            .unwrap()
            .map_coeff(&|value| Complex::new(value.re.to_f64(), value.im.to_f64()));
        for mut kernels in [direct, stepped, dispatched] {
            kernels = KernelSet::from_bytes(kernels.artifact_bytes().unwrap()).unwrap();
            for parameter in [4.0, -4.0] {
                kernels
                    .bind_parameters(&BTreeMap::from([(p, parameter)]))
                    .unwrap();
                let value = reference.evaluate_single(&[Complex::new(parameter, 0.0)]);
                assert!(value.re.is_finite() && value.im.is_finite());
                let mut output = [0.0; 2];
                kernels.sectors_mut()[0]
                    .evaluate(&[0.25], &mut output)
                    .unwrap();
                assert_close(&output, &[0.25 * value.re, 0.25 * value.im]);
                let mut context = kernels
                    .evaluation_context(0, ReplayPolicy::default())
                    .unwrap();
                let mut output = [0.0; 6];
                context
                    .evaluate_weighted_batch(&[0.2, 0.4, 0.6], &[1.0; 3], &mut output)
                    .unwrap();
                assert_close(
                    &output,
                    &[
                        0.2 * value.re,
                        0.2 * value.im,
                        0.4 * value.re,
                        0.4 * value.im,
                        0.6 * value.re,
                        0.6 * value.im,
                    ],
                );
            }
        }
    }
}

#[test]
fn runtime_branch_exact_pole_coefficient_rebinds_across_the_cut() {
    let x = symbol!("runtime_branch_exact::x");
    let p = symbol!("runtime_branch_exact::p");
    let eps = symbol!("runtime_branch_exact::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("runtime_branch_exact::p^(1/2)"),
            vec![Atom::var(eps) - 1],
            vec![],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    // The integral is exactly sqrt(p)/eps; its branch lives in the analytic
    // endpoint term, rather than in any sampled residual coefficient.
    assert!(generated.sectors().is_empty());
    let template = generated
        .compile_with_settings_parameters_and_progress(Default::default(), &[p], eager(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    assert_eq!(template.orders(), &[-1, -1, 0, 0]);
    let mut kernels = KernelSet::from_bytes(template.artifact_bytes().unwrap()).unwrap();
    for (parameter, expected) in [(4.0, [2.0, 0.0, 0.0, 0.0]), (-4.0, [0.0, 2.0, 0.0, 0.0])] {
        kernels
            .bind_parameters(&BTreeMap::from([(p, parameter)]))
            .unwrap();
        assert_close(kernels.exact_coefficients(), &expected);
    }
}

#[test]
fn runtime_branch_aliases_use_bodies_and_preserve_real_polynomial_routing() {
    let x = symbol!("runtime_branch_alias::x");
    let p = symbol!("runtime_branch_alias::p");
    let a = Atom::var(symbol!("runtime_branch_alias::a"; Real));
    let b = Atom::var(symbol!("runtime_branch_alias::b"; Real));
    let mut branch: AliasedAtom = a.clone().into();
    branch.register_alias(a.clone(), b.clone());
    branch.register_alias(b.clone(), parse!("runtime_branch_alias::p^(1/2)"));
    assert!(!is_real_with_parameters(&branch, &[x], &[p]));
    let program = build_with_settings(
        vec![x],
        &[p],
        &[branch],
        Cancellation::new(0, None, 1).unwrap(),
        eager(),
    )
    .unwrap();
    assert_eq!(program.real_coefficients, [false]);
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![program],
        vec![Atom::Zero],
        Default::default(),
        None,
        true,
        vec![p],
        eager(),
    )
    .unwrap();
    kernels
        .bind_parameters(&BTreeMap::from([(p, -4.0)]))
        .unwrap();
    let mut output = [0.0; 2];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut output)
        .unwrap();
    assert_close(&output, &[0.0, 2.0]);

    let mut polynomial: AliasedAtom = a.clone().into();
    polynomial.register_alias(a, b.clone() + 1);
    polynomial.register_alias(b, Atom::var(p) * Atom::var(x));
    polynomial.register_alias(parse!("runtime_branch_alias::unused"), parse!("1+𝑖"));
    assert!(is_real_with_parameters(&polynomial, &[x], &[p]));
    assert!(is_real_expression(
        &parse!("runtime_branch_alias::x+runtime_branch_alias::p"),
        &[x, p]
    ));
    assert!(!is_real_expression(
        &parse!("(runtime_branch_alias::x-runtime_branch_alias::p)^(1/2)"),
        &[x, p]
    ));
    let program = build_with_settings(
        vec![x],
        &[p],
        &[polynomial],
        Cancellation::new(0, None, 1).unwrap(),
        eager(),
    )
    .unwrap();
    assert_eq!(program.real_coefficients, [true]);
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![program],
        vec![Atom::Zero],
        Default::default(),
        None,
        false,
        vec![p],
        eager(),
    )
    .unwrap();
    assert_eq!(kernels.orders(), &[0]);
    assert_eq!(kernels.sectors()[0].statistics().arithmetic, "real");
    kernels
        .bind_parameters(&BTreeMap::from([(p, -4.0)]))
        .unwrap();
    let mut output = [0.0];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut output)
        .unwrap();
    assert_close(&output, &[-1.0]);
}

#[test]
fn runtime_branch_actual_poles_remain_nonfinite_errors() {
    let x = symbol!("runtime_branch_pole::x");
    let p = symbol!("runtime_branch_pole::p");
    let coefficient: AliasedAtom = parse!("1/runtime_branch_pole::p").into();
    assert!(is_real_with_parameters(&coefficient, &[x], &[p]));
    let program = build_with_settings(
        vec![x],
        &[p],
        &[coefficient],
        Cancellation::new(0, None, 1).unwrap(),
        eager(),
    )
    .unwrap();
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![program],
        vec![Atom::Zero],
        Default::default(),
        None,
        false,
        vec![p],
        eager(),
    )
    .unwrap();
    kernels
        .bind_parameters(&BTreeMap::from([(p, 0.0)]))
        .unwrap();
    let mut output = [0.0];
    assert!(matches!(
        kernels.sectors_mut()[0].evaluate(&[0.5], &mut output),
        Err(KernelError::NonFinite)
    ));
}

#[test]
fn runtime_branch_coordinate_endpoints_preserve_finite_values_and_log_errors() {
    let x = symbol!("runtime_branch_endpoint::x");
    for expression in [
        parse!("runtime_branch_endpoint::x"),
        parse!("runtime_branch_endpoint::x^(1/2)"),
        parse!("log(runtime_branch_endpoint::x)"),
    ] {
        let logarithm = expression == parse!("log(runtime_branch_endpoint::x)");
        let coefficient: AliasedAtom = expression.into();
        assert!(is_real_with_parameters(&coefficient, &[x], &[]));
        let program = build_with_settings(
            vec![x],
            &[],
            &[coefficient],
            Cancellation::new(0, None, 1).unwrap(),
            eager(),
        )
        .unwrap();
        let mut kernel = SectorKernel::from_program_with_backend(
            program,
            &Default::default(),
            false,
            EvaluatorBackend::Eager,
        )
        .unwrap();
        let mut output = [0.0];
        if logarithm {
            assert!(matches!(
                kernel.evaluate(&[0.0], &mut output),
                Err(KernelError::NonFinite)
            ));
        } else {
            kernel.evaluate(&[0.0], &mut output).unwrap();
            assert_eq!(output, [0.0]);
        }
    }
}

#[test]
fn runtime_branch_fixed_gamma_laurent_constants_retain_real_layout() {
    let x = symbol!("runtime_branch_gamma::x");
    let eps = symbol!("runtime_branch_gamma::eps");
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("gamma(3+runtime_branch_gamma::eps)"),
            vec![Atom::var(eps) - 1],
            vec![PolynomialFactor::new(
                parse!("1+runtime_branch_gamma::x"),
                parse!("-1-runtime_branch_gamma::eps"),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let kernels = generated.compile_with_settings(eager()).unwrap();
    assert_eq!(kernels.orders(), &[-1, 0, 1, 2]);
}

#[test]
fn runtime_branch_fixed_native_constant_domains_exclude_unknown_and_complex_arguments() {
    let p = symbol!("runtime_branch_fixed_domain::p");
    for expression in [
        parse!("polygamma(0,1/3)"),
        parse!("polygamma(1,3)"),
        parse!("polygamma(2,3)"),
    ] {
        assert!(is_real_expression(&expression, &[]));
    }
    for expression in [
        parse!("polygamma(1,runtime_branch_fixed_domain::p)"),
        parse!("polygamma(1/2,3)"),
        parse!("polygamma(1,3+𝑖/10^400)"),
        parse!("gamma(1+𝑖/10^400)"),
        parse!("(polygamma(1,1/3)-polygamma(1,3))^(1/2)"),
    ] {
        assert!(!is_real_expression(&expression, &[p]), "{expression}");
    }
}

#[test]
fn runtime_branch_legacy_gate_follows_noninlined_native_bodies() {
    let x = symbol!("runtime_branch_body::x");
    let argument = symbol!("runtime_branch_body::argument");
    let inner = symbol!("runtime_branch_body::inner");
    let outer = symbol!("runtime_branch_body::outer");
    for (body, expected) in [
        (parse!("log(runtime_branch_body::argument)"), true),
        (parse!("sin(runtime_branch_body::argument)"), false),
    ] {
        let mut functions = FunctionMap::new();
        functions
            .add_function_with_options(
                inner,
                vec![argument],
                body,
                FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
            )
            .unwrap();
        functions
            .add_function_with_options(
                outer,
                vec![argument],
                function!(inner, Atom::var(argument)),
                FunctionRegistrationOptions::new().inlining(InliningPolicy::Never),
            )
            .unwrap();
        let program = function!(outer, Atom::var(x))
            .evaluator(&[Atom::var(x)])
            .function_map(functions)
            .build()
            .unwrap();
        assert_eq!(legacy_real_branch(&program), expected);
    }
}

#[test]
fn runtime_branch_real_alias_cannot_hide_a_complex_intermediate() {
    let x = symbol!("runtime_branch_hidden::x");
    let p = symbol!("runtime_branch_hidden::p");
    let a = Atom::var(symbol!("runtime_branch_hidden::a"; Real));
    let b = Atom::var(symbol!("runtime_branch_hidden::b"; Real));
    let mut coefficient: AliasedAtom = a.clone().into();
    coefficient.register_alias(a, function!(Symbol::ABS, b.clone()));
    coefficient.register_alias(b, parse!("(-runtime_branch_hidden::p)^(1/2)"));
    assert!(!is_real_with_parameters(&coefficient, &[x], &[p]));
    let program = build_with_settings(
        vec![x],
        &[p],
        &[coefficient],
        Cancellation::new(0, None, 1).unwrap(),
        eager(),
    )
    .unwrap();
    let mut kernels = KernelSet::from_programs_for_load(
        vec![0],
        vec![program],
        vec![Atom::Zero],
        Default::default(),
        None,
        true,
        vec![p],
        eager(),
    )
    .unwrap();
    for parameter in [4.0, -4.0] {
        kernels
            .bind_parameters(&BTreeMap::from([(p, parameter)]))
            .unwrap();
        let mut output = [0.0; 2];
        kernels.sectors_mut()[0]
            .evaluate(&[0.5], &mut output)
            .unwrap();
        assert_close(&output, &[2.0, 0.0]);
    }
}
