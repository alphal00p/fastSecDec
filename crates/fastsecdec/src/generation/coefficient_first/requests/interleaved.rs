//! Restricted face scheduling; native Atom operations own all algebra.
use super::{Body, GenerationError};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::Pattern,
};

pub(super) fn resolve(
    body: &Body,
    depths: &[usize],
    arguments: &[Atom],
    poll: &mut impl FnMut() -> Result<(), GenerationError>,
) -> Result<Option<Atom>, GenerationError> {
    if body.parameters.len() != depths.len() || depths.len() != arguments.len() {
        return Err(GenerationError::Invariant(
            "interleaved native request shape".into(),
        ));
    }
    let mut faces = Vec::with_capacity(arguments.len());
    for (parameter, argument) in body.parameters.iter().zip(arguments) {
        if *argument == Atom::var(*parameter) {
            faces.push(None);
        } else if argument.is_zero() || argument.is_one() {
            faces.push(Some(argument));
        } else {
            return Ok(None);
        }
    }
    if faces.iter().all(Option::is_none) {
        return Ok(None);
    }
    let face = |value: Atom, parameter: Symbol, argument: &Atom| {
        value
            .replace(Pattern::Literal(Atom::var(parameter)))
            .with(argument.clone())
    };
    let mut value = body.value.clone();
    for ((parameter, depth), argument) in body.parameters.iter().zip(depths).zip(&faces) {
        if *depth == 0
            && let Some(argument) = argument
        {
            poll()?;
            value = face(value, *parameter, argument);
            poll()?;
        }
    }
    for ((parameter, depth), argument) in body.parameters.iter().zip(depths).zip(&faces) {
        if *depth == 0 {
            continue;
        }
        for _ in 0..*depth {
            poll()?;
            value = value.derivative(*parameter);
            poll()?;
        }
        if let Some(argument) = argument {
            poll()?;
            value = face(value, *parameter, argument);
            poll()?;
        }
    }
    Ok(Some(value))
}
