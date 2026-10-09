//! Small native determinant templates avoid repeated large pivot cancellation.
//!
//! Symbolica computes every determinant. Only its argument size changes: the
//! complete polynomial in short matrix entries is substituted before symbolic
//! subtraction, so derivatives always see the physical entries' dependence.
use crate::generation::GenerationError;
use std::sync::OnceLock;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::atom::AtomField,
    id::{Pattern, Replacement},
    symbol,
    tensors::matrix::Matrix,
};

#[cfg(test)]
mod tests;

// Generic determinant polynomials grow factorially. Bound retained template
// storage independently of sector count; larger dimensions keep the existing
// native path until a separately measured owner-supported approach is ready.
const FIRST_TEMPLATE: usize = 4;
const LAST_TEMPLATE: usize = 6;
static TEMPLATES: [OnceLock<Result<Template, String>>; LAST_TEMPLATE - FIRST_TEMPLATE + 1] =
    [const { OnceLock::new() }; LAST_TEMPLATE - FIRST_TEMPLATE + 1];

struct Template {
    entries: Vec<Atom>,
    determinant: Atom,
}

fn field() -> AtomField {
    // Bareiss pivot divisions must cancel exactly; retaining an uncancelled
    // quotient creates false numerical poles even when the determinant is regular.
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
