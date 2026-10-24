use fastsecdec_sectors::{
    DecompositionOptions, DecompositionPhase, GeometryCache, GeometryCompletion, GeometryJob,
    GeometryPlan, ParametricDomain, PolynomialSupport, SectorError, decompose,
};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use symbolica::utils::resources::{ResourceBudget, ResourceLimitError, ResourceLimits};

fn support(power: i64) -> Vec<PolynomialSupport> {
    vec![PolynomialSupport::new(vec![vec![0], vec![power]]).unwrap()]
}

#[test]
fn count_diagnostics_and_process_resources_are_disabled_by_default() {
    let options = DecompositionOptions::default();
    assert_eq!(options.max_support_pairs, None);
    assert_eq!(options.max_rays, None);
    assert_eq!(options.max_sectors, None);
    assert_eq!(options.resources, ResourceLimits::default());
    let limited = DecompositionOptions {
        resources: ResourceLimits {
            max_elapsed: Some(Duration::ZERO),
            ..ResourceLimits::default()
        },
        ..DecompositionOptions::default()
    };
    assert!(matches!(
        decompose(ParametricDomain::UnitCube, &support(1), &limited, |_| {
            ControlFlow::Continue(())
        }),
        Err(SectorError::Resource(_))
    ));
}

#[test]
fn queued_geometry_jobs_keep_the_callers_budget_on_another_thread() {
    let rss = Arc::new(AtomicU64::new(0));
    let measured = Arc::clone(&rss);
    let budget = ResourceBudget::with_memory_monitor(
        ResourceLimits {
            max_process_rss_bytes: Some(10),
            ..ResourceLimits::default()
        },
        move || Ok(measured.load(Ordering::Relaxed)),
    );
    let plan = budget
        .run(|| {
            GeometryPlan::new(
                ParametricDomain::UnitCube,
                support(1),
                DecompositionOptions::default(),
            )
        })
        .unwrap();
    let job = plan.charts().next().unwrap();
    rss.store(11, Ordering::Relaxed);
    let completed = thread::spawn(move || job.run(|_| ControlFlow::Continue(())))
        .join()
        .unwrap();
    assert!(matches!(
        completed.error(),
        Some(SectorError::Resource(ResourceLimitError::Memory { .. }))
    ));
}

#[test]
fn failed_cache_publication_preserves_the_previous_entry() {
    let options = DecompositionOptions::default();
    let old_support = support(1);
    let new_support = support(2);
    let mut failures = 0;
    let mut successes = 0;
    // Inject failures at successive checkpoints after the accepted Complete
    // callback, including the publication boundary, without relying on one
    // particular checkpoint count.
    for allowed_checks in 0..8 {
        let mut cache = GeometryCache::new(1);
        cache
            .decompose(ParametricDomain::UnitCube, &old_support, &options, |_| {
                ControlFlow::Continue(())
            })
            .unwrap();
        let armed = Arc::new(AtomicBool::new(false));
        let observed = Arc::new(AtomicUsize::new(0));
        let monitor_armed = Arc::clone(&armed);
        let monitor_observed = Arc::clone(&observed);
        let budget = ResourceBudget::with_memory_monitor(
            ResourceLimits {
                max_process_rss_bytes: Some(10),
                ..ResourceLimits::default()
            },
            move || {
                let exceeded = monitor_armed.load(Ordering::Relaxed)
                    && monitor_observed.fetch_add(1, Ordering::Relaxed) >= allowed_checks;
                Ok(if exceeded { 11 } else { 0 })
            },
        );
        let result = budget.run(|| {
            cache.decompose(
                ParametricDomain::UnitCube,
                &new_support,
                &options,
                |status| {
                    if status.phase == DecompositionPhase::Complete {
                        armed.store(true, Ordering::Relaxed);
                    }
                    ControlFlow::Continue(())
                },
            )
        });
        match result {
            Err(SectorError::Resource(ResourceLimitError::Memory { .. })) => {
                failures += 1;
                assert_eq!(cache.len(), 1);
                assert!(
                    cache
                        .decompose(ParametricDomain::UnitCube, &old_support, &options, |_| {
                            ControlFlow::Continue(())
                        })
                        .unwrap()
                        .reused
                );
            }
            Ok(result) => {
                successes += 1;
                assert!(!result.reused);
            }
            Err(error) => panic!("unexpected cache error: {error}"),
        }
    }
    assert!(failures > 0);
    assert!(successes > 0);
}

#[test]
fn callback_and_dispatch_errors_precede_resource_postchecks() {
    let options = DecompositionOptions::default();
    let supports = support(1);
    let rss = Arc::new(AtomicU64::new(0));
    let measured = Arc::clone(&rss);
    let budget = ResourceBudget::with_memory_monitor(
        ResourceLimits {
            max_process_rss_bytes: Some(10),
            ..ResourceLimits::default()
        },
        move || Ok(measured.load(Ordering::Relaxed)),
    );
    let mut cache = GeometryCache::new(1);
    cache
        .decompose(ParametricDomain::UnitCube, &supports, &options, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    rss.store(11, Ordering::Relaxed);
    assert!(matches!(
        budget.run(
            || cache.decompose(ParametricDomain::UnitCube, &supports, &options, |_| {
                ControlFlow::Continue(())
            })
        ),
        Err(SectorError::Resource(ResourceLimitError::Memory { .. }))
    ));
    assert_eq!(cache.len(), 1);
    rss.store(0, Ordering::Relaxed);
    let cancelled = budget.run(|| {
        cache.decompose(ParametricDomain::UnitCube, &supports, &options, |_| {
            rss.store(11, Ordering::Relaxed);
            ControlFlow::Break(())
        })
    });
    assert!(matches!(cancelled, Err(SectorError::Cancelled)));
    assert_eq!(cache.len(), 1);
    rss.store(0, Ordering::Relaxed);
    let mut dispatch = |_: &mut dyn ExactSizeIterator<Item = GeometryJob>| -> Result<
        Vec<GeometryCompletion>,
        SectorError,
    > {
        rss.store(11, Ordering::Relaxed);
        Err(SectorError::Geometry("original dispatch failure".into()))
    };
    let failed = budget.run(|| {
        cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &support(2),
            &options,
            &mut dispatch,
            || false,
            |_| ControlFlow::Continue(()),
        )
    });
    assert!(
        matches!(failed, Err(SectorError::Geometry(reason)) if reason == "original dispatch failure")
    );
    assert_eq!(cache.len(), 1);

    rss.store(0, Ordering::Relaxed);
    let mut dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
        Ok(jobs
            .map(|job| {
                job.run(|_| {
                    rss.store(11, Ordering::Relaxed);
                    ControlFlow::Break(())
                })
            })
            .collect::<Vec<_>>())
    };
    let cancelled = budget.run(|| {
        cache.decompose_with_dispatch(
            ParametricDomain::UnitCube,
            &support(2),
            &options,
            &mut dispatch,
            || false,
            |_| ControlFlow::Continue(()),
        )
    });
    assert!(matches!(cancelled, Err(SectorError::Cancelled)));
    assert_eq!(cache.len(), 1);
}
