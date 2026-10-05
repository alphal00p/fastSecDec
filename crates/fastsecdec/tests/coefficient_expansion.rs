//! Public opt-in admission, complete vectors, observers and caller-owned reuse.
use fastsecdec::{
    generation::{
        CoefficientExpansionMethod as Method, CoefficientExpansionOptions,
        CoefficientExpansionStage as Stage, ConditioningBasis, GeneratedIntegral,
        GenerationContext, GenerationError, GenerationEvent, GenerationOptions, GenerationPhase,
        GenerationProgress, GeometryCompletion, GeometryJob, SectorError, SubtractionStrategy,
        generate,
    },
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use std::{collections::BTreeSet, ops::ControlFlow};
use symbolica::{
    atom::{Atom, AtomCore},
    parse, symbol,
};

fn options(method: Method) -> GenerationOptions {
    GenerationOptions {
        max_order: 1,
        coefficient_expansion: CoefficientExpansionOptions {
            method,
            max_series_attempts: Some(8),
            max_relative_width: Some(32),
            max_unique_requests: Some(4096),
        },
        ..Default::default()
    }
}

fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("named_public::x")],
        symbol!("named_public::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("gamma(named_public::eps)"),
            vec![parse!("-2+named_public::eps")],
            vec![PolynomialFactor::new(
                parse!("1+named_public::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap()
}

fn run(input: &ParametricIntegrand, options: &GenerationOptions) -> GeneratedIntegral {
    generate(input, options, |_| ControlFlow::Continue(())).unwrap()
}

fn compare(a: &GeneratedIntegral, b: &GeneratedIntegral) {
    assert_eq!(a.orders(), b.orders());
    assert!(!a.orders().is_empty());
    assert_eq!(a.exact_coefficients().len(), a.orders().len());
    assert_eq!(b.exact_coefficients().len(), b.orders().len());
    assert_eq!(a.sectors().len(), b.sectors().len());
    for (a, b) in a.exact_coefficients().iter().zip(b.exact_coefficients()) {
        assert!((a - b).together().expand().is_zero());
    }
    let output_count = a.orders().len();
    for (a, b) in a.sectors().iter().zip(b.sectors()) {
        assert_eq!(a.coefficients().len(), output_count);
        assert_eq!(b.coefficients().len(), output_count);
        assert_eq!(a.parameters(), b.parameters());
        assert_eq!(a.map(), b.map());
        for (a, b) in a.coefficients().iter().zip(b.coefficients()) {
            let difference = a - b;
            assert!(difference.as_view().get_byte_size() < 128_000);
            assert!(difference.together().expand().is_zero(), "{difference}");
        }
    }
    assert_eq!(a.metadata().charts().len(), b.metadata().charts().len());
    for (a, b) in a.metadata().charts().iter().zip(b.metadata().charts()) {
        assert_eq!(a.source_index(), b.source_index());
        assert_eq!(a.representative(), b.representative());
        assert_eq!(
            a.representative_permutation(),
            b.representative_permutation()
        );
        assert_eq!(a.kernel_sector(), b.kernel_sector());
        assert_eq!(a.geometry(), b.geometry());
        assert_eq!(a.coordinates().images(), b.coordinates().images());
        assert_eq!(
            a.coordinates().measure_jacobian(),
            b.coordinates().measure_jacobian()
        );
    }
}

#[test]
fn full_public_vectors_preserve_both_strategies_and_conservative_profiles() {
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        for maximum in [-1, 1] {
            let physical = GenerationOptions {
                subtraction: strategy,
                max_order: maximum,
                ..options(Method::Physical)
            };
            let named = GenerationOptions {
                subtraction: strategy,
                max_order: maximum,
                ..options(Method::NativeNamed)
            };
            let a = run(&input(), &physical);
            let b = run(&input(), &named);
            compare(&a, &b);
            assert_eq!(b.orders(), &(-2..=maximum).collect::<Vec<_>>());
            assert_eq!(b.sectors().len(), 1);
            let bound = &b.sectors()[0];
            assert_eq!(
                bound.conditioning_basis(),
                ConditioningBasis::MappedEndpointBound
            );
            assert_eq!(bound.cancellation_terms(), &[vec![2]]);
            assert_eq!(bound.cancellation_degree(), 2);
            for row in a.sectors()[0].cancellation_terms() {
                assert!(
                    row.iter()
                        .zip(&bound.cancellation_terms()[0])
                        .all(|(physical, mapped)| physical <= mapped)
                );
            }
            assert_eq!(
                a.sectors()[0].conditioning_basis(),
                ConditioningBasis::RetainedRemainders
            );
        }
    }
}

#[test]
fn context_cold_warm_and_dispatched_named_generations_reuse_only_geometry() {
    let input = input();
    let options = options(Method::NativeNamed);
    let expected = run(&input, &options);
    let mut context = GenerationContext::new(1);
    let mut dispatch_calls = 0;
    let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>, SectorError> {
        dispatch_calls += 1;
        Ok(jobs.map(|job| job.run(|_| ControlFlow::Continue(()))).collect())
    };
    let cold = context
        .generate_with_dispatch(
            &input,
            &options,
            &mut dispatch,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert!(dispatch_calls > 0);
    compare(&expected, &cold);
    let mut reuse = Vec::new();
    let warm = context
        .generate(&input, &options, |event| {
            if let GenerationEvent::GeometryReuse(status) = event {
                reuse.push(status.reused);
            }
            ControlFlow::Continue(())
        })
        .unwrap();
    assert_eq!(reuse, [true]);
    compare(&expected, &warm);
    let mut never = |_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>, SectorError> { panic!("warm geometry dispatched") };
    let dispatched_warm = context
        .generate_with_dispatch(
            &input,
            &options,
            &mut never,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    compare(&expected, &dispatched_warm);
    // The same local function names must not retain the previous call's body.
    let changed = ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![ParametricTerm::new(
            parse!("3*gamma(named_public::eps)"),
            input.terms()[0].monomial_powers().to_vec(),
            vec![PolynomialFactor::new(
                parse!("2+named_public::x"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let changed_warm = context
        .generate(&changed, &options, |_| ControlFlow::Continue(()))
        .unwrap();
    compare(&run(&changed, &options), &changed_warm);
}

#[test]
fn geometry_multiplicity_and_exact_zero_support_survive_named_assembly() {
    let eps = symbol!("named_multiplicity::eps");
    let bubble = ParametricIntegrand::new(
        vec![
            symbol!("named_multiplicity::x"),
            symbol!("named_multiplicity::y"),
        ],
        eps,
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            parse!("gamma(named_multiplicity::eps)"),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(
                    parse!("named_multiplicity::x+named_multiplicity::y"),
                    parse!("-2+2*named_multiplicity::eps"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("named_multiplicity::x*named_multiplicity::y"),
                    parse!("-named_multiplicity::eps"),
                    FactorRole::Singularity,
                ),
            ],
        )],
    )
    .unwrap();
    let a = run(&bubble, &options(Method::Physical));
    let b = run(&bubble, &options(Method::NativeNamed));
    compare(&a, &b);
    assert!(b.metadata().charts().len() > b.sectors().len());
    let mut kernels = b.compile().unwrap();
    let mut values = vec![0.0; kernels.orders().len()];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut values)
        .unwrap();
    assert!(
        (values[0] - 2.0 / 2.25).abs() < 1e-13,
        "symmetry multiplicity applied once"
    );
    let zero = ParametricIntegrand::new(
        vec![symbol!("named_zero::x")],
        eps,
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    let generated = run(&zero, &options(Method::NativeNamed));
    assert!(generated.sectors().is_empty());
    assert!(generated.metadata().charts().is_empty());
    assert!(
        generated
            .exact_coefficients()
            .iter()
            .all(|coefficient| coefficient.is_zero())
    );
}

fn stages() -> Vec<Stage> {
    vec![
        Stage::Admission,
        Stage::RegularSeries,
        Stage::Naming,
        Stage::Endpoint,
        Stage::Composition,
        Stage::Coverage,
        Stage::Lowering,
        Stage::Complete,
    ]
}

#[test]
fn progress_resets_attempt_counts_and_uses_an_exclusive_phase() {
    let mut events = Vec::new();
    generate(&input(), &options(Method::NativeNamed), |event| {
        events.push(event.clone());
        ControlFlow::Continue(())
    })
    .unwrap();
    for stage in stages() {
        assert!(events.iter().any(|event| matches!(event, GenerationProgress::CoefficientExpansion { stage: actual, .. } if *actual == stage)), "missing {stage:?}");
    }
    let mut attempts = BTreeSet::new();
    for event in &events {
        if let GenerationProgress::CoefficientExpansion {
            attempt,
            relative_width,
            formal_pieces,
            requests,
            ..
        } = event
            && *attempt > 0
            && attempts.insert(*attempt)
        {
            assert!(*relative_width > 0);
            assert_eq!(*formal_pieces, 0);
            assert_eq!(*requests, Default::default());
        }
    }
    assert!(attempts.len() > 1);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event,
                GenerationProgress::PhaseTiming {
                    phase: GenerationPhase::CoefficientExpansion,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        GenerationProgress::Subtraction { .. }
            | GenerationProgress::LaurentExpansion { .. }
            | GenerationProgress::PhaseTiming {
                phase: GenerationPhase::Subtraction | GenerationPhase::Laurent,
                ..
            }
    )));
    let mut physical = options(Method::Physical);
    physical.coefficient_expansion.max_series_attempts = Some(0);
    physical.coefficient_expansion.max_relative_width = Some(0);
    physical.coefficient_expansion.max_unique_requests = Some(0);
    generate(&input(), &physical, |event| {
        assert!(!matches!(
            event,
            GenerationProgress::CoefficientExpansion { .. }
        ));
        ControlFlow::Continue(())
    })
    .unwrap();
}

#[test]
fn every_public_named_stage_can_cancel_without_completing_an_integral() {
    for stage in stages() {
        let mut cancelled = false;
        let mut complete = false;
        let mut context = GenerationContext::new(1);
        let result = context.generate(&input(), &options(Method::NativeNamed), |event| {
            assert!(!cancelled, "observer called after cancellation");
            if let GenerationEvent::Progress(GenerationProgress::Complete { .. }) = event {
                complete = true;
            }
            if let GenerationEvent::Progress(GenerationProgress::CoefficientExpansion {
                stage: actual,
                ..
            }) = event
                && *actual == stage
            {
                cancelled = true;
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        });
        assert!(cancelled && !complete && matches!(result, Err(GenerationError::Cancelled)));
    }
}

#[test]
fn public_limits_and_unregulated_fallback_fail_without_silent_route_changes() {
    for caps in [
        CoefficientExpansionOptions {
            max_series_attempts: Some(1),
            ..options(Method::NativeNamed).coefficient_expansion
        },
        CoefficientExpansionOptions {
            max_relative_width: Some(1),
            ..options(Method::NativeNamed).coefficient_expansion
        },
        CoefficientExpansionOptions {
            max_unique_requests: Some(0),
            ..options(Method::NativeNamed).coefficient_expansion
        },
    ] {
        let mut completed = false;
        let result = generate(
            &input(),
            &GenerationOptions {
                coefficient_expansion: caps,
                ..options(Method::NativeNamed)
            },
            |event| {
                completed |= matches!(event, GenerationProgress::Complete { .. });
                assert!(!matches!(
                    event,
                    GenerationProgress::CoefficientExpansion {
                        stage: Stage::PhysicalFallback,
                        ..
                    }
                ));
                ControlFlow::Continue(())
            },
        );
        assert!(matches!(result, Err(GenerationError::ResourceLimit(_))) && !completed);
    }
    let source = input();
    let unregulated = ParametricIntegrand::new(
        source.parameters().to_vec(),
        source.regulator(),
        source.domain(),
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::num(-1)],
            source.terms()[0].factors().to_vec(),
        )],
    )
    .unwrap();
    let mut fallback = false;
    let mut complete = false;
    let result = generate(
        &unregulated,
        &GenerationOptions {
            coefficient_expansion: CoefficientExpansionOptions {
                method: Method::NativeNamed,
                max_series_attempts: Some(0),
                ..Default::default()
            },
            ..Default::default()
        },
        |event| {
            complete |= matches!(event, GenerationProgress::Complete { .. });
            if let GenerationProgress::CoefficientExpansion {
                stage: Stage::PhysicalFallback,
                attempt,
                relative_width,
                formal_pieces,
                ..
            } = event
            {
                fallback = true;
                assert_eq!((*attempt, *relative_width, *formal_pieces), (0, 0, 0));
            }
            ControlFlow::Continue(())
        },
    );
    assert!(
        fallback && !complete && matches!(result, Err(GenerationError::UnregulatedEndpoint { .. }))
    );
}

#[test]
fn successful_exact_fallback_keeps_its_route_and_ignores_named_only_caps() {
    let x = symbol!("named_fallback_success::x");
    let eps = symbol!("named_fallback_success::eps");
    // Mapping groups these equal-power terms only after numerator valuation;
    // the resulting regular x still carries the original unregulated -1.
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![
            ParametricTerm::new(
                Atom::one(),
                vec![Atom::num(-1)],
                vec![PolynomialFactor::new(
                    Atom::one() + Atom::var(x),
                    Atom::one(),
                    FactorRole::Polynomial,
                )],
            ),
            ParametricTerm::new(-Atom::one(), vec![Atom::num(-1)], vec![]),
        ],
    )
    .unwrap();
    let options = GenerationOptions {
        coefficient_expansion: CoefficientExpansionOptions {
            method: Method::NativeNamed,
            max_series_attempts: Some(0),
            max_relative_width: Some(0),
            max_unique_requests: Some(0),
        },
        ..options(Method::NativeNamed)
    };
    let mut fallback = false;
    let mut completed = false;
    let mut phase_count = 0;
    let generated = generate(&input, &options, |event| {
        match event {
            GenerationProgress::CoefficientExpansion {
                stage,
                attempt,
                relative_width,
                formal_pieces,
                requests,
                ..
            } => {
                assert_eq!((*attempt, *relative_width, *formal_pieces), (0, 0, 0));
                assert_eq!(*requests, Default::default());
                fallback |= *stage == Stage::PhysicalFallback;
                if *stage == Stage::Complete {
                    assert!(fallback);
                    completed = true;
                }
            }
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::CoefficientExpansion,
                ..
            } => phase_count += 1,
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::Subtraction | GenerationPhase::Laurent,
                ..
            } => panic!("overlapping physical timing"),
            _ => {}
        }
        ControlFlow::Continue(())
    })
    .unwrap();
    assert!(fallback && completed);
    assert_eq!(phase_count, 1);
    assert_eq!(generated.exact_coefficients(), &[Atom::one(), Atom::Zero]);
    assert!(generated.sectors().is_empty());
    assert_eq!(generated.metadata().charts().len(), 1);
    assert_eq!(generated.metadata().charts()[0].kernel_sector(), None);
}

#[test]
fn cancellation_to_zero_preserves_admitted_chart_metadata() {
    let source = input();
    let cancelled = ParametricIntegrand::new(
        source.parameters().to_vec(),
        source.regulator(),
        source.domain(),
        [Atom::one(), -Atom::one()]
            .into_iter()
            .map(|prefactor| {
                ParametricTerm::new(
                    prefactor,
                    vec![Atom::Zero],
                    source.terms()[0].factors().to_vec(),
                )
            })
            .collect(),
    )
    .unwrap();
    let physical = run(&cancelled, &options(Method::Physical));
    let named = run(&cancelled, &options(Method::NativeNamed));
    compare(&physical, &named);
    assert_eq!(named.orders(), &[0, 1]);
    assert_eq!(named.exact_coefficients(), &[Atom::Zero, Atom::Zero]);
    assert!(named.sectors().is_empty());
    assert_eq!(named.metadata().charts().len(), 1);
    assert_eq!(named.metadata().charts()[0].kernel_sector(), None);
}
