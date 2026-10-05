use super::*;
use crate::generation::SubtractionStrategy;
use std::collections::BTreeSet;
use symbolica::{parse, symbol};

fn term(prefactor: Atom, regular: Atom, powers: Vec<Atom>) -> MappedTerm {
    MappedTerm {
        prefactor,
        regular,
        powers,
    }
}

fn limits() -> Limits {
    Limits {
        max_attempts: Some(8),
        max_relative_width: Some(64),
        requests: RequestLimits {
            max_unique_requests: Some(4096),
        },
    }
}

fn run(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    caps: Limits,
) -> Result<Completed, GenerationError> {
    expand(
        terms,
        parameters,
        regulator,
        options,
        caps,
        &mut TemplateCache::default(),
        &mut |_| ControlFlow::Continue(()),
    )
}

fn compare(
    terms: Vec<MappedTerm>,
    parameters: &[Symbol],
    regulator: Symbol,
    options: GenerationOptions,
) -> (BTreeMap<i32, Atom>, Completed) {
    let actual = run(&terms, parameters, regulator, &options, limits()).unwrap();
    let (density, pieces, cancellation_terms) =
        subtraction::subtract(terms, parameters, regulator, &options).unwrap();
    let expected = laurent::expand(
        &density,
        parameters,
        regulator,
        options.max_order,
        &mut TemplateCache::default(),
    )
    .unwrap();
    let restored = actual
        .coefficients
        .iter()
        .map(|(order, value)| (*order, value.clone().into_inner()))
        .collect::<BTreeMap<_, _>>();
    for order in restored
        .keys()
        .chain(expected.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let difference = restored.get(&order).cloned().unwrap_or(Atom::Zero)
            - expected
                .get(&order)
                .cloned()
                .map(AliasedAtom::into_inner)
                .unwrap_or(Atom::Zero);
        assert!(
            difference.as_view().get_byte_size() < 128_000,
            "only normalize small complete-vector controls"
        );
        assert!(
            difference.together().expand().is_zero(),
            "order {order}: {difference}"
        );
    }
    match &actual.route {
        Route::Native {
            attempts,
            relative_width,
            absolute_order,
            ..
        } => {
            assert!(*attempts > 0 && *relative_width > 0);
            assert!(*absolute_order > options.max_order);
        }
        Route::PhysicalFallback {
            pieces: actual_pieces,
            cancellation_terms: actual_terms,
        } => {
            assert_eq!(*actual_pieces, pieces);
            assert_eq!(actual_terms, &cancellation_terms);
            assert_eq!(actual.requests, RequestCounts::default());
        }
    }
    (restored, actual)
}

#[test]
fn complete_vectors_match_physical_taylor_and_ibp() {
    let x = symbol!("coefficient_controls::x");
    let y = symbol!("coefficient_controls::y");
    let eps = symbol!("coefficient_controls::eps");
    let e = Atom::var(eps);
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        for (prefactor, regular, powers, maximum) in [
            (
                Atom::one(),
                parse!("(1+coefficient_controls::x)^(-1-coefficient_controls::eps)"),
                vec![&e - 1, Atom::Zero],
                1,
            ),
            (
                parse!("gamma(2*coefficient_controls::eps)"),
                parse!("1/(1+coefficient_controls::x+coefficient_controls::y)"),
                vec![&e - 2, 2 * &e - 1],
                0,
            ),
            (
                parse!("gamma(2*coefficient_controls::eps)"),
                parse!("1/(1+coefficient_controls::x+coefficient_controls::y)"),
                vec![&e - 2, 2 * &e - 1],
                -1,
            ),
            (
                parse!("coefficient_controls::eps*gamma(coefficient_controls::eps)"),
                parse!("1/(1+coefficient_controls::x)^2"),
                vec![&e - 1, Atom::Zero],
                1,
            ),
            (
                parse!("gamma(coefficient_controls::eps)/coefficient_controls::eps^3"),
                parse!("exp(coefficient_controls::eps)-1-coefficient_controls::eps"),
                vec![&e - 1, Atom::Zero],
                -2,
            ),
            (
                Atom::one(),
                parse!("(1+coefficient_controls::x)^(-1-coefficient_controls::eps)"),
                vec![Atom::num((-3, 2)) + &e, Atom::Zero],
                0,
            ),
            (
                parse!("2+3*𝑖"),
                parse!(
                    "1+coefficient_controls::eps*coefficient_controls::x+coefficient_controls::eps^2*coefficient_controls::y"
                ),
                vec![&e - 1, 2 * &e - 1],
                0,
            ),
        ] {
            let (coefficients, _) = compare(
                vec![term(prefactor, regular, powers)],
                &[x, y],
                eps,
                GenerationOptions {
                    max_order: maximum,
                    subtraction: strategy,
                    ..GenerationOptions::default()
                },
            );
            assert!(!coefficients.is_empty(), "nonzero complete-vector control");
        }
    }
}

#[test]
fn native_coverage_retries_preserve_poles_cancellation_and_unknown_zero() {
    let x = symbol!("coefficient_bounds::x");
    let y = symbol!("coefficient_bounds::y");
    let eps = symbol!("coefficient_bounds::eps");
    let e = Atom::var(eps);
    let (coefficients, output) = compare(
        vec![term(Atom::one(), Atom::one(), vec![&e - 1, 2 * &e - 1])],
        &[x, y],
        eps,
        GenerationOptions::default(),
    );
    assert_eq!(coefficients, BTreeMap::from([(-2, Atom::num((1, 2)))]));
    let Route::Native {
        attempts,
        formal_pieces,
        absolute_order,
        ..
    } = output.route
    else {
        panic!("expected native route")
    };
    assert!(attempts > 1 && formal_pieces >= 4);
    assert_eq!(absolute_order, Rational::from(1));
    let prefactor = parse!("gamma(coefficient_bounds::eps)/coefficient_bounds::eps^3");
    let (zero, _) = compare(
        vec![
            term(prefactor.clone(), Atom::one(), vec![&e - 1]),
            term(prefactor, -Atom::one(), vec![&e - 1]),
        ],
        &[x],
        eps,
        GenerationOptions::default(),
    );
    assert!(zero.is_empty());
    let (positive, _) = compare(
        vec![term(e.pow(Atom::num(6)), Atom::one(), vec![Atom::Zero])],
        &[x],
        eps,
        GenerationOptions {
            max_order: -2,
            ..GenerationOptions::default()
        },
    );
    assert!(positive.is_empty());
    for maximum in [0, 2] {
        for terms in [
            vec![],
            vec![term(Atom::one(), Atom::Zero, vec![Atom::Zero])],
        ] {
            let result = run(
                &terms,
                &[x],
                eps,
                &GenerationOptions {
                    max_order: maximum,
                    ..GenerationOptions::default()
                },
                limits(),
            )
            .unwrap();
            assert!(result.coefficients.is_empty());
            let Route::Native { absolute_order, .. } = result.route else {
                panic!("zero must retain native coverage")
            };
            assert!(absolute_order > maximum);
        }
    }
}

#[test]
fn exact_unregulated_admission_preserves_pruned_polynomials_and_errors() {
    let x = symbol!("coefficient_fallback::x");
    let y = symbol!("coefficient_fallback::y");
    let eps = symbol!("coefficient_fallback::eps");
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let options = GenerationOptions {
            subtraction: strategy,
            ..GenerationOptions::default()
        };
        let (coefficients, output) = compare(
            vec![term(
                Atom::one(),
                Atom::var(x).pow(Atom::num(2)),
                vec![Atom::num(-2)],
            )],
            &[x],
            eps,
            options.clone(),
        );
        assert_eq!(coefficients, BTreeMap::from([(0, Atom::one())]));
        assert!(matches!(output.route, Route::PhysicalFallback { .. }));
        let regular =
            (Atom::one() + Atom::var(x)).pow(Atom::num(2)) * Atom::var(y).pow(Atom::num(2));
        let (coefficients, output) = compare(
            vec![term(
                Atom::one(),
                regular,
                vec![Atom::var(eps) - 3, Atom::num(-2)],
            )],
            &[x, y],
            eps,
            options.clone(),
        );
        assert_eq!(coefficients[&-1], Atom::one());
        assert_eq!(coefficients[&0], Atom::num((-5, 2)));
        assert!(matches!(output.route, Route::PhysicalFallback { .. }));
        for prefactor in [Atom::one(), Atom::Zero] {
            assert!(
                matches!(run(&[term(prefactor, Atom::one(), vec![Atom::num(-1)])], &[x], eps, &options, limits()), Err(GenerationError::UnregulatedEndpoint { parameter }) if parameter == x)
            );
        }
    }
}

#[test]
fn native_errors_and_early_endpoint_caps_are_not_hidden_by_series_work() {
    let x = symbol!("coefficient_errors::x");
    let eps = symbol!("coefficient_errors::eps");
    let options = GenerationOptions::default();
    assert!(matches!(
        run(
            &[term(
                parse!("coefficient_errors::eps^(1/2)"),
                Atom::one(),
                vec![Atom::Zero]
            )],
            &[x],
            eps,
            &GenerationOptions {
                max_order: 1,
                ..options.clone()
            },
            limits()
        ),
        Err(GenerationError::FractionalLaurent(_))
    ));
    let essential = parse!("exp(1/coefficient_errors::eps)");
    assert!(matches!(
        run(
            &[term(Atom::one(), essential.clone(), vec![Atom::Zero])],
            &[x],
            eps,
            &options,
            limits()
        ),
        Err(GenerationError::Series(_))
    ));
    let restricted = GenerationOptions {
        max_subtractions_per_axis: 1,
        ..options.clone()
    };
    // These must reject before attempting the unsupported regular Series.
    assert!(matches!(
        run(
            &[term(
                Atom::one(),
                essential.clone(),
                vec![Atom::var(eps) - 3]
            )],
            &[x],
            eps,
            &restricted,
            limits()
        ),
        Err(GenerationError::ResourceLimit("Taylor subtraction degree"))
    ));
    assert!(matches!(
        run(
            &[term(
                Atom::one(),
                essential,
                vec![Atom::var(eps).pow(Atom::num(2)) - 3]
            )],
            &[x],
            eps,
            &restricted,
            limits()
        ),
        Err(GenerationError::EndpointExponent(_))
    ));
    assert!(matches!(
        run(
            &[term(Atom::one(), Atom::one(), vec![])],
            &[x],
            eps,
            &options,
            limits()
        ),
        Err(GenerationError::Invariant(_))
    ));
}

#[test]
fn caller_caps_fail_before_partial_output_at_attempt_width_piece_and_request_boundaries() {
    let x = symbol!("coefficient_limits::x");
    let y = symbol!("coefficient_limits::y");
    let eps = symbol!("coefficient_limits::eps");
    let e = Atom::var(eps);
    let input = [term(Atom::one(), Atom::one(), vec![&e - 1, 2 * &e - 1])];
    for (caps, message) in [
        (
            Limits {
                max_attempts: Some(0),
                ..limits()
            },
            "coefficient-first series attempts",
        ),
        (
            Limits {
                max_attempts: Some(1),
                ..limits()
            },
            "coefficient-first series attempts",
        ),
        (
            Limits {
                max_relative_width: Some(0),
                ..limits()
            },
            "coefficient-first series width",
        ),
        (
            Limits {
                max_relative_width: Some(1),
                ..limits()
            },
            "coefficient-first series width",
        ),
    ] {
        assert!(
            matches!(run(&input, &[x, y], eps, &GenerationOptions::default(), caps), Err(GenerationError::ResourceLimit(actual)) if actual == message)
        );
    }
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let options = GenerationOptions {
            subtraction: strategy,
            max_subtraction_terms: 1,
            ..GenerationOptions::default()
        };
        assert!(matches!(
            run(
                &[term(Atom::one(), Atom::var(x) + 1, vec![&e - 3])],
                &[x],
                eps,
                &options,
                limits()
            ),
            Err(GenerationError::ResourceLimit(
                "coefficient-first subtraction terms"
            ))
        ));
        assert!(matches!(
            run(
                &[
                    term(Atom::one(), Atom::one(), vec![Atom::Zero]),
                    term(Atom::one(), Atom::one(), vec![Atom::Zero])
                ],
                &[x],
                eps,
                &options,
                limits()
            ),
            Err(GenerationError::ResourceLimit(
                "coefficient-first subtraction terms"
            ))
        ));
    }
    let no_requests = Limits {
        requests: RequestLimits {
            max_unique_requests: Some(0),
        },
        ..limits()
    };
    assert!(matches!(
        run(
            &[term(Atom::one(), Atom::var(x) + 1, vec![Atom::Zero])],
            &[x],
            eps,
            &GenerationOptions::default(),
            no_requests
        ),
        Err(GenerationError::ResourceLimit(_))
    ));
    let constant = run(
        &[term(Atom::one(), Atom::one(), vec![Atom::Zero])],
        &[x],
        eps,
        &GenerationOptions::default(),
        no_requests,
    )
    .unwrap();
    assert_eq!(constant.requests.unique_requests, 0);
}

#[test]
fn cancellation_stops_at_each_composition_boundary_without_further_callbacks() {
    let x = symbol!("coefficient_cancel::x");
    let eps = symbol!("coefficient_cancel::eps");
    let input = [term(
        parse!("gamma(coefficient_cancel::eps)"),
        parse!("1/(1+coefficient_cancel::x)"),
        vec![Atom::var(eps) - 2],
    )];
    let options = GenerationOptions::default();
    let mut events = Vec::new();
    expand(
        &input,
        &[x],
        eps,
        &options,
        limits(),
        &mut TemplateCache::default(),
        &mut |progress| {
            events.push(progress);
            ControlFlow::Continue(())
        },
    )
    .unwrap();
    // Each category is reached by a successful full-vector run before being
    // used as a cancellation target; request latch cases have separate tests.
    fn category(progress: &Progress) -> u8 {
        match progress {
            Progress::Admission { .. } => 0,
            Progress::Attempt { .. } => 1,
            Progress::RegularSeries { .. } => 2,
            Progress::ScalarSeries {
                kind: ScalarKind::EndpointInverse,
            } => 3,
            Progress::ScalarSeries {
                kind: ScalarKind::CoordinateWeight,
            } => 4,
            Progress::ScalarSeries {
                kind: ScalarKind::Prefactor,
            } => 5,
            Progress::Endpoint { .. } => 6,
            Progress::EndpointDegree { .. } => 7,
            Progress::ComposePiece { .. } => 8,
            Progress::ComposeGroup { .. } => 9,
            Progress::Coverage { .. } => 10,
            Progress::Requests(RequestProgress {
                stage: RequestStage::ReserveSymbols,
                ..
            }) => 11,
            Progress::Requests(RequestProgress {
                stage: RequestStage::NameCoefficients,
                ..
            }) => 12,
            Progress::Requests(RequestProgress {
                stage: RequestStage::LowerRequests,
                ..
            }) => 13,
            Progress::PhysicalFallback => 14,
            Progress::EmptySeries => 15,
        }
    }
    let categories = events.iter().map(category).collect::<BTreeSet<_>>();
    assert_eq!(categories, (0..14).collect());
    for target in categories {
        let mut cancelled = false;
        let result = expand(
            &input,
            &[x],
            eps,
            &options,
            limits(),
            &mut TemplateCache::default(),
            &mut |progress| {
                assert!(!cancelled, "callback after cancellation");
                if category(&progress) == target {
                    cancelled = true;
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        );
        assert!(cancelled && matches!(result, Err(GenerationError::Cancelled)));
    }
    for (terms, target) in [
        (vec![], 15),
        (
            vec![term(Atom::one(), Atom::var(x), vec![Atom::num(-1)])],
            14,
        ),
    ] {
        let mut cancelled = false;
        let result = expand(
            &terms,
            &[x],
            eps,
            &options,
            limits(),
            &mut TemplateCache::default(),
            &mut |progress| {
                assert!(!cancelled);
                if category(&progress) == target {
                    cancelled = true;
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        );
        assert!(cancelled && matches!(result, Err(GenerationError::Cancelled)));
    }
}
