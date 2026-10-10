use super::*;
use crate::contour::{
    ContourDefinitions, SmoothContourMap,
    dynamic::{
        DynamicEnvelope, displacement_cap_symbol, lambda_cap_symbol, safety_fraction_symbol,
    },
    functions::dynamic::{self, ProgramScope, RootProgram},
};
use std::sync::Arc;
use symbolica::{
    domains::{
        dual::HyperDual,
        float::{Complex, Float, FloatField, RealLike},
        rational::Rational,
    },
    evaluate::{Dualizer, ExpressionEvaluator},
};

fn equal(a: Atom, b: Atom) {
    assert!((a - b).expand().cancel().is_zero());
}

#[test]
fn structured_gradient_retains_strength_derivatives_and_faces() {
    let parameters = [symbol!("bordered_exact::x"), symbol!("bordered_exact::y")];
    let [x, y] = parameters.map(Atom::var);
    let f = x.pow(2) * &y + &x * y.pow(2) + &x * 3 - &y * 2;
    for strength in [Atom::num((1, 10)), (&x + &y + 1) * Atom::num((1, 10))] {
        let map = SmoothContourMap::new(&parameters, f.clone(), strength.clone()).unwrap();
        let expected = map.metadata().jacobian().clone();
        let actual = real_gradient_jacobian(&parameters, &f, &strength).unwrap();
        equal(actual.clone(), expected.clone());
        let mut a = actual.clone();
        let mut b = expected.clone();
        for axis in [0, 1, 0] {
            a = a.derivative(parameters[axis]);
            b = b.derivative(parameters[axis]);
            equal(a.clone(), b.clone());
        }
        for xx in [0, 1] {
            for yy in [0, 1] {
                let restrict = |v: &Atom| {
                    v.replace(parameters[0])
                        .with(xx)
                        .replace(parameters[1])
                        .with(yy)
                };
                equal(restrict(&actual), restrict(&expected));
                equal(restrict(&a), restrict(&b));
            }
        }
    }
}

#[test]
fn higher_dimensional_gradient_allows_zero_full_jacobian() {
    let parameters = (0..7)
        .map(|i| symbol!(format!("bordered_zero::x{i}")))
        .collect::<Vec<_>>();
    let x = Atom::var(parameters[0]) - Atom::num((1, 2));
    let y = Atom::var(parameters[1]) - Atom::num((1, 2));
    // At the centre v=(0,-2), Dv=[[0,1],[1,0]], lambda=1,
    // grad(lambda)=(1,0). A is invertible but det J is exactly zero.
    let f = Atom::num(4) * &x * &y - Atom::num(8) * &y;
    let strength = &x + 1;
    let map = SmoothContourMap::new(&parameters, f.clone(), strength).unwrap();
    let centre = |a: &Atom| {
        a.replace_multiple(parameters.iter().map(|p| {
            Replacement::new(
                Pattern::Literal(Atom::var(*p)),
                Pattern::Literal(Atom::num((1, 2))),
            )
        }))
    };
    assert!(
        centre(map.metadata().jacobian())
            .expand()
            .cancel()
            .is_zero()
    );
    let entries = map
        .metadata()
        .images()
        .iter()
        .flat_map(|z| parameters.iter().map(|p| centre(&z.derivative(*p))))
        .collect();
    assert!(native(entries, 7).unwrap().is_zero());
    let fixed = crate::contour::FixedContourMap::new(&parameters, f).unwrap();
    equal(
        centre(fixed.metadata().jacobian())
            .replace(crate::contour::lambda_symbol())
            .with(1),
        Atom::num(2),
    );
    // Complex F is inadmissible even when a syntactic determinant exists.
    assert!(
        SmoothContourMap::new(
            &parameters,
            Atom::i() * Atom::var(parameters[0]),
            Atom::one()
        )
        .is_err()
    );
}

#[test]
fn bordered_implicit_strength_preserves_native_higher_jets_on_faces() {
    let parameters = [symbol!("bordered_jets::x"), symbol!("bordered_jets::y")];
    let [x, y] = parameters.map(Atom::var);
    let f = Atom::one() + x.pow(3) + y.pow(3);
    let envelope = DynamicEnvelope::new(&parameters, f.clone(), &[]).unwrap();
    let coefficients = envelope.polynomial_coefficients().unwrap();
    assert!(coefficients.len() > 1);
    let helper = RootProgram::build(coefficients.len()).unwrap();
    let strength = dynamic::strength(
        &helper,
        &coefficients,
        &Atom::var(safety_fraction_symbol()),
        &Atom::var(lambda_cap_symbol()),
    )
    .unwrap();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let map = SmoothContourMap::new(&parameters, f.clone(), strength.clone()).unwrap();
    let outputs = [
        map.metadata().jacobian().clone(),
        real_gradient_jacobian(&parameters, &f, &strength).unwrap(),
    ];
    equal(outputs[0].clone(), outputs[1].clone());
    let inputs = [
        parameters[0],
        parameters[1],
        safety_fraction_symbol(),
        lambda_cap_symbol(),
        displacement_cap_symbol(),
    ]
    .map(Atom::var);
    let shape = HyperDual::<Complex<Rational>>::new(vec![vec![0], vec![1], vec![2], vec![3]]);
    let exact = Atom::evaluator_multiple(&outputs, &inputs)
        .horner_iterations(0)
        .cores(1)
        .build()
        .unwrap();
    let mut jets = exact
        .vectorize(&Dualizer::new(shape, vec![]))
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    for x in [0., 0.25, 1.] {
        let point = [x, 0.5, 0.75, 0.125, 1.]
            .into_iter()
            .enumerate()
            .flat_map(|(i, v)| [v, f64::from(i == 0), 0., 0.].map(|v| Complex::new(v, 0.)))
            .collect::<Vec<_>>();
        let mut values = vec![Complex::new(0., 0.); 8];
        let (_, failure) = dynamic::isolated_attempt(|| jets.evaluate(&point, &mut values));
        assert!(failure.is_none(), "x={x}: {failure:?}");
        for (a, b) in values[..4].iter().zip(&values[4..]) {
            assert!((a.re - b.re).abs() < 1e-9 * (1. + b.re.abs()));
            assert!((a.im - b.im).abs() < 1e-9 * (1. + b.im.abs()));
        }
    }
}

#[test]
fn eight_dimensional_implicit_map_matches_native_matrix_after_restore() {
    let parameters = (0..8)
        .map(|i| symbol!(format!("bordered_implicit::x{i}")))
        .collect::<Vec<_>>();
    let x = parameters
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let f = Atom::one() - Atom::num(2) * &x[0]
        + &x[0] * &x[1]
        + x.iter().enumerate().fold(Atom::Zero, |sum, (i, x)| {
            sum + x.pow(2) * Atom::num((1_i64, 1_i64 << (i + 2)))
        });
    let envelope = DynamicEnvelope::new(&parameters, f.clone(), &[]).unwrap();
    let coefficients = envelope.polynomial_coefficients().unwrap();
    let helper = RootProgram::build(coefficients.len()).unwrap();
    let (definitions, coefficients) =
        ContourDefinitions::coefficients(&parameters, &coefficients).unwrap();
    assert!(!definitions.is_empty());
    let strength = dynamic::strength(
        &helper,
        &coefficients,
        &Atom::var(safety_fraction_symbol()),
        &Atom::var(lambda_cap_symbol()),
    )
    .unwrap();
    let _scope = ProgramScope::new(std::slice::from_ref(&helper)).enter();
    let map = SmoothContourMap::with_definitions(&parameters, f, strength, Arc::new(definitions))
        .unwrap();
    let metadata = map.metadata();
    let mut outputs = vec![metadata.jacobian().clone()];
    // Independent full-J entries differentiate the images before restricting
    // any face; no determinant lemma is used by the numerical reference.
    outputs.extend(
        metadata
            .images()
            .iter()
            .flat_map(|z| parameters.iter().map(|p| z.derivative(*p))),
    );
    let inputs = parameters
        .iter()
        .copied()
        .chain([
            safety_fraction_symbol(),
            lambda_cap_symbol(),
            displacement_cap_symbol(),
        ])
        .map(Atom::var)
        .collect::<Vec<_>>();
    let exact = Atom::evaluator_multiple(&outputs, &inputs)
        .function_map(
            metadata
                .function_definitions()
                .function_map(&outputs)
                .unwrap(),
        )
        .horner_iterations(0)
        .cores(1)
        .build()
        .unwrap();
    let bytes = bincode::serde::encode_to_vec(&exact, bincode::config::standard()).unwrap();
    let (restored, consumed): (ExpressionEvaluator<Complex<Rational>>, _) =
        bincode::serde::borrow_decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(consumed, bytes.len());
    let mut mp = dynamic::with_precision(192, || {
        restored.clone().map_coeff_with_prec(
            &|c| Complex::new(c.re.to_multi_prec_float(192), c.im.to_multi_prec_float(192)),
            192,
        )
    });
    let mut eager = exact.map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    #[cfg(feature = "native")]
    let mut jit = restored
        .jit_compile::<Complex<f64>>(
            symbolica::evaluate::JITCompilationSettings::default()
                .optimization_level(2)
                .with_option("use_threads", "false"),
        )
        .unwrap();
    for case in 0..5 {
        let mut values = (0..8)
            .map(|axis| match case {
                0 => (axis + 1) as f64 / 16.,
                1 => 0.,
                2 => 1.,
                3 => {
                    if axis % 2 == 0 {
                        0.
                    } else {
                        0.5
                    }
                }
                _ => {
                    if axis % 2 == 0 {
                        1.
                    } else {
                        0.5
                    }
                }
            })
            .collect::<Vec<_>>();
        values.extend([0.75, 0.125, 1.]);
        let point = values
            .iter()
            .map(|v| Complex::new(Float::with_val(192, *v), Float::with_val(192, 0)))
            .collect::<Vec<_>>();
        let mut result = vec![Complex::new(Float::with_val(192, 0), Float::with_val(192, 0)); 65];
        let (_, failure) = dynamic::isolated_attempt(|| mp.evaluate(&point, &mut result));
        assert!(failure.is_none(), "case={case}: {failure:?}");
        let reference = Matrix::from_linear(
            result[1..].to_vec(),
            8,
            8,
            FloatField::from_rep(Complex::new(
                Float::with_val(192, 0),
                Float::with_val(192, 0),
            )),
        )
        .unwrap()
        .det()
        .unwrap();
        let delta = &result[0] - &reference;
        assert!(delta.re.to_f64().abs() < 1e-45 && delta.im.to_f64().abs() < 1e-45);
        let point = values
            .iter()
            .map(|v| Complex::new(*v, 0.))
            .collect::<Vec<_>>();
        let mut output = vec![Complex::new(0., 0.); 65];
        let (_, failure) = dynamic::isolated_attempt(|| eager.evaluate(&point, &mut output));
        assert!(failure.is_none(), "eager case={case}: {failure:?}");
        assert!((output[0].re - reference.re.to_f64()).abs() < 1e-10);
        assert!((output[0].im - reference.im.to_f64()).abs() < 1e-10);
        #[cfg(feature = "native")]
        {
            let (_, failure) = dynamic::isolated_attempt(|| jit.evaluate(&point, &mut output));
            assert!(failure.is_none(), "jit case={case}: {failure:?}");
            assert!((output[0].re - reference.re.to_f64()).abs() < 1e-10);
            assert!((output[0].im - reference.im.to_f64()).abs() < 1e-10);
        }
    }
}
