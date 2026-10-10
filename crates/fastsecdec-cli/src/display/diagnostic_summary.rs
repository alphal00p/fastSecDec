//! Presentation of measured invocation work; no estimator state is modified.
use super::number;
use fastsecdec::{integration::OperationalMetrics, status::EvaluationDiagnostics};

pub(super) fn fractions(d: &EvaluationDiagnostics) -> [String; 4] {
    if d.classified_points() == 0 && d.evaluations != 0 {
        return std::array::from_fn(|_| "unknown".into());
    }
    [
        d.f64_points,
        d.double_float_points,
        d.arbitrary_points,
        d.unstable_points,
    ]
    .map(|n| number::fraction(n, d.evaluations))
}

pub(super) fn effort(m: &OperationalMetrics) -> (f64, [f64; 3]) {
    let total =
        m.worker_seconds + m.coordinator_integrand_seconds + m.coordinator_integrator_seconds;
    let integrator =
        (m.worker_seconds - m.integrand_seconds).max(0.0) + m.coordinator_integrator_seconds;
    let integrand =
        (m.integrand_seconds - m.evaluator_seconds).max(0.0) + m.coordinator_integrand_seconds;
    (total, [integrator, integrand, m.evaluator_seconds])
}

pub(super) fn percentage(value: f64, total: f64) -> String {
    if total > 0.0 {
        format!("{:.1}%", 100.0 * value / total)
    } else {
        "—".into()
    }
}

pub(super) fn f64_mean(d: &EvaluationDiagnostics) -> Option<f64> {
    (d.f64_timing.calls > 0)
        .then(|| d.f64_timing.nanoseconds as f64 / d.f64_timing.calls as f64 / 1e9)
}

pub(super) fn arbitrary_label(mode: fastsecdec::kernel::StabilityMode) -> &'static str {
    match mode {
        fastsecdec::kernel::StabilityMode::Distance => "Arb<1000>",
        fastsecdec::kernel::StabilityMode::Validated => "Arb (variable)",
    }
}

pub(crate) fn diagnostic_summary(
    m: &OperationalMetrics,
    mode: fastsecdec::kernel::StabilityMode,
) -> Vec<[String; 2]> {
    diagnostic_summary_with_duration(m, mode, number::duration)
}

pub(super) fn diagnostic_summary_with_duration(
    m: &OperationalMetrics,
    mode: fastsecdec::kernel::StabilityMode,
    duration: fn(f64) -> String,
) -> Vec<[String; 2]> {
    let (total, times) = effort(m);
    let mut rows = vec![
        ["Measured work elapsed".into(), duration(total)],
        [
            "Aggregate worker elapsed".into(),
            duration(m.worker_seconds),
        ],
        [
            "Active coordinator elapsed".into(),
            duration(m.coordinator_integrand_seconds + m.coordinator_integrator_seconds),
        ],
    ];
    for (name, value) in [
        "Integrator overhead",
        "Integrand overhead",
        "Evaluator calls",
    ]
    .into_iter()
    .zip(times)
    {
        rows.push([
            name.into(),
            format!("{} ({})", duration(value), percentage(value, total)),
        ]);
    }
    rows.push([
        "f64 evaluator mean".into(),
        f64_mean(&m.diagnostics)
            .map(duration)
            .unwrap_or_else(|| "unavailable".into()),
    ]);
    let slowest = m
        .sectors
        .iter()
        .filter_map(|s| f64_mean(&s.diagnostics).map(|mean| (s.id, mean)))
        .max_by(|(a_id, a), (b_id, b)| a.total_cmp(b).then_with(|| b_id.cmp(a_id)));
    rows.push([
        "Slowest sector f64 mean".into(),
        slowest.map_or_else(
            || "unavailable".into(),
            |(id, mean)| format!("{} (sector {id})", duration(mean)),
        ),
    ]);
    for (name, fraction) in [
        "f64",
        "DoubleFloat (106 bits)",
        arbitrary_label(mode),
        "Unstable",
    ]
    .into_iter()
    .zip(fractions(&m.diagnostics))
    {
        rows.push([format!("Final {name} fraction"), fraction]);
    }
    rows.push([
        "Maximum successful-point precision".into(),
        if m.diagnostics.max_precision_bits == 0 {
            "unavailable".into()
        } else {
            format!("{} bits", m.diagnostics.max_precision_bits)
        },
    ]);
    rows.push([
        "Unstable outcomes".into(),
        format!(
            "{} cutoff zeros; {} failures",
            number::compact_count(m.diagnostics.cutoff_zero_points),
            number::compact_count(m.diagnostics.failures)
        ),
    ]);
    if m.diagnostics.unclassified_points() != 0 {
        rows.push([
            "Unclassified historical outcomes".into(),
            number::compact_count(m.diagnostics.unclassified_points()),
        ]);
    }
    if let Some(runtime) = &m.diagnostics.contour_runtime {
        for (phase, report) in [
            ("Adaptation", &runtime.adaptation),
            ("Production", &runtime.production),
        ] {
            let work = &report.evaluation;
            rows.push([
                format!("Contour {phase} calls"),
                format!(
                    "{} callbacks; {} solves; {} closed forms; {} conditioning calls",
                    work.callback_calls,
                    work.solver_calls,
                    work.closed_form_calls,
                    report.conditioning.callback_calls
                ),
            ]);
            rows.push([
                format!("Contour {phase} solve work"),
                format!(
                    "{} iterations; {} function evaluations; {} failures",
                    work.solver_iterations, work.solver_evaluations, work.solver_failures
                ),
            ]);
            rows.push([
                format!("Contour {phase} setup calls"),
                format!(
                    "{} preparation; {} exact; {} independent pilot",
                    report.preparation.callback_calls,
                    report.exact.callback_calls,
                    report.pilot.callback_calls
                ),
            ]);
            for (name, range) in [
                ("strength", &work.strength),
                ("normalized displacement", &work.normalized_displacement),
                ("physical displacement", &work.physical_displacement),
            ] {
                rows.push([
                    format!("{phase} {name} centres"),
                    match (range.minimum, range.maximum) {
                        (Some(low), Some(high)) => format!(
                            "{low:.4e} .. {high:.4e} ({} unavailable)",
                            range.unavailable
                        ),
                        _ => "unavailable".into(),
                    },
                ]);
            }
        }
    }
    rows
}
