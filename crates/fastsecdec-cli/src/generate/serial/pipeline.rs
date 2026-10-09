//! Ordered, recipe-local symmetry, formula and compilation pipeline.
use super::{
    jobs::{Compiled, Prepared, Request, Response},
    journal::Journal,
    runner::Runner,
};
use crate::{CliResult, display::Dashboard};
use fastsecdec::{
    generation::{GenerationMode, streaming as native},
    kernel::CompilationSettings,
    status::{FormulaPreparationSnapshot, GenerationStage},
};
use std::{collections::BTreeMap, path::Path, time::Instant};

pub(super) fn run(
    runner: &mut Runner,
    journal: &mut Journal,
    preparation_path: &Path,
    prepared: &mut Prepared,
    charts: Vec<native::DiscoveredSector>,
    evaluator: CompilationSettings,
    dashboard: &mut Dashboard,
) -> CliResult<Vec<Compiled>> {
    let recipe = prepared.native.program_recipe;
    let timings = &mut prepared.timings;
    let phase = Instant::now();
    let mut assignments = Vec::with_capacity(charts.len());
    if prepared.native.mode == GenerationMode::Symbolic {
        let mut representatives = BTreeMap::<String, Vec<native::DiscoveredSector>>::new();
        // Native exact admission is ordered. Canonicalization and heavyweight
        // mapped records were already built independently in the discovery pass.
        for chart in &charts {
            let key = chart
                .symmetry_key
                .clone()
                .ok_or("symbolic discovery omitted its symmetry bucket")?;
            let candidates = representatives.get(&key).cloned().unwrap_or_default();
            let assignment = if candidates.is_empty() {
                native::SymmetryAssignment {
                    program_recipe: chart.program_recipe,
                    source_id: chart.source_id.clone(),
                    source: chart.index,
                    representative: chart.index,
                    permutation: (0..chart.dimension).collect(),
                }
            } else {
                let response = runner
                    .run(
                        journal,
                        vec![(
                            format!("{}-symmetry-{}", recipe.name(), chart.index),
                            Request::Symmetry {
                                preparation: preparation_path.to_path_buf(),
                                chart: chart.clone(),
                                candidates,
                            },
                        )],
                        GenerationStage::Symmetry,
                        "Exact sector equivalence",
                        dashboard,
                    )?
                    .pop()
                    .ok_or("missing symmetry result")?;
                let Response::Symmetry(assignment) = response else {
                    return Err("invalid symmetry completion".into());
                };
                assignment
            };
            if assignment.representative == chart.index {
                representatives.entry(key).or_default().push(chart.clone());
            }
            assignments.push(assignment);
        }
    } else {
        assignments.extend(charts.iter().map(|chart| native::SymmetryAssignment {
            program_recipe: chart.program_recipe,
            source_id: chart.source_id.clone(),
            source: chart.index,
            representative: chart.index,
            permutation: (0..chart.dimension).collect(),
        }));
    }
    timings.symmetry_seconds = phase.elapsed().as_secs_f64();
    let phase = Instant::now();
    let mut formula_sources = BTreeMap::new();
    let formula_uses = charts
        .iter()
        .filter(|chart| chart.formula_key.is_some())
        .count();
    runner.formula_uses(formula_uses);
    for chart in &charts {
        if let Some(key) = &chart.formula_key {
            formula_sources
                .entry(key.clone())
                .or_insert_with(|| chart.clone());
        }
    }
    let formula_jobs = formula_sources
        .into_iter()
        .map(|(key, chart)| {
            (
                format!("{}-formula-{key}", recipe.name()),
                Request::Formula {
                    preparation: preparation_path.to_path_buf(),
                    chart,
                },
            )
        })
        .collect();
    let formula_results = if prepared.native.mode == GenerationMode::NumericalDual {
        runner.run(
            journal,
            formula_jobs,
            GenerationStage::FormulaPreparation,
            "Preparing reusable subtraction formulas",
            dashboard,
        )?
    } else {
        Vec::new()
    };
    let formulas = formula_results
        .into_iter()
        .map(|r| match r {
            Response::Formula(f) => Ok(f),
            _ => Err("invalid formula completion".into()),
        })
        .collect::<CliResult<Vec<_>>>()?;
    if prepared.native.mode == GenerationMode::NumericalDual {
        timings.formula_preparation_seconds = Some(phase.elapsed().as_secs_f64());
    }
    let plan = native::finish_preparation(&prepared.native, charts, assignments, formulas)?;

    if prepared.native.mode == GenerationMode::NumericalDual {
        prepared.generation.formula_preparation = Some(FormulaPreparationSnapshot {
            completed: plan.unique_formulas,
            total: plan.unique_formulas,
            sectors: formula_uses,
            reused: formula_uses.saturating_sub(plan.unique_formulas),
        });
    }
    let phase = Instant::now();
    let sector_jobs = plan
        .sectors
        .into_iter()
        .map(|job| {
            let output =
                journal
                    .root
                    .join(format!("{}-sector-{}.native", recipe.name(), job.index));
            (
                format!("{}-sector-{}", recipe.name(), job.index),
                Request::Sector {
                    job,
                    max_order: prepared.provenance.max_order,
                    evaluator,
                    output,
                },
            )
        })
        .collect();
    let mut compiled = runner
        .run(
            journal,
            sector_jobs,
            GenerationStage::CoefficientExpansion,
            "Complete, persist and release each sector",
            dashboard,
        )?
        .into_iter()
        .map(|r| match r {
            Response::Compiled(c) => Ok(c),
            _ => Err("invalid sector completion".into()),
        })
        .collect::<CliResult<Vec<_>>>()?;
    // Fused sector jobs overlap expansion, optimization, compilation and writes.
    // Record their enclosing wall interval once rather than double counting.
    timings.coefficient_expansion_seconds = phase.elapsed().as_secs_f64();
    compiled.sort_by_key(|c| c.source_index);
    let mut source_modes = BTreeMap::new();
    for unit in &compiled {
        source_modes.extend(unit.source_chart_modes.clone());
    }
    prepared.generation.source_chart_modes = Some(source_modes);
    Ok(compiled)
}
