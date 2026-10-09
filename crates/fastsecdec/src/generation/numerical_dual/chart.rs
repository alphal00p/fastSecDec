//! Map/valuation discovery and formula instantiation are separate native units.
use super::{DualSector, DualTerm, PreparedChart, formula::Key, pipeline::Context};
use crate::generation::{
    ChartRecord, CoefficientExpansionStage, CoordinateMap, GeneratedSector, GenerationError,
    GenerationEvent, GenerationProgress, PreSubtractionMetadata, coefficients,
    conditioning::Profile, context::emit, laurent, mapping::MappedTerm, support::SupportCache,
};
use fastsecdec_sectors::SectorMap;
use std::{ops::ControlFlow, sync::Arc};

pub(in crate::generation) struct DiscoveredChart {
    pub index: usize,
    pub map: SectorMap,
    pub coordinates: CoordinateMap,
    pub mapped: Vec<MappedTerm>,
    pub terms: Vec<DualTerm>,
    pub pre_subtraction: PreSubtractionMetadata,
    pub key: Option<Key>,
    pub contour: Option<crate::contour::ContourMetadata>,
}

pub(in crate::generation) fn discover(
    context: &Context,
    map: SectorMap,
    index: usize,
    total: usize,
    supports: &mut SupportCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<DiscoveredChart, GenerationError> {
    emit(
        progress,
        GenerationProgress::NumericalMapping {
            sector: index,
            total,
        },
    )?;
    let coordinates =
        crate::generation::mapping::coordinates(&context.input, &map, &context.parameters);
    let (mapped, terms, mut contour) = if context.options.contour {
        let (mapped, contour) = crate::generation::mapping::map_terms_with_contour(
            &context.input,
            &map,
            &coordinates,
            supports,
            true,
            false,
        )?;
        let terms = mapped
            .iter()
            .map(|_| DualTerm { factors: vec![] })
            .collect();
        (mapped, terms, contour)
    } else {
        let (mapped, terms) = super::mapping::prepare(
            &context.input,
            &map,
            &coordinates,
            supports,
            &context.valuations,
        )?;
        (mapped, terms, None)
    };
    let pre_subtraction = PreSubtractionMetadata::capture(
        &mapped,
        context.input.regulator(),
        context.options.max_subtractions_per_axis,
    )?;
    if let Some(contour) = &mut contour {
        contour.record_subtraction_faces(&pre_subtraction, context.options.subtraction);
    }
    let key = if context.eligible && !map.exponent_matrix.iter().flatten().any(|power| power < &0) {
        Key::discover(
            &mapped,
            context.parameters.len(),
            context.input.regulator(),
            &context.options,
        )?
    } else {
        None
    };
    Ok(DiscoveredChart {
        index,
        map,
        coordinates,
        mapped,
        terms,
        pre_subtraction,
        key,
        contour,
    })
}

pub(in crate::generation) fn instantiate(
    context: &Context,
    chart: DiscoveredChart,
    recipe: Option<Arc<super::subtraction::Recipe>>,
    total: usize,
    supports: &mut SupportCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<PreparedChart, GenerationError> {
    let DiscoveredChart {
        index,
        map,
        coordinates,
        mapped,
        terms,
        mut pre_subtraction,
        key,
        mut contour,
    } = chart;
    if key.is_some() != recipe.is_some() {
        return Err(GenerationError::Invariant(
            "prepared dual formula missing or assigned to fallback".into(),
        ));
    }
    let representative = coefficients::Representative {
        parameters: &context.parameters,
        regulator: context.input.regulator(),
        index,
        total,
    };
    let (coefficients, profile, deferred) = if let Some(recipe) = recipe {
        if let Some(contour) = &mut contour {
            use super::subtraction::Coordinate;
            contour.validation_faces = recipe
                .requests
                .iter()
                .map(|request| {
                    request
                        .coordinates
                        .iter()
                        .enumerate()
                        .filter_map(|(axis, coordinate)| match coordinate {
                            Coordinate::Variable(_) => None,
                            Coordinate::Zero => Some((axis, 0)),
                            Coordinate::One => Some((axis, 1)),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        let profile = Profile::mapped(
            &mapped,
            context.parameters.len(),
            context.input.regulator(),
            &context.options,
        )?;
        let coefficients = recipe.coefficients.clone();
        let orders = coefficients.keys().copied().collect();
        (
            coefficients,
            profile,
            Some(Arc::new(DualSector {
                programs: context.programs.clone(),
                source_parameters: context.input.parameters().to_vec(),
                regulator: context.input.regulator(),
                parameters: context.parameters.clone(),
                map: map.clone(),
                terms,
                mapped_regular: contour
                    .as_ref()
                    .map(|_| mapped.iter().map(|term| term.regular.clone()).collect()),
                recipe,
                orders,
            })),
        )
    } else {
        // Sampled jets cannot certify an unregulated face, signed infinity map
        // or non-polynomial epsilon dependence. Preserve exact admission.
        emit(
            progress,
            coefficients::Observation::default()
                .event(&representative, CoefficientExpansionStage::PhysicalFallback),
        )?;
        let physical = if context.options.contour {
            mapped
        } else {
            crate::generation::mapping::map_terms(&context.input, &map, &coordinates, supports)?
        };
        pre_subtraction = PreSubtractionMetadata::capture(
            &physical,
            context.input.regulator(),
            context.options.max_subtractions_per_axis,
        )?;
        // The enclosing assembly phase measures the complete fallback once.
        // Keep native activity/cancellation, but suppress nested phase timers.
        let mut observe = |event: &GenerationEvent| match event {
            GenerationEvent::Progress(GenerationProgress::PhaseTiming { .. }) => {
                ControlFlow::Continue(())
            }
            _ => progress(event),
        };
        let output = coefficients::expand(
            physical,
            representative,
            &context.options,
            &mut laurent::TemplateCache::default(),
            &mut observe,
        )?;
        (output.coefficients, output.conditioning, None)
    };
    let orders = coefficients.keys().copied().collect();
    let sector = GeneratedSector {
        cancellation_degree: profile.degree,
        cancellation_terms: profile.rows,
        endpoint_profiles: profile.endpoint_profiles,
        conditioning_basis: profile.basis,
        parameters: context.parameters.clone(),
        coefficients: coefficients.into_values().collect(),
        materialized: Default::default(),
        map: map.clone(),
        deferred,
    };
    let chart = ChartRecord {
        contour,
        source_index: index,
        representative: index,
        representative_permutation: (0..context.parameters.len()).collect(),
        kernel_sector: Some(index),
        coordinates,
        geometry: map,
        pre_subtraction: Some(pre_subtraction),
    };
    Ok(PreparedChart {
        chart,
        sector,
        orders,
    })
}
