use std::ops::ControlFlow;

use fastsecdec::{
    Atom,
    diagnostics::{
        AxisEndpoint, BoundaryAssessment, BoundaryCoverage, BoundaryGrowthEstimate,
        BoundaryGrowthOptions, BoundaryGrowthUnavailable, BoundaryOptions, BoundaryProbe,
        BoundaryReport, BoundaryScanAttempt, BoundaryScanOptions, BoundaryScanProgress,
        BoundaryScanReport, BoundarySector, BoundarySide, DiagnosticProgress, DiagnosticStop,
        analyze_boundary_growth, boundaries, scan_boundaries,
    },
    generation::{GenerationOptions, generate},
    kernel::KernelSet,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
    status::{CoefficientComponent, EvaluationDiagnostics},
};
use symbolica::{parse, symbol};

// Pure numeric fixtures intentionally do not initialize Symbolica. Both upper
// and lower face families are present, so a passing assessment has full coverage.
fn physical_samples(exponents: Vec<i32>, value: impl Fn(f64) -> Vec<f64>) -> BoundaryReport {
    let mut probes = Vec::new();
    for side in [BoundarySide::Lower, BoundarySide::Upper] {
        for &exponent in &exponents {
            let distance = 10f64.powi(-exponent);
            let x = match side {
                BoundarySide::Lower => distance,
                BoundarySide::Upper => 1.0 - distance,
            };
            let actual = match side {
                BoundarySide::Lower => x,
                BoundarySide::Upper => 1.0 - x,
            };
            let values = value(actual);
            probes.push(BoundaryProbe {
                sector: 7,
                endpoints: vec![AxisEndpoint { axis: 0, side }],
                exponent,
                distance,
                point: vec![x],
                finite: true,
                conditioning_checked: false,
                rescued: false,
                precision_bits: Some(53),
                max_absolute_value: Some(values.iter().map(|v| v.abs()).fold(0.0, f64::max)),
                values: Some(values),
                error: None,
            });
        }
    }
    let outputs = probes[0].values.as_ref().unwrap().len();
    let completed = probes.len();
    BoundaryReport {
        options: BoundaryOptions {
            exponents,
            max_codimension: 1,
            ..Default::default()
        },
        distance_scale: 1.0,
        orders: (0..outputs as i32).collect(),
        components: vec![CoefficientComponent::Real; outputs],
        sectors: vec![BoundarySector {
            sector: 7,
            dimension: 1,
        }],
        probes,
        coverage: BoundaryCoverage {
            max_codimension: 1,
            configured_probes: Some(completed as u64),
            all_face_pattern_probes: Some(completed as u64),
            planned_probes: completed,
            completed_probes: completed,
            truncated_by_budget: false,
            includes_all_face_patterns: true,
        },
        failures: 0,
        diagnostics: EvaluationDiagnostics::default(),
        stop: DiagnosticStop::Complete,
    }
}

#[test]
fn numeric_imported_attempt_index_cannot_overflow_display() {
    let samples = physical_samples(vec![3, 6], |_| vec![1.0]);
    let growth = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    let report = BoundaryScanReport {
        options: BoundaryScanOptions::default(),
        attempts: vec![BoundaryScanAttempt {
            index: 0,
            scale: 1.0,
            selected_sectors: vec![7],
            samples,
            growth,
        }],
        latest: Vec::new(),
        had_prior_flags: false,
        completed_probes: 4,
        diagnostics: EvaluationDiagnostics::default(),
        stop: DiagnosticStop::Complete,
        assessment: BoundaryAssessment::WithinThreshold,
    };
    let mut json = serde_json::to_value(report).unwrap();
    json["attempts"][0]["index"] = serde_json::json!(usize::MAX);
    let imported: BoundaryScanReport = serde_json::from_value(json).unwrap();
    assert!(imported.to_string().contains("invalid-attempt"));
}

#[test]
fn numeric_actual_upper_distances_and_component_growth_are_preserved() {
    let mut samples = physical_samples(vec![14, 15], |distance| vec![1e100, distance.powf(-0.75)]);
    samples.orders = vec![0, 0];
    samples.components[1] = CoefficientComponent::Imag;
    let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    assert_eq!(report.orders, vec![0, 0]);
    assert_eq!(report.components, samples.components);
    assert_eq!(report.assessment, BoundaryAssessment::Flagged);
    assert!(report.sectors[0].sampling_complete);
    for pair in &report.pairs {
        let BoundaryGrowthEstimate::Power(power) = pair.estimate else {
            panic!("{pair:?}")
        };
        if pair.component_index == 0 {
            assert_eq!(power, 0.0);
            assert_eq!(pair.assessment, BoundaryAssessment::WithinThreshold);
        } else {
            assert!((power - 0.75).abs() < 2e-12, "{pair:?}");
            assert_eq!(pair.assessment, BoundaryAssessment::Flagged);
        }
    }
    // A tau=0.5 policy intentionally flags x^-3/4 although its integral exists.
}

#[test]
fn numeric_log_differences_handle_extreme_finite_value_ranges() {
    let samples = physical_samples(vec![3, 6], |d| vec![if d > 1e-4 { 1e-300 } else { 1e300 }]);
    let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    let lower = report
        .pairs
        .iter()
        .find(|p| p.endpoints[0].side == BoundarySide::Lower)
        .unwrap();
    let BoundaryGrowthEstimate::Power(power) = lower.estimate else {
        panic!("{lower:?}")
    };
    assert!((power - 200.0).abs() < 1e-12);
    assert!(
        report
            .pairs
            .iter()
            .all(|p| matches!(p.estimate, BoundaryGrowthEstimate::Power(v) if v.is_finite()))
    );
}

#[test]
fn numeric_zeros_and_emergence_have_distinct_outcomes() {
    let samples = physical_samples(vec![2, 4], |d| {
        vec![
            0.0,
            if d > 1e-3 { 1.0 } else { 0.0 },
            if d > 1e-3 { 0.0 } else { 1.0 },
        ]
    });
    let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    for p in &report.pairs {
        match p.component_index {
            0 => assert_eq!(p.estimate, BoundaryGrowthEstimate::BothZero),
            1 => assert_eq!(p.estimate, BoundaryGrowthEstimate::DecreasesToZero),
            2 => {
                assert_eq!(p.estimate, BoundaryGrowthEstimate::EmergesFromZero);
                assert_eq!(p.assessment, BoundaryAssessment::Inconclusive)
            }
            _ => unreachable!(),
        }
    }
    assert_eq!(report.assessment, BoundaryAssessment::Inconclusive);
}

#[test]
fn numeric_missing_or_failed_intermediate_observations_are_never_bridged() {
    let original = physical_samples(vec![2, 3, 4], |_| vec![1.0]);
    for failed in [false, true] {
        let mut samples = original.clone();
        let index = samples
            .probes
            .iter()
            .position(|p| p.exponent == 3 && p.endpoints[0].side == BoundarySide::Lower)
            .unwrap();
        let expected = if failed {
            let probe = &mut samples.probes[index];
            probe.finite = false;
            probe.values = None;
            probe.max_absolute_value = None;
            probe.error = Some("deliberate failed sample".into());
            BoundaryGrowthUnavailable::EvaluationFailed
        } else {
            samples.probes.remove(index);
            samples.coverage.completed_probes -= 1;
            BoundaryGrowthUnavailable::MissingProbe
        };
        let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
        let lower = report
            .pairs
            .iter()
            .filter(|p| p.endpoints[0].side == BoundarySide::Lower)
            .collect::<Vec<_>>();
        assert_eq!(lower.len(), 2);
        assert!(
            lower
                .iter()
                .all(|p| p.estimate == BoundaryGrowthEstimate::Unavailable(expected))
        );
        assert_eq!(report.assessment, BoundaryAssessment::Inconclusive);
        assert_eq!(report.sectors[0].sampling_complete, failed);
    }
}

#[test]
fn numeric_metadata_and_old_value_rows_are_not_fabricated() {
    let original = physical_samples(vec![3, 6], |_| vec![1.0]);
    let mut old_values = original.clone();
    for probe in &mut old_values.probes {
        probe.values = None;
    }
    let report = analyze_boundary_growth(&old_values, &BoundaryGrowthOptions::default()).unwrap();
    assert!(report.pairs.iter().all(|p| p.estimate
        == BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::MissingValues)));
    old_values.orders.clear();
    old_values.components.clear();
    old_values.sectors.clear();
    assert!(analyze_boundary_growth(&old_values, &BoundaryGrowthOptions::default()).is_err());
    let mut invalid = original;
    invalid.probes[0].values = Some(vec![f64::INFINITY]);
    assert!(analyze_boundary_growth(&invalid, &BoundaryGrowthOptions::default()).is_err());
}

#[test]
fn numeric_collapsed_actual_distances_do_not_produce_a_slope() {
    let mut samples = physical_samples(vec![3, 6], |_| vec![1.0]);
    samples.probes[1].point = samples.probes[0].point.clone();
    let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    let lower = report
        .pairs
        .iter()
        .find(|p| p.endpoints[0].side == BoundarySide::Lower)
        .unwrap();
    assert_eq!(
        lower.estimate,
        BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::InvalidDistance)
    );
    assert_eq!(report.assessment, BoundaryAssessment::Inconclusive);
}

fn compile(terms: Vec<ParametricTerm>, max_order: i32) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("growth_test::x"), symbol!("growth_test::y")],
        symbol!("growth_test::eps"),
        ParametricDomain::UnitCube,
        terms,
    )
    .unwrap();
    generate(
        &input,
        &GenerationOptions {
            max_order,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap()
    .compile()
    .unwrap()
}

fn crossover() -> KernelSet {
    compile(
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0); 2],
            vec![PolynomialFactor::new(
                parse!("growth_test::x+1/10000000"),
                parse!("-3/4"),
                FactorRole::Singularity,
            )],
        )],
        0,
    )
}

#[test]
fn sampled_physical_components_and_codimension_threshold_keep_different_meanings() {
    let mut kernels = compile(
        vec![
            ParametricTerm::new(
                Atom::num(1_000_000_000_000i64),
                vec![Atom::num(0); 2],
                vec![],
            ),
            ParametricTerm::new(
                parse!("growth_test::eps"),
                vec![parse!("-3/4"), Atom::num(0)],
                vec![],
            ),
        ],
        1,
    );
    let samples = boundaries(
        &mut kernels,
        &BoundaryOptions {
            exponents: vec![6, 8],
            max_codimension: 2,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert!(
        samples
            .probes
            .iter()
            .all(|p| p.values.as_ref().unwrap().len() == 2)
    );
    assert!(
        samples
            .probes
            .iter()
            .all(|p| p.max_absolute_value == Some(1e12))
    );
    let report = analyze_boundary_growth(&samples, &BoundaryGrowthOptions::default()).unwrap();
    assert!(report.pairs.iter().any(|p| p.component_index == 1
        && p.endpoints.len() == 1
        && p.assessment == BoundaryAssessment::Flagged));
    assert!(
        report
            .pairs
            .iter()
            .all(|p| p.component_index != 0 || p.assessment == BoundaryAssessment::WithinThreshold)
    );
    assert!(
        report
            .pairs
            .iter()
            .filter(|p| p.endpoints.len() == 2)
            .all(|p| p.assessment == BoundaryAssessment::WithinThreshold)
    );
}

#[test]
fn scaled_retries_keep_initial_flags_and_stop_after_resolved_growth() {
    let report = scan_boundaries(
        &mut crossover(),
        &BoundaryScanOptions {
            sampling: BoundaryOptions {
                exponents: vec![4, 6],
                max_codimension: 1,
                ..Default::default()
            },
            retry_scales: vec![0.01, 0.0001],
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(report.attempts.len(), 2);
    assert_eq!(
        report.attempts[0].growth.assessment,
        BoundaryAssessment::Flagged
    );
    assert_eq!(
        report.attempts[1].growth.assessment,
        BoundaryAssessment::WithinThreshold
    );
    assert_eq!(report.assessment, BoundaryAssessment::WithinThreshold);
    assert!(report.had_prior_flags);
    assert_eq!(report.stop, DiagnosticStop::Complete);
    assert_eq!(report.completed_probes, 16);
    assert_eq!(report.diagnostics.evaluations, 16);
    assert_eq!(report.attempts[1].samples.distance_scale, 0.01);
    assert!(
        report
            .to_string()
            .contains("not an integrability certificate")
    );
}

#[test]
fn retry_budget_keeps_untouched_selected_sector_metadata_and_cancellation_rows() {
    let mut kernels = compile(
        vec![ParametricTerm::new(
            Atom::num(1),
            vec![Atom::num(0); 2],
            vec![
                PolynomialFactor::new(
                    parse!("growth_test::x+growth_test::y"),
                    parse!("-7/4"),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(
                    parse!("1+growth_test::x"),
                    Atom::num(1),
                    FactorRole::Polynomial,
                ),
            ],
        )],
        0,
    );
    assert!(kernels.sectors().len() >= 2);
    let first_count = kernels.sectors().len() * 8;
    let options = BoundaryScanOptions {
        sampling: BoundaryOptions {
            exponents: vec![4, 6],
            max_codimension: 1,
            max_probes: first_count + 1,
            ..Default::default()
        },
        retry_scales: vec![0.01],
        ..Default::default()
    };
    let report = scan_boundaries(&mut kernels, &options, |_| ControlFlow::Continue(())).unwrap();
    assert_eq!(report.stop, DiagnosticStop::ProbeBudget);
    assert_eq!(report.attempts.len(), 2);
    assert_eq!(report.completed_probes, first_count + 1);
    let retry = &report.attempts[1];
    assert!(retry.selected_sectors.len() >= 2);
    let untouched = retry
        .selected_sectors
        .iter()
        .find(|s| retry.samples.probes.iter().all(|p| p.sector != **s))
        .unwrap();
    let latest = report
        .latest
        .iter()
        .find(|s| s.sector.sector == *untouched)
        .unwrap();
    assert_eq!(latest.attempt, 1);
    assert_eq!(latest.sector.assessment, BoundaryAssessment::Inconclusive);
    assert_eq!(latest.sector.observed_pairs, 0);
    assert!(!latest.sector.sampling_complete);
    assert!(report.had_prior_flags);
    let mut calls = 0;
    let cancelled = scan_boundaries(&mut kernels, &options, |_| {
        calls += 1;
        ControlFlow::Break(())
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(cancelled.stop, DiagnosticStop::Cancelled);
    assert_eq!(cancelled.completed_probes, 0);
    assert_eq!(cancelled.latest.len(), kernels.sectors().len());
    assert!(
        cancelled
            .latest
            .iter()
            .all(|s| s.sector.assessment == BoundaryAssessment::Inconclusive)
    );
    let cancelled = scan_boundaries(&mut kernels, &options, |event| {
        if matches!(
            event,
            BoundaryScanProgress::Sampling {
                attempt: 1,
                progress: DiagnosticProgress::Boundary { completed: 1, .. },
                ..
            }
        ) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .unwrap();
    assert_eq!(cancelled.stop, DiagnosticStop::Cancelled);
    assert_eq!(cancelled.completed_probes, first_count + 1);
    assert_eq!(cancelled.attempts[1].samples.probes.len(), 1);
}

#[test]
fn retry_distances_are_preflighted_before_any_evaluation() {
    let mut kernels = crossover();
    for scales in [vec![0.01], vec![0.5, 0.7], vec![0.0], vec![f64::NAN]] {
        assert!(
            scan_boundaries(
                &mut kernels,
                &BoundaryScanOptions {
                    sampling: BoundaryOptions {
                        exponents: vec![14, 15],
                        ..Default::default()
                    },
                    retry_scales: scales,
                    ..Default::default()
                },
                |_| panic!("invalid retry reached evaluator")
            )
            .is_err()
        );
    }
    for growth in [
        BoundaryGrowthOptions {
            max_power_per_axis: -1.0,
            ..Default::default()
        },
        BoundaryGrowthOptions {
            numerical_slack: f64::INFINITY,
            ..Default::default()
        },
    ] {
        assert!(
            scan_boundaries(
                &mut kernels,
                &BoundaryScanOptions {
                    growth,
                    ..Default::default()
                },
                |_| panic!("invalid growth policy reached evaluator")
            )
            .is_err()
        );
    }
}

#[test]
fn analytically_constant_problem_has_no_stochastic_boundary_assessment() {
    let mut kernels = compile(
        vec![ParametricTerm::new(
            Atom::num(3),
            vec![Atom::num(0); 2],
            vec![],
        )],
        0,
    );
    assert!(kernels.sectors().is_empty());
    let report = scan_boundaries(
        &mut kernels,
        &BoundaryScanOptions {
            retry_scales: vec![0.5],
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(report.assessment, BoundaryAssessment::NotApplicable);
    assert_eq!(report.stop, DiagnosticStop::Complete);
    assert_eq!(report.attempts.len(), 1);
    assert_eq!(report.completed_probes, 0);
    assert!(report.latest.is_empty());
}
