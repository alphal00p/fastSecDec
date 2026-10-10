//! Lower the selected native envelope before differentiating the complete map.
use crate::{
    contour::{
        ContourDefinitions, FixedContourMap, SmoothContourMap,
        dynamic::{DynamicEnvelope, lambda_cap_symbol, safety_fraction_symbol},
        functions::{
            dynamic::{RootProgram, strength},
            smooth_positive::positive_part,
        },
    },
    generation::{GenerationError, program::ProgramData},
    kernel::{
        DynamicChartRecipe, DynamicCheckSource, NativeProgramDescriptor, indexed::ProgramRecipe,
    },
};
use std::sync::Arc;
use symbolica::atom::{Atom, Symbol};

pub(in crate::generation) fn build(
    recipe: ProgramRecipe,
    parameters: &[Symbol],
    causal: Atom,
    positive: &[Atom],
) -> Result<(SmoothContourMap, ProgramData), GenerationError> {
    match recipe {
        ProgramRecipe::FixedV1 => Ok((
            FixedContourMap::new(parameters, causal)?.into_inner(),
            ProgramData::default(),
        )),
        ProgramRecipe::DynamicPolynomialV1 | ProgramRecipe::DynamicSignAwareV1 => {
            let envelope = DynamicEnvelope::new(parameters, causal.clone(), positive)?;
            let coefficients = if recipe == ProgramRecipe::DynamicPolynomialV1 {
                envelope.polynomial_coefficients()?
            } else {
                envelope.sign_aware_coefficients_with(|part| {
                    positive_part(part.argument(), envelope.regularity())
                })?
            };
            let helper =
                RootProgram::build(coefficients.len()).map_err(GenerationError::Contour)?;
            let local_strength = strength(
                &helper,
                &coefficients,
                &Atom::var(safety_fraction_symbol()),
                &Atom::var(lambda_cap_symbol()),
            )
            .map_err(GenerationError::Contour)?;
            let chart = DynamicChartRecipe::from_envelope(0, &envelope, &helper)
                .map_err(|e| GenerationError::Contour(e.to_string()))?;
            let checks = vec![Arc::new(
                DynamicCheckSource::from_envelope(0, &envelope, recipe, local_strength.clone())
                    .map_err(|e| GenerationError::Contour(e.to_string()))?,
            )];
            let descriptor =
                NativeProgramDescriptor::dynamic(recipe, vec![chart], vec![helper.clone()])
                    .map_err(|e| GenerationError::Contour(e.to_string()))?;
            let (definitions, compact_coefficients) =
                ContourDefinitions::coefficients(parameters, &coefficients)
                    .map_err(GenerationError::Contour)?;
            let definitions = Arc::new(definitions);
            let compact_strength = strength(
                &helper,
                &compact_coefficients,
                &Atom::var(safety_fraction_symbol()),
                &Atom::var(lambda_cap_symbol()),
            )
            .map_err(GenerationError::Contour)?;
            let program = ProgramData {
                descriptor: Some(Arc::new(descriptor)),
                checks,
                definitions: vec![(0, definitions.clone())],
                ..ProgramData::default()
            };
            // The descriptor retains the callback helper during derivatives,
            // subtraction, optimization and independent job execution.
            let map = SmoothContourMap::with_definitions(
                parameters,
                causal,
                compact_strength,
                definitions,
            )?;
            Ok((map, program))
        }
        ProgramRecipe::UndeformedV1 => Err(GenerationError::Invariant(
            "undeformed recipe reached the contour map factory".into(),
        )),
    }
}
