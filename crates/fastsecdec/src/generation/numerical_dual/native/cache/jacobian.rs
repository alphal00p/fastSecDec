//! Compose native image derivatives into the smooth body before outer jets.
use super::{ExactProgram, Source};
use crate::contour::{ContourJacobianPlan, JacobianTemplate};
use symbolica::{
    atom::{Atom, AtomCore},
    evaluate::{EvaluatorComposer, Slot},
};

pub(super) fn build_prefix(
    source: &Source,
    plan: &ContourJacobianPlan,
) -> Result<ExactProgram, String> {
    let dimension = plan.parameters.len();
    if !(1..=6).contains(&dimension) || plan.images.len() != dimension {
        return Err("invalid dual Jacobian source dimension".into());
    }
    let inputs = source
        .inputs
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let images = build_outputs(source, &plan.images, &inputs)?;
    let partials =
        crate::contour::image_partials(images, &plan.parameters, &source.inputs, source.settings)?;
    let mut composer = EvaluatorComposer::new(inputs.len());
    let entries = composer
        .append(
            &partials,
            &(0..inputs.len()).map(Slot::Param).collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;
    let template = JacobianTemplate::new(dimension).map_err(|e| e.to_string())?;
    let determinant = template
        .expression
        .evaluator(
            &template
                .entries
                .iter()
                .copied()
                .map(Atom::var)
                .collect::<Vec<_>>(),
        )
        .optimization_settings(source.settings.native())
        .build()
        .map_err(|e| e.to_string())?;
    let determinant = composer
        .append(&determinant, &entries)
        .map_err(|e| e.to_string())?[0];
    composer
        .finish(&[determinant], source.settings.native())
        .map_err(|e| e.to_string())
}

pub(super) fn append_body(
    source: &Source,
    plan: &ContourJacobianPlan,
    prefix: &ExactProgram,
) -> Result<ExactProgram, String> {
    let inputs = source
        .inputs
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let mut composer = EvaluatorComposer::new(inputs.len());
    let slots = (0..inputs.len()).map(Slot::Param).collect::<Vec<_>>();
    let determinant = composer.append(prefix, &slots).map_err(|e| e.to_string())?[0];
    let mut body_inputs = inputs;
    body_inputs.push(plan.jacobian.clone());
    let body = build_outputs(
        source,
        std::slice::from_ref(&source.polynomial),
        &body_inputs,
    )?;
    let body_slots = (0..source.inputs.len())
        .map(Slot::Param)
        .chain([determinant])
        .collect::<Vec<_>>();
    let result = composer
        .append(&body, &body_slots)
        .map_err(|e| e.to_string())?;
    composer
        .finish(&result, source.settings.native())
        .map_err(|e| e.to_string())
}

pub(super) fn build_outputs(
    source: &Source,
    outputs: &[Atom],
    inputs: &[Atom],
) -> Result<ExactProgram, String> {
    let builder =
        Atom::evaluator_multiple(outputs, inputs).optimization_settings(source.settings.native());
    let builder = if let Some(definitions) = &source.definitions {
        builder.function_map(definitions.function_map(outputs.iter())?)
    } else {
        builder
    };
    builder.build().map_err(|e| e.to_string())
}
