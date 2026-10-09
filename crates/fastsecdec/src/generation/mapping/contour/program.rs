//! Lower the selected native envelope before differentiating the complete map.
use crate::{
    contour::{
        FixedContourMap, SmoothContourMap,
        dynamic::{DynamicEnvelope, lambda_cap_symbol, safety_fraction_symbol},
        functions::dynamic::{RootProgram, strength},
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
        ProgramRecipe::DynamicPolynomialV1 => {
            let envelope = DynamicEnvelope::new(parameters, causal.clone(), positive)?;
            let coefficients = envelope.polynomial_coefficients()?;
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
            let checks = vec![Arc::new(DynamicCheckSource::from_envelope(0, &envelope))];
            let descriptor = NativeProgramDescriptor::dynamic(recipe, vec![chart], vec![helper])
                .map_err(|e| GenerationError::Contour(e.to_string()))?;
            let program = ProgramData {
                descriptor: Some(Arc::new(descriptor)),
                checks,
            };
            // The descriptor retains the callback helper during derivatives,
            // subtraction, optimization and independent job execution.
            let map = SmoothContourMap::new(parameters, causal, local_strength)?;
            Ok((map, program))
        }
        ProgramRecipe::DynamicSignAwareV1 => Err(GenerationError::Contour(
            "sign-aware generation awaits cancellation-resistant positive-part callbacks".into(),
        )),
        ProgramRecipe::UndeformedV1 => Err(GenerationError::Invariant(
            "undeformed recipe reached the contour map factory".into(),
        )),
    }
}
