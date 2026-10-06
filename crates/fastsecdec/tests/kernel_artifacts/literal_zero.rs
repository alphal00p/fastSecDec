use super::*;
use fastsecdec::kernel::ReplayPolicy;

fn padded(complex: bool, tiny: bool) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("cold_zero::x")],
        symbol!("cold_zero::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            if complex {
                parse!("2+3𝑖")
            } else {
                Atom::one()
            },
            vec![Atom::Zero],
            vec![PolynomialFactor::new(
                if tiny {
                    parse!("cold_zero::x^2+cold_zero::eps^2*cold_zero::x^3")
                } else {
                    parse!("cold_zero::x+cold_zero::eps^2*cold_zero::x^2")
                },
                Atom::one(),
                FactorRole::Polynomial,
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
}

#[test]
fn cold_literal_zero_padding_does_not_trigger_weighted_range_rescue() {
    for complex in [false, true] {
        let fresh = padded(complex, false);
        let bytes = fresh.to_bytes().unwrap();
        let cold = KernelSet::from_bytes(&bytes).unwrap();
        assert_eq!(fresh.content_id(), cold.content_id());
        assert_eq!(cold.to_bytes().unwrap(), bytes);
        for kernels in [&fresh, &cold] {
            let mut worker = kernels
                .evaluation_context(0, ReplayPolicy::default())
                .unwrap();
            let mut output = vec![0.0; kernels.orders().len()];
            // The first sample still gets the policy's full-vector MPFR check.
            assert!(
                worker
                    .evaluate_weighted(&[0.5], 1.0, &mut output)
                    .unwrap()
                    .replayed
            );
            let report = worker.evaluate_weighted(&[0.5], 2.0, &mut output).unwrap();
            assert!(!report.weighted_check && !report.replayed);
            assert!(!report.precision.rescued);
            let expected = if complex {
                vec![2.0, 3.0, 0.0, 0.0, 1.0, 1.5]
            } else {
                vec![1.0, 0.0, 0.5]
            };
            assert_eq!(output, expected);
            assert_eq!(kernels.to_bytes().unwrap(), bytes);
        }
    }
}

#[test]
fn cold_literal_zero_proof_preserves_genuine_underflow_rescue() {
    for complex in [false, true] {
        let fresh = padded(complex, true);
        let bytes = fresh.to_bytes().unwrap();
        let cold = KernelSet::from_bytes(&bytes).unwrap();
        for kernels in [&fresh, &cold] {
            let mut worker = kernels
                .evaluation_context(0, ReplayPolicy::default())
                .unwrap();
            let mut output = vec![0.0; kernels.orders().len()];
            worker.evaluate_weighted(&[0.5], 1.0, &mut output).unwrap();
            let report = worker
                .evaluate_weighted(&[1e-160], 1e300, &mut output)
                .unwrap();
            assert!(!report.weighted_check && !report.replayed);
            assert!(report.precision.rescued);
            let expected = if complex {
                vec![2e-20, 3e-20, 0.0, 0.0, 2e-180, 3e-180]
            } else {
                vec![1e-20, 0.0, 1e-180]
            };
            for (actual, expected) in output.iter().zip(expected) {
                if expected == 0.0 {
                    assert_eq!(*actual, 0.0);
                } else {
                    assert!(
                        (actual / expected - 1.0).abs() < 1e-12,
                        "{actual} != {expected}"
                    );
                }
            }
            assert_eq!(kernels.to_bytes().unwrap(), bytes);
            assert_eq!(kernels.content_id(), fresh.content_id());
        }
    }
}
