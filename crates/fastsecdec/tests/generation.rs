use fastsecdec::{
    generation::{GenerationOptions, generate},
    parametric::{FactorRole, ParametricIntegrand, ParametricTerm, PolynomialFactor},
};
use fastsecdec_sectors::ParametricDomain;
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, Symbol},
    parse, symbol,
};

fn input(
    parameters: Vec<Symbol>,
    powers: Vec<Atom>,
    prefactor: Atom,
    factors: Vec<PolynomialFactor>,
) -> ParametricIntegrand {
    ParametricIntegrand::new(
        parameters,
        symbol!("eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(prefactor, powers, factors)],
    )
    .unwrap()
}
fn factor(polynomial: Atom, exponent: Atom) -> PolynomialFactor {
    PolynomialFactor::new(polynomial, exponent, FactorRole::Singularity)
}
fn run(input: &ParametricIntegrand) -> fastsecdec::generation::GeneratedIntegral {
    generate(input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
}
fn midpoint(integral: &fastsecdec::generation::GeneratedIntegral, n: usize) -> Vec<f64> {
    let mut kernels = integral.compile().unwrap();
    let mut result = kernels.exact_coefficients().to_vec();
    for kernel in kernels.sectors_mut() {
        assert_eq!(kernel.dimension(), 1);
        let mut values = vec![0.0; result.len()];
        for i in 0..n {
            kernel
                .evaluate(&[(i as f64 + 0.5) / n as f64], &mut values)
                .unwrap();
            for (sum, value) in result.iter_mut().zip(&values) {
                *sum += value / n as f64;
            }
        }
    }
    result
}

#[test]
fn exact_logarithmic_pole_and_zero_dimensional_piece() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("-1+eps")],
        Atom::one(),
        vec![],
    ));
    assert_eq!(generated.orders(), &[-1, 0]);
    assert!(generated.sectors().is_empty());
    assert_eq!(generated.exact_coefficients(), &[Atom::one(), Atom::Zero]);
    assert_eq!(
        generated.compile().unwrap().exact_coefficients(),
        &[1.0, 0.0]
    );
}

#[test]
fn endpoint_subtraction_integrates_a_nonconstant_remainder() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("-1+eps")],
        Atom::one(),
        vec![factor(parse!("1+x"), parse!("-1"))],
    ));
    assert_eq!(generated.orders(), &[-1, 0]);
    let result = midpoint(&generated, 2048);
    assert!((result[0] - 1.0).abs() < 1e-12);
    assert!((result[1] + 2f64.ln()).abs() < 1e-7);
}

#[test]
fn prefactor_pole_requests_extra_regular_orders() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("eps")],
        parse!("gamma(eps)"),
        vec![],
    ));
    assert_eq!(generated.orders(), &[-1, 0]);
    let result = midpoint(&generated, 65536);
    assert!((result[0] - 1.0).abs() < 1e-12);
    assert!((result[1] + 1.0 + std::f64::consts::EULER_GAMMA).abs() < 1e-5);
}

#[test]
fn higher_power_pole_uses_taylor_derivatives() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("-2+eps")],
        Atom::one(),
        vec![PolynomialFactor::new(
            parse!("1+3*x+x^2"),
            Atom::one(),
            FactorRole::Polynomial,
        )],
    ));
    let result = midpoint(&generated, 2048);
    assert_eq!(generated.orders(), &[-1, 0]);
    assert!((result[0] - 3.0).abs() < 1e-12);
    assert!(result[1].abs() < 1e-12); // -1 + 1
}

#[test]
fn massless_projective_bubble_matches_gamma_identity() {
    let input = ParametricIntegrand::new(
        vec![symbol!("x"), symbol!("y")],
        symbol!("eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            parse!("gamma(eps)"),
            vec![Atom::Zero; 2],
            vec![
                factor(parse!("x+y"), parse!("-2+2*eps")),
                factor(parse!("x*y"), parse!("-eps")),
            ],
        )],
    )
    .unwrap();
    let generated = run(&input);
    let result = midpoint(&generated, 65536);
    assert_eq!(generated.orders(), &[-1, 0]);
    assert!((result[0] - 1.0).abs() < 1e-9);
    assert!((result[1] - (2.0 - std::f64::consts::EULER_GAMMA)).abs() < 2e-5);
}

#[test]
fn orthant_inversion_charts_preserve_integral() {
    let input = ParametricIntegrand::new(
        vec![symbol!("x")],
        symbol!("eps"),
        ParametricDomain::PositiveOrthant,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![factor(parse!("1+x"), parse!("-2"))],
        )],
    )
    .unwrap();
    let generated = run(&input);
    let result = midpoint(&generated, 4096);
    assert_eq!(generated.orders(), &[0]);
    assert!((result[0] - 1.0).abs() < 1e-8);
}

#[test]
fn unregulated_divergence_and_nonlinear_endpoint_exponents_reject() {
    for power in [parse!("-1"), parse!("-2"), parse!("-1+eps^2")] {
        assert!(
            generate(
                &input(vec![symbol!("x")], vec![power], Atom::one(), vec![]),
                &GenerationOptions::default(),
                |_| ControlFlow::Continue(())
            )
            .is_err()
        );
    }
}

#[test]
fn threshold_assumptions_are_caller_responsibility_and_runtime_failures_remain_errors() {
    for polynomial in [parse!("1-2*x"), parse!("1-x+x^2")] {
        let input = input(
            vec![symbol!("x")],
            vec![Atom::Zero],
            Atom::one(),
            vec![factor(polynomial.clone(), parse!("-1"))],
        );
        for assume_no_threshold in [false, true] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    assume_no_threshold,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            let domain = generated.metadata().domain_assessment();
            assert_eq!(
                domain.branch_policy(),
                fastsecdec::generation::BranchPolicy::UserResponsible
            );
            assert!(!domain.relies_on_assertion());
            assert_eq!(domain.caller_asserted(), assume_no_threshold);
            assert_eq!(domain.factors()[0].polynomial(), &polynomial);
            assert_eq!(
                domain.factors()[0].certificate(),
                fastsecdec::generation::FactorCertificate::UncheckedUserResponsibility
            );
        }
    }
    // Formal generation is allowed; hitting a true numerical pole is not zero.
    let generated = run(&input(
        vec![symbol!("x")],
        vec![Atom::Zero],
        Atom::one(),
        vec![factor(parse!("1-2*x"), parse!("-1"))],
    ));
    let mut kernels = generated.compile().unwrap();
    assert!(
        kernels.sectors_mut()[0]
            .evaluate(&[0.5], &mut [0.0])
            .is_err()
    );
}

#[test]
fn worker_clone_and_content_identity_depend_on_whole_expression() {
    let first = run(&input(
        vec![symbol!("x")],
        vec![Atom::one()],
        Atom::one(),
        vec![],
    ));
    let second = run(&input(
        vec![symbol!("x")],
        vec![Atom::one()],
        Atom::num(2),
        vec![],
    ));
    let a = first.compile().unwrap();
    let b = second.compile().unwrap();
    assert_ne!(a.content_id(), b.content_id());
    assert_eq!(a.content_id(), first.compile().unwrap().content_id());
    let mut worker = a.sectors()[0].try_clone().unwrap();
    let mut output = [0.0];
    worker.evaluate(&[0.25], &mut output).unwrap();
    assert_eq!(output, [0.25]);
    assert!(worker.evaluate(&[-0.1], &mut output).is_err());
}

#[test]
fn full_double_pole_vector_retains_boundary_cancellations() {
    let generated = run(&input(
        vec![symbol!("x"), symbol!("y")],
        vec![parse!("-1+eps"), parse!("-1+2*eps")],
        Atom::one(),
        vec![PolynomialFactor::new(
            parse!("1+x+y"),
            Atom::one(),
            FactorRole::Polynomial,
        )],
    ));
    assert_eq!(generated.orders(), &[-2, -1, 0]);
    let mut kernels = generated.compile().unwrap();
    let mut sum = kernels.exact_coefficients().to_vec();
    for kernel in kernels.sectors_mut() {
        assert_eq!(kernel.dimension(), 2);
        let mut values = [0.0; 3];
        let n = 512;
        for i in 0..n {
            for j in 0..n {
                kernel
                    .evaluate(
                        &[(i as f64 + 0.5) / n as f64, (j as f64 + 0.5) / n as f64],
                        &mut values,
                    )
                    .unwrap();
                for (sum, value) in sum.iter_mut().zip(values) {
                    *sum += value / (n * n) as f64;
                }
            }
        }
    }
    assert!((sum[0] - 0.5).abs() < 1e-12);
    assert!((sum[1] - 1.5).abs() < 1e-12);
    assert!((sum[2] + 2.5).abs() < 0.002);
}

#[test]
fn native_cancellation_preserves_the_full_vector_across_precision_routes() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("-1+eps")],
        Atom::one(),
        vec![factor(parse!("1+x"), parse!("-1"))],
    ));
    let mut kernels = generated.compile().unwrap();
    for distance in [1e-12, 1e-40, 1e-100] {
        let mut output = [0.0; 2];
        let report = kernels.sectors_mut()[0]
            .evaluate_with_diagnostics(&[distance], &mut output)
            .unwrap();
        // Optimizer and routing choices may require a rescue; the scientific
        // contract is the same complete cancellation limit at every precision.
        assert!(report.bits >= 53);
        assert_eq!(output[0], 1.0);
        assert!(
            (output[1] + 1.0 / (1.0 + distance)).abs() < 1e-13,
            "distance={distance}, output={output:?}"
        );
    }
}

#[test]
fn portable_artifact_recompiles_and_rejects_modified_content() {
    let generated = run(&input(
        vec![symbol!("x")],
        vec![parse!("-1+eps")],
        Atom::one(),
        vec![factor(parse!("1+x"), parse!("-1"))],
    ));
    let kernels = generated.compile().unwrap();
    let bytes = kernels.to_bytes().unwrap();
    let mut loaded = fastsecdec::kernel::KernelSet::from_bytes(&bytes).unwrap();
    assert_eq!(kernels.content_id(), loaded.content_id());
    let mut output = [0.0; 2];
    loaded.sectors_mut()[0]
        .evaluate(&[1e-40], &mut output)
        .unwrap();
    assert_eq!(output, [1.0, -1.0]);
    let mut invalid = bytes.clone();
    let last = invalid.len() - 1;
    invalid[last] ^= 1;
    assert!(fastsecdec::kernel::KernelSet::from_bytes(&invalid).is_err());
}

#[test]
fn upper_cube_endpoint_is_recorded_without_certification() {
    // This input violates the origin-chart regularity responsibility; generation
    // keeps its formal expression and makes no claim to integrate the upper pole.
    let input = input(
        vec![symbol!("x")],
        vec![Atom::Zero],
        Atom::one(),
        vec![factor(parse!("1-x"), parse!("-1+eps"))],
    );
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(
        generated.metadata().domain_assessment().factors()[0].certificate(),
        fastsecdec::generation::FactorCertificate::UncheckedUserResponsibility
    );
    assert_eq!(
        generated.metadata().domain_assessment().factors()[0].polynomial(),
        &parse!("1-x")
    );
}

#[test]
fn massless_projective_triangle_matches_double_pole_gamma_identity() {
    let input = ParametricIntegrand::new(
        vec![symbol!("x"), symbol!("y"), symbol!("z")],
        symbol!("eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            parse!("-gamma(1+eps)"),
            vec![Atom::Zero; 3],
            vec![
                factor(parse!("x+y+z"), parse!("-1+2*eps")),
                factor(parse!("x*z"), parse!("-1-eps")),
            ],
        )],
    )
    .unwrap();
    let generated = run(&input);
    assert_eq!(generated.orders(), &[-2, -1, 0]);
    let mut kernels = generated.compile().unwrap();
    let mut sum = kernels.exact_coefficients().to_vec();
    for kernel in kernels.sectors_mut() {
        let n = 512;
        let mut output = [0.0; 3];
        for i in 0..n {
            for j in 0..n {
                kernel
                    .evaluate(
                        &[(i as f64 + 0.5) / n as f64, (j as f64 + 0.5) / n as f64],
                        &mut output,
                    )
                    .unwrap();
                for (total, value) in sum.iter_mut().zip(output) {
                    *total += value / (n * n) as f64;
                }
            }
        }
    }
    let gamma = std::f64::consts::EULER_GAMMA;
    assert!((sum[0] + 1.0).abs() < 1e-10, "{sum:?}");
    assert!((sum[1] - gamma).abs() < 0.002, "{sum:?}");
    assert!(
        (sum[2] - (std::f64::consts::PI.powi(2) / 12.0 - gamma * gamma / 2.0)).abs() < 0.004,
        "{sum:?}"
    );
}

#[test]
fn precision_rescue_resolves_logarithmic_taylor_cancellation() {
    let input = input(
        vec![symbol!("x")],
        vec![parse!("-1+eps")],
        Atom::one(),
        vec![factor(parse!("1+x"), parse!("-1-eps"))],
    );
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut kernels = generated.compile().unwrap();
    for distance in [1e-12, 1e-40, 1e-100] {
        let mut output = [0.0; 3];
        let report = kernels.sectors_mut()[0]
            .evaluate_with_diagnostics(&[distance], &mut output)
            .unwrap();
        assert!(report.rescued);
        let expected = -(distance.ln() + distance.ln_1p() / distance) / (1.0 + distance);
        assert!(
            (output[2] - expected).abs() < 1e-12,
            "{distance}: {output:?} expected {expected}"
        );
    }
}

#[test]
fn gamma_prefactor_constants_support_native_conditioning_domain() {
    let input = input(
        vec![symbol!("x")],
        vec![Atom::one()],
        parse!("gamma(3+eps)"),
        vec![],
    );
    let generated = generate(
        &input,
        &GenerationOptions {
            max_order: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let kernels = generated.compile().unwrap();
    let mut values = [0.0; 3];
    kernels.sectors()[0]
        .try_clone()
        .unwrap()
        .evaluate(&[0.5], &mut values)
        .unwrap();
    let psi = 1.5 - std::f64::consts::EULER_GAMMA;
    assert!((values[0] - 1.0).abs() < 1e-14);
    assert!((values[1] - psi).abs() < 1e-14);
    assert!(
        (values[2] - 0.5 * (psi * psi + std::f64::consts::PI.powi(2) / 6.0 - 1.25)).abs() < 1e-14
    );
}
