//! Cached views of native observations. Sorting changes presentation only.
use super::{GOLD, IntegrationWorkerActivity, TEAL, memory, number, panel};
use crate::terminal_policy::ColorPolicy;
use crossterm::event::KeyCode;
use fastsecdec::{
    integration::{
        IntegrationObservation, LiveObservation, LiveSource, LiveStatus, OperationalMetrics,
        SectorOperationalMetrics,
    },
    status::{CoefficientComponent, EvaluationDiagnostics},
};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier},
    text::Line,
    widgets::{Cell, Paragraph, Row, Table, TableState, Wrap},
};
use std::{cmp::Ordering, collections::BTreeSet};

#[derive(Clone)]
pub(super) struct Cached {
    pub observation: IntegrationObservation,
    pub live: Option<LiveObservation>,
    pub operational: OperationalMetrics,
    pub stability_mode: fastsecdec::kernel::StabilityMode,
    pub memory: memory::Snapshot,
    pub elapsed: f64,
    pub scope: fastsecdec::results::ResultScope,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sort {
    Id,
    Points,
    Real,
    Imag,
    RealError,
    ImagError,
    Time,
    RelativeError,
    F64Mean,
    Maximum,
    F64,
    Double,
    Arb,
    Unstable,
}
impl Sort {
    const ALL: [Self; 14] = [
        Self::Id,
        Self::Points,
        Self::Real,
        Self::Imag,
        Self::RealError,
        Self::ImagError,
        Self::Time,
        Self::RelativeError,
        Self::F64Mean,
        Self::Maximum,
        Self::F64,
        Self::Double,
        Self::Arb,
        Self::Unstable,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Id => "sector",
            Self::Points => "points",
            Self::Real => "Re",
            Self::Imag => "Im",
            Self::RealError => "Re error",
            Self::ImagError => "Im error",
            Self::Time => "worker time",
            Self::RelativeError => "relative error",
            Self::F64Mean => "f64 mean time",
            Self::Maximum => "max weighted",
            Self::F64 => "f64 fraction",
            Self::Double => "DoubleFloat fraction",
            Self::Arb => "Arb fraction",
            Self::Unstable => "unstable fraction",
        }
    }
}

pub(super) struct View {
    pub order: Option<i32>,
    selected_id: Option<u64>,
    sort: Sort,
    descending: bool,
    table: TableState,
}
impl Default for View {
    fn default() -> Self {
        Self {
            order: None,
            selected_id: None,
            sort: Sort::Id,
            descending: false,
            table: TableState::default(),
        }
    }
}
impl View {
    fn orders(data: &Cached) -> Vec<i32> {
        data.observation
            .contributions
            .orders
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    fn order(&self, data: &Cached) -> i32 {
        let orders = Self::orders(data);
        self.order
            .filter(|order| orders.contains(order))
            .or_else(|| orders.last().copied())
            .unwrap_or(0)
    }
    pub fn key(&mut self, key: KeyCode, data: &Cached) -> bool {
        let ids = self.ids(data);
        let position = self
            .selected_id
            .and_then(|id| ids.iter().position(|x| *x == id))
            .unwrap_or(0);
        let selected = match key {
            KeyCode::Up => Some(position.saturating_sub(1)),
            KeyCode::Down => Some(position.saturating_add(1).min(ids.len().saturating_sub(1))),
            KeyCode::PageUp => Some(position.saturating_sub(10)),
            KeyCode::PageDown => Some(position.saturating_add(10).min(ids.len().saturating_sub(1))),
            KeyCode::Home => Some(0),
            KeyCode::End => Some(ids.len().saturating_sub(1)),
            _ => None,
        };
        if let Some(position) = selected {
            self.selected_id = ids.get(position).copied();
            return true;
        }
        match key {
            KeyCode::Left | KeyCode::Right => {
                let orders = Self::orders(data);
                let index = orders
                    .iter()
                    .position(|order| *order == self.order(data))
                    .unwrap_or(0);
                let next = if key == KeyCode::Left {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(orders.len().saturating_sub(1))
                };
                self.order = orders.get(next).copied();
            }
            KeyCode::Tab | KeyCode::Char('s') | KeyCode::BackTab => {
                let index = Sort::ALL
                    .iter()
                    .position(|sort| *sort == self.sort)
                    .unwrap();
                let next = if key == KeyCode::BackTab {
                    (index + Sort::ALL.len() - 1) % Sort::ALL.len()
                } else {
                    (index + 1) % Sort::ALL.len()
                };
                self.sort = Sort::ALL[next];
                self.descending = self.sort != Sort::Id;
            }
            KeyCode::Char('r') => self.descending = !self.descending,
            _ => return false,
        }
        true
    }
    fn ids(&self, data: &Cached) -> Vec<u64> {
        let mut ids = data
            .observation
            .contributions
            .sectors
            .iter()
            .map(|row| row.progress.id)
            .collect::<Vec<_>>();
        let indices = indices(data, self.order(data));
        ids.sort_by(|&a, &b| {
            let compare = if self.sort == Sort::Id {
                a.cmp(&b)
            } else {
                let metric = |id| {
                    let (mean, error, points) = estimate(data, Some(id));
                    let op = operation(data, id);
                    let component = if matches!(self.sort, Sort::Imag | Sort::ImagError) {
                        indices.1
                    } else {
                        indices.0
                    };
                    match self.sort {
                        Sort::Points => Some(points as f64),
                        Sort::Real | Sort::Imag => {
                            component.and_then(|i| mean.and_then(|v| v.get(i))).copied()
                        }
                        Sort::RealError | Sort::ImagError => component
                            .and_then(|i| error.and_then(|v| v.get(i)))
                            .copied(),
                        Sort::Time => op.map(|o| o.worker_seconds),
                        Sort::RelativeError => relative_error(data, Some(id), indices),
                        Sort::F64Mean => {
                            op.and_then(|o| super::diagnostic_summary::f64_mean(&o.diagnostics))
                        }
                        Sort::Maximum => op.and_then(|o| {
                            o.maximum_weighted_contribution
                                .get(&self.order(data))
                                .copied()
                        }),
                        Sort::F64 | Sort::Double | Sort::Arb | Sort::Unstable => op.and_then(|o| {
                            let n = o.diagnostics.evaluations;
                            let counts = counts(&o.diagnostics);
                            let c = match self.sort {
                                Sort::F64 => counts[0],
                                Sort::Double => counts[1],
                                Sort::Arb => counts[2],
                                _ => counts[3],
                            };
                            (n > 0).then_some(c as f64 / n as f64)
                        }),
                        Sort::Id => unreachable!(),
                    }
                    .filter(|v| v.is_finite())
                };
                match (metric(a), metric(b)) {
                    (Some(a), Some(b)) => a.total_cmp(&b),
                    (Some(_), None) => return Ordering::Less,
                    (None, Some(_)) => return Ordering::Greater,
                    (None, None) => Ordering::Equal,
                }
            };
            (if self.descending {
                compare.reverse()
            } else {
                compare
            })
            .then_with(|| a.cmp(&b))
        });
        ids
    }
}

fn indices(data: &Cached, order: i32) -> (Option<usize>, Option<usize>) {
    let report = &data.observation.contributions;
    let find = |component| {
        report
            .orders
            .iter()
            .zip(&report.components)
            .position(|(&o, &c)| o == order && c == component)
    };
    (
        find(CoefficientComponent::Real),
        find(CoefficientComponent::Imag),
    )
}
fn operation(data: &Cached, id: u64) -> Option<&SectorOperationalMetrics> {
    data.operational.sectors.iter().find(|row| row.id == id)
}
type Estimate<'a> = (Option<&'a [f64]>, Option<&'a [f64]>, u64);
fn estimate(data: &Cached, id: Option<u64>) -> Estimate<'_> {
    if let Some(live) = &data.live {
        let row = if let Some(id) = id {
            live.sectors
                .iter()
                .find(|row| row.id == id)
                .map(|row| &row.estimate)
        } else {
            Some(&live.total)
        };
        return row.map_or((None, None, 0), |e| {
            (e.mean.as_deref(), e.standard_error.as_deref(), e.points)
        });
    }
    let report = &data.observation.contributions;
    let (value, points) = if let Some(id) = id {
        report
            .sectors
            .iter()
            .find(|row| row.progress.id == id)
            .map_or((None, 0), |row| (row.estimate.as_ref(), row.used_points))
    } else {
        (
            report.total.as_ref(),
            report.sectors.iter().map(|row| row.used_points).sum(),
        )
    };
    (
        value.map(|v| v.mean.as_slice()),
        value.map(|v| v.standard_error.as_slice()),
        points,
    )
}
fn is_qmc(data: &Cached) -> bool {
    matches!(
        data.observation.snapshot.method,
        fastsecdec::status::IntegrationMethod::DemocraticQmc
            | fastsecdec::status::IntegrationMethod::AdaptiveQmc
    )
}
fn shifts(data: &Cached, id: Option<u64>) -> usize {
    if let Some(live) = &data.live {
        id.map_or(Some(&live.total), |id| {
            live.sectors
                .iter()
                .find(|s| s.id == id)
                .map(|s| &s.estimate)
        })
        .map_or(0, |e| e.replicas)
    } else if let Some(id) = id {
        data.observation
            .contributions
            .sectors
            .iter()
            .find(|s| s.progress.id == id)
            .map_or(0, |s| s.used_replicas)
    } else {
        data.observation
            .contributions
            .sectors
            .iter()
            .map(|s| s.used_replicas)
            .min()
            .unwrap_or(0)
    }
}
fn used_points(data: &Cached, id: Option<u64>) -> String {
    let points = estimate(data, id).2;
    if is_qmc(data) {
        format!("{points} / {}", shifts(data, id))
    } else {
        points.to_string()
    }
}
fn value(data: &Cached, id: Option<u64>, index: Option<usize>) -> String {
    let Some(index) = index else {
        return "—".into();
    };
    let (mean, error, _) = estimate(data, id);
    mean.and_then(|v| v.get(index)).map_or_else(
        || {
            let status = data
                .live
                .as_ref()
                .and_then(|live| {
                    id.map_or(Some(&live.total), |id| {
                        live.sectors
                            .iter()
                            .find(|s| s.id == id)
                            .map(|s| &s.estimate)
                    })
                })
                .map(|e| e.status);
            match status {
                Some(LiveStatus::NumericalRange) => "numerical range unavailable",
                Some(LiveStatus::NotSampled) => "not sampled",
                _ => "unavailable",
            }
            .into()
        },
        |&value| number::uncertainty(value, error.and_then(|v| v.get(index)).copied()),
    )
}
fn counts(d: &EvaluationDiagnostics) -> [u64; 4] {
    [
        d.f64_points,
        d.double_float_points,
        d.arbitrary_points,
        d.unstable_points,
    ]
}
fn fractions(d: Option<&EvaluationDiagnostics>) -> [String; 4] {
    d.map_or_else(
        || std::array::from_fn(|_| "—".into()),
        super::diagnostic_summary::fractions,
    )
}
fn relative_error(
    data: &Cached,
    id: Option<u64>,
    (real, imag): (Option<usize>, Option<usize>),
) -> Option<f64> {
    let (means, errors, _) = estimate(data, id);
    let means = means?;
    let errors = errors?;
    let component =
        |values: &[f64], index: Option<usize>| index.map_or(Some(0.0), |i| values.get(i).copied());
    let mean = component(means, real)?.hypot(component(means, imag)?);
    let error = component(errors, real)?.hypot(component(errors, imag)?);
    (mean > 0.0 && mean.is_finite() && error.is_finite())
        .then(|| error / mean)
        .filter(|v| v.is_finite())
}
fn relative(data: &Cached, id: Option<u64>, indices: (Option<usize>, Option<usize>)) -> String {
    relative_error(data, id, indices).map_or_else(|| "—".into(), number::relative_percent)
}
fn f64_time(op: Option<&SectorOperationalMetrics>) -> String {
    op.and_then(|o| super::diagnostic_summary::f64_mean(&o.diagnostics))
        .map(number::duration)
        .unwrap_or_else(|| "—".into())
}
fn maximum(op: Option<&SectorOperationalMetrics>, order: i32) -> String {
    op.and_then(|o| o.maximum_weighted_contribution.get(&order))
        .map(|&v| number::scientific(v))
        .unwrap_or_else(|| "—".into())
}
fn source(data: &Cached) -> &'static str {
    match data.live.as_ref().map(|live| live.source) {
        Some(LiveSource::CurrentIteration) => "Current iteration · provisional",
        Some(LiveSource::SinceResume) => "Since resume · provisional",
        Some(LiveSource::CompleteLattices) => "Complete lattices · provisional",
        None => "Accepted contributions",
    }
}
fn timing(data: &Cached) -> Vec<String> {
    let mut rows = vec![format!(
        "This invocation · wall {} · process CPU {}",
        number::duration(data.elapsed),
        data.memory
            .process_cpu_seconds
            .map(number::duration)
            .unwrap_or_else(|| "unavailable".into())
    )];
    rows.extend(
        super::diagnostic_summary::diagnostic_summary(&data.operational, data.stability_mode)
            .into_iter()
            .map(|[name, value]| format!("{name}: {value}")),
    );
    rows
}

// Reuse tabled's Unicode-aware wrapping for multiline cells, avoiding byte or
// character-count approximations of terminal widths.
fn wrapped(text: &str, width: u16) -> Vec<Line<'static>> {
    use tabled::{
        builder::Builder,
        settings::{Padding, Style, Width},
    };
    let mut builder = Builder::new();
    builder.push_record([text]);
    let mut table = builder.build();
    table
        .with(Style::empty())
        .with(Padding::zero())
        .with(Width::wrap(usize::from(width.max(1))).keep_words(true));
    table
        .to_string()
        .lines()
        .map(|s| Line::from(s.trim_end().to_owned()))
        .collect()
}

pub(super) fn render(
    frame: &mut ratatui::Frame<'_>,
    data: &Cached,
    view: &mut View,
    colors: ColorPolicy,
    cancelling: bool,
    workers: &[IntegrationWorkerActivity],
) {
    let area = frame.area();
    let narrow = area.width < 120;
    let very_narrow = area.width < 60;
    let tiny = area.height < 25;
    let timing_height = if very_narrow {
        9
    } else if narrow {
        7
    } else {
        6
    };
    let chunks = Layout::vertical([
        Constraint::Length(if very_narrow { 3 } else { 2 }),
        Constraint::Length(if narrow { 3 } else { 4 }),
        Constraint::Min(7),
        Constraint::Length(timing_height),
        Constraint::Length(if very_narrow { 3 } else { 2 }),
    ])
    .split(area);
    let snapshot = &data.observation.snapshot;
    frame.render_widget(
        Paragraph::new(if very_narrow {
            let method = match snapshot.method {
                fastsecdec::status::IntegrationMethod::DemocraticQmc => "QMC",
                fastsecdec::status::IntegrationMethod::AdaptiveQmc => "adaptive QMC",
                fastsecdec::status::IntegrationMethod::HavanaMc => "MC",
                fastsecdec::status::IntegrationMethod::HavanaDiscreteMc => "discrete MC",
            };
            format!(
                "FastSecDec · {method} · {:?}\n{}\nAccepted {} / {}",
                snapshot.stage,
                source(data),
                snapshot.completed_points,
                snapshot.planned_points
            )
        } else {
            format!(
                "FastSecDec · {:?} {:?} · {}\nAccepted {} / {} · {}",
                snapshot.method,
                snapshot.stage,
                data.scope,
                snapshot.completed_points,
                snapshot.planned_points,
                source(data)
            )
        })
        .style(colors.foreground(TEAL))
        .wrap(Wrap { trim: true }),
        chunks[0],
    );
    let order = view.order(data);
    view.order = Some(order);
    let indices = indices(data, order);
    let (real, imag) = indices;
    let sum_label = if data.scope.is_full_integral() {
        "Full sum"
    } else {
        "Selected sum"
    };
    let sum_title = if is_qmc(data) {
        if very_narrow {
            format!(
                "ε{} {sum_label} · {}pts/{}sh",
                number::superscript(order),
                estimate(data, None).2,
                shifts(data, None)
            )
        } else {
            format!(
                "ε{} · {sum_label} · rel {} · {}pts/{} complete shifts",
                number::superscript(order),
                relative(data, None, indices),
                estimate(data, None).2,
                shifts(data, None)
            )
        }
    } else {
        format!(
            "ε{} · {sum_label} · relative error {}",
            number::superscript(order),
            relative(data, None, indices)
        )
    };
    let mut total = vec![
        Line::from(sum_title),
        Line::from(format!("Re {}", value(data, None, real))),
        Line::from(format!("Im {}", value(data, None, imag))),
    ];
    if !narrow && let Some(accepted) = &data.observation.contributions.total {
        let get = |i: Option<usize>| {
            i.and_then(|i| accepted.mean.get(i).zip(accepted.standard_error.get(i)))
                .map_or_else(|| "—".into(), |(&v, &e)| number::uncertainty(v, Some(e)))
        };
        total.push(Line::from(format!(
            "Accepted: Re {} · Im {}",
            get(real),
            get(imag)
        )));
    }
    frame.render_widget(
        Paragraph::new(total)
            .style(colors.foreground(GOLD))
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
    let ids = view.ids(data);
    let selected = view
        .selected_id
        .and_then(|id| ids.iter().position(|&x| x == id))
        .or((!ids.is_empty()).then_some(0));
    view.selected_id = selected.and_then(|i| ids.get(i).copied());
    view.table.select(selected);
    let show_fractions = area.width >= 170;
    let rows = ids.iter().map(|&id| {
        let op = operation(data,id);
        let fs = fractions(op.map(|o|&o.diagnostics));
        let points = used_points(data,Some(id));
        let re=value(data,Some(id),real); let im=value(data,Some(id),imag);
        let relative=relative(data,Some(id),indices); let avg=f64_time(op); let max=maximum(op,order);
        if narrow {
            let lines=wrapped(&format!("Sector {id} · used pts {points}\nRe {re}\nIm {im}\nRel {relative} · f64 {avg}\nMax |weighted| {max}"),area.width.saturating_sub(5));
            let height=lines.len().min(u16::MAX as usize) as u16;
            Row::new(vec![Cell::from(lines)]).height(height)
        } else {
            let mut row=vec![id.to_string(),points.to_string(),re,im,relative,avg,max];
            if show_fractions { row.extend(fs); }
            Row::new(row)
        }
    });
    let (mut widths, mut headers) = if narrow {
        (
            vec![Constraint::Min(1)],
            vec!["Estimates and invocation diagnostics"],
        )
    } else {
        (
            vec![
                Constraint::Length(6),
                Constraint::Length(12),
                Constraint::Min(23),
                Constraint::Min(23),
                Constraint::Length(9),
                Constraint::Length(15),
                Constraint::Length(21),
            ],
            vec![
                "Sector",
                if is_qmc(data) {
                    "Used pts/sh"
                } else {
                    "Points"
                },
                "Real (error)",
                "Imag (error)",
                "Rel error",
                "f64 mean",
                "Max |weighted|",
            ],
        )
    };
    if show_fractions {
        widths.extend([Constraint::Length(6); 4]);
        headers.extend(["f64", "DF106", "Arb", "Unstable"]);
    }
    frame.render_stateful_widget(
        Table::new(rows, widths)
            .header(Row::new(headers).style(colors.foreground(GOLD)))
            .row_highlight_style(colors.foreground(TEAL).add_modifier(Modifier::BOLD))
            .highlight_symbol("› ")
            .block(panel(
                &format!(
                    "Sectors · {} {} · ε{}",
                    view.sort.label(),
                    if view.descending { "↓" } else { "↑" },
                    number::superscript(order)
                ),
                colors,
            )),
        chunks[2],
        &mut view.table,
    );
    let m = &data.operational;
    let (total, times) = super::diagnostic_summary::effort(m);
    let cpu = data
        .memory
        .process_cpu_seconds
        .map(number::duration)
        .unwrap_or_else(|| "—".into());
    let fs = fractions(Some(&m.diagnostics));
    let arb = super::diagnostic_summary::arbitrary_label(data.stability_mode);
    let pct = |v| super::diagnostic_summary::percentage(v, total);
    let mut summary = vec![format!(
        "Wall {} · CPU {}",
        number::duration(data.elapsed),
        cpu
    )];
    if very_narrow {
        summary.push(format!(
            "Work {} (workers+active coordinator)",
            number::duration(total)
        ));
        summary.push(format!(
            "I {} · G {} · E {}",
            pct(times[0]),
            pct(times[1]),
            pct(times[2])
        ));
        summary.push(format!("f64 {} · DoubleFloat {}", fs[0], fs[1]));
        summary.push(format!("{arb} {} · Unstable {}", fs[2], fs[3]));
        summary.push(format!(
            "Cutoff zeros {} · failures {}",
            m.diagnostics.cutoff_zero_points, m.diagnostics.failures
        ));
    } else {
        summary.push(format!(
            "Measured work {}: integrator {} · integrand {} · evaluator {}",
            number::duration(total),
            pct(times[0]),
            pct(times[1]),
            pct(times[2])
        ));
        summary.push(format!(
            "Outcomes: f64 {} · DoubleFloat {} · {arb} {} · Unstable {}",
            fs[0], fs[1], fs[2], fs[3]
        ));
        summary.push(format!(
            "Cutoff zeros {} · failures {}",
            m.diagnostics.cutoff_zero_points, m.diagnostics.failures
        ));
    }
    let means = super::diagnostic_summary::diagnostic_summary(m, data.stability_mode);
    let mean = &means
        .iter()
        .find(|[name, _]| name == "f64 evaluator mean")
        .expect("mean row")[1];
    let slowest = &means
        .iter()
        .find(|[name, _]| name == "Slowest sector f64 mean")
        .expect("slowest row")[1];
    if very_narrow {
        summary.push(format!("f64 mean {mean}"));
        summary.push(format!("Slowest {slowest}"));
    } else {
        summary.push(format!("f64 mean {mean} · slowest {slowest}"));
    }
    if m.diagnostics.unclassified_points() != 0 {
        summary.push(format!(
            "Unknown historical classes: {}",
            m.diagnostics.unclassified_points()
        ));
    }
    if !tiny && !workers.is_empty() {
        summary.push(super::integration_activity::summary(workers));
    }
    frame.render_widget(
        Paragraph::new(summary.join("\n"))
            .style(colors.foreground(Color::DarkGray))
            .wrap(Wrap { trim: true }),
        chunks[3],
    );
    let help = if cancelling {
        "Cancelling · Ctrl-C again forces exit"
    } else if very_narrow {
        "↑↓/Pg: sector · ←→: ε · Tab: column\nr: reverse · Ctrl-C/q/Esc: stop\nI/G/E: integrator/integrand/evaluator"
    } else {
        "↑↓/Pg/Home/End: sector · ←→: ε · Tab/s: sort · r: reverse\nCtrl-C/q/Esc: stop · Work includes active coordinator"
    };
    frame.render_widget(Paragraph::new(help).wrap(Wrap { trim: true }), chunks[4]);
}

pub(super) fn plain(data: &Cached, view: &View) -> String {
    use tabled::{builder::Builder, settings::Style};
    let order = view.order(data);
    let (real, imag) = indices(data, order);
    let mut builder = Builder::new();
    builder.push_record([
        "Sector",
        if is_qmc(data) {
            "Used points / complete shifts"
        } else {
            "Points"
        },
        "Real",
        "Imag",
        "Rel error",
        "f64 mean",
        "Max |weighted|",
        "f64",
        "DoubleFloat",
        super::diagnostic_summary::arbitrary_label(data.stability_mode),
        "Unstable",
    ]);
    for id in view.ids(data) {
        let op = operation(data, id);
        let fs = fractions(op.map(|o| &o.diagnostics));
        builder.push_record([
            id.to_string(),
            used_points(data, Some(id)),
            value(data, Some(id), real),
            value(data, Some(id), imag),
            relative(data, Some(id), (real, imag)),
            f64_time(op),
            maximum(op, order),
            fs[0].clone(),
            fs[1].clone(),
            fs[2].clone(),
            fs[3].clone(),
        ]);
    }
    let mut table = builder.build();
    table.with(Style::modern());
    format!(
        "{} · {:?} {:?} · {}\nε{} sum Re {} · Im {}\nCoverage: {}\n{}\n{}",
        data.scope,
        data.observation.snapshot.method,
        data.observation.snapshot.stage,
        source(data),
        number::superscript(order),
        value(data, None, real),
        value(data, None, imag),
        if is_qmc(data) {
            format!(
                "{} used points / {} complete shifts",
                estimate(data, None).2,
                shifts(data, None)
            )
        } else {
            format!("{} sampled points", estimate(data, None).2)
        },
        table,
        timing(data).join("\n")
    )
}
