//! Native determinants for causal gradient maps.
//!
//! Symbolica computes every determinant. Small maps substitute physical entries
//! into native polynomial templates before symbolic subtraction, so derivatives
//! always see those entries' dependence.
//! Larger maps use the gradient structure to retain safe native pivot quotients.
use crate::generation::GenerationError;
use std::sync::OnceLock;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::atom::AtomField,
    id::{Pattern, Replacement},
    symbol,
    tensors::matrix::Matrix,
};

#[cfg(test)]
mod structured_tests;
#[cfg(test)]
mod tests;

// Generic determinant polynomials grow factorially. Bound retained template
// storage independently of sector count. Larger causal maps use the structured
// native determinant below, without constructing another generic template.
const FIRST_TEMPLATE: usize = 4;
const LAST_TEMPLATE: usize = 6;
static TEMPLATES: [OnceLock<Result<Template, String>>; LAST_TEMPLATE - FIRST_TEMPLATE + 1] =
    [const { OnceLock::new() }; LAST_TEMPLATE - FIRST_TEMPLATE + 1];

struct Template {
    entries: Vec<Atom>,
    determinant: Atom,
}

fn field() -> AtomField {
    // For arbitrary entries Bareiss pivot divisions must cancel exactly;
    // an uncancelled quotient can create false poles in a regular determinant.
    AtomField {
        statistical_zero_test: false,
        cancel_check_on_division: true,
        ..AtomField::new()
    }
}

fn native(entries: Vec<Atom>, dimension: u32) -> Result<Atom, String> {
    Matrix::from_linear(entries, dimension, dimension, field())?
        .det()
        .map_err(|error| error.to_string())
}

impl Template {
    fn build(dimension: u32) -> Result<Self, String> {
        let entries = (0..dimension * dimension)
            .map(|index| {
                Atom::var(symbol!(format!(
                    "fastsecdec::contour::det_entry_{dimension}_{index}"
                )))
            })
            .collect::<Vec<_>>();
        let determinant = native(entries.clone(), dimension)?;
        if determinant.is_polynomial(true, false).is_none() {
            return Err("native determinant template retained a pivot denominator".into());
        }
        Ok(Self {
            entries,
            determinant,
        })
    }

    fn substitute(&self, entries: Vec<Atom>) -> Atom {
        self.determinant
            .replace_multiple(self.entries.iter().zip(entries).map(|(source, target)| {
                Replacement::new(Pattern::Literal(source.clone()), Pattern::Literal(target))
            }))
    }
}

pub(super) fn determinant(entries: Vec<Atom>, dimension: u32) -> Result<Atom, GenerationError> {
    let dimension_usize = dimension as usize;
    let count = dimension_usize
        .checked_mul(dimension_usize)
        .ok_or(GenerationError::ResourceLimit("contour Jacobian entries"))?;
    if entries.len() != count {
        return Err(GenerationError::Contour(format!(
            "native Jacobian determinant: expected {count} entries, received {}",
            entries.len()
        )));
    }
    let result = if (FIRST_TEMPLATE..=LAST_TEMPLATE).contains(&dimension_usize) {
        TEMPLATES[dimension_usize - FIRST_TEMPLATE]
            .get_or_init(|| Template::build(dimension))
            .as_ref()
            .map(|template| template.substitute(entries))
            .map_err(Clone::clone)
    } else {
        native(entries, dimension)
    };
    result
        .map_err(|error| GenerationError::Contour(format!("native Jacobian determinant: {error}")))
}

/// Determinant of `z = x - i strength(x) v(x)`, `v_i=x_i(1-x_i) d_i F`.
///
/// This private boundary requires a real F and a caller-proved real smooth
/// strength on the admitted cube. `SmoothContourMap` checks F; fixed/dynamic
/// recipe admission supplies the strength premise. It is not a determinant
/// shortcut for arbitrary Jacobians or complex coordinate values.
///
/// Write A=I-i strength Dv. Every principal minor of A is nonzero: in the
/// interior Dv=D+W Hess(F) is similar to a real symmetric matrix, and a face
/// leaves a real diagonal block for the zero entries of W. Thus the leading
/// minors divided by native Bareiss are safe without symbolic cancellation.
/// The border retains the full native gradient of strength and introduces no
/// division by the full Jacobian, which is allowed to vanish.
pub(super) fn real_gradient_jacobian(
    parameters: &[Symbol],
    causal_polynomial: &Atom,
    strength: &Atom,
) -> Result<Atom, GenerationError> {
    let direction = parameters
        .iter()
        .map(|parameter| {
            let x = Atom::var(*parameter);
            &x * (Atom::one() - &x) * causal_polynomial.derivative(*parameter)
        })
        .collect::<Vec<_>>();
    let strength_gradient = parameters
        .iter()
        .map(|parameter| strength.derivative(*parameter))
        .collect::<Vec<_>>();
    let varying = strength_gradient.iter().any(|entry| !entry.is_zero());
    let size = parameters
        .len()
        .checked_add(usize::from(varying))
        .ok_or(GenerationError::ResourceLimit("contour Jacobian dimension"))?;
    let dimension = u32::try_from(size)
        .map_err(|_| GenerationError::ResourceLimit("contour Jacobian dimension"))?;
    let count = size
        .checked_mul(size)
        .ok_or(GenerationError::ResourceLimit("contour Jacobian entries"))?;
    let mut entries = Vec::with_capacity(count);
    for (row, v) in direction.iter().enumerate() {
        entries.extend(parameters.iter().enumerate().map(|(column, parameter)| {
            Atom::num(i32::from(row == column)) - Atom::i() * strength * v.derivative(*parameter)
        }));
        if varying {
            entries.push(Atom::i() * v);
        }
    }
    if varying {
        entries.extend(strength_gradient);
        entries.push(Atom::one());
    }
    Matrix::from_linear(
        entries,
        dimension,
        dimension,
        AtomField {
            statistical_zero_test: false,
            cancel_check_on_division: false,
            ..AtomField::new()
        },
    )
    .and_then(|matrix| matrix.det().map_err(|error| error.to_string()))
    .map_err(|error| GenerationError::Contour(format!("native gradient Jacobian: {error}")))
}
