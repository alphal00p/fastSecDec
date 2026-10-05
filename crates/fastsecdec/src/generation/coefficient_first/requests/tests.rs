use super::*;
use symbolica::{id::Pattern, parse, poly::series::SeriesDepth, symbol};

fn keep_going(_: RequestProgress) -> ControlFlow<()> {
    ControlFlow::Continue(())
}

fn registry(
    body: &Atom,
    parameters: &[Symbol],
    regulator: Symbol,
    maximum: Option<usize>,
) -> Requests {
    Requests::new(
        &[MappedTerm {
            powers: vec![Atom::Zero; parameters.len()],
            prefactor: Atom::one(),
            regular: body.clone(),
        }],
        parameters,
        regulator,
        RequestLimits {
            max_unique_requests: maximum,
        },
        &mut keep_going,
    )
    .unwrap()
}

#[test]
fn native_series_naming_preserves_bound_dependencies_and_deduplication() {
    let x = symbol!("requests_series::x");
    let y = symbol!("requests_series::y");
    let unused = symbol!("requests_series::unused");
    let eps = symbol!("requests_series::eps");
    let body = Atom::var(x) + Atom::var(y).pow(Atom::num(2));
    let series = (&body * (Atom::one() + Atom::var(eps)))
        .series(eps, 0, SeriesDepth::relative(4))
        .unwrap();
    let mut requests = registry(&body, &[x, y, unused], eps, None);
    let named = requests.wrap_series(&series, &mut keep_going).unwrap();
    assert_eq!(named.absolute_order(), series.absolute_order());
    assert_eq!(requests.counts.source_bodies, 1);
    assert_eq!(requests.bodies.values().next().unwrap().parameters, [x, y]);
    let coefficients = named
        .terms()
        .map(|(order, value)| {
            (
                i32::try_from(order.numerator().to_i64().unwrap()).unwrap(),
                value.clone(),
            )
        })
        .collect();
    let output = requests.lower(coefficients, &mut keep_going).unwrap();
    for (order, coefficient) in output.coefficients {
        assert_eq!(
            coefficient.into_inner(),
            series.coefficient(order.into()).unwrap()
        );
    }
    assert_eq!(output.counts.unique_requests, 1);
}

#[test]
fn interleaved_faces_preserve_native_partials_and_composed_fallbacks() {
    let x = symbol!("requests_faces::x");
    let y = symbol!("requests_faces::y");
    let z = symbol!("requests_faces::z");
    let eps = symbol!("requests_faces::eps");
    let parameters = [x, y, z];
    let own = parameters.map(Atom::var);
    for body in [
        parse!(
            "1+requests_faces::x*requests_faces::y+requests_faces::x^2*requests_faces::y^2+requests_faces::z"
        ),
        parse!("(2+requests_faces::x+2*requests_faces::y+3*requests_faces::z)^(-2)"),
        parse!("log(2+requests_faces::x+2*requests_faces::y+3*requests_faces::z)"),
    ] {
        let mut original = registry(&body, &parameters, eps, None);
        original.original_only = true;
        let mut candidate = registry(&body, &parameters, eps, None);
        let old = original.wrap(&body, &mut keep_going).unwrap();
        let new = candidate.wrap(&body, &mut keep_going).unwrap();
        let old = original.request(old.as_view()).unwrap().unwrap().0;
        let new = candidate.request(new.as_view()).unwrap().unwrap().0;
        for (index, (depths, args, interleaved)) in [
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
        ]
        .into_iter()
        .enumerate()
        {
            let before = candidate.counts.interleaved_requests;
            let key = (new, depths.clone(), args.clone());
            let actual = candidate.resolve(key.clone(), &mut keep_going).unwrap();
            let expected = original
                .resolve((old, depths, args), &mut keep_going)
                .unwrap();
            let difference = &actual - expected;
            assert!(difference.as_view().get_byte_size() < 128_000);
            assert!(difference.together().expand().is_zero());
            assert!(!actual.is_zero());
            assert_eq!(
                candidate.counts.interleaved_requests - before,
                usize::from(interleaved)
            );
            if index == 0 {
                assert!(candidate.partials.is_empty());
            }
            let count = candidate.counts.unique_requests;
            assert_eq!(candidate.resolve(key, &mut keep_going).unwrap(), actual);
            assert_eq!(candidate.counts.unique_requests, count);
        }
        assert!(
            candidate.counts.interleaved_requests > 0 && candidate.counts.fallback_requests > 0
        );
    }
}

#[test]
fn lowering_preserves_exact_zero_early_faces_and_composed_derivatives() {
    let x = symbol!("requests_lower::x");
    let y = symbol!("requests_lower::y");
    let eps = symbol!("requests_lower::eps");
    let body = Atom::one() + Atom::var(x) * Atom::var(y).pow(Atom::num(2));
    let mut requests = registry(&body, &[x, y], eps, None);
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    let face = formal
        .derivative(x)
        .replace(Pattern::Literal(Atom::var(y)))
        .with(Atom::Zero);
    let composed = formal
        .replace(Pattern::Literal(Atom::var(y)))
        .with(Atom::var(x) * Atom::var(y))
        .derivative(x)
        .derivative(y);
    let expected = body
        .replace(Pattern::Literal(Atom::var(y)))
        .with(Atom::var(x) * Atom::var(y))
        .derivative(x)
        .derivative(y);
    let result = requests
        .lower(
            BTreeMap::from([(-3, Atom::Zero), (-1, face), (0, composed)]),
            &mut keep_going,
        )
        .unwrap();
    assert_eq!(
        result.coefficients.keys().copied().collect::<Vec<_>>(),
        [-3, -1, 0]
    );
    assert!(result.coefficients[&-3].clone().into_inner().is_zero());
    assert!(result.coefficients[&-1].clone().into_inner().is_zero());
    assert!(
        (result.coefficients[&0].clone().into_inner() - expected)
            .expand()
            .is_zero()
    );
    let aliases = result.coefficients[&0].get_aliases();
    assert!(
        result
            .coefficients
            .values()
            .all(|coefficient| coefficient.get_aliases() == aliases)
    );
}

#[test]
fn unique_request_budget_counts_cache_keys_and_keeps_zero_outputs() {
    let x = symbol!("requests_budget::x");
    let eps = symbol!("requests_budget::eps");
    let body = Atom::var(x) + 1;
    let mut requests = registry(&body, &[x], eps, Some(1));
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    let result = requests
        .lower(
            BTreeMap::from([(-1, formal.clone()), (0, formal * 2)]),
            &mut keep_going,
        )
        .unwrap();
    assert_eq!(result.counts.unique_requests, 1);
    assert_eq!(result.counts.aliases, 1);
    let mut requests = registry(&body, &[x], eps, Some(0));
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    let result = requests
        .lower(
            BTreeMap::from([(-2, &formal - &formal), (0, Atom::num(7))]),
            &mut keep_going,
        )
        .unwrap();
    assert_eq!(result.counts.unique_requests, 0);
    assert!(result.coefficients[&-2].get_root().is_zero());
    let mut requests = registry(&body, &[x], eps, Some(1));
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    let face = formal
        .replace(Pattern::Literal(Atom::var(x)))
        .with(Atom::Zero);
    let result = requests.lower(BTreeMap::from([(-1, formal), (0, face)]), &mut keep_going);
    assert!(matches!(
        result,
        Err(GenerationError::ResourceLimit("named unique requests"))
    ));
    assert!(matches!(
        next_count(usize::MAX, None, "counter"),
        Err(GenerationError::ResourceLimit("counter"))
    ));
}

#[test]
fn malformed_owned_requests_and_symbol_counter_overflow_are_errors() {
    let x = symbol!("requests_errors::x");
    let eps = symbol!("requests_errors::eps");
    let body = Atom::var(x) + 1;
    let mut requests = registry(&body, &[x], eps, None);
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    assert!(matches!(
        requests.check_scalar(&formal),
        Err(GenerationError::Invariant(_))
    ));
    let source = requests.request(formal.as_view()).unwrap().unwrap().0;
    assert!(matches!(
        requests.request(source.call((x, x)).as_view()),
        Err(GenerationError::Invariant(_))
    ));
    for invalid in [Atom::num(-1), parse!("1/2"), Atom::var(x)] {
        assert!(matches!(
            Requests::depth(invalid.as_view()),
            Err(GenerationError::Invariant(_))
        ));
    }
    assert!(matches!(
        Requests::depth(parse!("18446744073709551616").as_view()),
        Err(GenerationError::ResourceLimit(_))
    ));
    assert!(matches!(
        requests.lower(BTreeMap::from([(0, Atom::var(source))]), &mut keep_going),
        Err(GenerationError::Invariant(_))
    ));
    let mut requests = registry(&body, &[x], eps, None);
    requests.next_body = usize::MAX;
    assert!(matches!(
        requests.wrap(&body, &mut keep_going),
        Err(GenerationError::ResourceLimit("named symbol counter"))
    ));
}

#[test]
fn native_callback_latches_preserve_first_cancellation_without_partial_output() {
    let x = symbol!("requests_cancel::x");
    let eps = symbol!("requests_cancel::eps");
    let body = Atom::var(x) + 1;
    let series = (&body * (Atom::one() + Atom::var(eps)))
        .series(eps, 0, SeriesDepth::relative(3))
        .unwrap();
    let mut requests = registry(&body, &[x], eps, None);
    let mut calls = 0;
    let result = requests.wrap_series(&series, &mut |_| {
        calls += 1;
        ControlFlow::Break(())
    });
    assert!(matches!(result, Err(GenerationError::Cancelled)));
    assert_eq!(calls, 1, "no callback after the first native-map error");
    drop(requests);
    let mut requests = registry(&body, &[x], eps, None);
    let first = requests.wrap(&body, &mut keep_going).unwrap();
    let second = requests
        .wrap(&(Atom::var(x).pow(Atom::num(2)) + 1), &mut keep_going)
        .unwrap();
    let mut calls = 0;
    let result = requests.lower(BTreeMap::from([(0, first + second)]), &mut |_| {
        calls += 1;
        if calls == 2 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    assert!(
        matches!(result, Err(GenerationError::Cancelled)),
        "unresolved-name checks must not replace the first error"
    );
    assert_eq!(calls, 2);
    let mut requests = registry(&body, &[x], eps, None);
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    let source = requests.request(formal.as_view()).unwrap().unwrap().0;
    let mut calls = 0;
    let result = requests.resolve((source, vec![3], vec![Atom::var(x)]), &mut |_| {
        calls += 1;
        if calls == 4 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    assert!(matches!(result, Err(GenerationError::Cancelled)));
    assert!(requests.faces.is_empty() && requests.partials.is_empty());
    drop(requests);
}

#[test]
fn allocator_rejects_foreign_metadata_and_skips_declared_input_handles() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    // Fixed high indices isolate hostile test fixtures from ordinary f0/a0
    // reuse. Production counters and namespaces remain unchanged.
    let foreign = SymbolBuilder::new(wrap_symbol!("fastsecdec::named_regular::f9000000"))
        .with_attributes(&[SymbolAttribute::Real])
        .build()
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    SymbolBuilder::new(wrap_symbol!("fastsecdec::named_regular::a9000000"))
        .with_normalization_function(move |_, _| {
            observed.fetch_add(1, Ordering::SeqCst);
        })
        .build()
        .unwrap();
    let x = symbol!("requests_hygiene::x");
    let eps = symbol!("requests_hygiene::eps");
    let body = Atom::var(x) + 1;
    let mut requests = registry(&body, &[x], eps, None);
    requests.next_body = 9_000_000;
    assert!(matches!(
        requests.wrap(&body, &mut keep_going),
        Err(GenerationError::ReservedSymbolConflict { .. })
    ));
    let mut requests = registry(&body, &[x, foreign], eps, None);
    requests.next_body = 9_000_000;
    let formal = requests.wrap(&body, &mut keep_going).unwrap();
    assert_ne!(
        requests.request(formal.as_view()).unwrap().unwrap().0,
        foreign
    );
    requests.next_alias = 9_000_000;
    assert!(matches!(
        requests.lower(BTreeMap::from([(0, formal)]), &mut keep_going),
        Err(GenerationError::ReservedSymbolConflict { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn repeated_jobs_reuse_names_and_live_vectors_keep_local_bodies() {
    let x = symbol!("requests_lifetime::x");
    let eps = symbol!("requests_lifetime::eps");
    let job = |offset: i64| {
        let body = Atom::var(x) + Atom::num(offset);
        let mut requests = registry(&body, &[x], eps, None);
        let formal = requests.wrap(&body, &mut keep_going).unwrap();
        let source = requests.request(formal.as_view()).unwrap().unwrap().0;
        let lowered = requests
            .lower(BTreeMap::from([(0, formal)]), &mut keep_going)
            .unwrap();
        (source, lowered)
    };
    let (first_source, first) = job(1);
    let (second_source, second) = job(10);
    assert_eq!(first_source, second_source);
    assert_eq!(
        first.coefficients[&0].get_root(),
        second.coefficients[&0].get_root()
    );
    assert_ne!(
        first.coefficients[&0].get_aliases(),
        second.coefficients[&0].get_aliases()
    );
    assert_eq!(
        first.coefficients[&0].clone().into_inner(),
        Atom::var(x) + 1
    );
    assert_eq!(
        second.coefficients[&0].clone().into_inner(),
        Atom::var(x) + 10
    );
    for _ in 0..16 {
        let (source, repeated) = job(1);
        assert_eq!(source, first_source);
        assert_eq!(repeated.counts, first.counts);
        assert_eq!(
            first.coefficients[&0].get_aliases(),
            repeated.coefficients[&0].get_aliases()
        );
        assert_eq!(
            first.coefficients[&0].get_root(),
            repeated.coefficients[&0].get_root()
        );
    }
    drop(second);
    assert_eq!(
        first.coefficients[&0].clone().into_inner(),
        Atom::var(x) + 1
    );
}
