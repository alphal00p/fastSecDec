use super::super::{
    MapJob, PreparedChartSource, PreparedRecipeSet, StreamingError,
    codec::{self, invalid},
    prepare::poll,
    records,
};
use crate::{
    generation::{
        GenerationMode, GenerationPhase, GenerationProgress, mapping,
        numerical_dual::{chart, pipeline::Context},
        support::SupportCache,
    },
    kernel::indexed::ProgramRecipe,
};
use std::{collections::BTreeSet, ops::ControlFlow, path::Path, time::Instant};

/// Extract one complete source chart, persist its native residuals, then release
/// it. Every selected recipe consumes this record without repeating extraction.
/// The optional undeformed dual payload uses its existing opaque source path.
pub fn prepare_chart_source(
    root: &Path,
    recipes: &PreparedRecipeSet,
    job: &MapJob,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<PreparedChartSource, StreamingError> {
    let first = recipes
        .recipes
        .first()
        .ok_or_else(|| invalid("empty prepared recipe directory"))?;
    let mut selected = BTreeSet::new();
    for recipe in &recipes.recipes {
        if !selected.insert(recipe.program_recipe)
            || recipe.source_identity != recipes.source_identity
            || recipe.mode != first.mode
            || recipe.dimension != first.dimension
            || recipe.source_scope != first.source_scope
            || recipe.charts.len() != first.charts.len()
            || recipe
                .charts
                .get(job.index)
                .is_none_or(|candidate| candidate.index != job.index || candidate.map != job.map)
        {
            return Err(invalid("inconsistent shared recipe preparation"));
        }
    }
    // Only the requested map belongs to this worker's admission. Rewalking
    // every compact catalogue here would make N independent jobs O(N²).
    let context = records::read_source(root, &first.source)?;
    if context.source_identity != recipes.source_identity
        || context.source_scope != first.source_scope
        || context.options.mode != first.mode
        || context.options.program_recipe != first.program_recipe
        || context.targets.len() != first.dimension
    {
        return Err(invalid("shared source context mismatch"));
    }
    let (map, _, _): (records::Map, _, _) = codec::read(root, &job.map, "map")?;
    let map = map.native()?;
    if map.dimension() != first.dimension {
        return Err(invalid("shared map dimension mismatch"));
    }
    poll(
        &mut progress,
        GenerationProgress::Factorization {
            sector: job.index,
            total: first.charts.len(),
        },
    )?;
    let started = Instant::now();
    let coordinates = mapping::coordinates(&context.input, &map, &context.targets);
    let mut supports = SupportCache::new(context.input.parameters());
    let retains_declarations = selected.iter().any(|recipe| recipe.is_contour());
    let terms = if first.mode == GenerationMode::Symbolic || retains_declarations {
        Some(mapping::prepare_terms(
            &context.input,
            &map,
            &coordinates,
            &mut supports,
            retains_declarations,
        )?)
    } else {
        None
    };
    let opaque = if first.mode == GenerationMode::NumericalDual
        && selected.contains(&ProgramRecipe::UndeformedV1)
    {
        let mut options = context.options.clone();
        options.program_recipe = ProgramRecipe::UndeformedV1;
        let dual = Context::new(&context.input, &options, context.targets.clone());
        Some(chart::prepare_undeformed(
            &dual,
            &map,
            &coordinates,
            &mut supports,
        )?)
    } else {
        None
    };
    let data = records::PreparedData {
        index: job.index,
        source_identity: recipes.source_identity.clone(),
        map_reference: job.map.clone(),
        map,
        parameters: context.targets,
        retains_declarations,
        terms,
        opaque,
    };
    let record = records::write_prepared(root, &data)?;
    poll(
        &mut progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    Ok(PreparedChartSource {
        index: job.index,
        source_identity: recipes.source_identity.clone(),
        map: job.map.clone(),
        dimension: first.dimension,
        record,
    })
}
