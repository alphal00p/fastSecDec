//! Lower the selected native envelope before differentiating the complete map.
use crate::{
    contour::{
        ContourDefinitions, ContourJacobian, ContourJacobianPlan, FixedContourMap,
        JacobianTemplate, SmoothContourMap,
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
    jacobian: ContourJacobian,
    parameters: &[Symbol],
    causal: Atom,
    positive: &[Atom],
) -> Result<(SmoothContourMap, ProgramData), GenerationError> {
    match recipe {
        ProgramRecipe::FixedV1 => {
            let mut map = if jacobian == ContourJacobian::Dual && !parameters.is_empty() {
                let mut template = JacobianTemplate::new(parameters.len())?;
                let (definitions, mut calls) = ContourDefinitions::with_required_bodies(
                    parameters,
                    std::slice::from_ref(&template.expression),
                    &[0],
                )
                .map_err(GenerationError::Contour)?;
                template.expression = calls.pop().expect("one requested determinant body");
                SmoothContourMap::with_jacobian(
                    parameters,
                    causal,
                    Atom::var(crate::contour::lambda_symbol()),
                    Arc::new(definitions),
                    Some(template),
                )?
            } else {
                let mut map = FixedContourMap::new(parameters, causal)?.into_inner();
                map.compact_fixed_density()?;
                map
            };
            let definitions = map.metadata().definitions.clone();
            let mut program = ProgramData {
                contour_jacobian: jacobian,
                ..Default::default()
            };
            if !definitions.is_empty() {
                program.definitions.push((0, definitions));
            }
            retain_plan(&mut program, &mut map, parameters);
            Ok((map, program))
        }
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
            let mut template = if jacobian == ContourJacobian::Dual && !parameters.is_empty() {
                Some(JacobianTemplate::new(parameters.len())?)
            } else {
                None
            };
            let (definitions, compact_coefficients) = if let Some(template) = &mut template {
                let mut bodies = coefficients.clone();
                bodies.push(template.expression.clone());
                let (definitions, mut calls) = ContourDefinitions::with_required_bodies(
                    parameters,
                    &bodies,
                    &[coefficients.len()],
                )
                .map_err(GenerationError::Contour)?;
                template.expression = calls.pop().expect("one requested determinant body");
                (definitions, calls)
            } else {
                ContourDefinitions::coefficients(parameters, &coefficients)
                    .map_err(GenerationError::Contour)?
            };
            let definitions = Arc::new(definitions);
            let compact_strength = strength(
                &helper,
                &compact_coefficients,
                &Atom::var(safety_fraction_symbol()),
                &Atom::var(lambda_cap_symbol()),
            )
            .map_err(GenerationError::Contour)?;
            let mut program = ProgramData {
                contour_jacobian: jacobian,
                descriptor: Some(Arc::new(descriptor)),
                checks,
                definitions: vec![(0, definitions.clone())],
                ..ProgramData::default()
            };
            // The descriptor retains the callback helper during derivatives,
            // subtraction, optimization and independent job execution.
            let mut map = SmoothContourMap::with_jacobian(
                parameters,
                causal,
                compact_strength,
                definitions,
                template,
            )?;
            retain_plan(&mut program, &mut map, parameters);
            Ok((map, program))
        }
        ProgramRecipe::ThresholdV1 | ProgramRecipe::UndeformedV1 => Err(
            GenerationError::Invariant("non-contour recipe reached the contour map factory".into()),
        ),
    }
}

fn retain_plan(program: &mut ProgramData, map: &mut SmoothContourMap, parameters: &[Symbol]) {
    if program.contour_jacobian == ContourJacobian::Dual && !parameters.is_empty() {
        program.jacobians.push((
            0,
            Arc::new(ContourJacobianPlan {
                parameters: parameters.to_vec(),
                images: map.metadata().images().to_vec(),
                jacobian: map.metadata().jacobian().clone(),
            }),
        ));
    }
}
