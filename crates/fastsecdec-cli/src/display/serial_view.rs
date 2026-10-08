//! Owned serial snapshots, shared by terminal and line-oriented presentation.
use super::{TEAL, number};
use crate::{driver::serial::SerialRunSnapshot, terminal_policy::ColorPolicy};

pub(super) fn summary(status: &SerialRunSnapshot) -> String {
    let covered = status
        .native
        .sectors
        .iter()
        .filter(|s| s.first_visit_complete)
        .count();
    format!(
        "Serial · first visits {covered}/{} · {} resident workers · {} reservations{}",
        status.native.sectors.len(),
        status.residents.len(),
        status.native.in_flight,
        if status.native.first_coverage_complete {
            " · error priority"
        } else {
            " · first coverage"
        }
    )
}

fn rows(status: &SerialRunSnapshot) -> Vec<Vec<String>> {
    status
        .residents
        .iter()
        .map(|resident| {
            let sector = status
                .native
                .sectors
                .iter()
                .find(|s| s.id == resident.sector);
            vec![
                resident.worker.to_string(),
                resident.pid.to_string(),
                resident.sector.to_string(),
                if resident.preparing {
                    "loading".into()
                } else {
                    number::sample_duration(resident.residence_seconds)
                },
                number::sample_duration(resident.loading_seconds),
                sector.map_or_else(|| "—".into(), |s| s.epoch.to_string()),
                sector.map_or_else(
                    || "—".into(),
                    |s| format!("{}/{}", s.replicas, s.target_replicas),
                ),
                sector
                    .and_then(|s| s.priority)
                    .map_or_else(|| "pending".into(), number::scientific),
            ]
        })
        .collect()
}

const HEADERS: [&str; 8] = [
    "Worker",
    "PID",
    "Sector",
    "Residence",
    "Load/JIT",
    "Epoch",
    "Replicas",
    "Priority",
];

pub(super) fn plain(status: &SerialRunSnapshot) -> String {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record(HEADERS);
    for row in rows(status) {
        builder.push_record(row);
    }
    let mut table = builder.build();
    table.with(tabled::settings::Style::modern());
    format!("{}\n{table}", summary(status))
}

pub(super) fn render(
    frame: &mut ratatui::Frame<'_>,
    area: ratatui::layout::Rect,
    status: &SerialRunSnapshot,
    colors: ColorPolicy,
) {
    use ratatui::{
        layout::Constraint,
        widgets::{Row, Table},
    };
    let widths = [5, 8, 6, 11, 10, 5, 11, 12].map(Constraint::Length);
    let shown = area.height.saturating_sub(3) as usize;
    let title = summary(status);
    let mut panel = super::panel(&title, colors);
    if status.residents.len() > shown {
        panel = panel.title_bottom(format!(
            " {} more residents in status stream ",
            status.residents.len() - shown
        ));
    }
    let table = Table::new(rows(status).into_iter().take(shown).map(Row::new), widths)
        .header(Row::new(HEADERS).style(colors.foreground(TEAL)))
        .column_spacing(1)
        .block(panel);
    frame.render_widget(table, area);
}
