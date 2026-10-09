use super::*;
use symbolica::{
    domains::{dual::HyperDual, float::RealLike, rational::Rational},
    evaluate::{Dualizer, ExpressionEvaluator},
};

fn fixture() -> (Vec<Atom>, Atom) {
    let inputs = ["smooth_positive_test::t", "smooth_positive_test::delta"]
        .map(|name| Atom::var(symbol!(name)))
        .to_vec();
    let expression = positive_part(&inputs[0], &inputs[1]);
    (inputs, expression)
}

#[test]
fn real_native_primitive_avoids_cancellation_and_overflow() {
    for (t, delta) in [
        (0_f64, 1e-300),
        (1e-300, 1e-300),
        (-1e-300, 1e-300),
        (1e308, 1e-3),
        (-1e308, 1e-3),
        (1e308, 1e308),
        (-1e308, 1e308),
        (2., 1e-3),
        (-2., 1e-3),
    ] {
        super::super::dynamic::take_failure();
        let result = real(&[t, delta], true);
        assert!(result.is_finite() && result > 0., "{t}, {delta}: {result}");
        let mp = real(
            &[Float::with_val(192, t), Float::with_val(192, delta)],
            true,
        );
        let expected = mp.to_f64();
        let tolerance = 8. * f64::EPSILON * expected.abs() + 8. * f64::from_bits(1);
        assert!(
            (result - expected).abs() <= tolerance,
            "{result} != {expected}"
        );
        assert!(super::super::dynamic::take_failure().is_none());
    }
}

#[test]
fn imaginary_uncertainty_is_not_erased_by_the_real_fast_path() {
    let tracked = |value| ErrorPropagatingFloat::new_with_accuracy(value, f64::INFINITY);
    for t in [-2., 0., 2.] {
        let input = [
            Complex::new(tracked(t), ErrorPropagatingFloat::new_with_accuracy(0., 7.)),
            Complex::new(tracked(0.1), tracked(0.)),
        ];
        let value = complex(&input, true);
        let expected = real(&[t, 0.1], true);
        assert!((value.re.to_f64() - expected).abs() <= 2e-14 * expected);
        assert_eq!(value.im.to_f64(), 0.);
        let expected_error = expected / t.hypot(0.1) * 1e-7;
        assert!(
            value.im.get_absolute_error() >= 0.99 * expected_error,
            "lost tracked imaginary direction: {} < {expected_error}",
            value.im.get_absolute_error()
        );
        let delta_uncertain = [
            Complex::new(tracked(t), tracked(0.)),
            Complex::new(
                tracked(0.1),
                ErrorPropagatingFloat::new_with_accuracy(0., 7.),
            ),
        ];
        let value = complex(&delta_uncertain, true);
        let expected_error = (0.1 / t.hypot(0.1)) / 2. * 1e-7;
        assert!(
            value.im.get_absolute_error() >= 0.99 * expected_error,
            "lost imaginary regularity uncertainty: {} < {expected_error}",
            value.im.get_absolute_error()
        );
    }
    let uncertain = [
        Complex::new(
            tracked(1e200),
            ErrorPropagatingFloat::new_with_accuracy(0., 7.),
        ),
        Complex::new(tracked(1e-3), tracked(0.)),
    ];
    super::super::dynamic::take_failure();
    let result = complex(&uncertain, true);
    assert!(!result.re.to_f64().is_finite());
    assert!(super::super::dynamic::take_failure().is_some());
}

#[test]
fn tracked_native_domains_retain_real_and_imaginary_input_errors() {
    let (inputs, expression) = fixture();
    let exact = expression.evaluator(&inputs).build().unwrap();
    let tracked = |value| {
        ErrorPropagatingFloat::new_with_accuracy(Float::with_val(192, value), f64::INFINITY)
    };
    let mut mp = exact.clone().map_coeff_with_prec(
        &|c| {
            Complex::new(
                ErrorPropagatingFloat::new_with_accuracy(
                    c.re.to_multi_prec_float(192),
                    f64::INFINITY,
                ),
                ErrorPropagatingFloat::new_with_accuracy(
                    c.im.to_multi_prec_float(192),
                    f64::INFINITY,
                ),
            )
        },
        192,
    );
    for imaginary in [false, true] {
        let uncertain = ErrorPropagatingFloat::new_with_accuracy(
            Float::with_val(192, if imaginary { 0. } else { -2. }),
            7.,
        );
        let t = if imaginary {
            Complex::new(tracked(-2.), uncertain)
        } else {
            Complex::new(uncertain, tracked(0.))
        };
        let value = mp.evaluate_single(&[t, Complex::new(tracked(0.1), tracked(0.))]);
        let expected = real(&[-2_f64, 0.1], true);
        assert!((value.re.to_f64() - expected).abs() < 1e-15);
        let component = if imaginary { &value.im } else { &value.re };
        assert!(component.get_absolute_error() >= 0.99 * expected / 2_f64.hypot(0.1) * 1e-7);
    }
    let mut tracked_real = exact
        .map_coeff(&|c| ErrorPropagatingFloat::new_with_accuracy(c.re.to_f64(), f64::INFINITY));
    let value = tracked_real.evaluate_single(&[
        ErrorPropagatingFloat::new_with_accuracy(-2., 7.),
        ErrorPropagatingFloat::new_with_accuracy(0.1, f64::INFINITY),
    ]);
    assert!(value.get_absolute_error() > 0.);
}

#[test]
fn complex_continuation_is_holomorphic_not_a_hermitian_norm() {
    let t = Complex::new(-0.3, 0.2);
    let delta = Complex::new(0.4, -0.02);
    let expected = (t + (t * t + delta * delta).sqrt()) / Complex::new(2., 0.);
    let actual = complex(&[t, delta], true);
    assert!((actual.re - expected.re).abs() < 2e-15);
    assert!((actual.im - expected.im).abs() < 2e-15);
    assert!(actual.im.abs() > 0.01);
}

#[test]
fn native_symbolic_jets_match_the_defining_smooth_function() {
    let x = symbol!("smooth_positive_test::jet_x");
    let t = Atom::var(x);
    let delta = Atom::num((1, 10)) + t.pow(2);
    let implicit = positive_part(&t, &delta);
    let explicit = (&t + (t.pow(2) + delta.pow(2)).sqrt()) / Atom::num(2);
    let dualizer = Dualizer::new(
        HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1], vec![2], vec![3]]),
        vec![],
    );
    let build = |expression: &Atom| {
        expression
            .evaluator(std::slice::from_ref(&t))
            .build()
            .unwrap()
            .vectorize(&dualizer)
            .unwrap()
            .map_coeff(&|c| c.re.to_f64())
    };
    let mut lhs = build(&implicit);
    let mut rhs = build(&explicit);
    for point in [-0.7, -0.1, 0., 0.1, 0.7] {
        let mut left = [0.; 4];
        let mut right = [0.; 4];
        lhs.evaluate(&[point, 1., 0., 0.], &mut left);
        rhs.evaluate(&[point, 1., 0., 0.], &mut right);
        for (a, b) in left.into_iter().zip(right) {
            assert!(
                (a - b).abs() < 1e-11 * (1. + b.abs()),
                "at {point}: {a} != {b}"
            );
        }
    }
}

#[test]
fn runtime_failures_preserve_first_error_and_never_fabricate_zero() {
    for arguments in [
        [1., 0.],
        [1., -1.],
        [f64::INFINITY, 1.],
        [0., f64::NAN],
        [-1e308, 1e-200],
    ] {
        super::super::dynamic::take_failure();
        assert!(real(&arguments, true).is_nan());
        let reason = super::super::dynamic::take_failure().unwrap();
        failure(reason.clone());
        assert!(real(&[0., -1.], false).is_nan());
        assert_eq!(super::super::dynamic::take_failure().unwrap(), reason);
    }
}

#[test]
fn evaluator_uses_native_double_double_and_multiprecision_domains() {
    let (inputs, expression) = fixture();
    let exact = expression.evaluator(&inputs).build().unwrap();
    let mut dd = exact.clone().map_coeff(&|c| DoubleFloat::from(&c.re));
    let mut mp = exact.map_coeff_with_prec(&|c| c.re.to_multi_prec_float(192), 192);
    for (t, delta) in [(-2., 0.1), (2., 0.1), (-1e200, 1e-3)] {
        let answer = dd.evaluate_single(&[t, delta].map(DoubleFloat::from));
        let expected = mp.evaluate_single(&[t, delta].map(|x| Float::with_val(192, x)));
        assert!((answer.to_f64() - expected.to_f64()).abs() <= 1e-15 * expected.to_f64());
        assert!(answer > DoubleFloat::from(0.));
    }
}

#[test]
fn native_derivative_optimization_avoids_twice_h_overflow() {
    let (inputs, expression) = fixture();
    let t = inputs[0].as_var_view().unwrap().get_symbol();
    let delta = inputs[1].as_var_view().unwrap().get_symbol();
    let outputs = [expression.derivative(t), expression.derivative(delta)];
    let exact = Atom::evaluator_multiple(&outputs, &inputs).build().unwrap();
    let mut eager = exact.clone().map_coeff(&|c| c.re.to_f64());
    let mut mp = exact.map_coeff_with_prec(&|c| c.re.to_multi_prec_float(192), 192);
    for t in [-1e308_f64, 1e308] {
        for delta in [1e-3, 1e308] {
            let mut answer = [0.; 2];
            eager.evaluate(&[t, delta], &mut answer);
            let mut expected = [Float::with_val(192, 0), Float::with_val(192, 0)];
            mp.evaluate(&[t, delta].map(|v| Float::with_val(192, v)), &mut expected);
            for (a, b) in answer.into_iter().zip(expected) {
                assert!(b > Float::with_val(192, 0));
                // The negative-t derivative can be below the f64 range; its
                // positive native MP value remains available to precision rescue.
                let b = b.to_f64();
                assert!(a.is_finite() && a >= 0.);
                assert!((a - b).abs() <= 8. * f64::EPSILON * b + 8. * f64::from_bits(1));
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn complex_symjit_batch_matches_native_callback_and_derivatives() {
    let (inputs, expression) = fixture();
    let t = inputs[0].as_view().as_var_view().unwrap().get_symbol();
    let outputs = [expression.clone(), expression.derivative(t)];
    let exact = Atom::evaluator_multiple(&outputs, &inputs).build().unwrap();
    let mut eager = exact
        .clone()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut jit = exact
        .jit_compile::<Complex<f64>>(
            symbolica::evaluate::JITCompilationSettings::default()
                .optimization_level(2)
                .direct_translation(true)
                .with_option("use_threads", "false"),
        )
        .unwrap();
    for rows in [1, 2, 19] {
        let input = (0..rows)
            .flat_map(|i| [Complex::new(i as f64 - 9., 0.), Complex::new(0.1, 0.)])
            .collect::<Vec<_>>();
        let mut expected = vec![Complex::new(0., 0.); rows * 2];
        for (point, values) in input.chunks_exact(2).zip(expected.chunks_exact_mut(2)) {
            eager.evaluate(point, values);
        }
        let mut actual = vec![Complex::new(0., 0.); rows * 2];
        jit.batch_evaluate(&input, &mut actual, rows);
        for (a, b) in actual.iter().zip(expected) {
            assert!((a.re - b.re).abs() < 2e-13 * (1. + b.re.abs()));
            assert!((a.im - b.im).abs() < 2e-13);
        }
    }
}

#[test]
fn saved_callback_restores_in_a_fresh_process() {
    let (inputs, expression) = fixture();
    let exact = expression.evaluator(&inputs).build().unwrap();
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(
        folder.path().join("program"),
        bincode::serde::encode_to_vec(&exact, bincode::config::standard()).unwrap(),
    )
    .unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "contour::functions::smooth_positive::tests::fresh_process_callback_child",
            "--nocapture",
        ])
        .env("FASTSECDEC_SMOOTH_CALLBACK_FIXTURE", folder.path())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(String::from_utf8_lossy(&child.stderr).contains("SMOOTH_CALLBACK_CHILD_OK"));
}

#[test]
fn fresh_process_callback_child() {
    let Some(folder) = std::env::var_os("FASTSECDEC_SMOOTH_CALLBACK_FIXTURE") else {
        return;
    };
    register();
    let bytes = std::fs::read(std::path::Path::new(&folder).join("program")).unwrap();
    let (exact, used): (ExpressionEvaluator<Complex<Rational>>, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    let mut evaluator = exact.map_coeff(&|c| c.re.to_f64());
    let actual = evaluator.evaluate_single(&[-2., 0.1]);
    assert!((actual - real(&[-2., 0.1], true)).abs() < 1e-16);
    eprintln!("SMOOTH_CALLBACK_CHILD_OK");
}
