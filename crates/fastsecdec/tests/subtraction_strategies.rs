use std::ops::ControlFlow;

use fastsecdec::{
    generation::{GenerationOptions, SubtractionStrategy, generate},
    kernel::KernelError,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{
    atom::Atom,
    domains::{float::Complex, rational::Rational},
    parse, symbol,
};

fn input(degree: i32, complex: bool, second_axis: bool) -> ParametricIntegrand {
    let mut parameters = vec![symbol!("strategy::x")];
    let mut powers = vec![parse!("strategy::eps") - Atom::num(degree)];
    if second_axis {
        parameters.push(symbol!("strategy::y"));
        powers.push(Atom::one());
    }
    ParametricIntegrand::new(
        parameters,
        symbol!("strategy::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                Atom::num(Complex::new(Rational::from(2), Rational::from(3)))
            } else {
                Atom::one()
            },
            powers,
            vec![PolynomialFactor::new(
                parse!("1+strategy::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap()
}

#[test]
fn ibp_and_taylor_have_equal_complete_integrals_and_correct_boundary_terms() {
    for degree in [2, 3, 8] {
        let expected = [
            (-1.0f64).powi(degree - 1),
            (0..degree - 1)
                .map(|k| (-1.0f64).powi(k) / f64::from(k - degree + 1))
                .sum::<f64>()
                + (-1.0f64).powi(degree) * 2.0f64.ln(),
        ];
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let generated = generate(
                &input(degree, false, false),
                &GenerationOptions {
                    subtraction,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.orders(), [-1, 0]);
            if subtraction == SubtractionStrategy::IntegrateByParts {
                assert!(
                    generated
                        .sectors()
                        .iter()
                        .all(|sector| sector.cancellation_degree() <= 1)
                );
            }
            let mut kernels = generated.compile().unwrap();
            let mut total = kernels.exact_coefficients().to_vec();
            for kernel in kernels.sectors_mut() {
                for sample in 0..4096 {
                    let mut values = [0.0; 2];
                    kernel
                        .evaluate(&[(sample as f64 + 0.5) / 4096.0], &mut values)
                        .unwrap();
                    for (target, value) in total.iter_mut().zip(values) {
                        *target += value / 4096.0;
                    }
                }
            }
            for (actual, expected) in total.iter().zip(expected) {
                assert!(
                    (actual - expected).abs() < 2e-6,
                    "{subtraction:?}, degree{degree}: {total:?} versus {expected}"
                );
            }
        }
    }
}

#[test]
fn unrelated_small_coordinates_do_not_trigger_real_or_complex_rescue() {
    for complex in [false, true] {
        let generated = generate(
            &input(1, complex, true),
            &GenerationOptions::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let active = (0..2)
            .filter(|axis| {
                generated.sectors()[0]
                    .cancellation_terms()
                    .iter()
                    .any(|row| row[*axis] != 0)
            })
            .collect::<Vec<_>>();
        assert_eq!(active.len(), 1);
        let mut unrelated = [0.5; 2];
        unrelated[1 - active[0]] = 1e-80;
        let mut singular = [0.5; 2];
        singular[active[0]] = 1e-8;
        let mut kernels = generated.compile().unwrap();
        let mut values = vec![0.0; kernels.orders().len()];
        let report = kernels.sectors_mut()[0]
            .evaluate_with_diagnostics(&unrelated, &mut values)
            .unwrap();
        assert!(!report.checked && !report.rescued);
        let report = kernels.sectors_mut()[0]
            .evaluate_with_diagnostics(&singular, &mut values)
            .unwrap();
        assert!(
            !report.checked,
            "distance routing performs no comparison check"
        );
        assert!(report.rescued);
        assert_eq!(report.class, fastsecdec::kernel::PrecisionClass::Arbitrary);
        let bytes = kernels.to_bytes().unwrap();
        let mut restored = fastsecdec::kernel::KernelSet::from_bytes(&bytes).unwrap();
        assert_eq!(kernels.content_id(), restored.content_id());
        assert!(
            !restored.sectors_mut()[0]
                .evaluate_with_diagnostics(&unrelated, &mut values)
                .unwrap()
                .checked
        );
    }
}

#[test]
fn compilation_progress_and_cancellation_are_explicit() {
    let generated = generate(
        &input(1, false, false),
        &GenerationOptions::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut completed = Vec::new();
    let kernels = generated
        .compile_with_progress(|status| {
            assert_eq!(status.total, generated.sectors().len());
            assert!(status.elapsed_seconds >= 0.0);
            completed.push(status.completed);
            ControlFlow::Continue(())
        })
        .unwrap();
    assert_eq!(completed, (0..=kernels.sectors().len()).collect::<Vec<_>>());
    assert!(matches!(
        generated.compile_with_progress(|_| ControlFlow::Break(())),
        Err(KernelError::Cancelled)
    ));
    assert!(matches!(
        generated.compile_with_progress(|status| if status.completed == 1 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }),
        Err(KernelError::Cancelled)
    ));
}

#[test]
fn a_nonnegative_integer_factor_remains_a_regular_polynomial_weight() {
    let input = ParametricIntegrand::new(
        vec![symbol!("strategy::x"), symbol!("strategy::y")],
        symbol!("strategy::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![PolynomialFactor::new(
                parse!("strategy::x-strategy::y"),
                Atom::num(2),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let mut options = GenerationOptions::default();
    options.decomposition.max_sectors = 1;
    let generated = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut kernels = generated.compile().unwrap();
    assert_eq!(kernels.sectors().len(), 1);
    let mut integral = 0.0;
    for i in 0..128 {
        for j in 0..128 {
            let mut value = [0.0];
            kernels.sectors_mut()[0]
                .evaluate(
                    &[(i as f64 + 0.5) / 128.0, (j as f64 + 0.5) / 128.0],
                    &mut value,
                )
                .unwrap();
            integral += value[0] / 128.0f64.powi(2);
        }
    }
    assert!((integral - 1.0 / 6.0).abs() < 2e-5);
}

#[test]
fn large_regular_polynomial_powers_keep_a_compact_factored_kernel() {
    for (polynomial, point) in [
        (parse!("(1+strategy::x)^10000"), [0.0, 0.0]),
        (parse!("(strategy::x+strategy::y)^10000"), [0.5, 0.5]),
    ] {
        let input = ParametricIntegrand::new(
            vec![symbol!("strategy::x"), symbol!("strategy::y")],
            symbol!("strategy::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::Zero; 2],
                vec![PolynomialFactor::new(
                    polynomial,
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            )],
        )
        .unwrap();
        let generated = generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        assert!(
            generated.sectors()[0].coefficients()[0]
                .as_view()
                .get_byte_size()
                < 256
        );
        let mut kernels = generated.compile().unwrap();
        let mut value = [0.0];
        kernels.sectors_mut()[0]
            .evaluate(&point, &mut value)
            .unwrap();
        assert_eq!(value, [1.0]);
    }
}
