use super::*;
use crate::contour::functions::dynamic::{self, ProgramScope, RootProgram};
use symbolica::{
    domains::{
        dual::HyperDual,
        float::{Complex, DoubleFloat, ErrorPropagatingFloat, Float, RealLike},
        rational::Rational,
    },
    evaluate::Dualizer,
    symbol,
};

#[test]
fn native_definitions_preserve_derivatives_and_remove_inert_arguments() {
    let (x, y, z) = symbol!(
        "compact_native::x",
        "compact_native::y",
        "compact_native::z"
    );
    let body = ((Atom::var(x) + 1).pow(8) * (Atom::var(z) + 2)).expand();
    let (definitions, calls) =
        ContourDefinitions::coefficients(&[x, y], std::slice::from_ref(&body)).unwrap();
    assert!(!definitions.is_empty());
    assert!(!calls[0].contains_symbol(y));
    definitions.validate(true).unwrap();
    let mut actual = calls[0].clone();
    let mut expected = body;
    for _ in 0..5 {
        assert_eq!(definitions.materialize(&actual).unwrap(), expected);
        actual = actual.derivative(x);
        expected = expected.derivative(x);
    }
    assert!(calls[0].derivative(y).is_zero());
}

#[test]
fn native_face_simplification_preserves_exact_and_zero_classification() {
    let (x, y, z) = symbol!("compact_face::x", "compact_face::y", "compact_face::z");
    let body = (Atom::var(x) * (Atom::var(y) + 1).pow(8) + Atom::var(z)).expand();
    let (definitions, calls) = ContourDefinitions::coefficients(&[x, y, z], &[body]).unwrap();
    assert!(!definitions.is_empty());
    let restricted = calls[0].replace(Atom::var(x)).with(Atom::Zero);
    let reduced = definitions.simplify(&restricted, &[x, y, z]).unwrap();
    assert!(!reduced.contains_symbol(y));
    assert_eq!(definitions.materialize(&reduced).unwrap(), Atom::var(z));
    let exact = reduced.replace(Atom::var(z)).with(Atom::num(3));
    assert_eq!(
        definitions.simplify(&exact, &[x, y, z]).unwrap(),
        Atom::num(3)
    );
    let zero = calls[0]
        .derivative(y)
        .replace(Atom::var(x))
        .with(Atom::Zero);
    assert!(definitions.simplify(&zero, &[x, y, z]).unwrap().is_zero());
    let (_, constants) = ContourDefinitions::coefficients(&[], &[Atom::num(123456789)]).unwrap();
    assert_eq!(constants, [Atom::num(123456789)]);
}

#[test]
fn correlated_compound_arguments_preserve_native_cancellation() {
    let (a, b, c, x, z) = symbol!(
        "compact_correlated::a",
        "compact_correlated::b",
        "compact_correlated::c",
        "compact_correlated::x",
        "compact_correlated::z"
    );
    let scale = Atom::num(1234567890123456789_i64);
    let body = &scale * (Atom::var(a) + Atom::var(b) + Atom::var(c));
    let (definitions, calls) = ContourDefinitions::coefficients(&[a, b, c], &[body]).unwrap();
    assert!(!definitions.is_empty());
    let call = calls[0].replace_multiple([
        Replacement::new(
            Pattern::Literal(Atom::var(a)),
            Pattern::Literal(Atom::var(x)),
        ),
        Replacement::new(
            Pattern::Literal(Atom::var(b)),
            Pattern::Literal(Atom::one() - Atom::var(x)),
        ),
        Replacement::new(
            Pattern::Literal(Atom::var(c)),
            Pattern::Literal(Atom::var(z)),
        ),
    ]);
    let expected = &scale * (Atom::one() + Atom::var(z));
    assert_eq!(definitions.materialize(&call).unwrap(), expected);
    let simplified = definitions.simplify(&call, &[x, z]).unwrap();
    assert_eq!(simplified, call);
    assert_eq!(definitions.materialize(&simplified).unwrap(), expected);
    // Whole-body coordinate independence remains a native exact-folding proof.
    let exact = call.replace(Atom::var(z)).with(Atom::num(2));
    assert_eq!(definitions.simplify(&exact, &[x, z]).unwrap(), 3 * scale);
}

#[test]
fn restored_definitions_require_canonical_unattributed_symbols() {
    let (x, y) = symbol!("compact_admission::x", "compact_admission::y");
    let body = (Atom::var(x) + Atom::num(2) * Atom::var(y) + Atom::one())
        .pow(9)
        .expand();
    let (definitions, _) = ContourDefinitions::coefficients(&[x, y], &[body]).unwrap();
    let definition = &definitions.entries()[0];
    let mut swapped = definition.parameters().to_vec();
    swapped.swap(0, 1);
    assert!(
        ContourDefinitions::from_parts(vec![(
            definition.function(),
            swapped,
            definition.body().clone()
        )])
        .is_err()
    );
    let attributed = SymbolBuilder::new(wrap_symbol!(
        "fastsecdec::contour::coefficient_v1_attributed_test"
    ))
    .with_attributes(&[SymbolAttribute::Symmetric])
    .build()
    .unwrap();
    assert!(attributed.is_exportable());
    assert!(
        ContourDefinitions::from_parts(vec![(
            attributed,
            definition.parameters().to_vec(),
            definition.body().clone()
        )])
        .is_err()
    );
}

#[test]
fn definition_scope_and_native_signature_are_checked() {
    let (x, y) = symbol!("compact_scope::x", "compact_scope::y");
    let body = (Atom::var(x) + 1).pow(12).expand();
    let (mut one, a) = ContourDefinitions::coefficients(&[x], std::slice::from_ref(&body)).unwrap();
    let (two, b) = ContourDefinitions::coefficients(&[x, y], &[body]).unwrap();
    assert!(one.function_map(&b).is_err());
    one.merge(&two).unwrap();
    assert!(
        one.function_map([&a[0], &b[0]])
            .unwrap_err()
            .contains("uniform")
    );
    let entry = &one.entries()[0];
    let corrupted = ContourDefinitions::from_parts(vec![(
        entry.function(),
        entry.parameters().to_vec(),
        entry.body() + 1,
    )])
    .unwrap();
    assert!(corrupted.validate(true).is_err());
    assert!(one.merge(&corrupted).is_err());
}

#[test]
fn registration_preserves_nested_arguments_and_native_mixed_derivatives() {
    let (x, y) = symbol!("compact_nested::x", "compact_nested::y");
    let bodies = [
        (Atom::var(x) + Atom::var(y) + 1).pow(4),
        (Atom::var(x) * Atom::var(y) + 2).pow(3),
    ];
    let (definitions, calls) =
        ContourDefinitions::with_required_bodies(&[x, y], &bodies, &[0, 1]).unwrap();
    let nested = calls[0].replace(Atom::var(y)).with(calls[1].clone());
    let outputs = [
        nested.clone(),
        nested.derivative(x),
        nested.derivative(x).derivative(y),
        nested.derivative(y).derivative(y).derivative(x),
    ];
    let expanded = outputs
        .iter()
        .map(|output| definitions.materialize(output).unwrap())
        .collect::<Vec<_>>();
    let parameters = [Atom::var(x), Atom::var(y)];
    let mut compact = Atom::evaluator_multiple(&outputs, &parameters)
        .function_map(definitions.function_map(&outputs).unwrap())
        .horner_iterations(0)
        .build()
        .unwrap()
        .map_coeff(&|value| Complex::new(value.re.to_f64(), value.im.to_f64()));
    let mut materialized = Atom::evaluator_multiple(&expanded, &parameters)
        .horner_iterations(0)
        .build()
        .unwrap()
        .map_coeff(&|value| Complex::new(value.re.to_f64(), value.im.to_f64()));
    for point in [[0., 0.], [0.125, 0.25], [0.5, 0.75]] {
        let point = point.map(|value| Complex::new(value, 0.));
        let mut actual = [Complex::new(0., 0.); 4];
        let mut expected = actual;
        compact.evaluate(&point, &mut actual);
        materialized.evaluate(&point, &mut expected);
        compare(&actual, &expected, 1e-12);
    }

    // Registration still visits nested calls, even though it does not need
    // owned copies of their actual arguments. Neither a bad arity nor a
    // missing nested definition may be hidden by an admitted outer function.
    let inner = calls[1].as_fun_view().unwrap().get_symbol();
    let bad = calls[0]
        .replace(Atom::var(y))
        .with(inner.call_args([Atom::var(x)]));
    assert!(
        definitions
            .function_map([&bad])
            .unwrap_err()
            .contains("arity")
    );
    let outer = calls[0].as_fun_view().unwrap().get_symbol();
    let outer_definition = definitions
        .entries()
        .iter()
        .find(|definition| definition.function() == outer)
        .unwrap();
    let missing = ContourDefinitions::from_parts(vec![(
        outer,
        outer_definition.parameters().to_vec(),
        outer_definition.body().clone(),
    )])
    .unwrap();
    assert!(
        missing
            .function_map([&nested])
            .unwrap_err()
            .contains("lacks its native definition")
    );
}

fn fixture() -> (
    RootProgram,
    ContourDefinitions,
    Vec<Atom>,
    Vec<Atom>,
    Vec<Atom>,
) {
    let (x, y, s, l, r) = symbol!(
        "compact_radius::x",
        "compact_radius::y",
        "compact_radius::s",
        "compact_radius::l",
        "compact_radius::r"
    );
    let input = [x, y, s, l, r].map(Atom::var).to_vec();
    let coefficients = [
        (Atom::one()
            + input[3].pow(2) * (&input[0] * (Atom::one() - &input[0])).pow(2) / input[4].pow(2))
        .expand(),
        (input[3].pow(4) * (Atom::one() + &input[0] + &input[1] + &input[0] * &input[1]).pow(4))
            .expand(),
    ];
    let (definitions, compact) = ContourDefinitions::coefficients(&[x, y], &coefficients).unwrap();
    assert!(!definitions.is_empty());
    let helper = RootProgram::build(2).unwrap();
    let original = dynamic::strength(&helper, &coefficients, &input[2], &input[3]).unwrap();
    let condensed = dynamic::strength(&helper, &compact, &input[2], &input[3]).unwrap();
    let outputs = |strength: Atom| {
        let image = &input[0]
            - Atom::i()
                * strength
                * &input[0]
                * (Atom::one() - &input[0])
                * (Atom::one() + &input[1]);
        vec![
            image.clone(),
            image.derivative(x),
            image.derivative(x).derivative(y),
            image.derivative(x).derivative(x),
        ]
    };
    let physical = outputs(original);
    let compact = outputs(condensed);
    (helper, definitions, input, physical, compact)
}

#[test]
fn compact_radius_maps_preserve_eager_jit_precision_and_native_jets() {
    let (helper, definitions, inputs, physical, compact) = fixture();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    for (a, b) in compact.iter().zip(&physical) {
        assert_eq!(definitions.materialize(a).unwrap(), *b);
    }
    let original = Atom::evaluator_multiple(&physical, &inputs)
        .build()
        .unwrap();
    let lowered = Atom::evaluator_multiple(&compact, &inputs)
        .function_map(definitions.function_map(&compact).unwrap())
        .build()
        .unwrap();
    let mut eager_original = original
        .clone()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut eager = lowered
        .clone()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut dd_original = original
        .clone()
        .map_coeff(&|c| Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im)));
    let mut dd = lowered
        .clone()
        .map_coeff(&|c| Complex::new(DoubleFloat::from(&c.re), DoubleFloat::from(&c.im)));
    let mut mp_original = dynamic::with_precision(192, || {
        original.clone().map_coeff_with_prec(
            &|c| Complex::new(c.re.to_multi_prec_float(192), c.im.to_multi_prec_float(192)),
            192,
        )
    });
    let mut mp = dynamic::with_precision(192, || {
        lowered.clone().map_coeff_with_prec(
            &|c| Complex::new(c.re.to_multi_prec_float(192), c.im.to_multi_prec_float(192)),
            192,
        )
    });
    #[cfg(feature = "native")]
    let mut jit = lowered
        .clone()
        .jit_compile::<Complex<f64>>(
            symbolica::evaluate::JITCompilationSettings::default()
                .optimization_level(2)
                .with_option("use_threads", "false"),
        )
        .unwrap();
    for x in [0., 0.03125, 0.3, 0.8, 1.] {
        let values = [x, 0.4, 0.8, 0.2, 1.];
        let point = values.map(|v| Complex::new(v, 0.));
        let mut expected = vec![Complex::new(0., 0.); 4];
        let mut actual = expected.clone();
        eager_original.evaluate(&point, &mut expected);
        eager.evaluate(&point, &mut actual);
        compare(&actual, &expected, 1e-12);
        #[cfg(feature = "native")]
        {
            jit.evaluate(&point, &mut actual);
            compare(&actual, &expected, 1e-11);
        }
        let point = values.map(|v| Complex::new(DoubleFloat::from(v), DoubleFloat::from(0.)));
        let mut a = vec![Complex::new(DoubleFloat::from(0.), DoubleFloat::from(0.)); 4];
        let mut b = a.clone();
        let (_, original_failure) =
            dynamic::isolated_attempt(|| dd_original.evaluate(&point, &mut a));
        let (_, compact_failure) = dynamic::isolated_attempt(|| dd.evaluate(&point, &mut b));
        if original_failure.is_some() || compact_failure.is_some() {
            // At the exact face, expanded DD arithmetic with binary cap .2
            // rounds a2 below its mathematical bound in BOTH programs. Keep
            // that explicit failure, then reevaluate the complete vector in
            // native MP below; no coefficient clamp or altered tolerance.
            assert_eq!(x, 1.);
            let expected = "dynamic envelope coefficients violate nonnegativity or a2 >= 1";
            for (failure, values) in [(&original_failure, &a), (&compact_failure, &b)] {
                if let Some(failure) = failure {
                    assert_eq!(failure, expected);
                    assert!(
                        values
                            .iter()
                            .all(|value| value.re.to_f64().is_nan() && value.im.to_f64().is_nan())
                    );
                } else {
                    assert!(values.iter().all(
                        |value| value.re.to_f64().is_finite() && value.im.to_f64().is_finite()
                    ));
                }
            }
        } else {
            assert!(original_failure.is_none() && compact_failure.is_none());
            for (index, (a, b)) in a.iter().zip(&b).enumerate() {
                assert!(
                    (a.re - b.re).to_f64().abs() < 1e-28 && (a.im - b.im).to_f64().abs() < 1e-28,
                    "DD x={x} output={index}: expanded={a:?}, compact={b:?}, delta={:?}",
                    ((a.re - b.re).to_f64(), (a.im - b.im).to_f64())
                );
            }
        }
        let point = values.map(|v| Complex::new(Float::with_val(192, v), Float::with_val(192, 0)));
        let mut a = vec![Complex::new(Float::with_val(192, 0), Float::with_val(192, 0)); 4];
        let mut b = a.clone();
        let (_, original_failure) =
            dynamic::isolated_attempt(|| mp_original.evaluate(&point, &mut a));
        let (_, compact_failure) = dynamic::isolated_attempt(|| mp.evaluate(&point, &mut b));
        assert!(original_failure.is_none() && compact_failure.is_none());
        for (index, (a, b)) in a.iter().zip(&b).enumerate() {
            assert!(
                (a.re.clone() - &b.re).to_f64().abs() < 1e-48
                    && (a.im.clone() - &b.im).to_f64().abs() < 1e-48,
                "Float192 x={x} output={index}: expanded={a:?}, compact={b:?}, delta={:?}, callback failure={:?}",
                (
                    (a.re.clone() - &b.re).to_f64(),
                    (a.im.clone() - &b.im).to_f64()
                ),
                dynamic::take_failure()
            );
        }
    }
    // A dyadic cap gives an independently finite DD boundary control through
    // the identical native programs and coefficient expressions.
    let point =
        [1., 0.4, 0.8, 0.25, 1.].map(|v| Complex::new(DoubleFloat::from(v), DoubleFloat::from(0.)));
    let mut a = vec![Complex::new(DoubleFloat::from(0.), DoubleFloat::from(0.)); 4];
    let mut b = a.clone();
    let (_, original_failure) = dynamic::isolated_attempt(|| dd_original.evaluate(&point, &mut a));
    let (_, compact_failure) = dynamic::isolated_attempt(|| dd.evaluate(&point, &mut b));
    assert!(original_failure.is_none() && compact_failure.is_none());
    for (a, b) in a.iter().zip(&b) {
        assert!((a.re - b.re).to_f64().abs() < 1e-28);
        assert!((a.im - b.im).to_f64().abs() < 1e-28);
    }
    let shape = HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1], vec![2]]);
    let dual = Dualizer::new(shape, vec![]);
    let mut a = original
        .vectorize(&dual)
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut b = lowered
        .vectorize(&dual)
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let mut point = Vec::new();
    for (i, v) in [0.2, 0.4, 0.8, 0.2, 1.].into_iter().enumerate() {
        point.extend([
            Complex::new(v, 0.),
            Complex::new(if i == 0 { 1. } else { 0. }, 0.),
            Complex::new(0., 0.),
        ]);
    }
    let mut av = vec![Complex::new(0., 0.); 12];
    let mut bv = av.clone();
    a.evaluate(&point, &mut av);
    b.evaluate(&point, &mut bv);
    compare(&av, &bv, 1e-10);
    assert!(dynamic::take_failure().is_none());
}

#[test]
fn compact_radius_preserves_tracked_complex_uncertainty() {
    let (helper, definitions, inputs, physical, compact) = fixture();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let original = Atom::evaluator_multiple(&physical, &inputs)
        .build()
        .unwrap();
    let lowered = Atom::evaluator_multiple(&compact, &inputs)
        .function_map(definitions.function_map(&compact).unwrap())
        .build()
        .unwrap();
    let map = |c: &Complex<Rational>| {
        Complex::new(
            ErrorPropagatingFloat::new_with_accuracy(c.re.to_f64(), f64::INFINITY),
            ErrorPropagatingFloat::new_with_accuracy(c.im.to_f64(), f64::INFINITY),
        )
    };
    let mut a = original.map_coeff(&map);
    let mut b = lowered.map_coeff(&map);
    let point = [0.2, 0.4, 0.8, 0.2, 1.].map(|v| {
        Complex::new(
            ErrorPropagatingFloat::new_with_accuracy(v, 12.),
            ErrorPropagatingFloat::new_with_accuracy(0., 12.),
        )
    });
    let zero = Complex::new(
        ErrorPropagatingFloat::new_with_accuracy(0., f64::INFINITY),
        ErrorPropagatingFloat::new_with_accuracy(0., f64::INFINITY),
    );
    let mut av = vec![zero; 4];
    let mut bv = av.clone();
    a.evaluate(&point, &mut av);
    b.evaluate(&point, &mut bv);
    for (a, b) in av.iter().zip(&bv) {
        assert!((a.re.to_f64() - b.re.to_f64()).abs() < 1e-12);
        assert!((a.im.to_f64() - b.im.to_f64()).abs() < 1e-12);
        assert!(b.re.get_absolute_error() > 0. && b.im.get_absolute_error() > 0.);
        assert!(
            (a.re.to_f64() - b.re.to_f64()).abs()
                <= a.re.get_absolute_error() + b.re.get_absolute_error()
        );
        assert!(
            (a.im.to_f64() - b.im.to_f64()).abs()
                <= a.im.get_absolute_error() + b.im.get_absolute_error()
        );
    }
    assert!(dynamic::take_failure().is_none());
}

fn compare(a: &[Complex<f64>], b: &[Complex<f64>], tolerance: f64) {
    for (a, b) in a.iter().zip(b) {
        assert!((a.re - b.re).abs() < tolerance * (1. + b.re.abs()));
        assert!((a.im - b.im).abs() < tolerance * (1. + b.im.abs()));
    }
}

#[test]
fn reserved_function_hooks_and_hidden_roots_are_rejected() {
    let (x, y) = symbol!("compact_hygiene::x", "compact_hygiene::y");
    let body = (Atom::num(3719) + Atom::var(x)).pow(17).expand();
    let formal = owned_symbol("fastsecdec::contour::coefficient_argument_0").unwrap();
    let normalized = body.replace(Atom::var(x)).with(Atom::var(formal));
    let name = format!("{PREFIX}{}", identity(&normalized, 1));
    SymbolBuilder::new(wrap_symbol!(&name))
        .with_derivative_function(|_, _, out| **out = Atom::Zero)
        .build()
        .unwrap();
    assert!(
        ContourDefinitions::coefficients(&[x], &[body])
            .unwrap_err()
            .contains("hook")
    );
    let helper = RootProgram::build(1).unwrap();
    let root = dynamic::strength(&helper, &[Atom::one()], &Atom::one(), &Atom::one()).unwrap();
    let name = owned_symbol(&format!("{PREFIX}{}", identity(&root, 1))).unwrap();
    assert!(ContourDefinitions::from_parts(vec![(name, vec![y], root)]).is_err());
    assert!(ContourDefinitions::from_parts(vec![(name, vec![], Atom::one())]).is_err());
}
