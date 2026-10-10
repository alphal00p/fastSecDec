//! Restore, finish and consume a single representative sector.
use super::{SectorJob, StreamingError, codec::invalid, prepare, records};
use crate::generation::{
    self, ChartRecord, GeneratedIntegral, GenerationEvent, GenerationMode, GenerationProgress,
    PreSubtractionMetadata,
    assembly::Assembly,
    coefficients, domain, laurent, mapping,
    numerical_dual::{
        self, chart::DiscoveredChart, formula::Key, pipeline::Context as DualContext,
    },
    support::SupportCache,
};
use std::{ops::ControlFlow, path::Path, sync::Arc};

/// Exists only inside the caller's active worker. Persist its compiled kernels
/// before returning a descriptor to the coordinator, then release the process.
pub struct GeneratedUnit {
    pub generated: GeneratedIntegral,
    /// Local chart ordinal -> original source chart ordinal.
    pub source_indices: Vec<usize>,
    pub runtime_parameters: Vec<symbolica::atom::Symbol>,
    pub runtime_mass_constraints: Vec<crate::kernel::RuntimeMassConstraint>,
}

pub fn generate_sector(
    root: &Path,
    job: &SectorJob,
    mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
) -> Result<GeneratedUnit, StreamingError> {
    let context = records::read_source(root, &job.source)?;
    if job.program_recipe != context.options.program_recipe {
        return Err(invalid("sector program recipe mismatch"));
    }
    let domain = domain::check_options(&context.input, &context.options)?;
    let mut observe = |event: &GenerationEvent| match event {
        GenerationEvent::Progress(status) => progress(status),
        GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
    };
    if job.charts.is_empty() {
        if !context.input.terms().is_empty() || job.formula.is_some() || job.total != 0 {
            return Err(invalid("empty sector job for a nonempty integral"));
        }
        return Ok(GeneratedUnit {
            generated: Assembly::new(context.options.max_order)
                .finish(domain, vec![], context.options.max_order)
                .preserve_empty_recipe(
                    context.options.program_recipe,
                    context.options.contour_jacobian,
                )?,
            source_indices: vec![],
            runtime_parameters: context.runtime_parameters,
            runtime_mass_constraints: context.runtime_mass_constraints,
        });
    }
    if job.charts[0].chart.index != job.index
        || job
            .charts
            .windows(2)
            .any(|pair| pair[0].chart.index >= pair[1].chart.index)
    {
        return Err(invalid("sector source-chart ordering mismatch"));
    }
    let data = records::read_chart(root, &job.charts[0].chart.record)?;
    if data.index != job.index || data.source_id != job.source.blake3 {
        return Err(invalid("representative chart index or source mismatch"));
    }
    let source_indices = job.charts.iter().map(|c| c.chart.index).collect();
    let generated = if context.options.mode == GenerationMode::Symbolic {
        if data.deferred.is_some() || job.formula.is_some() {
            return Err(invalid("symbolic sector contains deferred formula"));
        }
        let mut charts = Vec::with_capacity(job.charts.len());
        for (local, usage) in job.charts.iter().enumerate() {
            let restored;
            let chart = if local == 0 {
                &data
            } else {
                restored = records::read_chart(root, &usage.chart.record)?;
                &restored
            };
            if chart.index != usage.chart.index
                || usage.chart.program_recipe != job.program_recipe
                || chart.source_id != job.source.blake3
                || usage.chart.source_id != chart.source_id
                || usage.permutation.len() != context.targets.len()
            {
                return Err(invalid("associated chart identity or dimension mismatch"));
            }
            charts.push(ChartRecord {
                source_index: local,
                representative: 0,
                representative_permutation: usage.permutation.clone(),
                kernel_sector: None,
                coordinates: mapping::coordinates(&context.input, &chart.map, &context.targets),
                geometry: chart.map.clone(),
                pre_subtraction: Some(PreSubtractionMetadata::capture(
                    &chart.mapped,
                    context.input.regulator(),
                    context.options.max_subtractions_per_axis,
                )?),
                contour: chart.contour.clone(),
            });
        }
        let output = coefficients::expand(
            data.mapped,
            coefficients::Representative {
                parameters: &context.targets,
                regulator: context.input.regulator(),
                index: job.index,
                total: job.total,
            },
            &context.options,
            &mut laurent::TemplateCache::default(),
            &mut observe,
        )?;
        let mut assembly = Assembly::new(context.options.max_order);
        assembly.push(
            0,
            data.map,
            context.targets,
            job.charts.len(),
            output,
            data.program,
        )?;
        assembly.finish(domain, charts, context.options.max_order)
    } else {
        if job.charts.len() != 1 {
            return Err(invalid(
                "numerical-dual job cannot merge symbolic representatives",
            ));
        }
        let dual = DualContext::new(&context.input, &context.options, context.targets.clone());
        let key = prepare::key(&dual, &data)?;
        let recipe = match (&job.formula, &key) {
            (None, None) => None,
            (Some(formula), Some(key)) => {
                if formula.key != key.lookup_key()
                    || formula.source_id != job.source.blake3
                    || formula.program_recipe != job.program_recipe
                {
                    return Err(invalid("sector formula lookup key or source mismatch"));
                }
                let (recipe, signature) =
                    records::read_formula(root, &formula.record, &formula.key)?;
                let restored_key = Key::discover(
                    &signature,
                    context.targets.len(),
                    context.input.regulator(),
                    &context.options,
                )?;
                if restored_key.as_ref() != Some(key) {
                    return Err(invalid(
                        "restored subtraction formula is not natively equal to the required key",
                    ));
                }
                Some(Arc::new(recipe))
            }
            _ => return Err(invalid("sector formula/fallback admission mismatch")),
        };
        let coordinates = mapping::coordinates(&context.input, &data.map, &context.targets);
        let pre_subtraction = PreSubtractionMetadata::capture(
            &data.mapped,
            context.input.regulator(),
            context.options.max_subtractions_per_axis,
        )?;
        let discovered = DiscoveredChart {
            program: data.program,
            contour: data.contour,
            index: job.index,
            map: data.map,
            coordinates,
            mapped: data.mapped,
            terms: data
                .deferred
                .ok_or_else(|| invalid("missing deferred source factors"))?,
            pre_subtraction,
            key,
        };
        let mut prepared = numerical_dual::chart::instantiate(
            &dual,
            discovered,
            recipe,
            job.total,
            &mut SupportCache::new(context.input.parameters()),
            &mut observe,
        )?;
        prepared.chart.source_index = 0;
        prepared.chart.representative = 0;
        numerical_dual::finish(domain, vec![prepared], context.options.max_order)?
    };
    let generated = generated.preserve_empty_recipe(
        context.options.program_recipe,
        context.options.contour_jacobian,
    )?;
    if generated.sectors().len() > 1 {
        return Err(generation::GenerationError::Invariant(
            "one streamed representative produced multiple kernels".into(),
        )
        .into());
    }
    Ok(GeneratedUnit {
        generated,
        source_indices,
        runtime_parameters: context.runtime_parameters,
        runtime_mass_constraints: context.runtime_mass_constraints,
    })
}
