//! CLI process orchestration for bounded-memory generation and recovery.
pub(crate) mod jobs;
mod journal;
#[cfg(test)]
mod recovery_tests;
mod runner;

use crate::{
    CliResult, artifact::Artifact, config::RunCard, display::Dashboard,
    reference::PreparedReference,
};
use fastsecdec::{
    generation::{GenerationMode, streaming as native},
    kernel::indexed::IndexedWriter,
    status::{FormulaPreparationSnapshot, GenerationSnapshot, GenerationStage},
};
use jobs::{Prepared, Request, Response};
use journal::Journal;
use runner::Runner;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

/// All heavyweight work takes place in CLI-owned recyclable child processes.
/// This coordinator only sees compact native receipts and stream-copies bytes.
pub(crate) fn generate_with_overrides(
    path: &Path,
    output: &Path,
    dashboard: &mut Dashboard,
    reference: Option<&PreparedReference>,
    workers: usize,
    resume: bool,
    overrides: crate::config::GenerationOverrides,
) -> CliResult<Artifact> {
    crate::artifact::paths(output)?;
    if workers == 0 {
        return Err("generation workers must be positive".into());
    }
    let path = fs::canonicalize(path)?;
    let output = std::path::absolute(output)?;
    let mut card: RunCard = toml::from_str(&fs::read_to_string(&path)?)?;
    overrides.apply(&mut card);
    card.generation.evaluator.validate()?;
    dashboard.configure_generation(
        card.generation.mode,
        card.generation.coefficient_expansion.method,
    );
    let run_id = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let mut journal = Journal::open_with_overrides(&path, &output, resume, &run_id, overrides)?;
    if journal.completed() {
        let artifact = Artifact::load_metadata(&output)?;
        artifact.verify_input_sources(&path)?;
        if !artifact.dependencies_compatible() {
            return Err("completed generation dependencies changed".into());
        }
        return Ok(artifact);
    }
    let mut runner = Runner::new(workers, run_id, journal.residency_lock())?;
    let prepared = runner
        .run(
            &mut journal,
            vec![(
                "prepare".into(),
                Request::Prepare {
                    input: path.clone(),
                    workers,
                    overrides,
                },
            )],
            GenerationStage::Geometry,
            "Native source and geometry",
            dashboard,
        )?
        .pop()
        .ok_or("missing preparation result")?;
    let Response::Prepared(mut prepared) = prepared else {
        return Err("invalid preparation completion".into());
    };
    let preparation_path = journal.response_path("prepare");
    let mut timings = prepared.timings.clone();
    let phase = Instant::now();
    let discovery_jobs = prepared
        .native
        .charts
        .iter()
        .map(|map| {
            (
                format!("discover-{}", map.index),
                Request::Discover {
                    preparation: preparation_path.clone(),
                    program_recipe: prepared.native.program_recipe,
                    source_id: prepared.native.source.blake3.clone(),
                    dimension: prepared.native.dimension,
                    index: map.index,
                },
            )
        })
        .collect();
    let mut charts = runner
        .run(
            &mut journal,
            discovery_jobs,
            GenerationStage::Mapping,
            "Spooling independent mapped charts",
            dashboard,
        )?
        .into_iter()
        .map(|r| match r {
            Response::Discovered(c) => Ok(c),
            _ => Err("invalid discovery completion".into()),
        })
        .collect::<CliResult<Vec<_>>>()?;
    charts.sort_by_key(|c| c.index);
    timings.mapping_seconds = phase.elapsed().as_secs_f64();
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
                        &mut journal,
                        vec![(
                            format!("symmetry-{}", chart.index),
                            Request::Symmetry {
                                preparation: preparation_path.clone(),
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
                format!("formula-{key}"),
                Request::Formula {
                    preparation: preparation_path.clone(),
                    chart,
                },
            )
        })
        .collect();
    let formula_results = if prepared.native.mode == GenerationMode::NumericalDual {
        runner.run(
            &mut journal,
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
    prepared.generation.workers = workers;
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
            let output = journal.root.join(format!("sector-{}.native", job.index));
            (
                format!("sector-{}", job.index),
                Request::Sector {
                    job,
                    max_order: prepared.provenance.max_order,
                    evaluator: card.generation.evaluator,
                    output,
                },
            )
        })
        .collect();
    let mut compiled = runner
        .run(
            &mut journal,
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
    dashboard.generation_saving(runner.elapsed());
    let staged = journal.root.join("complete.fsd.dat");
    let mut writer = IndexedWriter::new(BufWriter::new(File::create(&staged)?))?;
    for unit in compiled {
        if dashboard.cancelled() {
            return Err("generation cancelled before publication; use --resume".into());
        }
        let mut data = File::open(unit.data)?;
        for receipt in unit.receipts {
            writer.append_record(&mut data, receipt)?;
        }
    }
    let (mut data, catalogue) = writer.finish()?;
    data.flush()?;
    data.get_ref().sync_all()?;
    drop(data);
    let sector_count = catalogue.sector_count();
    if let Some(reference) = reference.filter(|_| catalogue.runtime_parameters.is_empty()) {
        reference.validate_identity(&catalogue.content_id)?;
    }
    let Prepared {
        provenance,
        generation,
        ..
    } = *prepared;
    let mut artifact = Artifact::from_indexed_file(&staged, catalogue, provenance)?;
    artifact.reference = reference.map(|r| r.settings.clone());
    artifact.relocate_sources(
        path.parent().unwrap_or_else(|| Path::new(".")),
        output.parent().unwrap_or_else(|| Path::new(".")),
    )?;
    timings.total_seconds = runner.elapsed();
    artifact.generation_timings = Some(timings.clone());
    artifact.generation = Some(generation);
    artifact.save_staged(&output)?;
    journal.complete()?;
    let artifact = Artifact::load_metadata(&output)?;
    dashboard.generation(&GenerationSnapshot {
        stage: GenerationStage::Complete,
        completed: sector_count,
        total: Some(sector_count),
        sectors: sector_count,
        kernels: sector_count,
        elapsed_seconds: runner.elapsed(),
        timings,
        coefficient_expansion: None,
        formula_preparation: artifact
            .generation
            .as_ref()
            .and_then(|g| g.formula_preparation),
        detail: format!(
            "Saved {} · all sector processes released",
            crate::artifact::relative_display(&output)
        ),
    })?;
    journal.cleanup()?;
    Ok(artifact)
}
