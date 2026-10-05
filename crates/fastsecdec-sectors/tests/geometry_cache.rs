use fastsecdec_sectors::{
    Decomposition, DecompositionOptions, DecompositionPhase, GeometryCache, GeometryCacheOutcome,
    ParametricDomain, PolynomialSupport, SectorError, SectorMap, decompose,
};
use numerica::domains::{
    integer::Integer,
    rational::{Q, Rational},
};
use std::{ops::ControlFlow, sync::Arc};

fn support(rows: &[&[i64]]) -> PolynomialSupport {
    PolynomialSupport::new(rows.iter().map(|row| row.to_vec()).collect()).unwrap()
}

fn cached(cache: &mut GeometryCache, supports: &[PolynomialSupport]) -> GeometryCacheOutcome {
    cache
        .decompose(
            ParametricDomain::UnitCube,
            supports,
            &DecompositionOptions::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap()
}

fn same_geometry(a: &Decomposition, b: &Decomposition) {
    assert_eq!(a.domain, b.domain);
    assert_eq!(a.sectors, b.sectors);
    assert_eq!(a.candidate_vertices, b.candidate_vertices);
    assert_eq!(a.geometric_vertices, b.geometric_vertices);
}

/// Independent change-of-variables integral of x^2 y on the cube: 1/6.
fn cube_moment(result: &Decomposition) -> Rational {
    result.sectors.iter().fold(Rational::from(0), |sum, map| {
        assert_eq!(map.source_dimension(), 2);
        let denominator = (0..map.dimension()).fold(Integer::from(1), |product, axis| {
            product * (&map.exponent_matrix[0][axis] * 3 + &map.exponent_matrix[1][axis] * 2)
        });
        sum + Q.to_element(map.determinant.clone(), denominator, true)
    })
}

#[test]
fn complete_native_results_are_reused_with_cancellable_completion_only() {
    let mut cache = GeometryCache::new(8);
    let homogeneous = support(&[&[1, 0], &[0, 1]]);
    let full_rank = support(&[&[0, 0], &[1, 0], &[0, 1]]);
    let zero_dimensional = support(&[&[]]);
    for (domain, factors) in [
        (ParametricDomain::UnitCube, vec![homogeneous.clone()]),
        (ParametricDomain::ProjectiveSimplex, vec![homogeneous]),
        (ParametricDomain::PositiveOrthant, vec![full_rank]),
        (ParametricDomain::UnitCube, vec![zero_dimensional.clone()]),
        (ParametricDomain::PositiveOrthant, vec![zero_dimensional]),
    ] {
        let options = DecompositionOptions::default();
        let serial = decompose(domain, &factors, &options, |_| ControlFlow::Continue(())).unwrap();
        let mut miss_events = Vec::new();
        let fresh = cache
            .decompose(domain, &factors, &options, |status| {
                miss_events.push(status.clone());
                ControlFlow::Continue(())
            })
            .unwrap();
        assert!(!fresh.reused);
        same_geometry(&serial, &fresh.decomposition);
        assert!(miss_events.len() >= 2);
        let mut hit_events = Vec::new();
        let reused = cache
            .decompose(domain, &factors, &options, |status| {
                hit_events.push(status.clone());
                ControlFlow::Continue(())
            })
            .unwrap();
        assert!(reused.reused);
        assert!(Arc::ptr_eq(&fresh.decomposition, &reused.decomposition));
        assert_eq!(hit_events.len(), 1);
        assert_eq!(hit_events[0].phase, DecompositionPhase::Complete);
        assert_eq!(hit_events[0].sectors, serial.sectors.len());
        assert_eq!(hit_events[0].chart, miss_events.last().unwrap().chart);
    }
    assert_eq!(cache.len(), 5);
}

#[test]
fn ordered_support_axes_and_monomial_translations_keep_native_valuations() {
    let a = support(&[&[0, 0], &[1, 0], &[0, 2]]);
    let b = support(&[&[1, 0], &[0, 1]]);
    let translated = support(&[&[3, 1], &[4, 1], &[3, 3]]);
    let permuted = support(&[&[0, 0], &[0, 1], &[2, 0]]);
    let mut cache = GeometryCache::new(8);
    let base = cached(&mut cache, &[a.clone(), b.clone()]);
    let reversed = cached(&mut cache, &[b.clone(), a.clone()]);
    let shifted = cached(&mut cache, &[translated, b.clone()]);
    let axes = cached(&mut cache, &[permuted, b.clone()]);
    for result in [&base, &reversed, &shifted, &axes] {
        assert!(!result.reused);
        assert_eq!(cube_moment(&result.decomposition), Rational::from((1, 6)));
    }
    // Reordered factors keep the fan but must reorder the corresponding rows.
    assert_eq!(
        base.decomposition.sectors.len(),
        reversed.decomposition.sectors.len()
    );
    for (original, reversed) in base
        .decomposition
        .sectors
        .iter()
        .zip(&reversed.decomposition.sectors)
    {
        assert_eq!(original.exponent_matrix, reversed.exponent_matrix);
        assert_eq!(original.factor_valuations[0], reversed.factor_valuations[1]);
        assert_eq!(original.factor_valuations[1], reversed.factor_valuations[0]);
    }
    assert_eq!(
        base.decomposition.sectors.len(),
        shifted.decomposition.sectors.len()
    );
    for (original, shifted) in base
        .decomposition
        .sectors
        .iter()
        .zip(&shifted.decomposition.sectors)
    {
        assert_eq!(original.exponent_matrix, shifted.exponent_matrix);
        for axis in 0..original.dimension() {
            assert_eq!(
                shifted.factor_valuations[0][axis],
                &original.factor_valuations[0][axis]
                    + &original.exponent_matrix[0][axis] * 3
                    + &original.exponent_matrix[1][axis]
            );
        }
    }
    // Canonical duplicate monomial rows within one support are already native
    // equality; no new normalization is needed for this valid hit.
    let duplicate = support(&[&[0, 2], &[1, 0], &[0, 0], &[1, 0]]);
    let canonical = cached(&mut cache, &[duplicate, b]);
    assert!(canonical.reused);
    assert!(Arc::ptr_eq(&base.decomposition, &canonical.decomposition));
}

#[test]
fn warm_geometry_does_not_bypass_any_resource_limit() {
    let factors = [support(&[&[1, 0], &[0, 1]])];
    let mut cache = GeometryCache::new(8);
    let original = cached(&mut cache, &factors);
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
            max_sectors: 0,
            ..Default::default()
        },
    ] {
        let uncached = decompose(ParametricDomain::UnitCube, &factors, &options, |_| {
            ControlFlow::Continue(())
        });
        let result = cache.decompose(ParametricDomain::UnitCube, &factors, &options, |_| {
            ControlFlow::Continue(())
        });
        let Err(SectorError::ResourceLimit { resource, limit }) = uncached else {
            panic!("expected native limit")
        };
        assert!(
            matches!(result, Err(SectorError::ResourceLimit { resource: r, limit: n }) if r == resource && n == limit)
        );
        assert_eq!(cache.len(), 1);
    }
    let mut more_permissive = DecompositionOptions::default();
    more_permissive.max_sectors += 1;
    let changed = cache
        .decompose(
            ParametricDomain::UnitCube,
            &factors,
            &more_permissive,
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    assert!(!changed.reused);
    same_geometry(&original.decomposition, &changed.decomposition);
    assert!(cached(&mut cache, &factors).reused);
}

#[test]
fn invalid_and_cancelled_requests_never_insert_or_evict() {
    let factors = [support(&[&[1, 0], &[0, 1]])];
    let other = [support(&[&[0, 0], &[1, 0], &[0, 1]])];
    let options = DecompositionOptions::default();
    let mut cache = GeometryCache::new(1);
    for cancel_phase in [DecompositionPhase::Supports, DecompositionPhase::Complete] {
        let result = cache.decompose(ParametricDomain::UnitCube, &factors, &options, |status| {
            if status.phase == cancel_phase {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
        assert!(matches!(result, Err(SectorError::Cancelled)));
        assert!(cache.is_empty());
    }
    let valid = cached(&mut cache, &factors);
    assert!(matches!(
        cache.decompose(ParametricDomain::UnitCube, &factors, &options, |_| {
            ControlFlow::Break(())
        }),
        Err(SectorError::Cancelled)
    ));
    assert!(matches!(
        cache.decompose(ParametricDomain::UnitCube, &other, &options, |status| {
            if status.phase == DecompositionPhase::Complete {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }),
        Err(SectorError::Cancelled)
    ));
    assert!(matches!(
        cache.decompose(ParametricDomain::UnitCube, &[], &options, |_| {
            ControlFlow::Continue(())
        }),
        Err(SectorError::InvalidSupport(_))
    ));
    // The same support succeeds on the cube but is rank deficient on the full orthant.
    assert!(matches!(
        cache.decompose(
            ParametricDomain::PositiveOrthant,
            &factors,
            &options,
            |_| ControlFlow::Continue(())
        ),
        Err(SectorError::RankDeficient { .. })
    ));
    let retained = cached(&mut cache, &factors);
    assert!(retained.reused);
    assert!(Arc::ptr_eq(&valid.decomposition, &retained.decomposition));
    assert_eq!(cache.len(), 1);
}

#[test]
fn bounded_fifo_and_disabled_retention_preserve_outstanding_results() {
    let a = [support(&[&[0], &[1]])];
    let b = [support(&[&[0], &[2]])];
    let c = [support(&[&[0], &[3]])];
    let mut cache = GeometryCache::new(2);
    assert_eq!(cache.capacity(), 2);
    let original = cached(&mut cache, &a);
    let second = cached(&mut cache, &b);
    assert!(cached(&mut cache, &a).reused);
    assert!(!cached(&mut cache, &c).reused);
    assert_eq!(cache.len(), 2);
    assert!(Arc::ptr_eq(
        &second.decomposition,
        &cached(&mut cache, &b).decomposition
    ));
    let recomputed = cached(&mut cache, &a);
    assert!(!recomputed.reused);
    same_geometry(&original.decomposition, &recomputed.decomposition);
    cache.clear();
    assert!(cache.is_empty());
    assert!(!original.decomposition.sectors.is_empty());
    let mut disabled = GeometryCache::new(0);
    assert!(!cached(&mut disabled, &a).reused);
    assert!(!cached(&mut disabled, &a).reused);
    assert!(disabled.is_empty());
}

#[test]
fn native_geometry_can_be_shared_by_a_callers_scoped_threads() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<GeometryCache>();
    send_sync::<GeometryCacheOutcome>();
    send_sync::<PolynomialSupport>();
    send_sync::<SectorMap>();
    let factors = [support(&[&[0, 0], &[1, 0], &[0, 2]])];
    let mut cache = GeometryCache::new(1);
    let result = cached(&mut cache, &factors).decomposition;
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..2)
            .map(|_| {
                let shared = Arc::clone(&result);
                scope.spawn(move || cube_moment(&shared))
            })
            .collect();
        for reader in readers {
            assert_eq!(reader.join().unwrap(), Rational::from((1, 6)));
        }
    });
}
