//! Closed-form controls for the shared map; no dynamic artifact/runtime path.
use super::*;
use crate::contour::SmoothContourMap;
use std::collections::HashMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::Complex,
    parse, symbol,
};

fn strength(envelope: &DynamicEnvelope, sign_aware: bool) -> Atom {
    let coefficients = if sign_aware {
        envelope.sign_aware_coefficients().unwrap()
    } else {
        envelope.polynomial_coefficients().unwrap()
    };
    let root = match coefficients.as_slice() {
        [a] => Atom::one() / a.sqrt(),
        [a, b] => (Atom::num(2) / (a + (a.pow(2) + Atom::num(4) * b).sqrt())).sqrt(),
        _ => panic!("control needs a native-verified quadratic/quartic level"),
    };
    Atom::var(safety_fraction_symbol()) * Atom::var(lambda_cap_symbol()) * root
}

fn value(expression: &Atom, point: &[(Symbol, f64)]) -> Complex<f64> {
    expression
        .evaluate(
            &point
                .iter()
                .map(|(symbol, value)| (Atom::var(*symbol), Complex::new(*value, 0.0)))
                .collect::<HashMap<_, _>>(),
        )
        .unwrap()
}

fn settings() -> [(Symbol, f64); 3] {
    [
        (safety_fraction_symbol(), 0.6),
        (lambda_cap_symbol(), 0.4),
        (displacement_cap_symbol(), 0.3),
    ]
}

#[test]
fn local_strength_jacobian_matches_independent_image_differences() {
    let x = symbol!("local_map::x");
    let y = symbol!("local_map::y");
    let f = parse!("-1+local_map::x+2*local_map::y+local_map::x*local_map::y");
    let envelope = DynamicEnvelope::new(&[x, y], f.clone(), &[]).unwrap();
    let strength = strength(&envelope, false);
    let map = SmoothContourMap::new(&[x, y], f, strength.clone()).unwrap();
    let point = [
        (x, 0.31),
        (y, 0.63),
        settings()[0],
        settings()[1],
        settings()[2],
    ];
    let mut derivative = [[Complex::new(0., 0.); 2]; 2];
    for axis in 0..2 {
        let mut plus = point;
        let mut minus = point;
        plus[axis].1 += 1e-6;
        minus[axis].1 -= 1e-6;
        for (row, image) in map.metadata().images().iter().enumerate() {
            derivative[row][axis] = (value(image, &plus) - value(image, &minus)) / 2e-6;
        }
    }
    let independent = derivative[0][0] * derivative[1][1] - derivative[0][1] * derivative[1][0];
    let actual = value(map.metadata().jacobian(), &point);
    let delta = actual - independent;
    assert!(delta.re.hypot(delta.im) < 2e-8);
    // Missing grad(lambda) is detectably different, not another valid map.
    let v = envelope.direction();
    let i = Atom::i();
    let frozen = (Atom::one() - &i * &strength * v[0].derivative(x))
        * (Atom::one() - &i * &strength * v[1].derivative(y))
        + strength.pow(2) * v[0].derivative(y) * v[1].derivative(x);
    let delta = actual - value(&frozen, &point);
    assert!(delta.re.hypot(delta.im) > 1e-3);
    for (axis, coordinate) in [x, y].into_iter().enumerate() {
        assert!(
            map.metadata().images()[axis]
                .replace(coordinate)
                .with(0)
                .is_zero()
        );
        assert_eq!(
            map.metadata().images()[axis].replace(coordinate).with(1),
            Atom::one()
        );
    }
}

#[test]
fn quartic_strength_restricts_the_full_sector_function_on_a_face() {
    let x = symbol!("local_face::x");
    let y = symbol!("local_face::y");
    let f = parse!("1+local_face::x^3+local_face::y^3");
    let envelope = DynamicEnvelope::new(&[x, y], f.clone(), &[]).unwrap();
    let lambda = strength(&envelope, true);
    let map = SmoothContourMap::new(&[x, y], f.clone(), lambda.clone()).unwrap();
    let reduced = DynamicEnvelope::new(&[y], f.replace(x).with(0), &[]).unwrap();
    let incorrectly_rebuilt = strength(&reduced, true);
    let point = [
        (x, 0.0),
        (y, 0.7),
        settings()[0],
        settings()[1],
        settings()[2],
    ];
    let on_face = lambda.replace(x).with(0);
    let delta = value(&on_face, &point) - value(&incorrectly_rebuilt, &point);
    assert!(delta.re.hypot(delta.im) > 1e-8);
    assert_eq!(envelope.parameters().len(), 2);
    assert_eq!(map.metadata().images()[0].replace(x).with(0), Atom::Zero);
    assert!(
        !lambda
            .derivative(y)
            .derivative(y)
            .replace(x)
            .with(0)
            .is_zero()
    );
}

#[test]
fn stationary_causal_map_can_have_a_large_jacobian_without_displacement() {
    let x = symbol!("local_stationary::x");
    let k = symbol!("local_stationary::k");
    let f = parse!("-1+local_stationary::k*(local_stationary::x-1/2)^2");
    let envelope = DynamicEnvelope::new(&[x], f.clone(), &[]).unwrap();
    let lambda = strength(&envelope, false);
    let map = SmoothContourMap::new(&[x], f, lambda.clone()).unwrap();
    let center = Atom::num((1, 2));
    let expected = Atom::one()
        - Atom::i()
            * Atom::var(safety_fraction_symbol())
            * Atom::var(lambda_cap_symbol())
            * Atom::var(k)
            / Atom::num(2);
    assert!(
        (map.metadata().jacobian().replace(x).with(center.clone()) - expected)
            .expand()
            .is_zero()
    );
    assert_eq!(
        map.metadata().images()[0].replace(x).with(center.clone()),
        center
    );
    let point = [
        (x, 0.5),
        (k, 1e6),
        (safety_fraction_symbol(), 0.8),
        (lambda_cap_symbol(), 1.),
        (displacement_cap_symbol(), 1.),
    ];
    let jacobian = value(map.metadata().jacobian(), &point);
    assert_eq!(jacobian, Complex::new(1., -4e5));
    assert_eq!(value(&lambda, &point), Complex::new(0.8, 0.));
}
