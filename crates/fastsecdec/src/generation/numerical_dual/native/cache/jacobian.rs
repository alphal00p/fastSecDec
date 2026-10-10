//! Compose native image derivatives into the smooth body before outer jets.
use super::{ExactProgram, Source};
use crate::contour::{ContourJacobianPlan, JacobianTemplate};
use symbolica::{
    atom::{Atom, AtomCore},
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::{Dualizer, EvaluatorComposer, Slot},
};

pub(super) fn build_prefix(
    source: &Source,
    plan: &ContourJacobianPlan,
) -> Result<ExactProgram, String> {
    let dimension = plan.parameters.len();
    if !(1..=6).contains(&dimension) || plan.images.len() != dimension {
        return Err("invalid dual Jacobian source dimension".into());
    }
    let axes = plan
        .parameters
        .iter()
        .map(|coordinate| {
            source
                .inputs
                .iter()
                .position(|input| input == coordinate)
                .ok_or_else(|| "dual Jacobian coordinate missing from native inputs".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let inputs = source
        .inputs
        .iter()
        .copied()
        .map(Atom::var)
        .collect::<Vec<_>>();
    let mut shape = vec![vec![0; dimension]];
    for axis in 0..dimension {
        let mut unit = vec![0; dimension];
        unit[axis] = 1;
        shape.push(unit);
    }
    // Coordinate derivative seeds stay one on faces. Only non-coordinate
    // parameter seeds vanish; face restriction is applied by the outer recipe.
    let zeros = (0..inputs.len())
        .flat_map(|input| {
            axes.iter()
                .enumerate()
                .filter_map(move |(axis, index)| (input != *index).then_some((input, axis + 1)))
        })
        .collect();
    let images = build_outputs(source, &plan.images, &inputs)?
        .vectorize(&Dualizer::new(
            HyperDual::<Complex<Rational>>::new(shape),
            zeros,
        ))
        .map_err(|e| e.to_string())?;
    let mut composer = EvaluatorComposer::new(inputs.len());
    let constants = build_outputs(source, &[Atom::zero(), Atom::one()], &[])?;
    let constants = composer
        .append(&constants, &[])
        .map_err(|e| e.to_string())?;
    let constants = &constants;
    let seeds = (0..inputs.len())
        .flat_map(|input| {
            std::iter::once(Slot::Param(input)).chain(
                axes.iter()
                    .map(move |index| constants[usize::from(input == *index)]),
            )
        })
        .collect::<Vec<_>>();
    let images = composer
        .append(&images, &seeds)
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
    let entries = (0..dimension)
        .flat_map(|row| {
            let images = &images;
            (0..dimension).map(move |column| images[row * (dimension + 1) + column + 1])
        })
        .collect::<Vec<_>>();
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
