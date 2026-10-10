use super::tables::{bytes, section};
use crate::{
    CliResult,
    artifact::Artifact,
    generation_report::{facts_table, heading},
    math_display,
    terminal_policy::ColorPolicy,
};
use fastsecdec::{
    Atom, AtomCore,
    generation::{ChartRecord, PreSubtractionTerm},
    kernel::KernelSet,
};
use std::path::Path;
use tabled::settings::Color;

pub(super) fn render(
    path: &Path,
    artifact: &Artifact,
    kernels: &KernelSet,
    selected: Option<usize>,
    expressions: bool,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    let mut out = super::overview::render(
        path,
        artifact,
        &crate::artifact::KernelSummary::from_kernels(kernels),
        Some(kernels),
        width,
        colors,
    )?;
    if let Some(id) = selected {
        out.push('\n');
        out.push_str(&sector(artifact, kernels, id, expressions, width, colors)?);
    } else {
        out.push_str(&super::threshold::native_facts(kernels, width, colors));
        out.push('\n');
        let rows = super::ranked_sectors(kernels)
            .into_iter()
            .map(|id| {
                let kernel = &kernels.sectors()[id];
                let chart = representative(kernels, id);
                let groups = chart.map(monomials).unwrap_or_default();
                let context = chart.map(chart_context).unwrap_or_default();
                let leading = if groups.is_empty() {
                    "Not retained".into()
                } else {
                    let mut preview = groups
                        .iter()
                        .take(3)
                        .map(|(value, _)| {
                            math_display::in_context(
                                value,
                                &context,
                                colors,
                                width.saturating_sub(48).max(16),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    if groups.len() > 3 {
                        preview.push_str(&format!("\n+ {} more; --sector {id}", groups.len() - 3));
                    }
                    preview
                };
                vec![
                    id.to_string(),
                    bytes(kernel.statistics().exact_program_bytes),
                    kernel.dimension().to_string(),
                    chart
                        .and_then(|c| c.pre_subtraction())
                        .map(|p| p.terms().len().to_string())
                        .unwrap_or_else(|| "?".into()),
                    leading,
                ]
            })
            .collect();
        out.push_str(&section(
            "Largest sector evaluators · top 10",
            vec![
                "Sector",
                "Evaluator",
                "Vars",
                "Terms",
                "Pre-subtraction monomial",
            ],
            rows,
            width,
            colors,
        ));
        out.push_str(&heading(
            "Sorted by serialized evaluator bytes; IDs are zero-based saved kernel indices.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
        if kernels.sectors().is_empty() {
            out.push_str(&heading(
                "No numerical sectors; exact coefficients may still contribute.",
                width,
                colors,
                Color::FG_BRIGHT_BLACK,
            ));
        } else {
            out.push_str(&heading(
                "Use --sector <id> for monomials, evaluator details and coordinate maps.",
                width,
                colors,
                Color::FG_BRIGHT_BLACK,
            ));
        }
        if expressions {
            out.push('\n');
            out.push_str(&input_expressions(kernels, width, colors));
        }
    }
    if kernels.generation_metadata().is_none() && kernels.threshold_metadata().is_none() {
        out.push_str(&heading(
            "Retained chart metadata is unavailable in this older artifact.",
            width,
            colors,
            Color::FG_BRIGHT_BLACK,
        ));
    }
    Ok(out)
}

fn representative(kernels: &KernelSet, id: usize) -> Option<&ChartRecord> {
    kernels
        .generation_metadata()?
        .charts()
        .iter()
        .find(|chart| {
            chart.kernel_sector() == Some(id) && chart.source_index() == chart.representative()
        })
}

/// Assemble only the retained endpoint factors using native Atom arithmetic.
/// Equality is native Atom equality; no common-power inference or CAS is added.
fn monomials(chart: &ChartRecord) -> Vec<(Atom, usize)> {
    let mut groups: Vec<(Atom, usize)> = Vec::new();
    if let Some(record) = chart.pre_subtraction() {
        for term in record.terms() {
            let value = monomial(chart, term);
            if let Some((_, count)) = groups.iter_mut().find(|(old, _)| *old == value) {
                *count += 1;
            } else {
                groups.push((value, 1));
            }
        }
    }
    groups
}
fn monomial(chart: &ChartRecord, term: &PreSubtractionTerm) -> Atom {
    chart
        .coordinates()
        .target_parameters()
        .iter()
        .zip(term.powers())
        .fold(Atom::num(1), |value, (parameter, power)| {
            value * Atom::var(*parameter).pow(power.exponent())
        })
}

fn sector(
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
    expressions: bool,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    sector_labeled(artifact, kernels, id, id, None, expressions, width, colors)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_record(
    path: &Path,
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
    sources: &[usize],
    expressions: bool,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    let mut output = super::overview::render(
        path,
        artifact,
        &artifact.kernel_summary()?,
        None,
        width,
        colors,
    )?;
    output.push('\n');
    output.push_str(&sector_labeled(
        artifact,
        kernels,
        0,
        id,
        Some(sources),
        expressions,
        width,
        colors,
    )?);
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn sector_labeled(
    artifact: &Artifact,
    kernels: &KernelSet,
    id: usize,
    display_id: usize,
    source_indices: Option<&[usize]>,
    expressions: bool,
    width: usize,
    colors: ColorPolicy,
) -> CliResult<String> {
    let chart_id = |id| source_indices.map_or(id, |indices| indices[id]);
    let kernel = &kernels.sectors()[id];
    let stats = kernel.statistics();
    let chart_modes = super::source_chart_modes_mapped(artifact, kernels, id, source_indices);
    let mode_names = chart_modes
        .as_ref()
        .into_iter()
        .flat_map(|modes| modes.values())
        .map(|mode| mode.name())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let content_id = kernels.sector_content_id(id)?;
    let mut out = heading(
        &format!("Sector {display_id}"),
        width,
        colors,
        Color::FG_GREEN,
    );
    let ops = stats.operations;
    out.push_str(&facts_table(
        vec![
            ["Sector ID".into(), display_id.to_string()],
            [
                "Generation mode".into(),
                if mode_names.is_empty() {
                    "Not recorded".into()
                } else {
                    mode_names.join(", ")
                },
            ],
            ["Content ID".into(), content_id],
            ["Coordinates".into(), kernel.dimension().to_string()],
            ["Arithmetic".into(), stats.arithmetic.clone()],
            [
                "Backend".into(),
                match stats.backend.as_str() {
                    "symjit_o2" => "SymJIT · O2",
                    "symbolica_interpreter" => "Symbolica interpreter",
                    other => other,
                }
                .into(),
            ],
            [
                "Evaluator data".into(),
                format!(
                    "{} ({} bytes)",
                    bytes(stats.exact_program_bytes),
                    stats.exact_program_bytes
                ),
            ],
            [
                "SymJIT data".into(),
                stats
                    .symjit_ir_bytes
                    .map(|value| format!("{} ({value} bytes)", bytes(value)))
                    .unwrap_or_else(|| "Not applicable".into()),
            ],
            [
                "Inputs".into(),
                format!("{} (coordinates + runtime parameters)", stats.inputs),
            ],
            [
                "Outputs".into(),
                format!(
                    "{} {} coefficients → {} scalar components",
                    stats.outputs,
                    stats.arithmetic,
                    kernel.output_count()
                ),
            ],
            [
                "Operations".into(),
                format!(
                    "{} additions; {} multiplications; {} inversions; {} function calls",
                    ops.additions, ops.multiplications, ops.inversions, ops.function_calls
                ),
            ],
        ],
        width,
        colors,
    ));
    out.push('\n');
    out.push_str(&heading("Evaluator size is serialized exact program data; SymJIT size is its compressed application. Neither is machine-code size.",width,colors,Color::FG_BRIGHT_BLACK));
    out.push_str(&heading("Operations describe the shared Laurent-vector program after native symbolic optimization, before real/complex lowering and backend optimization.",width,colors,Color::FG_BRIGHT_BLACK));
    out.push_str(&super::threshold::native_facts(kernels, width, colors));
    let Some(metadata) = kernels.generation_metadata() else {
        return Ok(out);
    };
    let charts = metadata
        .charts()
        .iter()
        .filter(|chart| chart.kernel_sector() == Some(id))
        .collect::<Vec<_>>();
    out.push('\n');
    if let Some(chart) = representative(kernels, id) {
        let groups = monomials(chart);
        let context = chart_context(chart);
        let cap = if expressions { usize::MAX } else { 10 };
        let rows = groups
            .iter()
            .take(cap)
            .enumerate()
            .map(|(index, (value, count))| {
                vec![
                    index.to_string(),
                    count.to_string(),
                    math_display::in_context(value, &context, colors, width.saturating_sub(30)),
                ]
            })
            .collect();
        out.push_str(&section(
            &format!(
                "Pre-subtraction monomial factors · representative chart {}",
                chart_id(chart.source_index())
            ),
            vec!["Group", "Terms", "Monomial"],
            rows,
            width,
            colors,
        ));
        if groups.len() > cap {
            out.push_str(&heading(&format!("Showing {cap} of {} groups; --expressions shows every group and term prefactor.",groups.len()),width,colors,Color::FG_BRIGHT_BLACK));
        }
        out.push_str(&heading("Term counts precede symmetry multiplicity. The regular body is not retained and may vanish; these factors do not determine surviving poles.",width,colors,Color::FG_BRIGHT_BLACK));
        if expressions && let Some(record) = chart.pre_subtraction() {
            let rows = record
                .terms()
                .iter()
                .enumerate()
                .map(|(index, term)| {
                    vec![
                        index.to_string(),
                        bytes(term.regular_expression_bytes()),
                        math_display::in_context(
                            term.prefactor(),
                            &context,
                            colors,
                            width.saturating_sub(30),
                        ),
                    ]
                })
                .collect();
            out.push('\n');
            out.push_str(&section(
                "Pre-subtraction term prefactors",
                vec![
                    "Term",
                    match chart_modes
                        .as_ref()
                        .and_then(|modes| modes.get(&chart_id(chart.source_index())))
                    {
                        Some(fastsecdec::generation::GenerationMode::NumericalDual) => {
                            "Source bytes"
                        }
                        Some(fastsecdec::generation::GenerationMode::Symbolic) => "Mapped bytes",
                        None => "Body bytes",
                    },
                    "Prefactor",
                ],
                rows,
                width,
                colors,
            ));
        }
    }
    out.push('\n');
    out.push_str(&heading(
        &format!("Coordinate maps · {} contributing charts", charts.len()),
        width,
        colors,
        Color::FG_CYAN,
    ));
    out.push_str(&heading("Maps and positive measure factors describe the density before endpoint subtraction. Compiled boundary terms can have fewer active coordinates.",width,colors,Color::FG_BRIGHT_BLACK));
    for chart in charts {
        out.push('\n');
        let map = chart.coordinates();
        let context = chart_context(chart);
        let mut rows = Vec::new();
        for (source, image) in map.source_parameters().iter().zip(map.images()) {
            rows.push(vec![
                math_display::in_context(&Atom::var(*source), &context, colors, width),
                math_display::in_context(image, &context, colors, width.saturating_sub(20)),
            ]);
        }
        rows.push(vec![
            "Positive measure".into(),
            math_display::in_context(
                map.measure_jacobian(),
                &context,
                colors,
                width.saturating_sub(20),
            ),
        ]);
        out.push_str(&section(
            &format!(
                "Chart {} → representative {}",
                chart_id(chart.source_index()),
                chart_id(chart.representative())
            ),
            vec!["Input", "Sector expression"],
            rows,
            width,
            colors,
        ));
        if let Some(fixed) = map.projective_fixed_parameter() {
            out.push_str(&heading(&format!("Projective gauge: input parameter {fixed} is fixed to 1; images are not normalized simplex coordinates."),width,colors,Color::FG_BRIGHT_BLACK));
        }
        if chart.source_index() != chart.representative()
            && let Some(target) = metadata
                .charts()
                .iter()
                .find(|r| r.source_index() == chart.representative())
        {
            let mut context = chart_context(chart);
            context.extend(chart_context(target));
            let rows = map
                .target_parameters()
                .iter()
                .zip(chart.representative_permutation())
                .map(|(from, &to)| {
                    let pair = [
                        Atom::var(*from),
                        Atom::var(target.coordinates().target_parameters()[to]),
                    ];
                    vec![
                        math_display::in_context(&pair[0], &context, colors, width),
                        math_display::in_context(&pair[1], &context, colors, width),
                    ]
                })
                .collect();
            out.push_str(&section(
                "Permutation into representative coordinates",
                vec!["Chart coordinate", "Representative"],
                rows,
                width,
                colors,
            ));
        }
    }
    Ok(out)
}

fn chart_context(chart: &ChartRecord) -> Vec<Atom> {
    let map = chart.coordinates();
    let mut context = map
        .source_parameters()
        .iter()
        .chain(map.target_parameters())
        .map(|symbol| Atom::var(*symbol))
        .chain(map.images().iter().cloned())
        .collect::<Vec<_>>();
    context.push(map.measure_jacobian().clone());
    if let Some(record) = chart.pre_subtraction() {
        context.push(Atom::var(record.regulator()));
        for term in record.terms() {
            context.push(term.prefactor().clone());
            context.extend(term.powers().iter().map(|power| power.exponent().clone()));
        }
    }
    context
}

fn input_expressions(kernels: &KernelSet, width: usize, colors: ColorPolicy) -> String {
    let Some(metadata) = kernels.generation_metadata() else {
        return String::new();
    };
    let context = metadata
        .domain_assessment()
        .factors()
        .iter()
        .flat_map(|factor| [factor.polynomial().clone(), factor.exponent().clone()])
        .collect::<Vec<_>>();
    let rows = metadata
        .domain_assessment()
        .factors()
        .iter()
        .map(|factor| {
            vec![
                factor.term_index().to_string(),
                factor.factor_index().to_string(),
                math_display::in_context(
                    &factor.polynomial().pow(factor.exponent()),
                    &context,
                    colors,
                    width.saturating_sub(30),
                ),
            ]
        })
        .collect();
    section(
        "Retained input factors",
        vec!["Term", "Factor", "Expression"],
        rows,
        width,
        colors,
    )
}
