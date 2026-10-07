//! Operational diagnostics, separate from accepted statistical estimates.
use super::render::{BLUE, PURPLE, bold, card, right};
use super::*;
use crate::display::{GOLD, TEAL, diagnostic_summary as diagnostics};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

fn facts(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: Vec<(String, String)>,
    color: ratatui::style::Color,
    colors: ColorPolicy,
) {
    let label_width = rows
        .iter()
        .map(|(label, _)| Line::from(label.as_str()).width() as u16)
        .max()
        .unwrap_or(0)
        .min(area.width / 2);
    let value_width = area.width.saturating_sub(label_width + 1);
    frame.render_widget(
        Table::new(
            rows.into_iter().map(|(a, b)| {
                Row::new(vec![
                    ratatui::widgets::Cell::from(a),
                    right(super::sectors::fitted(b, value_width)),
                ])
            }),
            [Constraint::Length(label_width), Constraint::Fill(1)],
        )
        .column_spacing(1)
        .style(colors.foreground(color)),
        area,
    );
}
fn times(worker: f64, integrand: f64, evaluator: f64) -> Option<[f64; 3]> {
    // Worker and callback counters can be observed between two atomic updates.
    // Keep their raw values, but do not manufacture exclusive fractions yet.
    (worker.is_finite()
        && integrand.is_finite()
        && evaluator.is_finite()
        && worker >= integrand
        && integrand >= evaluator)
        .then_some([worker - integrand, integrand - evaluator, evaluator])
}
fn timed(value: f64, total: f64) -> String {
    format!(
        "{} {}",
        number::sample_duration(value),
        diagnostics::percentage(value, total)
    )
}
fn outcome_rows(
    d: &EvaluationDiagnostics,
    mode: fastsecdec::kernel::StabilityMode,
    calls: bool,
) -> Vec<(String, String)> {
    let fractions = if calls {
        compact_fractions(d)
    } else {
        fractions(Some(d))
    };
    let counts = [
        Some(d.f64_timing.calls),
        Some(d.double_float_timing.calls),
        Some(d.arbitrary_timing.calls),
        None,
    ];
    [
        "f64",
        "DF106",
        diagnostics::arbitrary_label(mode),
        "Unstable",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, name)| {
        let value = if calls {
            counts[i].map_or_else(
                || fractions[i].clone(),
                |n| format!("{} · {}", fractions[i], number::compact_count(n)),
            )
        } else {
            fractions[i].clone()
        };
        (
            if name == "Arb (variable)" {
                "Arb (var)"
            } else {
                name
            }
            .into(),
            value,
        )
    })
    .collect()
}
fn compact_fractions(d: &EvaluationDiagnostics) -> [String; 4] {
    fractions(Some(d)).map(|s| {
        s.strip_suffix(".0%")
            .map_or_else(|| s.clone(), |v| format!("{v}%"))
    })
}
pub(super) fn selected(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &Cached,
    view: &View,
    colors: ColorPolicy,
) {
    let Some(id) = view.selected_id else {
        frame.render_widget(
            Paragraph::new("No numerical sectors; the sum may consist of exact offsets.")
                .block(card(" Sector detail ", GOLD, colors)),
            area,
        );
        return;
    };
    let op = operation(data, id);
    let panel = card(
        format!(" ▶ Sector {id} · invocation detail · final class fraction / attempted calls "),
        GOLD,
        colors,
    )
    .title_bottom(Line::from(
        " ¹Accepted: current phase · times exclude coordinator ",
    ));
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    let columns = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ])
    .spacing(2)
    .split(inner);
    let accepted = data
        .observation
        .contributions
        .sectors
        .iter()
        .find(|s| s.progress.id == id)
        .map_or(0, |s| s.progress.completed_points);
    let assessed = op.map_or(0, |o| o.evaluations);
    let sample = op
        .filter(|o| o.evaluations > 0)
        .map(|o| number::sample_duration(o.worker_seconds / o.evaluations as f64))
        .unwrap_or_else(|| "—".into());
    let used = if is_qmc(data) {
        format!(
            "{} / {}sh",
            number::compact_count(estimate(data, Some(id)).2),
            number::compact_count(shifts(data, Some(id)) as u64)
        )
    } else {
        number::compact_count(estimate(data, Some(id)).2)
    };
    let mut coverage = vec![
        ("Assessed".into(), number::compact_count(assessed)),
        ("Accepted¹".into(), number::compact_count(accepted)),
    ];
    if inner.height >= 5 {
        coverage.push(("View pts".into(), used));
    }
    coverage.extend([
        ("Mean/point".into(), sample.clone()),
        ("Max |wgt|".into(), maximum(op, view.order(data))),
    ]);
    if inner.height < 4 {
        coverage = vec![
            (
                "Assess/acc".into(),
                format!(
                    "{}/{}",
                    number::compact_count(assessed),
                    number::compact_count(accepted)
                ),
            ),
            ("Mean/point".into(), sample.clone()),
            ("Max |wgt|".into(), maximum(op, view.order(data))),
        ];
    }
    facts(frame, columns[0], coverage, BLUE, colors);
    if let Some(op) = op {
        let mut rows = outcome_rows(&op.diagnostics, data.stability_mode, true);
        rows.push((
            "Zero/fail".into(),
            format!(
                "{} / {}",
                number::compact_count(op.diagnostics.cutoff_zero_points),
                number::compact_count(op.diagnostics.failures)
            ),
        ));
        if inner.height < 4 {
            let f = compact_fractions(&op.diagnostics);
            rows = vec![
                ("f64 / DF".into(), format!("{} / {}", f[0], f[1])),
                ("Arb/unst.".into(), format!("{} / {}", f[2], f[3])),
                (
                    "Calls f/D/A".into(),
                    format!(
                        "{}/{}/{}",
                        number::compact_count(op.diagnostics.f64_timing.calls),
                        number::compact_count(op.diagnostics.double_float_timing.calls),
                        number::compact_count(op.diagnostics.arbitrary_timing.calls)
                    ),
                ),
            ];
        }
        facts(frame, columns[1], rows, PURPLE, colors);
        let mut rows = if let Some(t) = times(
            op.worker_seconds,
            op.integrand_seconds,
            op.evaluator_seconds,
        ) {
            ["Integrator", "Integrand", "Evaluator"]
                .into_iter()
                .zip(t)
                .map(|(k, t)| (k.into(), timed(t, op.worker_seconds)))
                .collect()
        } else {
            vec![
                ("Timing".into(), "pending".into()),
                ("Worker".into(), number::sample_duration(op.worker_seconds)),
                (
                    "Callback".into(),
                    number::sample_duration(op.integrand_seconds),
                ),
            ]
        };
        rows.push(("f64 mean".into(), f64_time(Some(op))));
        rows.push((
            "Unknown".into(),
            number::compact_count(op.diagnostics.unclassified_points()),
        ));
        facts(frame, columns[2], rows, TEAL, colors);
    } else {
        frame.render_widget(
            Paragraph::new("No assessed points in this invocation.")
                .style(colors.foreground(PURPLE)),
            columns[1],
        );
    }
}
pub(super) fn global(frame: &mut Frame<'_>, area: Rect, data: &Cached, colors: ColorPolicy) {
    let m = &data.operational;
    let (total, effort) = diagnostics::effort(m);
    let slowest = m
        .sectors
        .iter()
        .filter_map(|s| diagnostics::f64_mean(&s.diagnostics).map(|v| (s.id, v)))
        .max_by(|(ia, a), (ib, b)| a.total_cmp(b).then_with(|| ib.cmp(ia)));
    let slowest = slowest.map_or_else(
        || "—".into(),
        |(id, t)| format!("{} (#{id})", number::sample_duration(t)),
    );
    let d = &m.diagnostics;
    let unknown = if d.unclassified_points() > 0 {
        format!(
            " · unknown {}",
            number::compact_count(d.unclassified_points())
        )
    } else {
        String::new()
    };
    let bottom = format!(
        " Slowest f64 {slowest} · zeros {} · failures {}{unknown} ",
        number::compact_count(d.cutoff_zero_points),
        number::compact_count(d.failures)
    );
    let panel = card(
        format!(
            " Invocation diagnostics · measured work {} ",
            number::sample_duration(total)
        ),
        TEAL,
        colors,
    )
    .title_bottom(Line::from(bottom));
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    let columns = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ])
    .spacing(2)
    .split(inner);
    facts(
        frame,
        columns[0],
        vec![
            ("Wall".into(), number::sample_duration(data.elapsed)),
            (
                "CPU".into(),
                data.memory
                    .process_cpu_seconds
                    .map(number::sample_duration)
                    .unwrap_or_else(|| "—".into()),
            ),
            ("Workers".into(), number::sample_duration(m.worker_seconds)),
            (
                "Coord.".into(),
                number::sample_duration(
                    m.coordinator_integrand_seconds + m.coordinator_integrator_seconds,
                ),
            ),
        ],
        BLUE,
        colors,
    );
    let mut rows = if times(m.worker_seconds, m.integrand_seconds, m.evaluator_seconds).is_some() {
        ["Integrator", "Integrand", "Evaluator"]
            .into_iter()
            .zip(effort)
            .map(|(k, t)| (k.into(), timed(t, total)))
            .collect()
    } else {
        vec![
            ("Timing".into(), "pending attribution".into()),
            (
                "Integrand".into(),
                number::sample_duration(m.integrand_seconds),
            ),
            (
                "Evaluator".into(),
                number::sample_duration(m.evaluator_seconds),
            ),
        ]
    };
    rows.push((
        "f64 mean".into(),
        diagnostics::f64_mean(d)
            .map(number::sample_duration)
            .unwrap_or_else(|| "—".into()),
    ));
    facts(frame, columns[1], rows, TEAL, colors);
    if inner.height >= 4 {
        facts(
            frame,
            columns[2],
            outcome_rows(d, data.stability_mode, false),
            PURPLE,
            colors,
        );
    } else {
        let fs = compact_fractions(d);
        facts(
            frame,
            columns[2],
            vec![
                ("f64/DF106".into(), format!("{} / {}", fs[0], fs[1])),
                ("Arb/unst.".into(), format!("{} / {}", fs[2], fs[3])),
                (
                    "Zero/fail".into(),
                    format!(
                        "{} / {}",
                        number::compact_count(d.cutoff_zero_points),
                        number::compact_count(d.failures)
                    ),
                ),
            ],
            PURPLE,
            colors,
        );
    }
    // Accented separators distinguish the three diagnostic families without
    // constructing an ANSI string or affecting NO_COLOR policy.
    for column in columns.iter().skip(1) {
        if column.x > 0 {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled("│", bold(colors, GOLD)))),
                Rect::new(column.x - 1, inner.y, 1, inner.height),
            );
        }
    }
}
