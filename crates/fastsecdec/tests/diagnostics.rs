use std::{collections::BTreeSet, ops::ControlFlow};

use fastsecdec::{
    Atom,
    diagnostics::{
        BenchmarkOptions, BoundaryOptions, BoundarySide, DiagnosticProgress, DiagnosticStop,
        benchmark, boundaries,
    },
    generation::{GenerationOptions, generate},
    kernel::KernelSet,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{parse, symbol};

fn kernels() -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![
            symbol!("diagnostic_test::x"),
            symbol!("diagnostic_test::y"),
            symbol!("diagnostic_test::z"),
        ],
        symbol!("diagnostic_test::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0); 3],
            vec![PolynomialFactor::new(
                parse!("1+diagnostic_test::x+diagnostic_test::y+diagnostic_test::z"),
                Atom::num(1),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    let generated = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let kernels = generated.compile().unwrap();
    assert_eq!(kernels.sectors().len(), 1);
    assert_eq!(kernels.sectors()[0].dimension(), 3);
    kernels
}

#[test]
fn boundary_coverage_includes_every_requested_face_intersection() {
    let report = boundaries(
        &mut kernels(),
        &BoundaryOptions {
            exponents: vec![3],
            max_codimension: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    // Cube has six faces, twelve edges, eight vertices. Codimension2 includes
    // every edge, including mixed upper/lower pairs omitted by old CLI patterns.
    assert_eq!(report.probes.len(), 18);
    assert_eq!(report.coverage.configured_probes, Some(18));
    assert_eq!(report.coverage.all_face_pattern_probes, Some(26));
    assert!(!report.coverage.includes_all_face_patterns);
    assert_eq!(report.failures, 0);
    let unique = report
        .probes
        .iter()
        .map(|p| p.endpoints.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 18);
    for a in 0..3 {
        for b in a + 1..3 {
            for first in [BoundarySide::Lower, BoundarySide::Upper] {
                for second in [BoundarySide::Lower, BoundarySide::Upper] {
                    assert!(report.probes.iter().any(|p| p.endpoints.len() == 2
                        && p.endpoints[0].axis == a
                        && p.endpoints[0].side == first
                        && p.endpoints[1].axis == b
                        && p.endpoints[1].side == second));
                }
            }
        }
    }
    for probe in &report.probes {
        assert!(probe.point.iter().all(|x| *x > 0.0 && *x < 1.0));
        // Sector axes may be permuted relative to the original parameters.
        let expected = 1.0 + probe.point.iter().sum::<f64>();
        assert!(
            (probe.max_absolute_value.unwrap() - expected).abs() < 1e-12,
            "point {:?}: observed {:?}, expected {}",
            probe.point,
            probe.max_absolute_value,
            expected
        );
    }
    let all = boundaries(
        &mut kernels(),
        &BoundaryOptions {
            exponents: vec![3],
            max_codimension: 3,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(all.probes.len(), 26);
    assert!(all.coverage.includes_all_face_patterns);
    assert_eq!(all.stop, DiagnosticStop::Complete);
}

#[test]
fn boundary_budget_and_cancellation_keep_completed_rows() {
    let options = BoundaryOptions {
        exponents: vec![3, 6],
        max_codimension: 3,
        max_probes: 7,
        ..Default::default()
    };
    let budget = boundaries(&mut kernels(), &options, |_| ControlFlow::Continue(())).unwrap();
    assert_eq!(budget.probes.len(), 7);
    assert_eq!(budget.coverage.configured_probes, Some(52));
    assert_eq!(budget.stop, DiagnosticStop::ProbeBudget);
    assert!(budget.coverage.truncated_by_budget);
    assert!(!budget.coverage.includes_all_face_patterns);
    let mut streamed = Vec::new();
    let cancelled = boundaries(&mut kernels(), &options, |event| {
        if let DiagnosticProgress::Boundary {
            completed,
            probe: Some(probe),
            ..
        } = event
        {
            streamed.push(probe.point.clone());
            if *completed == 3 {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    })
    .unwrap();
    assert_eq!(cancelled.stop, DiagnosticStop::Cancelled);
    assert_eq!(cancelled.coverage.completed_probes, 3);
    assert_eq!(
        cancelled
            .probes
            .iter()
            .map(|p| p.point.clone())
            .collect::<Vec<_>>(),
        streamed
    );
    assert_eq!(cancelled.diagnostics.evaluations, 3);
}

#[test]
fn requests_reject_unrepresentable_endpoints_and_invalid_counts() {
    let mut kernels = kernels();
    for exponents in [
        vec![],
        vec![0],
        vec![-1],
        vec![3, 3],
        vec![17],
        vec![i32::MIN],
        vec![i32::MAX],
    ] {
        assert!(
            boundaries(
                &mut kernels,
                &BoundaryOptions {
                    exponents,
                    ..Default::default()
                },
                |_| panic!("invalid request reached callback")
            )
            .is_err()
        );
    }
    for options in [
        BenchmarkOptions {
            points: 0,
            ..Default::default()
        },
        BenchmarkOptions {
            batch_size: 0,
            ..Default::default()
        },
        BenchmarkOptions {
            points: usize::MAX,
            ..Default::default()
        },
    ] {
        assert!(
            benchmark(&mut kernels, &options, |_| panic!(
                "invalid request reached callback"
            ))
            .is_err()
        );
    }
}

#[test]
fn benchmark_cancels_inside_repetition_and_preserves_complete_median() {
    let report = benchmark(
        &mut kernels(),
        &BenchmarkOptions {
            points: 20,
            repetitions: 3,
            warmup: 2,
            batch_size: 4,
        },
        |event| match event {
            DiagnosticProgress::Benchmark {
                repetition: Some(1),
                completed_evaluations: 8,
                ..
            } => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        },
    )
    .unwrap();
    assert_eq!(report.stop, DiagnosticStop::Cancelled);
    let row = &report.sectors[0];
    assert_eq!(row.warmup_evaluations, 2);
    assert_eq!(row.measurements.len(), 2);
    assert!(row.measurements[0].complete);
    assert_eq!(row.measurements[0].evaluations, 20);
    assert!(!row.measurements[1].complete);
    assert_eq!(row.measurements[1].evaluations, 8);
    assert_eq!(row.median_seconds, Some(row.measurements[0].seconds));
    assert_eq!(row.diagnostics.evaluations, 30);
    assert!(serde_json::to_value(&report).unwrap().is_object());
}

#[test]
fn immediate_cancellation_performs_no_evaluations() {
    let boundary = boundaries(&mut kernels(), &BoundaryOptions::default(), |_| {
        ControlFlow::Break(())
    })
    .unwrap();
    assert_eq!(boundary.stop, DiagnosticStop::Cancelled);
    assert!(boundary.probes.is_empty());
    let benchmark = benchmark(&mut kernels(), &BenchmarkOptions::default(), |_| {
        ControlFlow::Break(())
    })
    .unwrap();
    assert_eq!(benchmark.stop, DiagnosticStop::Cancelled);
    assert_eq!(benchmark.sectors[0].diagnostics.evaluations, 0);
    assert!(benchmark.sectors[0].median_seconds.is_none());
}
