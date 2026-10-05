use fastsecdec_sectors::{
    Decomposition, DecompositionOptions, DecompositionPhase, GeometryCompletion, GeometryJob,
    GeometryJobId, GeometryPlan, GeometryWorkError, ParametricDomain, PolynomialSupport,
    PreparedGeometry, SectorError, SectorMap, decompose,
};
use numerica::domains::{
    integer::Integer,
    rational::{Q, Rational},
};
use std::ops::ControlFlow;

fn support(rows: &[&[i64]]) -> PolynomialSupport {
    PolynomialSupport::new(rows.iter().map(|row| row.to_vec()).collect()).unwrap()
}
fn go(_: &fastsecdec_sectors::DecompositionProgress) -> ControlFlow<()> {
    ControlFlow::Continue(())
}
fn plan(
    domain: ParametricDomain,
    supports: Vec<PolynomialSupport>,
    options: DecompositionOptions,
) -> GeometryPlan {
    GeometryPlan::new(domain, supports, options).unwrap()
}

// The crate supplies no pool. This test caller chooses scoped threads, at most
// two jobs in flight, and deliberately returns each batch in reverse order.
fn execute(jobs: impl Iterator<Item = GeometryJob>, workers: usize) -> Vec<GeometryCompletion> {
    let mut jobs = jobs.peekable();
    let mut results = Vec::new();
    while jobs.peek().is_some() {
        let batch = jobs.by_ref().take(workers).collect::<Vec<_>>();
        std::thread::scope(|scope| {
            let handles = batch
                .into_iter()
                .map(|job| scope.spawn(move || job.run(go)))
                .collect::<Vec<_>>();
            for handle in handles.into_iter().rev() {
                results.push(handle.join().unwrap());
            }
        });
    }
    results.reverse();
    results
}
fn run(
    domain: ParametricDomain,
    supports: Vec<PolynomialSupport>,
    options: DecompositionOptions,
    workers: usize,
) -> Result<Decomposition, SectorError> {
    let plan = GeometryPlan::new(domain, supports, options)?;
    let charts = execute(plan.charts(), workers);
    let prepared = plan.prepare(charts, || false)?;
    let cones = execute(prepared.cones(), workers);
    let mut final_events = Vec::new();
    let result = prepared.finish(
        cones,
        || false,
        |status| {
            final_events.push(status.clone());
            ControlFlow::Continue(())
        },
    )?;
    assert_eq!(final_events.len(), 1);
    assert_eq!(final_events[0].phase, DecompositionPhase::Complete);
    assert_eq!(final_events[0].sectors, result.sectors.len());
    Ok(result)
}
fn same(a: &Decomposition, b: &Decomposition) {
    assert_eq!(a.domain, b.domain);
    assert_eq!(a.sectors, b.sectors);
    assert_eq!(a.candidate_vertices, b.candidate_vertices);
    assert_eq!(a.geometric_vertices, b.geometric_vertices);
}
fn error<T>(result: Result<T, SectorError>) -> SectorError {
    match result {
        Err(e) => e,
        Ok(_) => panic!("expected error"),
    }
}

#[test]
fn caller_threads_preserve_all_native_maps_domains_and_order() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<GeometryPlan>();
    send_sync::<PreparedGeometry>();
    send_sync::<GeometryJob>();
    send_sync::<GeometryCompletion>();
    send_sync::<SectorMap>();
    let mut cross = Vec::new();
    for axis in 0..4 {
        for offset in [-1, 1] {
            let mut row = vec![1; 4];
            row[axis] += offset;
            cross.push(row);
        }
    }
    let cross = PolynomialSupport::new(cross).unwrap();
    let cases = vec![
        (
            ParametricDomain::UnitCube,
            vec![support(&[&[1, 0], &[0, 1]])],
        ),
        (ParametricDomain::UnitCube, vec![cross.clone()]),
        (ParametricDomain::PositiveOrthant, vec![cross]),
        (
            ParametricDomain::PositiveOrthant,
            vec![support(&[&[0, 0], &[1, 0], &[0, 1]])],
        ),
        (
            ParametricDomain::ProjectiveSimplex,
            vec![
                support(&[&[2, 0, 0], &[0, 2, 0], &[0, 0, 2]]),
                support(&[&[1, 1, 0], &[1, 0, 1], &[0, 1, 1]]),
            ],
        ),
        (ParametricDomain::UnitCube, vec![support(&[&[]])]),
        (ParametricDomain::PositiveOrthant, vec![support(&[&[]])]),
        (ParametricDomain::ProjectiveSimplex, vec![support(&[&[3]])]),
    ];
    for (domain, supports) in cases {
        let serial = decompose(domain, &supports, &Default::default(), go).unwrap();
        for workers in [1, 2] {
            same(
                &serial,
                &run(domain, supports.clone(), Default::default(), workers).unwrap(),
            );
        }
    }
}

#[test]
fn exact_cube_moment_valuations_and_infinity_identity_are_preserved() {
    let a = support(&[&[0, 0], &[1, 0], &[0, 2]]);
    let shifted = support(&[&[3, 1], &[4, 1], &[3, 3]]);
    let b = support(&[&[1, 0], &[0, 1]]);
    let base = run(
        ParametricDomain::UnitCube,
        vec![a, b.clone()],
        Default::default(),
        2,
    )
    .unwrap();
    let translated = run(
        ParametricDomain::UnitCube,
        vec![shifted, b],
        Default::default(),
        2,
    )
    .unwrap();
    for geometry in [&base, &translated] {
        let moment = geometry.sectors.iter().fold(Rational::from(0), |sum, map| {
            let denominator = (0..2).fold(Integer::from(1), |p, j| {
                p * (&map.exponent_matrix[0][j] * 3 + &map.exponent_matrix[1][j] * 2)
            });
            sum + Q.to_element(map.determinant.clone(), denominator, true)
        });
        assert_eq!(moment, Rational::from((1, 6)));
    }
    for (a, b) in base.sectors.iter().zip(&translated.sectors) {
        assert_eq!(a.exponent_matrix, b.exponent_matrix);
        for axis in 0..2 {
            assert_eq!(
                b.factor_valuations[0][axis],
                &a.factor_valuations[0][axis]
                    + &a.exponent_matrix[0][axis] * 3
                    + &a.exponent_matrix[1][axis]
            );
        }
    }
    let infinity = run(
        ParametricDomain::PositiveOrthant,
        vec![support(&[&[0], &[1]])],
        Default::default(),
        2,
    )
    .unwrap();
    assert_eq!(infinity.sectors.len(), 2);
    // Each signed chart maps dx/(1+x)^2 to dt/(1+t)^2, integral 1/2.
    let mut powers = Vec::new();
    for map in &infinity.sectors {
        assert_eq!(map.determinant, Integer::from(1));
        let m = map.exponent_matrix[0][0].to_i64().unwrap();
        let v = map.factor_valuations[0][0].to_i64().unwrap();
        assert_eq!(map.jacobian_powers[0].to_i64().unwrap() - 2 * v, 0);
        powers.push(m);
    }
    powers.sort();
    assert_eq!(powers, vec![-1, 1]);
}

#[test]
fn all_native_limits_and_admission_errors_remain_failures() {
    let factors = vec![support(&[&[1, 0], &[0, 1]])];
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
        DecompositionOptions {
            max_sectors: 1,
            ..Default::default()
        },
    ] {
        let native = error(decompose(
            ParametricDomain::UnitCube,
            &factors,
            &options,
            go,
        ));
        let parallel = error(run(ParametricDomain::UnitCube, factors.clone(), options, 2));
        assert_eq!(native.to_string(), parallel.to_string());
    }
    let oct = support(&[
        &[0, 1, 1],
        &[2, 1, 1],
        &[1, 0, 1],
        &[1, 2, 1],
        &[1, 1, 0],
        &[1, 1, 2],
    ]);
    for limit in [0, 1, 2, 3, 11] {
        let options = DecompositionOptions {
            max_sectors: limit,
            ..Default::default()
        };
        let native = error(decompose(
            ParametricDomain::PositiveOrthant,
            std::slice::from_ref(&oct),
            &options,
            go,
        ));
        let parallel = error(run(
            ParametricDomain::PositiveOrthant,
            vec![oct.clone()],
            options,
            2,
        ));
        assert_eq!(native.to_string(), parallel.to_string());
    }
    for (domain, factors) in [
        (ParametricDomain::UnitCube, vec![]),
        (ParametricDomain::ProjectiveSimplex, vec![support(&[&[]])]),
        (
            ParametricDomain::ProjectiveSimplex,
            vec![support(&[&[0], &[1]])],
        ),
        (
            ParametricDomain::UnitCube,
            vec![support(&[&[0]]), support(&[&[0, 0]])],
        ),
        (ParametricDomain::PositiveOrthant, factors),
    ] {
        let native = error(decompose(domain, &factors, &Default::default(), go));
        let parallel = error(run(domain, factors, Default::default(), 2));
        assert_eq!(native.to_string(), parallel.to_string());
    }
}

fn cube_plan() -> GeometryPlan {
    plan(
        ParametricDomain::UnitCube,
        vec![support(&[&[1, 0], &[0, 1]])],
        Default::default(),
    )
}
fn ready() -> PreparedGeometry {
    let p = cube_plan();
    let done = p.charts().map(|j| j.run(go)).collect::<Vec<_>>();
    p.prepare(done, || false).unwrap()
}

#[test]
fn completion_admission_rejects_missing_duplicate_foreign_and_wrong_stage() {
    assert!(matches!(
        error(cube_plan().prepare([], || false)),
        SectorError::Work(GeometryWorkError::MissingJobs { count: 1, .. })
    ));
    let p = cube_plan();
    let job = p.charts().next().unwrap();
    assert!(matches!(
        error(p.prepare([job.run(go), job.run(go)], || false)),
        SectorError::Work(GeometryWorkError::DuplicateJob(_))
    ));
    let foreign = cube_plan().charts().next().unwrap().run(go);
    assert!(matches!(
        error(cube_plan().prepare([foreign], || false)),
        SectorError::Work(GeometryWorkError::ForeignPlan)
    ));
    let p = ready();
    let jobs = p.cones().collect::<Vec<_>>();
    assert_eq!(jobs.len(), 2);
    assert!(matches!(
        error(p.finish([jobs[0].run(go)], || false, go)),
        SectorError::Work(GeometryWorkError::MissingJobs { count: 1, .. })
    ));
    let p = ready();
    let j = p.cones().next().unwrap();
    assert!(matches!(
        error(p.finish([j.run(go), j.run(go)], || false, go)),
        SectorError::Work(GeometryWorkError::DuplicateJob(_))
    ));
    // Independently prepared equal-input stages cannot exchange cone results.
    let foreign = ready().cones().next().unwrap().run(go);
    assert!(matches!(
        error(ready().finish([foreign], || false, go)),
        SectorError::Work(GeometryWorkError::ForeignPlan)
    ));
    let chart = cube_plan().charts().next().unwrap().run(go);
    assert!(matches!(
        error(ready().finish([chart], || false, go)),
        SectorError::Work(GeometryWorkError::WrongStage)
    ));
    let cone = ready().cones().next().unwrap().run(go);
    assert!(matches!(
        error(cube_plan().prepare([cone], || false)),
        SectorError::Work(GeometryWorkError::WrongStage)
    ));
}

#[test]
fn cancellation_at_worker_stage_merge_and_final_observer_never_returns_a_fan() {
    assert!(matches!(
        error(cube_plan().prepare([], || true)),
        SectorError::Cancelled
    ));
    let p = cube_plan();
    let stopped = p.charts().next().unwrap().run(|_| ControlFlow::Break(()));
    assert!(matches!(stopped.error(), Some(SectorError::Cancelled)));
    assert!(matches!(
        error(p.prepare([stopped], || false)),
        SectorError::Cancelled
    ));
    let p = ready();
    let stopped = p.cones().next().unwrap().run(|_| ControlFlow::Break(()));
    assert!(matches!(
        error(p.finish([stopped], || false, go)),
        SectorError::Cancelled
    ));
    let p = ready();
    let completed = p.cones().map(|j| j.run(go)).collect::<Vec<_>>();
    let count = completed.len();
    let mut polls = 0;
    let mut callbacks = 0;
    let result = p.finish(
        completed,
        || {
            polls += 1;
            polls > count + 5
        },
        |_| {
            callbacks += 1;
            ControlFlow::Continue(())
        },
    );
    assert!(matches!(result, Err(SectorError::Cancelled)));
    assert_eq!(callbacks, 0);
    let p = ready();
    let completed = p.cones().map(|j| j.run(go)).collect::<Vec<_>>();
    let mut final_count = 0;
    assert!(matches!(
        p.finish(
            completed,
            || false,
            |s| {
                assert_eq!(s.phase, DecompositionPhase::Complete);
                assert_eq!(s.sectors, 2);
                final_count += 1;
                ControlFlow::Break(())
            }
        ),
        Err(SectorError::Cancelled)
    ));
    assert_eq!(final_count, 1);
}

#[test]
fn progress_distinguishes_prepared_facets_local_cones_and_serial_trivial_charts() {
    let p = cube_plan();
    let mut facet_jobs = 0;
    let completed = p
        .charts()
        .map(|job| {
            let mut facet_seen = false;
            let result = job.run(|s| {
                facet_seen |= s.phase == DecompositionPhase::Facets;
                assert_ne!(s.phase, DecompositionPhase::Complete);
                ControlFlow::Continue(())
            });
            facet_jobs += usize::from(facet_seen);
            result
        })
        .collect::<Vec<_>>();
    assert_eq!(facet_jobs, 1);
    let p = p.prepare(completed, || false).unwrap();
    let completed = p
        .cones()
        .map(|job| {
            assert!(matches!(job.id(), GeometryJobId::Cone { chart: 0, .. }));
            job.run(|s| {
                assert_eq!(s.phase, DecompositionPhase::Triangulation);
                ControlFlow::Continue(())
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(p.finish(completed, || false, go).unwrap().sectors.len(), 2);
    for (domain, supports) in [
        (ParametricDomain::UnitCube, vec![support(&[&[]])]),
        (ParametricDomain::ProjectiveSimplex, vec![support(&[&[3]])]),
    ] {
        let mut trace = Vec::new();
        decompose(domain, &supports, &Default::default(), |s| {
            trace.push((
                s.phase,
                s.chart,
                s.completed_constraints,
                s.total_constraints,
                s.rays,
                s.sectors,
            ));
            ControlFlow::Continue(())
        })
        .unwrap();
        assert_eq!(
            trace,
            vec![
                (DecompositionPhase::Supports, 0, 0, 0, 0, 0),
                (DecompositionPhase::Complete, 0, 0, 0, 0, 1)
            ]
        );
        let p = plan(domain, supports, Default::default());
        let done = p.charts().map(|j| j.run(go)).collect::<Vec<_>>();
        let p = p.prepare(done, || false).unwrap();
        assert_eq!(p.cones().len(), 0);
        assert_eq!(p.finish([], || false, go).unwrap().sectors.len(), 1);
    }
}
