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
    let (total, times) = effort(m);
    let mut rows = vec![
        ["Measured work elapsed".into(), number::duration(total)],
        [
            "Aggregate worker elapsed".into(),
            number::duration(m.worker_seconds),
        ],
        [
            "Active coordinator elapsed".into(),
            number::duration(m.coordinator_integrand_seconds + m.coordinator_integrator_seconds),
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
            format!("{} ({})", number::duration(value), percentage(value, total)),
        ]);
    }
    rows.push([
        "f64 evaluator mean".into(),
        f64_mean(&m.diagnostics)
            .map(number::duration)
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
            |(id, mean)| format!("{} (sector {id})", number::duration(mean)),
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
            m.diagnostics.cutoff_zero_points, m.diagnostics.failures
        ),
    ]);
    if m.diagnostics.unclassified_points() != 0 {
        rows.push([
            "Unclassified historical outcomes".into(),
            m.diagnostics.unclassified_points().to_string(),
        ]);
    }
    rows
}
