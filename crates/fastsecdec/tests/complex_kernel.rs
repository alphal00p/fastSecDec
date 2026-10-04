//! Complex scalar weights use native complex kernels and numerical component
//! covariance, without a symbolic real/imaginary expansion.
use fastsecdec::{
    Atom, AtomCore,
    generation::{GenerationOptions, generate},
    integration::{CoefficientComponent, IntegrationProblem, QmcSession, QmcSettings, SectorSpec},
    kernel::KernelSet,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    domains::{float::Complex, rational::Rational},
    parse, symbol,
};

fn integral(power: i32) -> KernelSet {
    let weight = Atom::num(Complex::new(Rational::from(2), Rational::from(3)));
    let input = ParametricIntegrand::new(
        vec![symbol!("complex_integral::x")],
        symbol!("complex_integral::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            weight * parse!("complex_integral::eps^-1"),
            vec![Atom::num(power)],
            vec![],
        )],
    )
    .unwrap();
    generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile()
    .unwrap()
}

#[test]
fn complete_complex_laurent_layout_survives_portable_roundtrip() {
    let kernels = integral(1);
    let mut restored = KernelSet::from_bytes(&kernels.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.content_id(), kernels.content_id());
    assert_eq!(restored.orders(), &[-1, -1, 0, 0]);
    assert_eq!(
        restored.components(),
        &[
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
            CoefficientComponent::Real,
            CoefficientComponent::Imag,
        ]
    );
    assert_eq!(restored.sectors()[0].dimension(), 1);
    let mut output = [0.0; 4];
    restored.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert_eq!(output, [0.5, 0.75, 0.0, 0.0]);

    let exact = integral(0);
    assert!(exact.sectors().is_empty());
    assert_eq!(exact.exact_coefficients(), &[2.0, 3.0, 0.0, 0.0]);
}

#[test]
fn complex_scalar_weight_preserves_real_imaginary_covariance() {
    let mut kernels = integral(1);
    let problem = IntegrationProblem::new_with_components(
        kernels.content_id().to_owned(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        vec![SectorSpec {
            id: 0,
            dimension: 1,
        }],
        kernels.exact_coefficients().to_vec(),
    )
    .unwrap();
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 1024,
            shifts: 8,
            seed: 2195,
            ..Default::default()
        },
    )
    .unwrap();
    let mut worker = session.worker_context(0).unwrap();
    while let Some(task) = session.next_work().unwrap() {
        session
            .submit(
                worker
                    .evaluate(task, |point, output| {
                        kernels.sectors_mut()[0].evaluate(point, output)
                    })
                    .unwrap(),
            )
            .unwrap();
    }
    let estimate = session.estimate().unwrap();
    assert_eq!(estimate.components, kernels.components());
    assert!((estimate.mean[0] - 1.0).abs() < 6.0 * estimate.standard_error[0] + 1e-10);
    assert!((estimate.mean[1] - 1.5).abs() < 6.0 * estimate.standard_error[1] + 1e-10);
    assert_eq!(&estimate.mean[2..], &[0.0, 0.0]);
    let variance = estimate.covariance_of_mean[0];
    assert!(variance > 0.0);
    assert!((estimate.covariance_of_mean[1] / variance - 1.5).abs() < 1e-4);
    assert!((estimate.covariance_of_mean[5] / variance - 2.25).abs() < 1e-4);
}

#[test]
fn gamma_constants_and_complex_boundary_rescue_remain_numeric_on_workers() {
    let build = |weight: Atom| {
        let input = ParametricIntegrand::new(
            vec![symbol!("complex_gamma::x")],
            symbol!("complex_gamma::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                weight * parse!("gamma(3+complex_gamma::eps)"),
                vec![parse!("complex_gamma::eps-1")],
                vec![PolynomialFactor::new(
                    parse!("1+complex_gamma::x"),
                    parse!("-1-complex_gamma::eps"),
                    FactorRole::Singularity,
                )],
            )],
        )
        .unwrap();
        generate(
            &input,
            &GenerationOptions {
                max_order: 2,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
        .compile()
        .unwrap()
    };
    let mut real = build(Atom::num(1));
    let complex = build(Atom::num(Complex::new(
        Rational::from(2),
        Rational::from(3),
    )));
    assert_eq!(real.sectors().len(), 1);
    let mut expected = vec![0.0; real.orders().len()];
    assert!(
        real.sectors_mut()[0]
            .evaluate_with_diagnostics(&[1e-80], &mut expected)
            .unwrap()
            .rescued
    );
    let mut worker = complex.sectors()[0].try_clone().unwrap();
    let (actual, report) = std::thread::spawn(move || {
        let mut output = vec![0.0; worker.output_count()];
        let report = worker
            .evaluate_with_diagnostics(&[1e-80], &mut output)
            .unwrap();
        (output, report)
    })
    .join()
    .unwrap();
    assert!(report.rescued);
    for (pair, expected) in actual.as_chunks::<2>().0.iter().zip(expected) {
        assert!((pair[0] - 2.0 * expected).abs() <= 1e-11 * expected.abs().max(1.0));
        assert!((pair[1] - 3.0 * expected).abs() <= 1e-11 * expected.abs().max(1.0));
    }
}

#[test]
fn native_canonical_complex_coefficients_preserve_precedence() {
    for source in [
        "(2+3𝑖)*x",
        "(-2-3𝑖)*(x+y)",
        "(1/2-3𝑖/4)*f(x)",
        "3𝑖*x",
        "(2+3𝑖)^x",
        "x^(2+3𝑖)",
        "-(2+3𝑖)*x*(1-y)",
        "f((2+3𝑖)*x,2+3𝑖)",
    ] {
        let original = Atom::parse(source, "complex_roundtrip", Default::default()).unwrap();
        let canonical = original.to_canonical_string();
        let restored = Atom::parse(&canonical, "unrelated_default", Default::default()).unwrap();
        assert_eq!(original, restored, "source {source}; canonical {canonical}");
    }
}
