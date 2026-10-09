//! File-backed preparation and exact deterministic representative admission.
use super::{
    codec::{self, invalid},
    records::{self, ChartData},
    *,
};
use crate::generation::{
    GenerationError, GenerationEvent, GenerationMode, GenerationOptions, GenerationProgress,
    numerical_dual::{self, formula::Key, pipeline::Context as DualContext},
    support::SupportCache,
    symmetry, work,
};
use crate::parametric::ParametricIntegrand;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
    path::Path,
};

pub(super) fn event(
    progress: &mut impl FnMut(&GenerationProgress) -> ControlFlow<()>,
    status: &GenerationEvent,
) -> ControlFlow<()> {
    match status {
        GenerationEvent::Progress(status) => progress(status),
        GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
    }
}
pub(super) fn poll(
    progress: &mut impl FnMut(&GenerationProgress) -> ControlFlow<()>,
    status: GenerationProgress,
) -> Result<(), StreamingError> {
    if progress(&status).is_break() {
        Err(GenerationError::Cancelled.into())
    } else {
        Ok(())
    }
}

/// Shared native source/geometry preparation. Run this in a recyclable caller
/// process: it owns no pool and returns only immutable file descriptors.
pub fn prepare(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    root: &Path,
    progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<Preparation, StreamingError> {
    prepare_with_runtime(input, options, &[], &[], root, progress)
}

/// Freeze runtime symbols and mass constraints in the native source context,
/// so sector workers never reparse a graph or reconstruct symbol definitions.
pub fn prepare_with_runtime(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    runtime_parameters: &[symbolica::atom::Symbol],
    runtime_mass_constraints: &[crate::kernel::RuntimeMassConstraint],
    root: &Path,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<Preparation, StreamingError> {
    let mut prepared = super::shared::prepare_recipes_with_runtime(
        input,
        options,
        &[options.program_recipe],
        runtime_parameters,
        runtime_mass_constraints,
        root,
        &mut progress,
    )?;
    Ok(prepared.recipes.remove(0))
}

/// Map one source chart and persist it immediately; no completed chart owner
/// escapes this call. Numerical-dual discovery keeps source expressions opaque.
pub fn discover(
    root: &Path,
    preparation: &Preparation,
    job: &MapJob,
    progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<DiscoveredSector, StreamingError> {
    if preparation
        .charts
        .get(job.index)
        .is_none_or(|j| j.map != job.map)
    {
        return Err(invalid("foreign geometry job"));
    }
    let context = records::read_source(root, &preparation.source)?;
    if context.options.mode != preparation.mode
        || context.options.program_recipe != preparation.program_recipe
        || context.source_identity != preparation.source_identity
    {
        return Err(invalid("source generation mode mismatch"));
    }
    let (map, _, _): (records::Map, _, _) = codec::read(root, &job.map, "map")?;
    let map = map.native()?;
    discover_loaded(root, preparation, context, map, job.index, None, progress)
}

pub(super) fn discover_loaded(
    root: &Path,
    preparation: &Preparation,
    context: records::Context,
    map: fastsecdec_sectors::SectorMap,
    index: usize,
    prepared: Option<records::PreparedData>,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<DiscoveredSector, StreamingError> {
    if context.options.mode != preparation.mode
        || context.options.program_recipe != preparation.program_recipe
        || context.source_identity != preparation.source_identity
        || context.targets.len() != preparation.dimension
    {
        return Err(invalid("source generation context mismatch"));
    }
    let mut supports = SupportCache::new(context.input.parameters());
    let (data, symmetry_key, formula_key) = if context.options.mode == GenerationMode::Symbolic {
        let chart = if let Some(prepared) = prepared {
            work::map_prepared_chart(
                &context.input,
                &context.options,
                map,
                context.targets.clone(),
                prepared
                    .terms
                    .ok_or_else(|| invalid("prepared symbolic source lacks residual terms"))?,
                &mut |status| event(&mut progress, status),
            )?
        } else {
            work::map_chart(
                &context.input,
                &context.options,
                map,
                context.targets.clone(),
                &mut supports,
                &mut |status| event(&mut progress, status),
            )?
        };
        let prepared = symmetry::prepare_mapped(index, &chart.parameters, &chart.mapped, || {
            if progress(&GenerationProgress::SymmetryPreparation {
                sector: index,
                total: preparation.charts.len(),
            })
            .is_break()
            {
                Err(GenerationError::Cancelled)
            } else {
                Ok(())
            }
        })?;
        let key = prepared.lookup_key();
        drop(prepared);
        (
            ChartData {
                index,
                source_id: preparation.source.blake3.clone(),
                map: chart.map,
                mapped: chart.mapped,
                deferred: None,
                contour: chart.contour,
                program: chart.program,
            },
            Some(key),
            None,
        )
    } else {
        let dual = DualContext::new(&context.input, &context.options, context.targets.clone());
        let chart = if let Some(prepared) = prepared {
            numerical_dual::chart::discover_prepared(
                &dual,
                map,
                index,
                preparation.charts.len(),
                prepared.terms,
                prepared.opaque,
                &mut |status| event(&mut progress, status),
            )?
        } else {
            numerical_dual::chart::discover(
                &dual,
                map,
                index,
                preparation.charts.len(),
                &mut supports,
                &mut |status| event(&mut progress, status),
            )?
        };
        let key = chart.key.as_ref().map(Key::lookup_key);
        (
            ChartData {
                index,
                source_id: preparation.source.blake3.clone(),
                map: chart.map,
                mapped: chart.mapped,
                deferred: Some(chart.terms),
                contour: chart.contour,
                program: chart.program.select(&[index])?,
            },
            None,
            key,
        )
    };
    let record = records::write_chart(root, &data)?;
    Ok(DiscoveredSector {
        program_recipe: preparation.program_recipe,
        index,
        source_id: preparation.source.blake3.clone(),
        dimension: context.targets.len(),
        record,
        symmetry_key,
        formula_key,
    })
}

/// Compare an ordered list of earlier representatives. The caller selects the
/// matching lookup bucket and admits replies in original source-chart order.
/// One candidate is restored at a time; lookup hashes never establish equality.
pub fn compare_symmetry(
    root: &Path,
    preparation: &Preparation,
    chart: &DiscoveredSector,
    candidates: &[DiscoveredSector],
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<SymmetryAssignment, StreamingError> {
    let context = records::read_source(root, &preparation.source)?;
    let data = records::read_chart(root, &chart.record)?;
    if chart.program_recipe != preparation.program_recipe {
        return Err(invalid("chart program recipe mismatch"));
    }
    if data.index != chart.index
        || data.source_id != preparation.source.blake3
        || chart.source_id != data.source_id
    {
        return Err(invalid("chart index or source identity mismatch"));
    }
    let identity = SymmetryAssignment {
        program_recipe: preparation.program_recipe,
        source_id: preparation.source.blake3.clone(),
        source: chart.index,
        representative: chart.index,
        permutation: (0..context.targets.len()).collect(),
    };
    if context.options.mode == GenerationMode::NumericalDual {
        if !candidates.is_empty() {
            return Err(invalid(
                "numerical-dual charts do not use symbolic symmetry admission",
            ));
        }
        return Ok(identity);
    }
    let mut abort = || {
        if progress(&GenerationProgress::Symmetry {
            completed: chart.index,
            total: preparation.charts.len(),
        })
        .is_break()
        {
            Err(GenerationError::Cancelled)
        } else {
            Ok(())
        }
    };
    let prepared =
        symmetry::prepare_mapped(chart.index, &context.targets, &data.mapped, &mut abort)?;
    let _helper_owner = data.program.descriptor.clone();
    drop(data);
    let mut previous = None;
    for candidate in candidates {
        if candidate.program_recipe != preparation.program_recipe
            || candidate.source_id != preparation.source.blake3
            || candidate.index >= chart.index
            || previous.is_some_and(|p| p >= candidate.index)
        {
            return Err(invalid(
                "symmetry candidates must be earlier ordered representatives",
            ));
        }
        previous = Some(candidate.index);
        abort()?;
        if candidate.symmetry_key != chart.symmetry_key {
            continue;
        }
        let target = records::read_chart(root, &candidate.record)?;
        if target.index != candidate.index
            || target.source_id != preparation.source.blake3
            || candidate.source_id != target.source_id
        {
            return Err(invalid("candidate chart index or source identity mismatch"));
        }
        // The prepared symmetry form contains callback expressions but does
        // not itself own their native helper programs. Retain this candidate's
        // owner independently of the active chart and its helper arity.
        let _candidate_helper_owner = target.program.descriptor.clone();
        let target = symmetry::prepare_mapped(
            candidate.index,
            &context.targets,
            &target.mapped,
            &mut abort,
        )?;
        if let Some(permutation) = prepared.equivalent_to(&target)? {
            return Ok(SymmetryAssignment {
                program_recipe: preparation.program_recipe,
                source_id: preparation.source.blake3.clone(),
                source: chart.index,
                representative: candidate.index,
                permutation,
            });
        }
    }
    Ok(identity)
}

pub(super) fn key(context: &DualContext, data: &ChartData) -> Result<Option<Key>, GenerationError> {
    if context.eligible && !data.map.exponent_matrix.iter().flatten().any(|p| p < &0) {
        Key::discover(
            &data.mapped,
            context.parameters.len(),
            context.input.regulator(),
            &context.options,
        )
    } else {
        Ok(None)
    }
}

/// Build one exact formula shared by all charts with the identical native key.
pub fn build_formula(
    root: &Path,
    preparation: &Preparation,
    chart: &DiscoveredSector,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<FormulaRecord, StreamingError> {
    let source = records::read_source(root, &preparation.source)?;
    let context = DualContext::new(&source.input, &source.options, source.targets);
    let data = records::read_chart(root, &chart.record)?;
    if chart.program_recipe != preparation.program_recipe {
        return Err(invalid("chart program recipe mismatch"));
    }
    if data.index != chart.index
        || data.source_id != preparation.source.blake3
        || chart.source_id != data.source_id
    {
        return Err(invalid("formula chart source mismatch"));
    }
    let key = key(&context, &data)?.ok_or_else(|| {
        invalid("chart requires exact physical fallback, not a reusable dual formula")
    })?;
    let lookup = key.lookup_key();
    if chart.formula_key.as_ref() != Some(&lookup) {
        return Err(invalid("formula key mismatch"));
    }
    let _helper_owner = data.program.descriptor.clone();
    drop(data);
    let recipe = numerical_dual::formula::build(
        &key,
        &context,
        0,
        numerical_dual::formula::Counts {
            total: 1,
            sectors: 1,
        },
        &mut |status| event(&mut progress, status),
    )?;
    let record = records::write_formula(
        root,
        &lookup,
        &key.terms(context.input.regulator()),
        &recipe,
    )?;
    Ok(FormulaRecord {
        program_recipe: preparation.program_recipe,
        key: lookup,
        source_id: preparation.source.blake3.clone(),
        record,
    })
}

/// Assemble work descriptors only. This is safe in the long-lived coordinator;
/// it never reads a native record or interns an expression.
pub fn finish_preparation(
    preparation: &Preparation,
    mut charts: Vec<DiscoveredSector>,
    mut assignments: Vec<SymmetryAssignment>,
    formulas: Vec<FormulaRecord>,
) -> Result<PreparedGeneration, StreamingError> {
    charts.sort_by_key(|c| c.index);
    assignments.sort_by_key(|a| a.source);
    if charts.len() != preparation.charts.len()
        || assignments.len() != charts.len()
        || charts.iter().enumerate().any(|(i, c)| {
            c.index != i
                || c.program_recipe != preparation.program_recipe
                || c.source_id != preparation.source.blake3
                || c.dimension != preparation.dimension
        })
        || assignments.iter().enumerate().any(|(i, a)| {
            a.program_recipe != preparation.program_recipe
                || a.source_id != preparation.source.blake3
                || a.source != i
                || a.representative > i
                || a.permutation.len() != preparation.dimension
        })
    {
        return Err(invalid(
            "incomplete, foreign or duplicate preparation completions",
        ));
    }
    let mut by_formula = BTreeMap::new();
    for f in formulas {
        if f.source_id != preparation.source.blake3
            || f.program_recipe != preparation.program_recipe
        {
            return Err(invalid("foreign formula source"));
        }
        if by_formula.insert(f.key.clone(), f).is_some() {
            return Err(invalid("duplicate formula completion"));
        }
    }
    let unique_formulas = by_formula.len();
    let mut groups = BTreeMap::<usize, Vec<ChartUse>>::new();
    let mut used_formulas = BTreeSet::new();
    for (chart, assignment) in charts.into_iter().zip(&assignments) {
        if assignments[assignment.representative].representative != assignment.representative {
            return Err(invalid("symmetry representative is not a root"));
        }
        let dimension = assignment.permutation.len();
        if assignment
            .permutation
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            != (0..dimension).collect::<BTreeSet<_>>()
        {
            return Err(invalid("invalid representative permutation"));
        }
        groups
            .entry(assignment.representative)
            .or_default()
            .push(ChartUse {
                chart,
                permutation: assignment.permutation.clone(),
            });
    }
    let total = groups.len();
    let mut sectors = Vec::with_capacity(total);
    for (index, charts) in groups {
        let expected = charts[0].chart.formula_key.as_ref();
        if charts
            .iter()
            .any(|c| c.chart.formula_key.as_ref() != expected)
        {
            return Err(invalid(
                "symmetry group has inconsistent formula requirements",
            ));
        }
        let formula = expected
            .map(|key| {
                used_formulas.insert(key.clone());
                by_formula
                    .get(key)
                    .cloned()
                    .ok_or_else(|| invalid("missing subtraction formula"))
            })
            .transpose()?;
        sectors.push(SectorJob {
            program_recipe: preparation.program_recipe,
            index,
            source: preparation.source.clone(),
            charts,
            formula,
            total,
        });
    }
    if used_formulas.len() != by_formula.len() {
        return Err(invalid("foreign formula completion"));
    }
    if sectors.is_empty() {
        sectors.push(SectorJob {
            program_recipe: preparation.program_recipe,
            index: 0,
            source: preparation.source.clone(),
            charts: vec![],
            formula: None,
            total: 0,
        });
    }
    Ok(PreparedGeneration {
        source_identity: preparation.source_identity.clone(),
        program_recipe: preparation.program_recipe,
        source: preparation.source.clone(),
        source_charts: preparation.charts.len(),
        unique_formulas,
        sectors,
    })
}
