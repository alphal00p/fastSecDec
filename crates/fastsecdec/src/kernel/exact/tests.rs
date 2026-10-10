use super::*;
use crate::kernel::{evaluator::MappingRequirements, program::ExactProgram};
use symbolica::{parse, symbol};
mod checked;

#[test]
fn complete_native_vector_shares_callbacks_and_keeps_branches_lazy() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use symbolica::{atom::EvaluationInfo, function};
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let f = symbol!(
        "exact_vector::counted",
        eval = EvaluationInfo::new().register(move |x: &[f64]| {
            counter.fetch_add(1, Ordering::SeqCst);
            x[0] * x[0]
        })
    );
    let invalid = symbol!(
        "exact_vector::dead",
        eval = EvaluationInfo::new()
            .register(|_: &[f64]| -> f64 { panic!("dead exact branch executed") })
    );
    let p = symbol!("exact_vector::p");
    let condition = symbol!("exact_vector::condition");
    let live = function!(f, Atom::var(p));
    let expressions = vec![
        function!(
            Symbol::IF,
            Atom::var(condition),
            function!(invalid, Atom::var(p)),
            live.clone()
        ),
        &live + 1,
        &live * 2,
    ];
    assert_eq!(
        evaluate(
            &expressions,
            &BTreeMap::from([(p, 2.), (condition, 0.)]),
            false
        )
        .unwrap(),
        [4., 5., 8.]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn aggregate_atoms_cancel_exact_singularities_before_native_vector_evaluation() {
    use symbolica::{atom::EvaluationInfo, function};
    let invalid = symbol!(
        "exact_vector::cancelled",
        eval = EvaluationInfo::new()
            .register(|_: &[f64]| -> f64 { panic!("cancelled exact callback executed") })
    );
    let p = symbol!("exact_vector::zero_parameter");
    let left = Atom::one() / Atom::var(p) + function!(invalid, Atom::var(p));
    let right = -left.clone();
    let raw = [&left + &right, &left + &right + 6];
    // Native Add normalization does not distribute a numeric sign into the
    // negative sum. Exercise the same narrow operation as both generation and
    // selected-record assembly, before the direct numeric owner sees the sum.
    assert!(!raw[0].is_zero());
    let summed = raw
        .iter()
        .map(crate::generation::normalize_exact_coefficient)
        .collect::<Vec<_>>();
    assert_eq!(summed, [Atom::Zero, Atom::num(6)]);
    assert_eq!(
        evaluate(&summed, &BTreeMap::from([(p, 0.)]), false).unwrap(),
        [0., 6.]
    );
}

#[test]
fn exact_numeric_weight_normalization_preserves_dynamic_root_keys_and_products() {
    use crate::contour::functions::dynamic::{RootProgram, strength};
    let p = Atom::var(symbol!("exact_normalization::p"));
    let q = Atom::var(symbol!("exact_normalization::q"));
    let helper = RootProgram::build(1).unwrap();
    let root = strength(
        &helper,
        &[2 * (&p + 1)],
        &(Atom::num((1, 5)) * (&q + 1)),
        &Atom::one(),
    )
    .unwrap();
    let normalize = crate::generation::normalize_exact_coefficient;
    // Count/semantic tags and the deliberately factored callback arguments are
    // opaque to Symbolica's expand_num, preserving the saved exact association.
    assert_eq!(normalize(&root), root);
    let factored = (&p + 1) * (&q + 1);
    assert_eq!(normalize(&factored), factored);
    let aggregate: Atom = &root + 2 * (Atom::one() / &p + 1) - 2 / &p;
    let normalized = normalize(&aggregate);
    assert_eq!(normalized, &root + 2);
    assert!(normalized.contains(root.as_view()));
}

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

#[test]
fn dynamic_exact_offsets_scope_precision_and_retry_without_stale_failure() {
    use crate::contour::functions::dynamic::{
        ProgramScope, RootProgram, failure, strength, take_failure,
    };
    let helper = RootProgram::build(1).unwrap();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let p = symbol!("dynamic_exact_offset::p");
    let root = strength(
        &helper,
        &[Atom::var(p).pow(2)],
        &Atom::num((4, 5)),
        &Atom::one(),
    )
    .unwrap();
    for complex in [false, true] {
        // Both fixed-exponent domains overflow a2. The native Float attempt
        // must map the callback at its actual precision and recover lambda.
        let value = evaluate(
            std::slice::from_ref(&root),
            &BTreeMap::from([(p, 1e200)]),
            complex,
        )
        .unwrap();
        assert!(value[0] > 0.0);
        assert!((value[0] / 8e-201 - 1.0).abs() < 2e-14);
        if complex {
            assert_eq!(value[1], 0.0);
        }
        assert!(
            take_failure().is_none(),
            "failed attempts leaked diagnostics"
        );

        failure("outer callback failure".into());
        let value = evaluate(
            std::slice::from_ref(&root),
            &BTreeMap::from([(p, 2.0)]),
            complex,
        )
        .unwrap();
        assert!((value[0] - 0.4).abs() < 1e-15);
        assert_eq!(take_failure().as_deref(), Some("outer callback failure"));
    }
    let invalid = strength(&helper, &[Atom::num(4)], &Atom::Zero, &Atom::one()).unwrap();
    assert!(matches!(
        evaluate(std::slice::from_ref(&invalid), &BTreeMap::new(), false),
        Err(KernelError::Contour(reason)) if reason.contains("safety fraction")
    ));
    assert!(take_failure().is_none());
    // Native expression evaluation may return an error after an earlier
    // callback has already failed. That attempt still needs a precision retry.
    assert!(matches!(
        evaluate(&[invalid, parse!("unknown_dynamic_exact_function(1)")], &BTreeMap::new(), false),
        Err(KernelError::Contour(reason)) if reason.contains("safety fraction")
    ));
    assert!(take_failure().is_none());
}
