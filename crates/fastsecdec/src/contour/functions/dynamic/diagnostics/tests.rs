use super::*;
use crate::contour::functions::dynamic::{self, ProgramScope, RootProgram};
use symbolica::{
    atom::{Atom, AtomCore},
    domains::float::{Complex, ErrorPropagatingFloat, RealLike},
    symbol,
};

fn config() -> Configuration {
    Configuration {
        mode: ContourDiagnosticsMode::Aggregate,
        displacement_cap: 3.,
    }
}
fn fixture(count: usize) -> (RootProgram, Vec<Atom>, Atom) {
    let helper = RootProgram::build(count).unwrap();
    let input = (0..count + 2)
        .map(|i| Atom::var(symbol!(&format!("root_diagnostics_test::p{i}"))))
        .collect::<Vec<_>>();
    let root =
        dynamic::strength(&helper, &input[..count], &input[count], &input[count + 1]).unwrap();
    (helper, input, root)
}

#[test]
fn factory_selection_preserves_values_and_distinguishes_rounded_displacement() {
    let (helper, input, root) = fixture(1);
    let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let exact = root.evaluator(&input).build().unwrap();
    let mut plain = exact.clone().map_coeff(&|c| c.re.to_f64());
    let mut aggregate = {
        let _mode = config().enter();
        exact.map_coeff(&|c| c.re.to_f64())
    };
    let mut history = Accumulator::default();
    let expected = plain.evaluate_single(&[4., 0.8, 2.]);
    // A Disabled factory ignores even an unrelated active aggregate scope.
    history.measure(config(), Phase::Evaluation, || {
        assert_eq!(plain.evaluate_single(&[4., 0.8, 2.]), expected)
    });
    assert_eq!(
        history.take().unwrap().unwrap().evaluation.callback_calls,
        0
    );
    history.measure(config(), Phase::Evaluation, || {
        assert_eq!(aggregate.evaluate_single(&[4., 0.8, 2.]), expected);
        assert_eq!(aggregate.evaluate_single(&[1., 0.8, 2.]), 1.6);
    });
    let report = history.snapshot().unwrap().unwrap();
    let work = report.evaluation;
    assert_eq!(
        (
            work.callback_calls,
            work.closed_form_calls,
            work.correction_evaluations
        ),
        (2, 2, 2)
    );
    assert_eq!(
        (
            work.solver_calls,
            work.solver_iterations,
            work.solver_evaluations
        ),
        (0, 0, 0)
    );
    assert_eq!(work.strength.count, 2);
    assert_eq!(
        (
            work.normalized_displacement.count,
            work.normalized_displacement.unavailable
        ),
        (1, 1)
    );
    let expected_normalized = (expected / 2.) * 3_f64.sqrt();
    assert_eq!(
        work.normalized_displacement.minimum,
        Some(expected_normalized)
    );
    assert_eq!(
        work.physical_displacement.minimum,
        Some(3. * expected_normalized)
    );
    assert!(dynamic::take_failure().is_none());
}

#[test]
fn native_solver_work_and_failed_callbacks_are_separate() {
    let (helper, input, root) = fixture(2);
    let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let _mode = config().enter();
    let mut evaluator = root
        .evaluator(&input)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    let mut history = Accumulator::default();
    history.measure(config(), Phase::Evaluation, || {
        let value = evaluator.evaluate_single(&[2., 3., 0.8, 1.]);
        let expected = 0.8 * (2_f64 / (2. + 16_f64.sqrt())).sqrt();
        assert!((value - expected).abs() < 1e-14);
        assert!(evaluator.evaluate_single(&[0., 3., 0.8, 1.]).is_nan());
    });
    let work = history.take().unwrap().unwrap().evaluation;
    assert_eq!(
        (
            work.callback_calls,
            work.callback_failures,
            work.solver_calls,
            work.solver_successes,
            work.solver_failures
        ),
        (2, 1, 1, 1, 0)
    );
    assert!(work.solver_iterations > 0 && work.solver_evaluations >= work.solver_iterations);
    assert_eq!(work.maximum_solver_iterations, work.solver_iterations);
    assert_eq!(
        work.solver_numerical_zero + work.solver_bracket_width + work.solver_newton_correction,
        1
    );
    assert_eq!(work.correction_evaluations, 1);
    assert!(dynamic::take_failure().unwrap().contains("a2 >= 1"));
}

#[test]
fn overflow_is_sticky_and_does_not_change_native_value_or_failure_state() {
    let (helper, input, root) = fixture(1);
    let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let _mode = config().enter();
    let mut evaluator = root
        .evaluator(&input)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    let mut history = Accumulator::default();
    let value = history.measure(config(), Phase::Evaluation, || {
        ATTEMPT.with_borrow_mut(|s| s.as_mut().unwrap().work.callback_calls = u64::MAX);
        evaluator.evaluate_single(&[4., 0.8, 1.])
    });
    assert_eq!(value, 0.4);
    assert!(dynamic::take_failure().is_none());
    assert!(history.snapshot().is_err());
    assert!(history.take().is_err());
    assert!(history.snapshot().is_err());
    assert!(ATTEMPT.with_borrow(|s| s.is_none()));
}

#[test]
fn nested_and_unwound_scopes_restore_independent_histories() {
    let mut outer = Accumulator::default();
    let mut inner = Accumulator::default();
    outer.measure(config(), Phase::Exact, || {
        record(Event::default());
        inner.measure(config(), Phase::Pilot, || record(Event::default()));
        let _ = std::panic::catch_unwind(|| {
            let mut discarded = Accumulator::default();
            discarded.measure(config(), Phase::Preparation, || {
                panic!("scope restoration probe")
            });
        });
        record(Event::default());
    });
    assert_eq!(outer.snapshot().unwrap().unwrap().exact.callback_calls, 2);
    assert_eq!(inner.snapshot().unwrap().unwrap().pilot.callback_calls, 1);
    assert!(ATTEMPT.with_borrow(|s| s.is_none()));
    assert!(outer.clone().snapshot().unwrap().is_none());
    let mut report = outer.snapshot().unwrap().unwrap();
    let old = report.clone();
    let mut incoming = ContourRuntimeReport::default();
    incoming.exact.callback_calls = u64::MAX;
    assert!(report.merge(&incoming).is_err());
    assert_eq!(report, old);
}

#[test]
fn aggregate_preserves_complex_tracked_uncertainty() {
    let (helper, input, root) = fixture(1);
    let _owner = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let exact = root.evaluator(&input).build().unwrap();
    let map = |c: &Complex<symbolica::domains::rational::Rational>| {
        Complex::new(
            ErrorPropagatingFloat::new_with_accuracy(c.re.to_f64(), f64::INFINITY),
            ErrorPropagatingFloat::new_with_accuracy(c.im.to_f64(), f64::INFINITY),
        )
    };
    let mut plain = exact.clone().map_coeff(&map);
    let mut observed = {
        let _mode = config().enter();
        exact.map_coeff(&map)
    };
    let tracked = |x| ErrorPropagatingFloat::new_with_accuracy(x, f64::INFINITY);
    let values = [
        Complex::new(
            tracked(4.),
            ErrorPropagatingFloat::new_with_accuracy(0., 6.),
        ),
        Complex::new(tracked(0.8), tracked(0.)),
        Complex::new(tracked(1.), tracked(0.)),
    ];
    let expected = plain.evaluate_single(&values);
    let mut history = Accumulator::default();
    let actual = history.measure(config(), Phase::Evaluation, || {
        observed.evaluate_single(&values)
    });
    assert_eq!(actual.re.to_f64(), expected.re.to_f64());
    assert_eq!(
        actual.im.get_absolute_error(),
        expected.im.get_absolute_error()
    );
    assert!(actual.im.get_absolute_error() > 0.);
    assert_eq!(
        history
            .snapshot()
            .unwrap()
            .unwrap()
            .evaluation
            .callback_calls,
        1
    );
}
