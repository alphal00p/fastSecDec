use super::super::{
    MapJob, Preparation, PreparedRecipeSet, StreamingError,
    codec::{self, Atoms, invalid},
    prepare::{event, poll},
    records,
};
use crate::{
    generation::{
        GenerationError, GenerationOptions, GenerationPhase, GenerationProgress, domain,
        geometry::{Geometry, GeometrySource},
        mapping,
        support::SupportCache,
    },
    kernel::{RuntimeMassConstraint, indexed::ProgramRecipe},
    parametric::ParametricIntegrand,
};
use fastsecdec_sectors::PolynomialSupport;
use std::{collections::BTreeSet, ops::ControlFlow, path::Path, time::Instant};
use symbolica::atom::Symbol;

/// Validate all selected recipes, compute their geometry union once, and return
/// compact source contexts referencing the same immutable geometry records.
/// `recipes` replaces the single `options.program_recipe` for this call; other
/// generation options are shared. The library owns no worker or execution loop.
pub fn prepare_recipes_with_runtime(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    recipes: &[ProgramRecipe],
    runtime_parameters: &[Symbol],
    runtime_mass_constraints: &[RuntimeMassConstraint],
    root: &Path,
    progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<PreparedRecipeSet, StreamingError> {
    prepare_recipes(
        input,
        options,
        recipes,
        runtime_parameters,
        runtime_mass_constraints,
        root,
        None,
        progress,
    )
}

/// The same shared preparation with native geometry jobs scheduled by the caller.
#[allow(clippy::too_many_arguments)]
pub fn prepare_recipes_with_runtime_and_dispatch(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    recipes: &[ProgramRecipe],
    runtime_parameters: &[Symbol],
    runtime_mass_constraints: &[RuntimeMassConstraint],
    root: &Path,
    dispatch: &mut fastsecdec_sectors::GeometryDispatch<'_>,
    progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<PreparedRecipeSet, StreamingError> {
    prepare_recipes(
        input,
        options,
        recipes,
        runtime_parameters,
        runtime_mass_constraints,
        root,
        Some(dispatch),
        progress,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_recipes(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    recipes: &[ProgramRecipe],
    runtime_parameters: &[Symbol],
    runtime_mass_constraints: &[RuntimeMassConstraint],
    root: &Path,
    dispatch: Option<&mut fastsecdec_sectors::GeometryDispatch<'_>>,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<PreparedRecipeSet, StreamingError> {
    let selected = recipes.iter().copied().collect::<BTreeSet<_>>();
    if selected.is_empty() || selected.len() != recipes.len() {
        return Err(invalid(
            "recipe preparation requires a nonempty distinct recipe set",
        ));
    }
    let started = Instant::now();
    for recipe in &selected {
        let mut settings = options.clone();
        settings.program_recipe = *recipe;
        domain::check_options(input, &settings)?;
    }
    poll(
        &mut progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Domain,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    let started = Instant::now();
    let contour = selected
        .iter()
        .any(|recipe| *recipe != ProgramRecipe::UndeformedV1);
    let mut cache = SupportCache::new(input.parameters());
    let mut supports = Vec::new();
    for factor in input.terms().iter().flat_map(|term| term.factors()) {
        if domain::is_geometry_factor(factor, contour) {
            let support = cache.get(factor).map_err(GenerationError::from)?;
            if !supports.contains(support) {
                supports.push(support.clone());
            }
        }
    }
    if supports.is_empty() {
        supports.push(
            PolynomialSupport::new(vec![vec![0; input.parameters().len()]])
                .map_err(GenerationError::from)?,
        );
    }
    let mut geometry_cache = fastsecdec_sectors::GeometryCache::new(0);
    let source = match dispatch {
        Some(dispatch) => GeometrySource::Dispatched {
            cache: &mut geometry_cache,
            dispatch,
            cancelled: &|| false,
        },
        None => GeometrySource::Uncached,
    };
    let mut geometry = if input.terms().is_empty() {
        None
    } else {
        Some(Geometry::compute(
            input.domain(),
            &supports,
            &options.decomposition,
            source,
            &mut |status| event(&mut progress, status),
        )?)
    };
    let total = geometry.as_ref().map_or(0, Geometry::len);
    let selection = crate::generation::selection::SourceSectorSelection::resolve(
        options.source_sectors.as_deref(),
        total,
    )?;
    let source_scope = selection.as_ref().map(|selection| selection.full_scope());
    poll(
        &mut progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Geometry,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    let mut maps = geometry.iter_mut().flat_map(Geometry::maps).peekable();
    let dimension = maps.peek().map_or(0, |map| map.dimension());
    let targets = mapping::target_parameters(input, dimension);
    let selected_total = selection
        .as_ref()
        .map_or(total, |selection| selection.source_sectors().len());
    let mut charts = Vec::with_capacity(selected_total);
    for (index, (_, map)) in maps
        .enumerate()
        .filter(|(original, _)| {
            selection
                .as_ref()
                .is_none_or(|selection| selection.source_sectors().binary_search(original).is_ok())
        })
        .enumerate()
    {
        poll(
            &mut progress,
            GenerationProgress::Factorization {
                sector: index,
                total: selected_total,
            },
        )?;
        let map = codec::write(
            root,
            &format!("map-{index}"),
            "map",
            &records::Map::from(map.as_ref()),
            Atoms::default(),
            vec![],
        )?;
        charts.push(MapJob { index, map });
    }
    let mut prepared = Vec::with_capacity(selected.len());
    let mut physical_identity = None;
    for recipe in selected {
        let mut settings = options.clone();
        settings.program_recipe = recipe;
        let (source, source_identity) = records::write_source(
            root,
            input,
            &targets,
            &settings,
            source_scope.as_ref(),
            runtime_parameters,
            runtime_mass_constraints,
        )?;
        if physical_identity
            .as_ref()
            .is_some_and(|previous| previous != &source_identity)
        {
            return Err(invalid("recipe contexts have different physical inputs"));
        }
        physical_identity = Some(source_identity.clone());
        prepared.push(Preparation {
            source_scope: source_scope.clone(),
            source_identity,
            program_recipe: recipe,
            source,
            charts: charts.clone(),
            mode: options.mode,
            dimension,
        });
    }
    Ok(PreparedRecipeSet {
        source_identity: physical_identity.unwrap(),
        recipes: prepared,
    })
}
