//! Test-only constant-face scheduling. Native Atom operations own all algebra.
use super::*;

/// Return None for every shape outside own-coordinate/zero/one requests.
/// A face in one independent coordinate commutes with derivatives in another;
/// every derivative in the replaced coordinate must precede its own face.
pub(super) fn resolve(body: &Body, depths: &[usize], arguments: &[Atom]) -> Option<Atom> {
    assert_eq!(body.parameters.len(), depths.len());
    assert_eq!(body.parameters.len(), arguments.len());
    let mut faces = Vec::with_capacity(arguments.len());
    for (parameter, argument) in body.parameters.iter().zip(arguments) {
        if *argument == Atom::var(*parameter) {
            faces.push(None);
        } else if argument.is_zero() || argument.is_one() {
            faces.push(Some(argument));
        } else {
            return None;
        }
    }
    if faces.iter().all(Option::is_none) {
        return None;
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
            value = face(value, *parameter, argument);
        }
    }
    for ((parameter, depth), argument) in body.parameters.iter().zip(depths).zip(&faces) {
        if *depth == 0 {
            continue;
        }
        for _ in 0..*depth {
            value = value.derivative(*parameter);
        }
        if let Some(argument) = argument {
            value = face(value, *parameter, argument);
        }
    }
    Some(value)
}
