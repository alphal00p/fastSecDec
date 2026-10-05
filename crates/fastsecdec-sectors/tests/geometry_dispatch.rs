use fastsecdec_sectors::*;
use std::{
    ops::ControlFlow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn poly(rows: &[&[i64]]) -> PolynomialSupport {
    PolynomialSupport::new(rows.iter().map(|r| r.to_vec()).collect()).unwrap()
}
fn go(_: &DecompositionProgress) -> ControlFlow<()> {
    ControlFlow::Continue(())
}
fn threads(
    jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>,
) -> Result<Vec<GeometryCompletion>, SectorError> {
    let mut results = Vec::new();
    loop {
        let batch = (&mut *jobs).take(2).collect::<Vec<_>>();
        if batch.is_empty() {
            break;
        }
        std::thread::scope(|scope| {
            let workers = batch
                .into_iter()
                .map(|job| scope.spawn(move || job.run(go)))
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
fn native_cache_dispatch_preserves_all_domains_and_skips_work_on_hits() {
    let cases = [
        (ParametricDomain::UnitCube, poly(&[&[1, 0], &[0, 1]])),
        (ParametricDomain::PositiveOrthant, poly(&[&[0], &[1]])),
        (
            ParametricDomain::ProjectiveSimplex,
            poly(&[&[1, 0], &[0, 1]]),
        ),
        (ParametricDomain::UnitCube, poly(&[&[]])),
        (ParametricDomain::PositiveOrthant, poly(&[&[]])),
        (ParametricDomain::ProjectiveSimplex, poly(&[&[2]])),
    ];
    for (domain, factor) in cases {
        let factors = [factor];
        let options = DecompositionOptions::default();
        let expected = decompose(domain, &factors, &options, go).unwrap();
        let mut cache = GeometryCache::new(1);
        let mut stage_sizes = Vec::new();
        let mut dispatcher = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
            stage_sizes.push(jobs.len());
            threads(jobs)
        };
        let mut events = Vec::new();
        let cold = cache
            .decompose_with_dispatch(
                domain,
                &factors,
                &options,
                &mut dispatcher,
                || false,
                |s| {
                    events.push(s.clone());
                    ControlFlow::Continue(())
                },
            )
            .unwrap();
        assert!(!cold.reused);
        assert_eq!(stage_sizes.len(), 2);
        assert_eq!(stage_sizes[1] == 0, expected.sectors[0].dimension() == 0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].phase, DecompositionPhase::Complete);
        assert_eq!(cold.decomposition.sectors, expected.sectors);
        assert_eq!(cold.decomposition.domain, expected.domain);
        assert_eq!(
            cold.decomposition.candidate_vertices,
            expected.candidate_vertices
        );
        assert_eq!(
            cold.decomposition.geometric_vertices,
            expected.geometric_vertices
        );
        let mut never = |_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>,SectorError> {panic!("warm cache dispatched")};
        let warm = cache
            .decompose_with_dispatch(domain, &factors, &options, &mut never, || false, go)
            .unwrap();
        assert!(warm.reused && Arc::ptr_eq(&cold.decomposition, &warm.decomposition));
        let serial_hit = cache.decompose(domain, &factors, &options, go).unwrap();
        assert!(serial_hit.reused && Arc::ptr_eq(&cold.decomposition, &serial_hit.decomposition));
        cache.clear();
        let serial = cache.decompose(domain, &factors, &options, go).unwrap();
        let reused = cache
            .decompose_with_dispatch(domain, &factors, &options, &mut never, || false, go)
            .unwrap();
        assert!(reused.reused && Arc::ptr_eq(&serial.decomposition, &reused.decomposition));
        let mut disabled = GeometryCache::new(0);
        for _ in 0..2 {
            assert!(
                !disabled
                    .decompose_with_dispatch(domain, &factors, &options, &mut threads, || false, go)
                    .unwrap()
                    .reused
            );
            assert!(disabled.is_empty());
        }
    }
}

#[test]
fn invalid_dispatched_work_and_native_limits_do_not_insert_or_evict() {
    let original = [poly(&[&[0], &[1]])];
    let other = [poly(&[&[1, 0], &[0, 1]])];
    let options = DecompositionOptions::default();
    let mut cache = GeometryCache::new(1);
    let retained = cache
        .decompose(ParametricDomain::UnitCube, &original, &options, go)
        .unwrap();
    for mode in 0..6 {
        let mut stage = 0;
        let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
            stage += 1;
            if mode == 2 && stage == 1 {
                let foreign =
                    GeometryPlan::new(ParametricDomain::UnitCube, other.to_vec(), options.clone())
                        .unwrap();
                return Ok(foreign.charts().map(|job| job.run(go)).collect());
            }
            if stage == 1 {
                return threads(jobs);
            }
            match mode {
                0 => Ok(Vec::new()),
                1 => {
                    let job = jobs.next().unwrap();
                    let mut result = vec![job.run(go), job.run(go)];
                    result.extend(jobs.map(|j| j.run(go)));
                    Ok(result)
                }
                3 => Err(SectorError::Geometry("caller scheduling failure".into())),
                4 => Ok(jobs.map(|j| j.run(|_| ControlFlow::Break(()))).collect()),
                5 => {
                    let wrong = GeometryPlan::new(
                        ParametricDomain::UnitCube,
                        other.to_vec(),
                        options.clone(),
                    )
                    .unwrap();
                    Ok(wrong.charts().map(|j| j.run(go)).collect())
                }
                _ => unreachable!(),
            }
        };
        let result = cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &other,
            &options,
            &mut dispatch,
            || false,
            go,
        );
        match mode {
            0 => assert!(matches!(
                result,
                Err(SectorError::Work(GeometryWorkError::MissingJobs { .. }))
            )),
            1 => assert!(matches!(
                result,
                Err(SectorError::Work(GeometryWorkError::DuplicateJob(_)))
            )),
            2 => assert!(matches!(
                result,
                Err(SectorError::Work(GeometryWorkError::ForeignPlan))
            )),
            3 => assert!(matches!(result, Err(SectorError::Geometry(_)))),
            4 => assert!(matches!(result, Err(SectorError::Cancelled))),
            5 => assert!(matches!(
                result,
                Err(SectorError::Work(GeometryWorkError::WrongStage))
            )),
            _ => unreachable!(),
        }
        assert_eq!(cache.len(), 1);
        let hit = cache
            .decompose(ParametricDomain::UnitCube, &original, &options, go)
            .unwrap();
        assert!(hit.reused && Arc::ptr_eq(&retained.decomposition, &hit.decomposition));
    }
    for options in [
        DecompositionOptions {
            max_support_pairs: 0,
            ..Default::default()
        },
        DecompositionOptions {
            max_rays: 0,
            ..Default::default()
        },
        DecompositionOptions {
            max_sectors: 1,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            cache.decompose_with_dispatch(
                ParametricDomain::UnitCube,
                &other,
                &options,
                &mut threads,
                || false,
                go
            ),
            Err(SectorError::ResourceLimit { .. })
        ));
        assert_eq!(cache.len(), 1);
    }
    let hit = cache
        .decompose(ParametricDomain::UnitCube, &original, &options, go)
        .unwrap();
    assert!(hit.reused && Arc::ptr_eq(&retained.decomposition, &hit.decomposition));
}

#[test]
fn cancellation_before_between_and_after_native_stages_preserves_complete_cache() {
    let factors = [poly(&[&[1, 0], &[0, 1]])];
    let options = DecompositionOptions::default();
    let mut cache = GeometryCache::new(1);
    let mut never=|_: &mut dyn ExactSizeIterator<Item=GeometryJob>| -> Result<Vec<GeometryCompletion>,SectorError> {panic!("cancelled before dispatch")};
    assert!(matches!(
        cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &factors,
            &options,
            &mut never,
            || true,
            go
        ),
        Err(SectorError::Cancelled)
    ));
    let token = AtomicBool::new(false);
    let mut calls = 0;
    let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
        calls += 1;
        let result = threads(jobs);
        token.store(true, Ordering::Relaxed);
        result
    };
    assert!(matches!(
        cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &factors,
            &options,
            &mut dispatch,
            || token.load(Ordering::Relaxed),
            go
        ),
        Err(SectorError::Cancelled)
    ));
    assert_eq!(calls, 1);
    assert!(cache.is_empty());
    for use_token in [false, true] {
        token.store(false, Ordering::Relaxed);
        let result = cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &factors,
            &options,
            &mut threads,
            || token.load(Ordering::Relaxed),
            |status| {
                assert_eq!(status.phase, DecompositionPhase::Complete);
                if use_token {
                    token.store(true, Ordering::Relaxed);
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            },
        );
        assert!(matches!(result, Err(SectorError::Cancelled)));
        assert!(cache.is_empty());
    }
    let complete = cache
        .decompose(ParametricDomain::UnitCube, &factors, &options, go)
        .unwrap();
    assert!(matches!(
        cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &factors,
            &options,
            &mut never,
            || false,
            |_| ControlFlow::Break(())
        ),
        Err(SectorError::Cancelled)
    ));
    let retained = cache
        .decompose(ParametricDomain::UnitCube, &factors, &options, go)
        .unwrap();
    assert!(retained.reused && Arc::ptr_eq(&complete.decomposition, &retained.decomposition));
}
