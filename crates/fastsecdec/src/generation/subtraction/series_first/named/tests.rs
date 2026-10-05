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
fn native_interleaved_faces_match_mixed_requests_without_partial_cache_contamination() {
    let x = symbol!("interleaved_control::x");
    let y = symbol!("interleaved_control::y");
    let z = symbol!("interleaved_control::z");
    let eps = symbol!("interleaved_control::eps");
    let parameters = [x, y, z];
    let own = parameters.map(Atom::var);
    for body in [
        parse!(
            "1+interleaved_control::x*interleaved_control::y+interleaved_control::x^2*interleaved_control::y^2+interleaved_control::z"
        ),
        parse!("(2+interleaved_control::x+2*interleaved_control::y+3*interleaved_control::z)^(-2)"),
        parse!("log(2+interleaved_control::x+2*interleaved_control::y+3*interleaved_control::z)"),
    ] {
        let terms = [MappedTerm {
            prefactor: Atom::one(),
            regular: body,
            powers: vec![Atom::Zero; 3],
        }];
        let original = Coefficients::new(&terms, &parameters, eps);
        let candidate =
            Coefficients::with_resolution(&terms, &parameters, eps, Resolution::InterleavedFaces);
        let original_source = original
            .request(original.wrap(&terms[0].regular).as_view())
            .unwrap()
            .0;
        let candidate_source = candidate
            .request(candidate.wrap(&terms[0].regular).as_view())
            .unwrap()
            .0;
        // Reuse the same source and depths under different faces and then an
        // unsubstituted/composed request. Face-dependent cache pollution would
        // alter these later independent native results.
        let cases = [
            (
                vec![1, 1, 0],
                vec![Atom::Zero, Atom::one(), own[2].clone()],
                true,
            ),
            (
                vec![1, 1, 0],
                vec![own[0].clone(), Atom::Zero, Atom::one()],
                true,
            ),
            (
                vec![2, 1, 0],
                vec![Atom::one(), own[1].clone(), Atom::Zero],
                true,
            ),
            (vec![1, 1, 0], own.to_vec(), false),
            (
                vec![1, 1, 0],
                vec![own[1].clone(), own[0].clone(), own[2].clone()],
                false,
            ),
            (
                vec![1, 1, 0],
                vec![&own[0] * &own[1], own[1].clone(), own[2].clone()],
                false,
            ),
            (
                vec![1, 1, 0],
                vec![Atom::num(2), own[1].clone(), own[2].clone()],
                false,
            ),
            (
                vec![1, 1, 0],
                vec![Atom::num(-1), own[1].clone(), own[2].clone()],
                false,
            ),
        ];
        for (index, (depths, arguments, specialized)) in cases.into_iter().enumerate() {
            let before = candidate.state.borrow().interleaved_requests;
            let actual = candidate.resolve((candidate_source, depths.clone(), arguments.clone()));
            let expected = original.resolve((original_source, depths.clone(), arguments.clone()));
            let difference = &actual - &expected;
            assert!(difference.as_view().get_byte_size() < 128_000);
            assert!(difference.together().expand().is_zero());
            assert!(!actual.is_zero(), "nonvacuous mixed partial");
            assert_eq!(
                candidate.state.borrow().interleaved_requests - before,
                usize::from(specialized)
            );
            if index == 0 {
                assert!(candidate.state.borrow().partials.is_empty());
            }
            let cached_before = candidate.state.borrow().faces.len();
            assert_eq!(
                candidate.resolve((candidate_source, depths, arguments)),
                actual
            );
            assert_eq!(candidate.state.borrow().faces.len(), cached_before);
        }
        assert!(candidate.state.borrow().original_requests > 0);
        assert!(candidate.state.borrow().interleaved_requests > 0);
    }
}

#[test]
fn native_interleaved_faces_keep_exact_zero_and_early_zero_depth_faces() {
    let x = symbol!("interleaved_zero::x");
    let y = symbol!("interleaved_zero::y");
    let eps = symbol!("interleaved_zero::eps");
    let terms = [MappedTerm {
        prefactor: Atom::one(),
        regular: parse!("1+interleaved_zero::x*interleaved_zero::y^2"),
        powers: vec![Atom::Zero; 2],
    }];
    let original = Coefficients::new(&terms, &[x, y], eps);
    let candidate =
        Coefficients::with_resolution(&terms, &[x, y], eps, Resolution::InterleavedFaces);
    let old = original
        .request(original.wrap(&terms[0].regular).as_view())
        .unwrap()
        .0;
    let new = candidate
        .request(candidate.wrap(&terms[0].regular).as_view())
        .unwrap()
        .0;
    for (depths, args, expected) in [
        (vec![2, 0], vec![Atom::one(), Atom::var(y)], Atom::Zero),
        (vec![1, 0], vec![Atom::var(x), Atom::Zero], Atom::Zero),
        (vec![1, 1], vec![Atom::Zero, Atom::one()], Atom::num(2)),
    ] {
        assert_eq!(
            candidate.resolve((new, depths.clone(), args.clone())),
            expected
        );
        assert_eq!(original.resolve((old, depths, args)), expected);
    }
    assert_eq!(candidate.state.borrow().interleaved_requests, 3);
    assert!(candidate.state.borrow().partials.is_empty());
}

#[test]
fn native_named_composition_preserves_explicit_errors() {
    let x = symbol!("named_errors::x");
    let eps = symbol!("named_errors::eps");
    let options = GenerationOptions {
        max_order: 1,
        ..GenerationOptions::default()
    };
    for resolution in [Resolution::Original, Resolution::InterleavedFaces] {
        let fractional = MappedTerm {
            prefactor: parse!("named_errors::eps^(1/2)"),
            regular: Atom::one(),
            powers: vec![Atom::Zero],
        };
        assert!(matches!(
            expand_named_with_resolution(&[fractional], &[x], eps, &options, resolution),
            Err(GenerationError::FractionalLaurent(_))
        ));
        let essential = MappedTerm {
            prefactor: Atom::one(),
            regular: parse!("exp(1/named_errors::eps)"),
            powers: vec![Atom::Zero],
        };
        assert!(matches!(
            expand_named_with_resolution(&[essential], &[x], eps, &options, resolution),
            Err(GenerationError::Series(_))
        ));
        let unregulated = MappedTerm {
            prefactor: Atom::one(),
            regular: Atom::one(),
            powers: vec![Atom::num(-1)],
        };
        assert!(matches!(
            expand_named_with_resolution(&[unregulated], &[x], eps, &options, resolution),
            Err(GenerationError::UnregulatedEndpoint { .. })
        ));
    }
}
