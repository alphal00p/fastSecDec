//! Private fault controls exercise completion precedence without admitting any
//! fabricated public geometry or changing native mathematical operations.
use super::*;
use std::ops::ControlFlow;
fn go(_: &DecompositionProgress) -> ControlFlow<()> {
    ControlFlow::Continue(())
}
fn poly(rows: Vec<Vec<i64>>) -> PolynomialSupport {
    PolynomialSupport::new(rows).unwrap()
}
fn failure<T>(result: Result<T, SectorError>) -> SectorError {
    match result {
        Err(e) => e,
        Ok(_) => panic!("expected error"),
    }
}
fn octahedron(limit: usize) -> PreparedGeometry {
    let input = poly(vec![
        vec![0, 1, 1],
        vec![2, 1, 1],
        vec![1, 0, 1],
        vec![1, 2, 1],
        vec![1, 1, 0],
        vec![1, 1, 2],
    ]);
    let p = GeometryPlan::new(
        ParametricDomain::PositiveOrthant,
        vec![input],
        DecompositionOptions {
            max_sectors: limit,
            ..Default::default()
        },
    )
    .unwrap();
    let results = p.charts().map(|j| j.run(go)).collect::<Vec<_>>();
    p.prepare(results, || false).unwrap()
}

#[test]
fn native_prefix_limit_precedes_a_later_conversion_error_in_the_same_cone() {
    for keep_prefix in [false, true] {
        let p = octahedron(2);
        let mut results = p.cones().map(|j| j.run(go)).collect::<Vec<_>>();
        let Payload::Cone { maps, .. } = &results[0].payload else {
            panic!()
        };
        assert_eq!(maps.len(), 2);
        let Payload::Cone { maps, error, .. } = &mut results[1].payload else {
            panic!()
        };
        assert_eq!(maps.len(), 2);
        maps.truncate(usize::from(keep_prefix));
        *error = Some(SectorError::Geometry(
            "injected conversion failure after native prefix".into(),
        ));
        let error = failure(p.finish(results, || false, go));
        if keep_prefix {
            assert!(matches!(
                error,
                SectorError::ResourceLimit {
                    resource: "sectors",
                    limit: 2
                }
            ));
        } else {
            assert!(
                matches!(error,SectorError::Geometry(reason) if reason.contains("injected conversion"))
            );
        }
    }
    // Native pulling fails before any simplex conversion at this limit. Do not
    // turn that error into an output-sector limit or a fabricated prefix.
    let p = octahedron(1);
    let results = p.cones().map(|j| j.run(go)).collect::<Vec<_>>();
    let Payload::Cone { maps, error, .. } = &results[0].payload else {
        panic!()
    };
    assert!(maps.is_empty());
    assert!(matches!(
        error,
        Some(SectorError::ResourceLimit {
            resource: "triangulation simplices",
            limit: 1
        })
    ));
    assert!(matches!(
        p.finish(results, || false, go),
        Err(SectorError::ResourceLimit {
            resource: "triangulation simplices",
            limit: 1
        })
    ));
}

#[test]
fn an_earlier_chart_cone_limit_precedes_a_later_chart_preparation_error() {
    let input = poly(vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1]]);
    let p = GeometryPlan::new(
        ParametricDomain::ProjectiveSimplex,
        vec![input],
        DecompositionOptions {
            max_sectors: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut charts = p.charts().map(|j| j.run(go)).collect::<Vec<_>>();
    charts[1].payload = Payload::Chart(Err(SectorError::Geometry(
        "injected later chart failure".into(),
    )));
    let prepared = p.prepare(charts, || false).unwrap();
    let jobs = prepared.cones().collect::<Vec<_>>();
    assert!(
        jobs.iter()
            .all(|j| matches!(j.id(), GeometryJobId::Cone { chart: 0, .. }))
    );
    let results = jobs.into_iter().map(|j| j.run(go)).collect::<Vec<_>>();
    assert!(matches!(
        prepared.finish(results, || false, go),
        Err(SectorError::ResourceLimit {
            resource: "sectors",
            limit: 1
        })
    ));
}

#[test]
fn private_unknown_id_is_rejected_before_payload_access() {
    let p = GeometryPlan::new(
        ParametricDomain::UnitCube,
        vec![poly(vec![vec![0], vec![1]])],
        Default::default(),
    )
    .unwrap();
    let mut result = p.charts().next().unwrap().run(go);
    result.id = GeometryJobId::Chart { chart: usize::MAX };
    assert!(matches!(
        p.prepare([result], || false),
        Err(SectorError::Work(GeometryWorkError::UnknownJob(
            GeometryJobId::Chart { chart: usize::MAX }
        )))
    ));
}

#[test]
fn prepared_chart_facets_are_computed_once_and_shared_by_all_cones() {
    let input = poly(vec![vec![1, 1, 0], vec![1, 0, 1], vec![0, 1, 1]]);
    let p = GeometryPlan::new(
        ParametricDomain::ProjectiveSimplex,
        vec![input],
        Default::default(),
    )
    .unwrap();
    crate::stages::FACET_PREPARATIONS.with(|count| count.set(0));
    let charts = p.charts().map(|job| job.run(go)).collect::<Vec<_>>();
    assert_eq!(charts.len(), 3);
    crate::stages::FACET_PREPARATIONS.with(|count| assert_eq!(count.get(), 3));
    let prepared = p.prepare(charts, || false).unwrap();
    let mut jobs = prepared.cones().collect::<Vec<_>>();
    assert!(jobs.len() > 3);
    jobs.reverse();
    let results = jobs.into_iter().map(|job| job.run(go)).collect::<Vec<_>>();
    let fan = prepared.finish(results, || false, go).unwrap();
    assert!(!fan.sectors.is_empty());
    crate::stages::FACET_PREPARATIONS.with(|count| assert_eq!(count.get(), 3));
}
