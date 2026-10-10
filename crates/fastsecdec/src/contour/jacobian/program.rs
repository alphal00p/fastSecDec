//! Native first derivatives of the contour image vector only.
use crate::kernel::CompilationSettings;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::{Dualizer, EvaluatorComposer, ExpressionEvaluator, Slot},
};

type ExactProgram = ExpressionEvaluator<Complex<Rational>>;

pub(crate) fn image_partials(
    images: ExactProgram,
    coordinates: &[Symbol],
    inputs: &[Symbol],
    settings: CompilationSettings,
) -> Result<ExactProgram, String> {
    let dimension = coordinates.len();
    if dimension == 0
        || images.get_output_len() != dimension
        || images.get_input_len() != inputs.len()
    {
        return Err("invalid contour image program shape".into());
    }
    let axes = coordinates
        .iter()
        .map(|coordinate| {
            inputs
                .iter()
                .position(|input| input == coordinate)
                .ok_or_else(|| "contour image coordinate missing from native inputs".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut shape = vec![vec![0; dimension]];
    for axis in 0..dimension {
        let mut unit = vec![0; dimension];
        unit[axis] = 1;
        shape.push(unit);
    }
    let zeros = (0..inputs.len())
        .flat_map(|input| {
            axes.iter()
                .enumerate()
                .filter_map(move |(axis, index)| (input != *index).then_some((input, axis + 1)))
        })
        .collect();
    // There are only value and first-order contour-coordinate components.
    // The endpoint-subtracted density is never passed to this Dualizer.
    let images = images
        .vectorize(&Dualizer::new(
            HyperDual::<Complex<Rational>>::new(shape),
            zeros,
        ))
        .map_err(|error| error.to_string())?;
    let constants = Atom::evaluator_multiple(&[Atom::zero(), Atom::one()], &[] as &[Atom])
        .optimization_settings(settings.native())
        .build()
        .map_err(|error| error.to_string())?;
    let mut composer = EvaluatorComposer::new(inputs.len());
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
    let outputs = composer
        .append(&images, &seeds)
        .map_err(|e| e.to_string())?;
    let entries = (0..dimension)
        .flat_map(|row| {
            let outputs = &outputs;
            (0..dimension).map(move |column| outputs[row * (dimension + 1) + column + 1])
        })
        .collect::<Vec<_>>();
    composer
        .finish(&entries, settings.native())
        .map_err(|error| error.to_string())
}
