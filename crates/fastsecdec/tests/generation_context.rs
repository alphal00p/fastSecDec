use fastsecdec::{
    generation::{
        GeneratedIntegral, GenerationContext, GenerationError, GenerationEvent, GenerationOptions,
        GenerationProgress, GeometryReuseStatus, generate,
    },
    parametric::{FactorRole, ParametricIntegrand, ParametricTerm, PolynomialFactor},
};
use fastsecdec_sectors::{DecompositionPhase, ParametricDomain, SectorError};
use std::ops::ControlFlow;
use symbolica::{atom::Atom, parse, symbol};

fn input(polynomial: Atom, exponent: Atom, measure: Atom) -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("x")],
        symbol!("eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![measure],
            vec![PolynomialFactor::new(
                polynomial,
                exponent,
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap()
}

fn run(
    context: &mut GenerationContext,
    input: &ParametricIntegrand,
    options: &GenerationOptions,
) -> (GeneratedIntegral, Vec<GeometryReuseStatus>) {
    let mut geometry = Vec::new();
    let result = context
        .generate(input, options, |event| {
            if let GenerationEvent::GeometryReuse(status) = event {
                geometry.push(*status);
            }
            ControlFlow::Continue(())
        })
        .unwrap();
    (result, geometry)
}

fn same_output(a: &GeneratedIntegral, b: &GeneratedIntegral) {
    assert_eq!(a.orders(), b.orders());
    assert_eq!(a.exact_coefficients(), b.exact_coefficients());
    assert_eq!(a.sectors().len(), b.sectors().len());
    for (a, b) in a.sectors().iter().zip(b.sectors()) {
        assert_eq!(a.parameters(), b.parameters());
        assert_eq!(a.map(), b.map());
        assert_eq!(a.cancellation_terms(), b.cancellation_terms());
        assert_eq!(a.cancellation_degree(), b.cancellation_degree());
        assert_eq!(
            a.aliased_coefficients().len(),
            b.aliased_coefficients().len()
        );
        for (a, b) in a
            .aliased_coefficients()
            .iter()
            .zip(b.aliased_coefficients())
        {
            assert_eq!(a.get_root(), b.get_root());
            assert_eq!(a.get_aliases(), b.get_aliases());
        }
    }
    let a = a.metadata();
    let b = b.metadata();
    assert_eq!(
        a.domain_assessment().domain(),
        b.domain_assessment().domain()
    );
    assert_eq!(
        a.domain_assessment().parameters(),
        b.domain_assessment().parameters()
    );
    assert_eq!(
        a.domain_assessment().caller_asserted(),
        b.domain_assessment().caller_asserted()
    );
    assert_eq!(
        a.domain_assessment().factors().len(),
        b.domain_assessment().factors().len()
    );
    for (a, b) in a
        .domain_assessment()
        .factors()
        .iter()
        .zip(b.domain_assessment().factors())
    {
        assert_eq!(a.polynomial(), b.polynomial());
        assert_eq!(a.exponent(), b.exponent());
        assert_eq!(a.certificate(), b.certificate());
    }
    assert_eq!(a.charts().len(), b.charts().len());
    for (a, b) in a.charts().iter().zip(b.charts()) {
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
fn cold_warm_and_uncached_generation_keep_compact_full_vectors_and_metadata() {
    let input = input(parse!("1+x"), parse!("-1"), parse!("-1+eps"));
    let options = GenerationOptions {
        max_order: 1,
        ..Default::default()
    };
    let plain = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut context = GenerationContext::new(2);
    let (cold, cold_status) = run(&mut context, &input, &options);
    let (warm, warm_status) = run(&mut context, &input, &options);
    assert_eq!(
        cold_status,
        vec![GeometryReuseStatus {
            reused: false,
            sectors: cold.metadata().charts().len()
        }]
    );
    assert_eq!(
        warm_status,
        vec![GeometryReuseStatus {
            reused: true,
            sectors: warm.metadata().charts().len()
        }]
    );
    same_output(&plain, &cold);
    same_output(&plain, &warm);
    assert_eq!(warm.orders(), &[-1, 0, 1]);
    // Compilation consumes the compact native bodies; no materialized coefficient
    // accessor or polynomial expansion is used by this comparison.
    let mut reference = plain.compile().unwrap();
    let mut cached = warm.compile().unwrap();
    assert_eq!(reference.content_id(), cached.content_id());
    // Only wholly constant vectors are extracted as exact offsets. This pole
    // stays beside its coordinate-dependent higher coefficients in the kernel.
    assert_eq!(cached.exact_coefficients(), &[0.0, 0.0, 0.0]);
    assert_eq!(cached.sectors().len(), 1);
    for x in [0.25_f64, 0.5, 0.75] {
        let mut a = [0.0; 3];
        let mut b = [0.0; 3];
        reference.sectors_mut()[0].evaluate(&[x], &mut a).unwrap();
        cached.sectors_mut()[0].evaluate(&[x], &mut b).unwrap();
        assert_eq!(a, b);
        assert_eq!(b[0], 1.0);
        assert!((b[1] + 1.0 / (1.0 + x)).abs() < 1e-13);
        assert!((b[2] + x.ln() / (1.0 + x)).abs() < 1e-13);
    }
    let encoded = serde_json::to_vec(&warm_status[0]).unwrap();
    assert_eq!(
        serde_json::from_slice::<GeometryReuseStatus>(&encoded).unwrap(),
        warm_status[0]
    );
}

#[test]
fn equal_support_reuses_only_geometry_and_reassesses_changed_coefficients() {
    let options = GenerationOptions::default();
    let mut context = GenerationContext::new(2);
    run(
        &mut context,
        &input(parse!("1+x"), parse!("-1"), parse!("-1+eps")),
        &options,
    );
    let changed = input(parse!("2+x"), parse!("-1"), parse!("-1+eps"));
    let (result, status) = run(&mut context, &changed, &options);
    assert!(status[0].reused);
    assert_eq!(
        result.metadata().domain_assessment().factors()[0].polynomial(),
        &parse!("2+x")
    );
    assert_eq!(result.exact_coefficients(), &[Atom::Zero, Atom::Zero]);
    let mut changed_kernels = result.compile().unwrap();
    let mut changed_vector = [0.0; 2];
    changed_kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut changed_vector)
        .unwrap();
    assert_eq!(changed_vector[0], 0.5);
    assert!((changed_vector[1] + 0.2).abs() < 1e-13);
    same_output(
        &result,
        &generate(&changed, &options, |_| ControlFlow::Continue(())).unwrap(),
    );
    let weighted = ParametricIntegrand::new(
        changed.parameters().to_vec(),
        changed.regulator(),
        changed.domain(),
        vec![ParametricTerm::new(
            Atom::num(2),
            vec![parse!("-1+eps")],
            vec![
                PolynomialFactor::new(parse!("2+x"), parse!("-1"), FactorRole::Singularity),
                PolynomialFactor::new(parse!("1+3*x"), Atom::one(), FactorRole::Polynomial),
            ],
        )],
    )
    .unwrap();
    let (weighted, status) = run(&mut context, &weighted, &options);
    assert!(status[0].reused);
    let mut kernels = weighted.compile().unwrap();
    assert_eq!(kernels.exact_coefficients(), &[0.0, 0.0]);
    let mut vector = [0.0; 2];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut vector)
        .unwrap();
    assert_eq!(vector[0], 1.0);
    assert!((vector[1] - 2.0).abs() < 1e-13); // 5/(2+x): both new weights applied once.

    let asserted = GenerationOptions {
        assume_no_threshold: true,
        ..Default::default()
    };
    for options in [&options, &asserted] {
        for (polynomial, exponent) in [
            (parse!("1-2*x"), parse!("-1")),
            (parse!("-1-x"), parse!("-1/2")),
            (parse!("1-x"), parse!("-1")),
        ] {
            let (generated, reuse) = run(
                &mut context,
                &input(polynomial.clone(), exponent, Atom::Zero),
                options,
            );
            assert!(reuse[0].reused);
            let domain = generated.metadata().domain_assessment();
            assert_eq!(
                domain.branch_policy(),
                fastsecdec::generation::BranchPolicy::UserResponsible
            );
            assert_eq!(domain.caller_asserted(), options.assume_no_threshold);
            assert_eq!(domain.factors()[0].polynomial(), &polynomial);
            assert_eq!(
                domain.factors()[0].certificate(),
                fastsecdec::generation::FactorCertificate::UncheckedUserResponsibility
            );
        }
    }
    assert_eq!(context.geometry_cache().len(), 1);
}

#[test]
fn caller_provenance_and_input_domain_are_not_replaced_by_cached_geometry() {
    let mut context = GenerationContext::new(4);
    let ordinary = GenerationOptions::default();
    let asserted = GenerationOptions {
        assume_no_threshold: true,
        ..Default::default()
    };
    run(
        &mut context,
        &input(parse!("1+x+x^2"), parse!("-1"), Atom::Zero),
        &ordinary,
    );
    let unknown = input(parse!("1-x+x^2"), parse!("-1"), Atom::Zero);
    for options in [&ordinary, &asserted, &ordinary] {
        let (accepted, observation) = run(&mut context, &unknown, options);
        assert!(observation[0].reused);
        let domain = accepted.metadata().domain_assessment();
        assert_eq!(domain.caller_asserted(), options.assume_no_threshold);
        assert!(!domain.relies_on_assertion());
        assert_eq!(
            domain.factors()[0].certificate(),
            fastsecdec::generation::FactorCertificate::UncheckedUserResponsibility
        );
    }

    let cube = input(parse!("1+x"), parse!("-2"), Atom::Zero);
    run(&mut context, &cube, &ordinary);
    let orthant = ParametricIntegrand::new(
        cube.parameters().to_vec(),
        cube.regulator(),
        ParametricDomain::PositiveOrthant,
        cube.terms().to_vec(),
    )
    .unwrap();
    let (full, observation) = run(&mut context, &orthant, &ordinary);
    assert!(!observation[0].reused);
    assert_eq!(
        full.metadata().domain_assessment().domain(),
        ParametricDomain::PositiveOrthant
    );
    same_output(
        &full,
        &generate(&orthant, &ordinary, |_| ControlFlow::Continue(())).unwrap(),
    );
}

#[test]
fn cancellation_preserves_only_completed_geometry_and_never_partial_integrals() {
    let input = input(parse!("1+x"), parse!("-1"), parse!("-1+eps"));
    let options = GenerationOptions::default();
    let mut context = GenerationContext::new(1);
    assert!(matches!(
        context.generate(&input, &options, |_| ControlFlow::Break(())),
        Err(GenerationError::Cancelled)
    ));
    assert!(context.geometry_cache().is_empty());
    let cancel_completion = |event: &GenerationEvent| {
        if matches!(event, GenerationEvent::Progress(GenerationProgress::Decomposition(status)) if status.phase == DecompositionPhase::Complete)
        {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    assert!(matches!(
        context.generate(&input, &options, cancel_completion),
        Err(GenerationError::Geometry(SectorError::Cancelled))
    ));
    assert!(context.geometry_cache().is_empty());
    for reused in [false, true] {
        let mut saw_reuse = false;
        assert!(matches!(
            context.generate(&input, &options, |event| {
                if let GenerationEvent::GeometryReuse(status) = event {
                    assert_eq!(status.reused, reused);
                    saw_reuse = true;
                    return ControlFlow::Break(());
                }
                assert!(!matches!(
                    event,
                    GenerationEvent::Progress(GenerationProgress::Factorization { .. })
                ));
                ControlFlow::Continue(())
            }),
            Err(GenerationError::Cancelled)
        ));
        assert!(saw_reuse);
        assert_eq!(context.geometry_cache().len(), 1);
    }
    assert!(matches!(
        context.generate(&input, &options, cancel_completion),
        Err(GenerationError::Geometry(SectorError::Cancelled))
    ));
    let (_, status) = run(&mut context, &input, &options);
    assert!(status[0].reused);
}

#[test]
fn empty_integral_skips_geometry_and_context_retains_no_expression_state() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<GenerationContext>();
    let zero = ParametricIntegrand::new(
        vec![symbol!("x")],
        symbol!("eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    let mut context = GenerationContext::new(1);
    let (result, geometry) = run(&mut context, &zero, &GenerationOptions::default());
    assert!(geometry.is_empty());
    assert!(context.geometry_cache().is_empty());
    assert!(result.sectors().is_empty());
    assert_eq!(result.exact_coefficients(), &[Atom::Zero]);
}

fn scoped_geometry(
    jobs: &mut dyn ExactSizeIterator<Item = fastsecdec::generation::GeometryJob>,
) -> Result<Vec<fastsecdec::generation::GeometryCompletion>, SectorError> {
    let mut results = Vec::new();
    loop {
        let batch = (&mut *jobs).take(2).collect::<Vec<_>>();
        if batch.is_empty() {
            break;
        }
        std::thread::scope(|scope| {
            let workers = batch
                .into_iter()
                .map(|job| scope.spawn(move || job.run(|_| ControlFlow::Continue(()))))
                .collect::<Vec<_>>();
            for worker in workers.into_iter().rev() {
                results.push(worker.join().unwrap());
            }
        });
    }
    results.reverse();
    Ok(results)
}

#[test]
fn caller_dispatch_preserves_generated_vectors_metadata_and_native_analytic_control() {
    use fastsecdec::generation::GeometryJob;
    let input = input(parse!("1+x"), parse!("-1"), parse!("-1+eps"));
    let options = GenerationOptions {
        max_order: 1,
        ..Default::default()
    };
    let plain = generate(&input, &options, |_| ControlFlow::Continue(())).unwrap();
    let mut stages = Vec::new();
    let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
        stages.push(jobs.len());
        scoped_geometry(jobs)
    };
    let mut context = GenerationContext::new(1);
    let mut geometry = Vec::new();
    let cold = context
        .generate_with_dispatch(
            &input,
            &options,
            &mut dispatch,
            || false,
            |event| {
                if let GenerationEvent::GeometryReuse(status) = event {
                    geometry.push(*status);
                }
                if let GenerationEvent::Progress(GenerationProgress::Decomposition(status)) = event
                {
                    assert_eq!(
                        status.phase,
                        DecompositionPhase::Complete,
                        "local worker status stays caller-owned"
                    );
                }
                ControlFlow::Continue(())
            },
        )
        .unwrap();
    assert_eq!(stages.len(), 2);
    assert!(stages.iter().all(|count| *count > 0));
    assert_eq!(geometry.len(), 1);
    assert!(!geometry[0].reused);
    same_output(&plain, &cold);
    let mut never=|_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<fastsecdec::generation::GeometryCompletion>,SectorError> {panic!("warm request dispatched")};
    let warm = context
        .generate_with_dispatch(
            &input,
            &options,
            &mut never,
            || false,
            |event| {
                if let GenerationEvent::GeometryReuse(status) = event {
                    assert!(status.reused);
                }
                ControlFlow::Continue(())
            },
        )
        .unwrap();
    same_output(&plain, &warm);
    let mut kernels = warm.compile().unwrap();
    assert_eq!(kernels.orders(), &[-1, 0, 1]);
    assert_eq!(kernels.exact_coefficients(), &[0.0, 0.0, 0.0]);
    for x in [0.25_f64, 0.5, 0.75] {
        let mut value = [0.0; 3];
        kernels.sectors_mut()[0].evaluate(&[x], &mut value).unwrap();
        assert_eq!(value[0], 1.0);
        assert!((value[1] + 1.0 / (1.0 + x)).abs() < 1e-13);
        assert!((value[2] + x.ln() / (1.0 + x)).abs() < 1e-13);
    }
    let weighted = ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![parse!("-1+eps")],
            input.terms()[0].factors().to_vec(),
        )],
    )
    .unwrap();
    let expected = generate(&weighted, &options, |_| ControlFlow::Continue(())).unwrap();
    let changed = context
        .generate_with_dispatch(
            &weighted,
            &options,
            &mut never,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    same_output(&expected, &changed);
    let mut kernels = changed.compile().unwrap();
    let mut value = [0.0; 3];
    kernels.sectors_mut()[0]
        .evaluate(&[0.5], &mut value)
        .unwrap();
    assert_eq!(value[0], 3.0);
    assert!((value[1] + 2.0).abs() < 1e-13);
}

#[test]
fn dispatched_generation_retains_caller_provenance_and_never_admits_incomplete_work() {
    use fastsecdec::generation::{GeometryCompletion, GeometryJob};
    let options = GenerationOptions::default();
    let good = input(parse!("1+x"), parse!("-1"), parse!("-1+eps"));
    let mut context = GenerationContext::new(1);
    context
        .generate_with_dispatch(
            &good,
            &options,
            &mut scoped_geometry,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let mut never=|_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>,SectorError> {panic!("cached/empty input dispatched")};
    let formal = context
        .generate_with_dispatch(
            &input(parse!("1-2*x"), parse!("-1"), Atom::Zero),
            &options,
            &mut never,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert_eq!(
        formal.metadata().domain_assessment().branch_policy(),
        fastsecdec::generation::BranchPolicy::UserResponsible
    );
    assert_eq!(
        formal.metadata().domain_assessment().factors()[0].polynomial(),
        &parse!("1-2*x")
    );
    assert_eq!(context.geometry_cache().len(), 1);
    let zero = ParametricIntegrand::new(
        good.parameters().to_vec(),
        good.regulator(),
        good.domain(),
        Vec::new(),
    )
    .unwrap();
    let empty = context
        .generate_with_dispatch(
            &zero,
            &options,
            &mut never,
            || false,
            |event| {
                assert!(!matches!(event, GenerationEvent::GeometryReuse(_)));
                ControlFlow::Continue(())
            },
        )
        .unwrap();
    assert!(empty.sectors().is_empty());
    assert_eq!(empty.exact_coefficients(), &[Atom::Zero]);
    let mut fresh = GenerationContext::new(1);
    let mut missing = |_: &mut dyn ExactSizeIterator<Item = GeometryJob>| Ok(Vec::new());
    assert!(matches!(
        fresh.generate_with_dispatch(
            &good,
            &options,
            &mut missing,
            || false,
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Geometry(SectorError::Work(
            fastsecdec_sectors::GeometryWorkError::MissingJobs { .. }
        )))
    ));
    assert!(fresh.geometry_cache().is_empty());
}

#[test]
fn dispatched_generation_cancellation_retains_only_accepted_geometry() {
    use fastsecdec::generation::{GeometryCompletion, GeometryJob};
    use std::sync::atomic::{AtomicBool, Ordering};
    let input = input(parse!("1+x"), parse!("-1"), parse!("-1+eps"));
    let options = GenerationOptions::default();
    let token = AtomicBool::new(true);
    let mut context = GenerationContext::new(1);
    let mut never=|_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>,SectorError> {panic!("cancelled/hit dispatched")};
    assert!(matches!(
        context.generate_with_dispatch(
            &input,
            &options,
            &mut never,
            || token.load(Ordering::Relaxed),
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Cancelled)
    ));
    token.store(false, Ordering::Relaxed);
    let mut interrupted = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
        Ok(jobs
            .map(|job| {
                job.run(|_| {
                    token.store(true, Ordering::Relaxed);
                    ControlFlow::Break(())
                })
            })
            .collect())
    };
    assert!(matches!(
        context.generate_with_dispatch(
            &input,
            &options,
            &mut interrupted,
            || token.load(Ordering::Relaxed),
            |_| ControlFlow::Continue(())
        ),
        Err(GenerationError::Geometry(SectorError::Cancelled))
    ));
    assert!(context.geometry_cache().is_empty());
    token.store(false, Ordering::Relaxed);
    assert!(matches!(context.generate_with_dispatch(&input,&options,&mut scoped_geometry,||false,|event|{
        if matches!(event,GenerationEvent::Progress(GenerationProgress::Decomposition(status)) if status.phase==DecompositionPhase::Complete) {ControlFlow::Break(())} else {ControlFlow::Continue(())}
    }),Err(GenerationError::Geometry(SectorError::Cancelled))));
    assert!(context.geometry_cache().is_empty());
    assert!(matches!(
        context.generate_with_dispatch(
            &input,
            &options,
            &mut scoped_geometry,
            || false,
            |event| {
                if matches!(event, GenerationEvent::GeometryReuse(_)) {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        ),
        Err(GenerationError::Cancelled)
    ));
    assert_eq!(context.geometry_cache().len(), 1);
    assert!(matches!(
        context.generate_with_dispatch(
            &input,
            &options,
            &mut never,
            || token.load(Ordering::Relaxed),
            |event| {
                if matches!(event, GenerationEvent::GeometryReuse(_)) {
                    token.store(true, Ordering::Relaxed);
                }
                ControlFlow::Continue(())
            }
        ),
        Err(GenerationError::Cancelled)
    ));
    assert_eq!(context.geometry_cache().len(), 1);
    token.store(false, Ordering::Relaxed);
    context
        .generate_with_dispatch(
            &input,
            &options,
            &mut never,
            || false,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
}
