use fastsecdec::{
    generation::{GenerationOptions, generate},
    kernel::{KernelError, KernelSet, ReplayPolicy, ReplayState},
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    atom::Atom,
    domains::{float::Complex, rational::Rational},
    parse, symbol,
};

fn kernels(complex: bool, scale: i64) -> KernelSet {
    let prefactor = if complex {
        Atom::num(Complex::new(Rational::from(2), Rational::from(3)))
    } else {
        Atom::num(scale)
    };
    let input = ParametricIntegrand::new(
        vec![symbol!("weighted::x")],
        symbol!("weighted::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            prefactor,
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                parse!("weighted::x+weighted::eps*weighted::x^2"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    let mut result = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap();
    // These controls exercise the explicitly retained validated replay policy.
    result
        .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
        .unwrap();
    result
}

#[test]
fn weighted_growth_replays_and_weights_the_complete_vector_once() {
    for complex in [false, true] {
        let kernels = kernels(complex, 1);
        let mut context = kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap();
        let mut output = vec![0.0; kernels.orders().len()];
        let first = context
            .evaluate_weighted(&[0.25], 2.0, &mut output)
            .unwrap();
        assert!(first.weighted_check && first.replayed && first.precision.rescued);
        let expected = if complex {
            vec![0.5, 0.75, 0.125, 0.1875]
        } else {
            vec![0.25, 0.0625]
        };
        assert_eq!(output, expected.iter().map(|x| x * 2.0).collect::<Vec<_>>());
        assert_eq!(
            context.state().maxima(),
            expected.iter().map(|x| x * 2.0).collect::<Vec<_>>()
        );
        let repeated = context
            .evaluate_weighted(&[0.25], 2.0, &mut output)
            .unwrap();
        assert!(!repeated.weighted_check && !repeated.replayed);
        let growing = context
            .evaluate_weighted(&[0.25], 100.0, &mut output)
            .unwrap();
        assert!(growing.weighted_check && growing.replayed);
        assert_eq!(
            output,
            expected.iter().map(|x| x * 100.0).collect::<Vec<_>>()
        );
        assert_eq!(
            context.state().maxima(),
            expected.iter().map(|x| x * 100.0).collect::<Vec<_>>()
        );
    }
}

#[test]
fn zero_maxima_invalid_weights_and_overflow_preserve_accepted_state() {
    let kernels = kernels(false, 2);
    let mut context = kernels
        .evaluation_context(0, ReplayPolicy::default())
        .unwrap();
    let mut output = vec![0.0; kernels.orders().len()];
    assert!(
        context
            .evaluate_weighted(&[0.0], 1.0, &mut output)
            .unwrap()
            .replayed
    );
    assert!(context.state().verified());
    assert_eq!(context.state().maxima(), [0.0, 0.0]);
    assert!(
        context
            .evaluate_weighted(&[0.25], 1.0, &mut output)
            .unwrap()
            .replayed
    );
    let state = context.state().clone();
    for weight in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            context.evaluate_weighted(&[0.5], weight, &mut output),
            Err(KernelError::InvalidWeight)
        ));
        assert_eq!(context.state(), &state);
    }
    assert!(matches!(
        context.evaluate_weighted(&[1.0], f64::MAX, &mut output),
        Err(KernelError::PrecisionExhausted { .. })
    ));
    assert_eq!(context.state(), &state);
    assert!(
        !context
            .evaluate_weighted(&[0.5], 0.0, &mut output)
            .unwrap()
            .replayed
    );
    assert_eq!(context.state(), &state);
}

#[test]
fn replay_checkpoint_is_identity_bound_and_merges_idempotently() {
    let kernels = kernels(false, 1);
    let policy = ReplayPolicy::default();
    let mut context = kernels.evaluation_context(0, policy.clone()).unwrap();
    context
        .evaluate_weighted(&[0.5], 1.0, &mut [0.0; 2])
        .unwrap();
    let serialized = serde_json::to_vec(context.state()).unwrap();
    let state: ReplayState = serde_json::from_slice(&serialized).unwrap();
    let mut restored = kernels
        .restore_evaluation_context(0, policy.clone(), &state)
        .unwrap();
    assert!(
        !restored
            .evaluate_weighted(&[0.5], 1.0, &mut [0.0; 2])
            .unwrap()
            .replayed
    );
    restored.merge_state(&state).unwrap();
    assert_eq!(restored.state(), &state);
    assert!(
        kernels
            .restore_evaluation_context(
                0,
                ReplayPolicy {
                    growth_factor: 32.0,
                    ..policy.clone()
                },
                &state
            )
            .is_err()
    );
    assert!(
        kernels
            .restore_evaluation_context(1, policy.clone(), &state)
            .is_err()
    );
    let different = self::kernels(false, 2);
    assert!(
        different
            .restore_evaluation_context(0, policy, &state)
            .is_err()
    );
    let before = restored.state().clone();
    let mut malformed = serde_json::to_value(&state).unwrap();
    malformed["maximum_absolute_weighted"][0] = serde_json::json!(-1.0);
    let malformed: ReplayState = serde_json::from_value(malformed).unwrap();
    assert!(restored.merge_state(&malformed).is_err());
    assert_eq!(restored.state(), &before);
}

#[test]
fn replay_runs_on_a_numeric_worker_without_symbolic_operations() {
    let kernels = kernels(true, 1);
    let mut context = kernels
        .evaluation_context(0, ReplayPolicy::default())
        .unwrap();
    std::thread::spawn(move || {
        let mut output = [0.0; 4];
        assert!(
            context
                .evaluate_weighted(&[0.5], 1.0, &mut output)
                .unwrap()
                .replayed
        );
        assert_eq!(output, [1.0, 1.5, 0.5, 0.75]);
    })
    .join()
    .unwrap();
}

#[test]
fn native_weighting_recovers_subnormal_and_underflowed_full_vectors() {
    for complex in [false, true] {
        let prefactor = if complex {
            Atom::num(Complex::new(Rational::from(2), Rational::from(3)))
        } else {
            Atom::one()
        };
        let input = ParametricIntegrand::new(
            vec![symbol!("weighted_range::x")],
            symbol!("weighted_range::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                prefactor,
                vec![Atom::Zero],
                vec![PolynomialFactor::new(
                    parse!("weighted_range::x^2+weighted_range::eps*weighted_range::x^3"),
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            )],
        )
        .unwrap();
        let mut kernels = generate(
            &input,
            &GenerationOptions {
                max_order: 1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
        .compile()
        .unwrap();
        kernels
            .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
            .unwrap();
        let mut context = kernels
            .evaluation_context(0, ReplayPolicy::default())
            .unwrap();
        let mut output = vec![0.0; kernels.orders().len()];
        // Establish large maxima first: the later range check must work even
        // when the suspicious tiny value does not trigger the growth heuristic.
        context.evaluate_weighted(&[0.5], 1.0, &mut output).unwrap();
        let report = context
            .evaluate_weighted(&[1e-160], 1e300, &mut output)
            .unwrap();
        assert!(!report.weighted_check);
        assert!(report.precision.rescued);
        let expected = if complex {
            vec![2e-20, 3e-20, 2e-180, 3e-180]
        } else {
            vec![1e-20, 1e-180]
        };
        for (got, expected) in output.iter().zip(expected) {
            assert!((got / expected - 1.0).abs() < 1e-12, "{got} != {expected}");
        }
        context
            .evaluate_weighted(&[1e-200], 1e300, &mut output)
            .unwrap();
        let expected = if complex { 2e-100 } else { 1e-100 };
        assert!((output[0] / expected - 1.0).abs() < 1e-12);
    }
}

#[test]
fn native_weighting_recovers_an_unrepresentable_unweighted_value() {
    // The Laurent remainder is finite over the cube but overflows f64 at x=1:
    // scaling in native precision recovers the representable final sample.
    let input = ParametricIntegrand::new(
        vec![symbol!("weighted_large::x")],
        symbol!("weighted_large::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("10^400"),
            vec![Atom::one()],
            vec![],
        )],
    )
    .unwrap();
    let mut kernels = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile()
    .unwrap();
    kernels
        .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
        .unwrap();
    let mut context = kernels
        .evaluation_context(0, ReplayPolicy::default())
        .unwrap();
    let mut output = [0.0];
    assert!(
        context
            .evaluate_weighted(&[0.5], 1e-300, &mut output)
            .unwrap()
            .precision
            .rescued
    );
    assert!((output[0] / 5e99 - 1.0).abs() < 1e-12);
}

#[test]
fn zero_weight_does_not_hide_a_singular_endpoint_evaluation() {
    let input = ParametricIntegrand::new(
        vec![symbol!("weighted_endpoint::x")],
        symbol!("weighted_endpoint::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::var(symbol!("weighted_endpoint::eps"))],
            vec![],
        )],
    )
    .unwrap();
    let mut kernels = generate(
        &input,
        &GenerationOptions {
            max_order: 1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap();
    kernels
        .set_stability_settings(&fastsecdec::kernel::StabilitySettings::validated())
        .unwrap();
    let mut context = kernels
        .evaluation_context(0, ReplayPolicy::default())
        .unwrap();
    let original = context.state().clone();
    let mut output = [0.0; 2];
    // The epsilon coefficient log(x) is not finite at the exact endpoint.
    assert!(context.evaluate_weighted(&[0.0], 0.0, &mut output).is_err());
    assert_eq!(context.state(), &original);
    context.evaluate_weighted(&[0.5], 0.0, &mut output).unwrap();
    assert_eq!(output, [0.0, 0.0]);
    assert_eq!(context.state(), &original);
}
