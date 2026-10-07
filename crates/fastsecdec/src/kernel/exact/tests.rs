use super::*;
use crate::kernel::{evaluator::MappingRequirements, program::ExactProgram};
use symbolica::{parse, symbol};

#[test]
fn direct_native_offsets_match_compiled_complex_branches_and_callbacks() {
    let _ = symbolica::transcendental::gamma();
    let p = symbol!("direct_offsets::p");
    let expressions = vec![
        parse!("direct_offsets::p^(1/2)"),
        parse!("log(direct_offsets::p)"),
        parse!("(2+3𝑖)*polygamma(1,3)"),
    ];
    let program: ExactProgram = Atom::evaluator_multiple(&expressions, &[Atom::var(p)])
        .build()
        .unwrap();
    let mut previous = MappingRequirements::new(&program)
        .unwrap()
        .map(&program, |v| Complex::new(v.re.to_f64(), v.im.to_f64()), 53)
        .unwrap();
    for value in [4.0, -4.0] {
        let direct = evaluate(&expressions, &BTreeMap::from([(p, value)]), true).unwrap();
        let mut reference = vec![Complex::new(0.0, 0.0); expressions.len()];
        previous.evaluate(&[Complex::new(value, 0.0)], &mut reference);
        for (actual, expected) in direct
            .iter()
            .zip(reference.into_iter().flat_map(|v| [v.re, v.im]))
        {
            assert!((actual - expected).abs() < 2e-14 * (1.0 + expected.abs()));
        }
    }
}

#[test]
fn direct_offsets_preserve_range_rescue_and_confirm_whole_value_zero() {
    let p = symbol!("direct_offsets::p");
    let q = symbol!("direct_offsets::q");
    let expression = parse!("direct_offsets::p^2*direct_offsets::q");
    for complex in [false, true] {
        let scale = if complex {
            parse!("2+3𝑖")
        } else {
            Atom::one()
        };
        for (p_value, q_value, expected) in [(1e200, 1e-200, 1e200), (1e-200, 1e200, 1e-200)] {
            // Squaring p over/underflows f64 before the representable product.
            let output = evaluate(
                &[expression.clone() * &scale],
                &BTreeMap::from([(p, p_value), (q, q_value)]),
                complex,
            )
            .unwrap();
            let reference = if complex {
                vec![2.0 * expected, 3.0 * expected]
            } else {
                vec![expected]
            };
            for (actual, expected) in output.iter().zip(reference) {
                assert!(*actual > 0.0);
                assert!((actual - expected).abs() < 2e-14 * expected);
            }
        }
        let zero = evaluate(
            &[parse!("direct_offsets::p-4")],
            &BTreeMap::from([(p, 4.0)]),
            complex,
        )
        .unwrap();
        assert!(zero.iter().all(|value| *value == 0.0));
    }
    assert!(matches!(
        evaluate(
            &[parse!("1/direct_offsets::p")],
            &BTreeMap::from([(p, 0.0)]),
            false
        ),
        Err(KernelError::NonFinite)
    ));
    assert!(
        evaluate(
            &[parse!("undefined_offset_function(direct_offsets::p)")],
            &BTreeMap::from([(p, 1.0)]),
            true
        )
        .is_err()
    );
}
