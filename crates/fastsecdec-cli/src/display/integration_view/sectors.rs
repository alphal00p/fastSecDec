//! Tables and hit regions use the same native Ratatui layout constraints.
use super::render::{BLUE, bold, card, right};
use super::*;
use crate::display::{GOLD, TEAL};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier},
    text::Line,
    widgets::{Cell, HighlightSpacing, Row, Table},
};

pub(super) fn split(value: String) -> (String, String) {
    value.split_once(" ·10").map_or_else(
        || (value.clone(), String::new()),
        |(mantissa, exponent)| (mantissa.to_owned(), format!("·10{exponent}")),
    )
}
fn heading(text: &str, sort: Sort, view: &View) -> String {
    if view.sort == sort {
        format!("{text}{}", if view.descending { "↓" } else { "↑" })
    } else {
        text.to_owned()
    }
}
fn two(a: String, b: String) -> Cell<'static> {
    Cell::from(vec![
        Line::from(a).right_aligned(),
        Line::from(b).right_aligned(),
    ])
}
fn left_two(a: String, b: String) -> Cell<'static> {
    Cell::from(vec![Line::from(a), Line::from(b)])
}
pub(super) fn fitted(value: String, width: u16) -> String {
    if Line::from(value.as_str()).width() <= usize::from(width) {
        value
    } else {
        "…".into()
    }
}
pub(super) fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    data: &Cached,
    view: &mut View,
    colors: ColorPolicy,
) {
    let ids = view.ids(data);
    let order = view.order(data);
    let (real, imag) = indices(data, order);
    // Measure the complete formatted suffix, including the explicit unknown
    // uncertainty marker. A fixed exponent width silently clipped that marker
    // while the flexible mantissa columns still held unused space.
    let values = ids
        .iter()
        .map(|&id| {
            (
                split(value(data, Some(id), real)),
                split(value(data, Some(id), imag)),
                split(maximum(operation(data, id), order)),
            )
        })
        .collect::<Vec<_>>();
    let suffix_width = |part: usize| {
        values
            .iter()
            .map(|(re, im, max)| {
                let suffix = match part {
                    0 => &re.1,
                    1 => &im.1,
                    _ => &max.1,
                };
                Line::from(suffix.as_str()).width()
            })
            .max()
            .unwrap_or(0)
            .min(u16::MAX as usize) as u16
    };
    let [re_width, im_width, max_width] = [suffix_width(0), suffix_width(1), suffix_width(2)];
    // Keep usable mantissa space; use the existing two-line table when the
    // complete suffixes and auxiliary columns would crowd both estimates out.
    let wide_minimum = 5 + 12 + 10 + re_width + 10 + im_width + 8 + 10 + 12 + max_width + 9 * 2 + 4; // column gaps, selection marker and borders
    let wide = area.width >= 120 && area.width >= wide_minimum;
    let tiny = area.width < 65;
    let compact_rows = !wide && !tiny && area.height <= 7;
    let header_height = if wide || tiny || compact_rows { 1 } else { 2 };
    let panel = card(
        format!(
            " Sectors · {} {} · ε{} · {} ",
            view.sort.label(),
            if view.descending { "↓" } else { "↑" },
            number::superscript(order),
            source(data)
        ),
        BLUE,
        colors,
    );
    let inner = panel.inner(area);
    let widths = if tiny {
        vec![Constraint::Fill(1)]
    } else if wide {
        vec![
            Constraint::Length(5),
            Constraint::Length(12),
            Constraint::Fill(1),
            Constraint::Length(re_width),
            Constraint::Fill(1),
            Constraint::Length(im_width),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(max_width),
        ]
    } else {
        vec![
            Constraint::Length(4),
            Constraint::Length(11),
            Constraint::Fill(1),
            Constraint::Length(re_width),
            Constraint::Fill(1),
            Constraint::Length(im_width.max(max_width)),
        ]
    };
    let spacing = if wide { 2 } else { 1 };
    let [_, columns_area] =
        Layout::horizontal([Constraint::Length(2), Constraint::Fill(0)]).areas(inner);
    let columns = Layout::horizontal(widths.clone())
        .flex(Flex::Start)
        .spacing(spacing)
        .split(columns_area);
    let mut heights = Vec::with_capacity(ids.len());
    let rows=ids.iter().zip(values).enumerate().map(|(index,(&id,((re,re_exp),(im,im_exp),(max,max_exp))))| {
        let op=operation(data,id);
        let points=used_points(data,Some(id));
        let relative=relative(data,Some(id),(real,imag));
        let f64=f64_time(op);
        let fit=|text:String,i:usize| fitted(text,columns[i].width);
        let (cells,height)=if tiny {
            let text=format!("#{id} · {points} pts\nRe {re} {re_exp}\nIm {im} {im_exp}\nRel {relative} · f64 {f64} · Max |wgt| {max} {max_exp}");
            let lines=wrapped(&text,inner.width.saturating_sub(2));
            let h=lines.len().min(u16::MAX as usize) as u16;
            (vec![Cell::from(lines)],h)
        } else if wide {
            (vec![right(id.to_string()),right(fit(points,1)),right(fit(re,2)),Cell::from(fit(re_exp,3)),right(fit(im,4)),Cell::from(fit(im_exp,5)),right(fit(relative,6)),right(fit(f64,7)),right(fit(max,8)),Cell::from(fit(max_exp,9))],1)
        } else if compact_rows {
            (vec![right(id.to_string()),right(fit(points,1)),right(fit(re,2)),Cell::from(fit(re_exp,3)),right(fit(im,4)),Cell::from(fit(im_exp,5))],1)
        } else {
            (vec![right(id.to_string()),two(fit(points,1),fit(relative,1)),two(fit(re,2),fit(f64,2)),left_two(fit(re_exp,3),String::new()),two(fit(im,4),fit(max,4)),left_two(fit(im_exp,5),fit(max_exp,5))],2)
        };
        heights.push(height);
        let mut style=colors.foreground(if index%2==0 {Color::White}else{Color::Rgb(191,208,224)});
        if colors.enabled() && index%2==1 {style=style.bg(Color::Rgb(20,28,39));}
        Row::new(cells).height(height).style(style)
    }).collect::<Vec<_>>();
    let header = if tiny {
        vec![right("Sector estimates")]
    } else if wide {
        vec![
            right(heading("ID", Sort::Id, view)),
            right(heading(
                if is_qmc(data) { "Pts / sh" } else { "Points" },
                Sort::Points,
                view,
            )),
            right(heading("Real (error)", Sort::Real, view)),
            right(""),
            right(heading("Imag (error)", Sort::Imag, view)),
            right(""),
            right(heading("Rel err", Sort::RelativeError, view)),
            right(heading("f64 mean", Sort::F64Mean, view)),
            right(heading("Max |wgt|", Sort::Maximum, view)),
            right(""),
        ]
    } else if compact_rows {
        vec![
            right(heading("ID", Sort::Id, view)),
            right(heading(
                if is_qmc(data) { "Pts/sh" } else { "Points" },
                Sort::Points,
                view,
            )),
            right(heading("Real", Sort::Real, view)),
            right(""),
            right(heading("Imag", Sort::Imag, view)),
            right(""),
        ]
    } else {
        vec![
            right(heading("ID", Sort::Id, view)),
            two(
                heading(
                    if is_qmc(data) { "Pts/sh" } else { "Points" },
                    Sort::Points,
                    view,
                ),
                heading("Rel err", Sort::RelativeError, view),
            ),
            two(
                heading("Real", Sort::Real, view),
                heading("f64 mean", Sort::F64Mean, view),
            ),
            right(""),
            two(
                heading("Imag", Sort::Imag, view),
                heading("Max |wgt|", Sort::Maximum, view),
            ),
            right(""),
        ]
    };
    let mut highlight = bold(colors, TEAL);
    if colors.enabled() {
        highlight = highlight.bg(Color::Rgb(22, 62, 64));
    } else {
        highlight = highlight.add_modifier(Modifier::REVERSED);
    }
    frame.render_stateful_widget(
        Table::new(rows, widths.clone())
            .header(
                Row::new(header)
                    .height(header_height)
                    .style(bold(colors, GOLD)),
            )
            .column_spacing(spacing)
            .flex(Flex::Start)
            .highlight_spacing(HighlightSpacing::Always)
            .row_highlight_style(highlight)
            .highlight_symbol("› ")
            .block(panel),
        area,
        &mut view.table,
    );

    // Header hit rectangles reuse the exact Layout used above, including the
    // reserved Unicode-width-two selection marker and native column spacing.
    let sorts = if tiny {
        vec![Sort::Id]
    } else if wide {
        vec![
            Sort::Id,
            Sort::Points,
            Sort::Real,
            Sort::Real,
            Sort::Imag,
            Sort::Imag,
            Sort::RelativeError,
            Sort::F64Mean,
            Sort::Maximum,
            Sort::Maximum,
        ]
    } else {
        vec![
            Sort::Id,
            Sort::Points,
            Sort::Real,
            Sort::Real,
            Sort::Imag,
            Sort::Imag,
        ]
    };
    for (column, sort) in columns.iter().zip(sorts) {
        view.hits.headers.push((
            Rect::new(
                column.x,
                inner.y,
                column.width,
                header_height.min(inner.height),
            ),
            sort,
        ));
    }
    if !wide && !tiny && !compact_rows && inner.height > 1 {
        // Second-line compact headers are independent clickable columns. Insert
        // them first so their narrow rectangles override the spanning top rows.
        for (index, sort) in [
            (1, Sort::RelativeError),
            (2, Sort::F64Mean),
            (4, Sort::Maximum),
            (5, Sort::Maximum),
        ] {
            let c = columns[index];
            view.hits
                .headers
                .insert(0, (Rect::new(c.x, inner.y + 1, c.width, 1), sort));
        }
    }
    let body = Rect::new(
        inner.x,
        inner.y + header_height.min(inner.height),
        inner.width,
        inner.height.saturating_sub(header_height),
    );
    view.hits.body = body;
    let mut y = body.y;
    for (&id, &height) in ids.iter().zip(&heights).skip(view.table.offset()) {
        let visible = height.min(body.bottom().saturating_sub(y));
        if visible == 0 {
            break;
        }
        view.hits
            .rows
            .push((Rect::new(body.x, y, body.width, visible), id));
        y = y.saturating_add(height);
    }
}
