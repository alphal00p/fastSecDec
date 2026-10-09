use super::*;
use std::collections::HashMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, integer::Integer},
    id::{Pattern, Replacement},
    parse, symbol,
};

fn value(expression: &Atom, inputs: &[(Symbol, f64)]) -> f64 {
    let inputs = inputs
        .iter()
        .map(|(s, x)| (Atom::var(*s), *x))
        .collect::<HashMap<_, _>>();
    expression.evaluate(&inputs).unwrap()
}
fn substitute(expression: &Atom, inputs: &[(Symbol, Atom)]) -> Atom {
    expression.replace_multiple(inputs.iter().map(|(s, v)| {
        Replacement::new(Pattern::Literal(Atom::var(*s)), Pattern::Literal(v.clone()))
    }))
}
fn settings() -> [(Symbol, f64); 2] {
    [(lambda_cap_symbol(), 0.7), (displacement_cap_symbol(), 1.2)]
}

#[test]
fn native_hessian_ray_matches_independent_full_ray_without_v_differentiation() {
    let x = symbol!("dynamic_test::x");
    let y = symbol!("dynamic_test::y");
    let t = symbol!("dynamic_test::t");
    let f = parse!(
        "1-4*dynamic_test::x*dynamic_test::y+(dynamic_test::x+dynamic_test::y)^3+dynamic_test::x^3*dynamic_test::y^2"
    );
    let envelope = DynamicEnvelope::new(&[x, y], f.clone(), &[]).unwrap();
    assert_eq!(envelope.causal_terms().len(), 2);
    let full_ray = substitute(
        &f,
        &[
            (x, Atom::var(x) + Atom::var(t) * &envelope.direction()[0]),
            (y, Atom::var(y) + Atom::var(t) * &envelope.direction()[1]),
        ],
    )
    .series(t, 0, 5)
    .unwrap();
    for term in envelope.causal_terms() {
        let contraction = (0..2)
            .flat_map(|i| (0..2).map(move |j| (i, j)))
            .map(|(i, j)| {
                &envelope.direction()[i] * &term.hessian_ray()[i * 2 + j] * &envelope.direction()[j]
            })
            .sum::<Atom>();
        let exact = Atom::num(Integer::factorial(term.order()))
            * full_ray
                .coefficient(i64::from(term.order()).into())
                .unwrap();
        assert!((contraction - exact).expand().is_zero());
    }
    let derivative = envelope.causal_terms()[0].squared_bound().derivative(x);
    assert!(
        !derivative.is_zero(),
        "physical derivatives must retain v(x)"
    );
}

#[test]
fn positive_u_certificate_is_exact_sufficient_and_never_floored() {
    let x = symbol!("dynamic_u::x");
    let f = parse!("1-3*dynamic_u::x");
    let envelope = DynamicEnvelope::new(
        &[x],
        f.clone(),
        &[parse!("2+3*dynamic_u::x+dynamic_u::x^2")],
    )
    .unwrap();
    assert_eq!(
        envelope.positive_factors()[0].certified_lower_bound(),
        &Atom::num(2)
    );
    assert_eq!(envelope.positive_factors()[0].terms().len(), 1);
    let even = &envelope.positive_factors()[0].terms()[0];
    assert!(
        (even.ray_coefficient() + envelope.direction()[0].pow(2))
            .expand()
            .is_zero()
    );
    for unsupported in [
        parse!("dynamic_u::x"),
        parse!("1-dynamic_u::x/2"),
        parse!("dynamic_u::mass+dynamic_u::x"),
        parse!("-1-dynamic_u::x"),
        parse!("2^(1/2)+dynamic_u::x"),
    ] {
        assert!(DynamicEnvelope::new(&[x], f.clone(), &[unsupported]).is_err());
    }
    // The signed-coefficient example above is positive, but this narrow
    // certificate deliberately does not pretend to establish that fact.
    let constant = DynamicEnvelope::new(&[], Atom::num(-2), &[Atom::num(3)]).unwrap();
    assert!(constant.direction().is_empty());
    assert_eq!(
        constant.polynomial_level(),
        &Atom::var(radius_fraction_symbol()).pow(2)
    );
}

#[test]
fn native_low_degree_specializations_are_available_and_exact() {
    let x = symbol!("dynamic_low_degree::x");
    let fraction = Atom::var(radius_fraction_symbol());
    let quadratic = DynamicEnvelope::new(
        &[x],
        parse!("1-5*dynamic_low_degree::x*(1-dynamic_low_degree::x)"),
        &[Atom::one() + Atom::var(x)],
    )
    .unwrap();
    assert_eq!(quadratic.polynomial_exponents(), vec![2]);
    let a = quadratic
        .polynomial_level()
        .replace(radius_fraction_symbol())
        .with(1);
    let root = Atom::one() / a.sqrt();
    assert!(
        (quadratic
            .polynomial_level()
            .replace(radius_fraction_symbol())
            .with(root)
            - 1)
        .cancel()
        .is_zero()
    );

    let quartic = DynamicEnvelope::new(
        &[x],
        parse!("1-3*dynamic_low_degree::x+dynamic_low_degree::x^3"),
        &[Atom::one() + Atom::var(x).pow(2)],
    )
    .unwrap();
    assert_eq!(quartic.polynomial_exponents(), vec![2, 4]);
    let polynomial = quartic
        .polynomial_level()
        .to_polynomial_in_vars::<u32>([fraction]);
    let a = polynomial.coefficient(&[2]).unwrap();
    let b = polynomial.coefficient(&[4]).unwrap();
    let closed_root = (Atom::num(2) / (&a + (a.pow(2) + Atom::num(4) * &b).sqrt())).sqrt();
    for coordinate in [0.0, 0.2, 0.5, 0.8, 1.0] {
        let inputs = [(x, coordinate), settings()[0], settings()[1]];
        let radius = value(&closed_root, &inputs);
        let inputs = [
            inputs[0],
            inputs[1],
            inputs[2],
            (radius_fraction_symbol(), radius),
        ];
        assert!((value(quartic.polynomial_level(), &inputs) - 1.0).abs() < 1e-12);
    }
    // Prove the specialization for independent coefficients, without asking
    // the CAS to expand the potentially large physical coefficient bodies.
    let a = Atom::var(symbol!("dynamic_low_degree::a"));
    let b = Atom::var(symbol!("dynamic_low_degree::b"));
    let s = symbol!("dynamic_low_degree::sqrt_discriminant");
    let root_squared = Atom::num(2) / (&a + Atom::var(s));
    let equation = (&a * &root_squared + &b * root_squared.pow(2) - 1).cancel();
    let residual = (equation * (&a + Atom::var(s)).pow(2))
        .together()
        .cancel()
        .expand();
    assert!(
        residual
            .replace(Atom::var(s).pow(2))
            .with(a.pow(2) + Atom::num(4) * b)
            .expand()
            .is_zero(),
        "native specialization residual: {residual}"
    );
    assert_eq!(
        quartic.strength(),
        Atom::var(safety_fraction_symbol())
            * Atom::var(lambda_cap_symbol())
            * Atom::var(radius_fraction_symbol())
    );
}

#[test]
fn dense_native_coefficients_keep_schema_and_reconstruct_both_levels() {
    let x = symbol!("dynamic_coefficients::x");
    let envelope = DynamicEnvelope::new(
        &[x],
        parse!("1+dynamic_coefficients::x^3+dynamic_coefficients::x^5"),
        &[parse!("1+dynamic_coefficients::x^4")],
    )
    .unwrap();
    assert_eq!(envelope.maximum_even_order(), 8);
    let polynomial = envelope.polynomial_coefficients().unwrap();
    let sign_aware = envelope.sign_aware_coefficients().unwrap();
    assert_eq!(polynomial.len(), 4);
    assert_eq!(sign_aware.len(), 4);
    assert!(polynomial[2].is_zero());
    for (coefficients, level) in [
        (&polynomial, envelope.polynomial_level()),
        (&sign_aware, envelope.sign_aware_level()),
    ] {
        let reconstructed = coefficients
            .iter()
            .enumerate()
            .map(|(index, coefficient)| {
                coefficient * Atom::var(radius_fraction_symbol()).pow((2 * index + 2) as i64)
            })
            .sum::<Atom>();
        let residual = (reconstructed - level).expand().together().cancel();
        assert!(residual.is_zero(), "residual: {residual}");
        assert!(
            coefficients
                .iter()
                .all(|coefficient| !coefficient.contains_symbol(radius_fraction_symbol()))
        );
        // Restriction acts on the existing coefficient vector, not its shape.
        let face = coefficients
            .iter()
            .map(|coefficient| coefficient.replace(x).with(0))
            .collect::<Vec<_>>();
        assert_eq!(face.len(), 4);
    }
    assert_eq!(polynomial[0].replace(x).with(0), Atom::one());
    assert!(
        polynomial[1..]
            .iter()
            .all(|coefficient| coefficient.replace(x).with(0).is_zero())
    );
    let mut aliases = Vec::new();
    let lowered = envelope
        .sign_aware_coefficients_with(|part| {
            let alias = symbol!(&format!("dynamic_coefficients::positive_{}", aliases.len()));
            aliases.push((alias, part.expression().clone()));
            Atom::var(alias)
        })
        .unwrap();
    assert_eq!(aliases.len(), 4);
    assert_eq!(lowered.len(), sign_aware.len());
    for (coefficient, expected) in lowered.iter().zip(&sign_aware) {
        assert!(
            (substitute(coefficient, &aliases) - expected)
                .expand()
                .together()
                .cancel()
                .is_zero()
        );
    }
}

#[test]
fn both_symbolic_bounds_control_complete_homotopy_and_positive_factors() {
    let x = symbol!("dynamic_bound::x");
    let y = symbol!("dynamic_bound::y");
    let f = parse!(
        "1-6*dynamic_bound::x+3*dynamic_bound::y+(dynamic_bound::x+dynamic_bound::y)^3+2*dynamic_bound::x^3*dynamic_bound::y^2"
    );
    let u = parse!(
        "1+dynamic_bound::x+dynamic_bound::y+dynamic_bound::x*dynamic_bound::y+(dynamic_bound::x+dynamic_bound::y)^4"
    );
    let envelope = DynamicEnvelope::new(&[x, y], f.clone(), std::slice::from_ref(&u)).unwrap();
    for px in [0.0, 0.2, 0.5, 0.8, 1.0] {
        for py in [0.25, 0.7] {
            let bindings = [(x, px), (y, py), settings()[0], settings()[1]];
            let v = envelope
                .direction()
                .iter()
                .map(|d| value(d, &bindings))
                .collect::<Vec<_>>();
            let a = value(envelope.leading_causal_magnitude(), &bindings);
            for level in [envelope.polynomial_level(), envelope.sign_aware_level()] {
                let bound = substitute(
                    level,
                    &bindings
                        .iter()
                        .map(|(s, p)| (*s, Atom::num(*p)))
                        .collect::<Vec<_>>(),
                );
                // Native Symbolica root solving is an independent test oracle.
                // Production requires the prepared owner API; no solver here.
                let root = (bound.clone() - 1)
                    .nsolve(radius_fraction_symbol(), 0.25, 1e-13, 100)
                    .unwrap();
                assert!(root > 0.0 && root <= 1.0 + 1e-12);
                assert!((value(&bound, &[(radius_fraction_symbol(), root)]) - 1.0).abs() < 2e-10);
                for fraction in [0.01, 0.2, 0.5, 0.8] {
                    let lambda = fraction * 0.7 * root;
                    let point = HashMap::from([
                        (Atom::var(x), Complex::new(px, -lambda * v[0])),
                        (Atom::var(y), Complex::new(py, -lambda * v[1])),
                    ]);
                    let fz = f.evaluate(&point).unwrap();
                    let uz = u.evaluate(&point).unwrap();
                    assert!(a > 0.0 && fz.im < 0.0, "F={fz:?}, A={a}, lambda={lambda}");
                    assert!(uz.re > 0.0, "U={uz:?}, lambda={lambda}");
                    let displacement = lambda * v.iter().map(|q| q * q).sum::<f64>().sqrt();
                    assert!(displacement <= fraction * 1.2 * (1.0 + 1e-12));
                }
            }
        }
    }
}

#[test]
fn stationary_points_and_faces_restrict_the_same_full_sector_function() {
    let x = symbol!("dynamic_stationary::x");
    let y = symbol!("dynamic_stationary::y");
    let f = parse!("-1+(dynamic_stationary::x-1/2)^2+(dynamic_stationary::y-1/2)^4");
    let envelope = DynamicEnvelope::new(&[x, y], f, &[]).unwrap();
    let center = [
        (x, 0.5),
        (y, 0.5),
        settings()[0],
        settings()[1],
        (radius_fraction_symbol(), 1.0),
    ];
    assert_eq!(value(envelope.leading_causal_magnitude(), &center), 0.0);
    assert_eq!(value(envelope.polynomial_level(), &center), 1.0);
    assert!(value(envelope.sign_aware_level(), &center).is_finite());
    for level in [envelope.polynomial_level(), envelope.sign_aware_level()] {
        assert!(value(&level.derivative(x), &center).is_finite());
        assert!(value(&level.derivative(y).derivative(y), &center).is_finite());
    }
    let faces = [(x, Atom::Zero), (y, parse!("1/3"))];
    let restricted = substitute(envelope.sign_aware_level(), &faces);
    assert_eq!(envelope.parameters().len(), 2);
    assert_eq!(envelope.causal_terms().len(), 1);
    assert!(
        restricted
            .derivative(radius_fraction_symbol())
            .contains_symbol(lambda_cap_symbol())
    );
    // Even where a direction vanishes, its physical derivative need not.
    assert_ne!(value(&envelope.direction()[0].derivative(x), &center), 0.0);
    let unresolved =
        DynamicEnvelope::new(&[x], (Atom::var(x) - Atom::num((1, 2))).pow(2), &[]).unwrap();
    assert_eq!(value(unresolved.causal_polynomial(), &center), 0.0);
    assert_eq!(value(unresolved.leading_causal_magnitude(), &center), 0.0);
    assert_eq!(value(unresolved.polynomial_level(), &center), 1.0);
    // The radius equation stays smooth, but it does not certify avoidance of
    // a stationary F zero. Runtime admission must diagnose this separately.
}

#[test]
fn spectral_sum_of_squares_matches_native_trace_identity_and_is_smooth() {
    let x = symbol!("dynamic_spectral::x");
    let y = symbol!("dynamic_spectral::y");
    let z = symbol!("dynamic_spectral::z");
    let f = parse!("1+(dynamic_spectral::x+2*dynamic_spectral::y-dynamic_spectral::z)^3");
    let envelope = DynamicEnvelope::new(&[x, y, z], f, &[]).unwrap();
    let term = &envelope.causal_terms()[0];
    let scale = Atom::var(lambda_cap_symbol()).pow(2) / Atom::num(6);
    let trace = Atom::num(3) * term.spectral_mean();
    let norm = (0..3)
        .flat_map(|i| (0..3).map(move |j| (i, j)))
        .map(|(i, j)| {
            scale.pow(2)
                * &envelope.weights()[i]
                * &envelope.weights()[j]
                * term.hessian_ray()[i * 3 + j].pow(2)
        })
        .sum::<Atom>();
    let trace_identity = Atom::num((2, 3)) * (norm - trace.pow(2) / Atom::num(3));
    assert!(
        (term.spectral_gap_squared() - trace_identity)
            .expand()
            .is_zero()
    );
    let corner = [(x, 0.0), (y, 0.0), (z, 0.0), settings()[0], settings()[1]];
    assert_eq!(value(term.spectral_gap_squared(), &corner), 0.0);
    assert!(
        value(
            &term
                .positive_bound()
                .expression()
                .derivative(x)
                .derivative(y),
            &corner
        )
        .is_finite()
    );
    let bound = term.positive_bound();
    let at_negative = [(x, 0.1), (y, 0.4), (z, 0.9), settings()[0], settings()[1]];
    let direct = value(bound.expression(), &at_negative);
    let rationalized = value(bound.negative_argument_expression(), &at_negative);
    assert!((direct - rationalized).abs() < 2e-8 * direct.max(1.0));
}

#[test]
fn reject_nonpolynomial_complex_and_reserved_inputs_without_silent_fallback() {
    let x = symbol!("dynamic_admission::x");
    let y = symbol!("dynamic_admission::y");
    assert!(DynamicEnvelope::new(&[x, y], Atom::var(x) / Atom::var(y), &[]).is_err());
    for f in [
        parse!("1/(1+dynamic_admission::x)"),
        Atom::one() + Atom::i() * Atom::var(x),
        Atom::var(lambda_cap_symbol()) + Atom::var(x),
    ] {
        assert!(DynamicEnvelope::new(&[x], f, &[]).is_err());
    }
    assert!(DynamicEnvelope::new(&[x, x], Atom::one() + Atom::var(x), &[]).is_err());
    assert!(DynamicEnvelope::new(&[radius_fraction_symbol()], Atom::one(), &[]).is_err());
}
