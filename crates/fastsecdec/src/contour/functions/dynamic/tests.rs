use super::*;
use symbolica::{
    domains::{
        dual::HyperDual,
        float::{Complex, Real, RealLike},
        rational::Rational,
    },
    evaluate::{Dualizer, ExpressionEvaluator},
};

fn fixture(count: usize) -> (RootProgram, Vec<Atom>, Atom) {
    let helper = RootProgram::build(count).unwrap();
    let inputs = (0..count + 2)
        .map(|i| Atom::var(symbol!(&format!("dynamic_runtime_tests::p_{i}"))))
        .collect::<Vec<_>>();
    let expression = strength(
        &helper,
        &inputs[..count],
        &inputs[count],
        &inputs[count + 1],
    )
    .unwrap();
    (helper, inputs, expression)
}

#[test]
fn prepared_root_callback_preserves_owner_lifetimes_and_clone_independence() {
    let (helper, inputs, expression) = fixture(3);
    let weak = std::sync::Arc::downgrade(&helper.0);
    let exact = expression.evaluator(&inputs).build().unwrap();
    let mut eager = exact.clone().map_coeff(&|c| c.re.to_f64());
    let mut other = eager.clone();
    drop(helper);
    assert!(weak.upgrade().is_some());
    std::thread::scope(|scope| {
        scope.spawn(move || {
            for _ in 0..1000 {
                assert_eq!(other.evaluate_single(&[16., 0., 0., 0.8, 1.]), 0.2);
            }
        });
        for _ in 0..1000 {
            assert_eq!(eager.evaluate_single(&[4., 0., 0., 0.8, 1.]), 0.4);
        }
    });
    drop(eager);
    drop(exact);
    assert!(
        weak.upgrade().is_none(),
        "registry retained a dead resident helper"
    );
}

#[test]
fn implicit_strength_jets_match_native_closed_forms_including_zero_coefficients() {
    let helper = RootProgram::build(2).unwrap();
    let x = symbol!("dynamic_runtime_tests::jet_x");
    let a = Atom::one() + Atom::var(x).pow(2);
    let b = (Atom::one() - Atom::var(x)).pow(2);
    let safety = Atom::num((4, 5));
    let cap = Atom::one() + Atom::var(x);
    let implicit = strength(&helper, &[a.clone(), b.clone()], &safety, &cap).unwrap();
    let explicit =
        safety * cap * (Atom::num(2) / (&a + (a.pow(2) + Atom::num(4) * b).sqrt())).sqrt();
    let dualizer = Dualizer::new(
        HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1], vec![2], vec![3]]),
        vec![],
    );
    let mut left = implicit
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap()
        .vectorize(&dualizer)
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    let mut right = explicit
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap()
        .vectorize(&dualizer)
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    for x in [0., 0.15, 0.5, 0.93, 1.] {
        let mut lhs = [0.; 4];
        let mut rhs = [0.; 4];
        left.evaluate(&[x, 1., 0., 0.], &mut lhs);
        right.evaluate(&[x, 1., 0., 0.], &mut rhs);
        for (a, b) in lhs.into_iter().zip(rhs) {
            assert!((a - b).abs() < 2e-11, "x={x}: {a} != {b}");
        }
    }
}

fn imaginary_uncertainty<T: numeric::Number>(bits: u32)
where
    ErrorPropagatingFloat<T>: numeric::Number,
    Complex<ErrorPropagatingFloat<T>>: symbolica::evaluate::EvaluationDomain,
{
    let (helper, inputs, expression) = fixture(1);
    let exact = expression.evaluator(&inputs).build().unwrap();
    let one = T::one_at(bits);
    let tracked = |value: T| ErrorPropagatingFloat::new_with_accuracy(value, f64::INFINITY);
    let mut evaluator = with_precision(bits, || {
        exact.map_coeff_with_prec(
            &|c| {
                Complex::new(
                    tracked(one.from_rational(&c.re)),
                    tracked(one.from_rational(&c.im)),
                )
            },
            bits,
        )
    });
    for a in [1, 4] {
        let input = [
            Complex::new(
                tracked(one.from_usize(a)),
                ErrorPropagatingFloat::new_with_accuracy(one.zero(), 6.),
            ),
            Complex::new(
                tracked(one.from_usize(4) / one.from_usize(5)),
                tracked(one.zero()),
            ),
            Complex::new(tracked(one.clone()), tracked(one.zero())),
        ];
        let value = evaluator.evaluate_single(&input);
        let expected = 0.8 / f64::sqrt(a as f64);
        assert!((value.re.to_f64() - expected).abs() < 1e-14);
        assert_eq!(value.im.to_f64(), 0.);
        let expected_error = expected * 1e-6 / (2. * a as f64);
        assert!(
            value.im.get_absolute_error() >= 0.99 * expected_error,
            "lost imaginary coefficient uncertainty: {} < {expected_error}",
            value.im.get_absolute_error()
        );
    }
    drop(helper);
}
#[test]
fn complex_root_uncertainty_survives_zero_imaginary_centres_in_all_native_domains() {
    imaginary_uncertainty::<f64>(53);
    imaginary_uncertainty::<Float>(192);
}

#[test]
fn callback_failures_do_not_fabricate_zero_with_validation_absent() {
    let (_helper, inputs, expression) = fixture(2);
    let mut eager = expression
        .evaluator(&inputs)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    for data in [
        [0., 1., 0.8, 1.],
        [1., -1., 0.8, 1.],
        [1., 1., 0., 1.],
        [1., 1., 1., 1.],
        [1., 1., 0.8, 0.],
        [1., 1., 1e-300, 1e-300],
        [1., f64::NAN, 0.8, 1.],
    ] {
        take_failure();
        assert!(eager.evaluate_single(&data).is_nan());
        assert!(take_failure().is_some());
    }
}

#[test]
fn preparation_precision_is_nested_and_unwind_safe() {
    assert_eq!(preparation::precision(), None);
    with_precision(192, || {
        assert_eq!(preparation::precision(), Some(192));
        with_precision(256, || assert_eq!(preparation::precision(), Some(256)));
        assert_eq!(preparation::precision(), Some(192));
    });
    assert_eq!(preparation::precision(), None);
    let result =
        std::panic::catch_unwind(|| with_precision(128, || panic!("expected unwind probe")));
    assert!(result.is_err());
    assert_eq!(preparation::precision(), None);
}

#[test]
fn saved_helper_and_callback_restore_in_a_fresh_process() {
    let (helper, inputs, expression) = fixture(3);
    let exact = expression.evaluator(&inputs).build().unwrap();
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("helper"), helper.bytes()).unwrap();
    let bytes = bincode::serde::encode_to_vec(&exact, bincode::config::standard()).unwrap();
    std::fs::write(folder.path().join("main"), bytes).unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "contour::functions::dynamic::tests::fresh_process_callback_child",
            "--nocapture",
        ])
        .env("FASTSECDEC_ROOT_CALLBACK_FIXTURE", folder.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
fn fresh_process_callback_child() {
    let Some(folder) = std::env::var_os("FASTSECDEC_ROOT_CALLBACK_FIXTURE") else {
        return;
    };
    let folder = std::path::PathBuf::from(folder);
    register();
    let helper = RootProgram::from_bytes(&std::fs::read(folder.join("helper")).unwrap()).unwrap();
    let bytes = std::fs::read(folder.join("main")).unwrap();
    let (exact, used): (ExpressionEvaluator<Complex<Rational>>, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    let mut eager = exact.map_coeff(&|c| c.re.to_f64());
    assert!((eager.evaluate_single(&[4., 0., 0., 0.8, 1.]) - 0.4).abs() < 1e-14);
    assert_eq!(helper.coefficient_count(), 3);
}

#[cfg(feature = "native")]
#[test]
fn native_complex_batches_and_implicit_derivatives_match_eager() {
    let (_helper, inputs, root) = fixture(3);
    let i = Atom::i();
    let expression = &i * &root + &inputs[0] / (Atom::one() + i * root);
    let coordinate = inputs[0].as_var_view().unwrap().get_symbol();
    let exact = Atom::evaluator_multiple(
        &[expression.clone(), expression.derivative(coordinate)],
        &inputs,
    )
    .build()
    .unwrap();
    let mut eager = exact
        .clone()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut native = exact
        .jit_compile::<Complex<f64>>(
            symbolica::evaluate::JITCompilationSettings::default()
                .optimization_level(2)
                .direct_translation(true)
                .with_option("use_threads", "false"),
        )
        .unwrap();
    let rows = 19;
    let arguments = (0..rows)
        .flat_map(|i| {
            [1. + i as f64, (i % 7) as f64, (i % 5) as f64, 0.8, 1.]
                .map(|value| Complex::new(value, 0.))
        })
        .collect::<Vec<_>>();
    let mut expected = vec![Complex::new(0., 0.); 2 * rows];
    for (point, value) in arguments.chunks_exact(5).zip(expected.chunks_exact_mut(2)) {
        eager.evaluate(point, value);
    }
    let mut actual = vec![Complex::new(0., 0.); 2 * rows];
    native.batch_evaluate(&arguments, &mut actual, rows);
    for (actual, expected) in actual.iter().zip(&expected) {
        assert!((actual.re - expected.re).abs() < 2e-12);
        assert!((actual.im - expected.im).abs() < 2e-12);
    }
}

#[test]
fn centre_zero_real_coefficients_keep_their_root_uncertainty() {
    let (_helper, inputs, expression) = fixture(2);
    let exact = expression.evaluator(&inputs).build().unwrap();
    let tracked = |value| ErrorPropagatingFloat::new_with_accuracy(value, f64::INFINITY);
    let mut evaluator = exact.map_coeff(&|c| tracked(c.re.to_f64()));
    let output = evaluator.evaluate_single(&[
        tracked(4.),
        ErrorPropagatingFloat::new_with_accuracy(0., 6.),
        tracked(0.8),
        tracked(1.),
    ]);
    assert!((output.to_f64() - 0.4).abs() < 1e-14);
    assert!(output.get_absolute_error() >= 0.99 * 1.25e-8);
}

#[test]
fn root_keeps_native_double_double_and_multiprecision_domains() {
    let (_helper, inputs, expression) = fixture(3);
    let exact = expression.evaluator(&inputs).build().unwrap();
    let mut dd = exact.clone().map_coeff(&|c| DoubleFloat::from(&c.re));
    let answer = dd.evaluate_single(&[4., 0., 0., 0.8, 1.].map(DoubleFloat::from));
    assert!((answer - DoubleFloat::from(0.4)).norm().to_f64() < 1e-29);
    let mut mp = with_precision(192, || {
        exact.map_coeff_with_prec(&|c| c.re.to_multi_prec_float(192), 192)
    });
    let answer = mp.evaluate_single(&[4., 0., 0., 0.8, 1.].map(|x| Float::with_val(192, x)));
    assert!((answer - Float::with_val(192, 0.4)).norm().to_f64() < 1e-50);
}
