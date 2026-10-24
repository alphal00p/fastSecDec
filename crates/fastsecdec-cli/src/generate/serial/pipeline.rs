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
use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    time::Instant,
};

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
    let assignments = if prepared.native.mode == GenerationMode::Symbolic {
        symmetry(
            runner,
            journal,
            preparation_path,
            recipe.name(),
            &charts,
            dashboard,
        )?
    } else {
        charts
            .iter()
            .map(|chart| native::SymmetryAssignment {
                program_recipe: chart.program_recipe,
                source_id: chart.source_id.clone(),
                source: chart.index,
                representative: chart.index,
                permutation: (0..chart.dimension).collect(),
            })
            .collect()
    };
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

/// Exact symmetry admission in original source-chart order within each
/// lookup bucket. A chart is compared only with the earlier representatives
/// of its own bucket, so buckets are independent ordered chains: every round
/// submits the next chart of each bucket that has candidates. Every chart
/// receives the same candidates, job key and assignment as one-at-a-time
/// admission, while independent comparisons use all workers.
fn symmetry(
    runner: &mut Runner,
    journal: &mut Journal,
    preparation_path: &Path,
    recipe: &str,
    charts: &[native::DiscoveredSector],
    dashboard: &mut Dashboard,
) -> CliResult<Vec<native::SymmetryAssignment>> {
    let mut buckets = BTreeMap::<String, VecDeque<usize>>::new();
    for (position, chart) in charts.iter().enumerate() {
        let key = chart
            .symmetry_key
            .clone()
            .ok_or("symbolic discovery omitted its symmetry bucket")?;
        buckets.entry(key).or_default().push_back(position);
    }
    let mut representatives = BTreeMap::<String, Vec<native::DiscoveredSector>>::new();
    let mut assignments = vec![None; charts.len()];
    loop {
        let mut jobs = Vec::new();
        let mut round = BTreeMap::new();
        for (key, chain) in &mut buckets {
            while let Some(position) = chain.pop_front() {
                let chart = &charts[position];
                let candidates = representatives.get(key).cloned().unwrap_or_default();
                if candidates.is_empty() {
                    representatives
                        .entry(key.clone())
                        .or_default()
                        .push(chart.clone());
                    assignments[position] = Some(native::SymmetryAssignment {
                        program_recipe: chart.program_recipe,
                        source_id: chart.source_id.clone(),
                        source: chart.index,
                        representative: chart.index,
                        permutation: (0..chart.dimension).collect(),
                    });
                    continue;
                }
                jobs.push((
                    format!("{recipe}-symmetry-{}", chart.index),
                    Request::Symmetry {
                        preparation: preparation_path.to_path_buf(),
                        chart: chart.clone(),
                        candidates,
                    },
                ));
                round.insert(chart.index, (position, key.clone()));
                break;
            }
        }
        if jobs.is_empty() {
            break;
        }
        let responses = runner.run(
            journal,
            jobs,
            GenerationStage::Symmetry,
            "Exact sector equivalence",
            dashboard,
        )?;
        let mut replies = BTreeMap::new();
        for response in responses {
            let Response::Symmetry(assignment) = response else {
                return Err("invalid symmetry completion".into());
            };
            if !round.contains_key(&assignment.source)
                || replies.insert(assignment.source, assignment).is_some()
            {
                return Err("unexpected symmetry completion".into());
            }
        }
        for (index, (position, key)) in round {
            let assignment = replies.remove(&index).ok_or("missing symmetry result")?;
            if assignment.representative == index {
                representatives
                    .entry(key)
                    .or_default()
                    .push(charts[position].clone());
            }
            assignments[position] = Some(assignment);
        }
    }
    assignments
        .into_iter()
        .map(|assignment| assignment.ok_or_else(|| "unassigned symmetry chart".into()))
        .collect()
}
