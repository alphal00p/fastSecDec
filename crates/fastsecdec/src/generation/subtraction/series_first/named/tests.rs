use super::*;
use symbolica::parse;

#[test]
fn native_named_partials_and_faces_follow_composed_arguments() {
    let x = symbol!("named_requests::x");
    let y = symbol!("named_requests::y");
    let unused = symbol!("named_requests::unused");
    let eps = symbol!("named_requests::eps");
    let body = parse!("1+named_requests::x*named_requests::y^2");
    let terms = [MappedTerm {
        prefactor: Atom::one(),
        regular: body.clone(),
        powers: vec![Atom::Zero; 3],
    }];
    let names = Coefficients::new(&terms, &[x, y, unused], eps);
    let formal = names.wrap(&body);
    assert_eq!(names.state.borrow().bodies.len(), 1);
    assert_eq!(
        names
            .state
            .borrow()
            .bodies
            .values()
            .next()
            .unwrap()
            .parameters,
        [x, y]
    );
    assert_eq!(names.wrap(&body), formal, "equal native bodies deduplicate");
    assert!(names.restore(&formal.derivative(x).derivative(x)).is_zero());
    let image = Atom::var(x) * Atom::var(y);
    let composed = formal
        .replace(Pattern::Literal(Atom::var(y)))
        .with(image.clone());
    let original = body.replace(Pattern::Literal(Atom::var(y))).with(image);
    let actual = names.restore(&composed.derivative(x).derivative(y));
    let expected = original.derivative(x).derivative(y);
    assert!((actual - expected).expand().is_zero());
    let at_face = formal
        .derivative(x)
        .derivative(y)
        .replace(Pattern::Literal(Atom::var(y)))
        .with(Atom::Zero);
    assert!(names.restore(&at_face).is_zero());
    assert!(names.state.borrow().partials.len() >= 3);
}

#[test]
#[should_panic(expected = "formal coefficient used as a scalar multiplier")]
fn native_named_scalar_guard_rejects_a_formal_coefficient() {
    let x = symbol!("named_guard::x");
    let eps = symbol!("named_guard::eps");
    let terms = [MappedTerm {
        prefactor: Atom::one(),
        regular: Atom::var(x) + 1,
        powers: vec![Atom::Zero],
    }];
    let names = Coefficients::new(&terms, &[x], eps);
    let formal = names.wrap(&terms[0].regular);
    names.assert_independent(&formal);
}

#[test]
fn native_named_composition_preserves_explicit_errors() {
    let x = symbol!("named_errors::x");
    let eps = symbol!("named_errors::eps");
    let options = GenerationOptions {
        max_order: 1,
        ..GenerationOptions::default()
    };
    let fractional = MappedTerm {
        prefactor: parse!("named_errors::eps^(1/2)"),
        regular: Atom::one(),
        powers: vec![Atom::Zero],
    };
    assert!(matches!(
        expand_named(&[fractional], &[x], eps, &options),
        Err(GenerationError::FractionalLaurent(_))
    ));
    let essential = MappedTerm {
        prefactor: Atom::one(),
        regular: parse!("exp(1/named_errors::eps)"),
        powers: vec![Atom::Zero],
    };
    assert!(matches!(
        expand_named(&[essential], &[x], eps, &options),
        Err(GenerationError::Series(_))
    ));
    let unregulated = MappedTerm {
        prefactor: Atom::one(),
        regular: Atom::one(),
        powers: vec![Atom::num(-1)],
    };
    assert!(matches!(
        expand_named(&[unregulated], &[x], eps, &options),
        Err(GenerationError::UnregulatedEndpoint { .. })
    ));
}
