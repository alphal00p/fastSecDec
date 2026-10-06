//! Width budgets for the inspector's specific tables. tabled owns all text
//! wrapping, including native Symbolica ANSI escapes and Unicode cell widths.
use crate::{
    generation_report::{facts_table, heading},
    terminal_policy::ColorPolicy,
};
use tabled::{
    builder::Builder,
    settings::{
        Alignment, Color, Padding, Style, Width,
        object::{Columns, Rows},
        style::BorderColor,
    },
};

pub(super) fn section(
    title: &str,
    headers: Vec<&str>,
    rows: Vec<Vec<String>>,
    width: usize,
    colors: ColorPolicy,
) -> String {
    let mut result = heading(title, width, colors, Color::FG_CYAN);
    if rows.is_empty() {
        result.push_str("None\n");
        return result;
    }
    if width < 64 && headers.len() > 2 {
        for row in rows {
            result.push_str(&facts_table(
                headers
                    .iter()
                    .zip(row)
                    .map(|(label, value)| [(*label).to_owned(), value])
                    .collect(),
                width,
                colors,
            ));
            result.push('\n');
        }
        return result;
    }
    if width < 32 {
        let mut table = Builder::from_iter(rows.into_iter().map(|row| {
            [headers
                .iter()
                .zip(row)
                .map(|(label, value)| format!("{label}: {value}"))
                .collect::<Vec<_>>()
                .join("\n")]
        }))
        .build();
        table
            .with(Style::empty())
            .with(Padding::zero())
            .modify(Columns::first(), Width::wrap(width).keep_words(true));
        result.push_str(&format!("{table}\n"));
        return result;
    }
    let columns = headers.len();
    let available = width.saturating_sub(3 * columns + 1);
    let widths = match columns {
        2 => vec![24.min(available / 2), available - 24.min(available / 2)],
        3 => vec![
            available.min(10),
            available.min(10),
            available.saturating_sub(20),
        ],
        5 => vec![6, 11, 4, 5, available.saturating_sub(26)],
        _ => vec![available / columns; columns],
    };
    let mut table = Builder::from_iter(
        [headers.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>()]
            .into_iter()
            .chain(rows),
    )
    .build();
    table.with(Style::rounded()).with(Alignment::left());
    for (column, cap) in widths.into_iter().enumerate() {
        table.modify(
            Columns::new(column..column + 1),
            Width::wrap(cap.max(1)).keep_words(true),
        );
    }
    if colors.enabled() {
        table
            .with(BorderColor::filled(Color::FG_BRIGHT_BLACK))
            .modify(Rows::first(), Color::FG_CYAN | Color::BOLD);
        if headers.get(1) == Some(&"Evaluator") {
            table.modify(Columns::new(1..2), Color::FG_GREEN | Color::BOLD);
        }
    }
    result.push_str(&format!("{table}\n"));
    result
}

pub(super) fn bytes(value: usize) -> String {
    if value < 1024 {
        format!("{value} B")
    } else if value < 1024 * 1024 {
        format!("{:.2} KiB", value as f64 / 1024.)
    } else if value < 1024 * 1024 * 1024 {
        format!("{:.2} MiB", value as f64 / (1024. * 1024.))
    } else {
        format!("{:.2} GiB", value as f64 / (1024. * 1024. * 1024.))
    }
}
