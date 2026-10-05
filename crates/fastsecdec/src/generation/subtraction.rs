use super::{GenerationError, GenerationOptions, SubtractionStrategy, mapping::MappedTerm};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::Pattern,
};

pub(super) mod endpoints;
#[cfg(test)]
use endpoints::endpoint_power;
pub(super) use endpoints::rational;

#[cfg(test)]
pub(super) mod series_first;

struct Piece {
    powers: Vec<Option<Atom>>,
    prefactor: Atom,
    regular: Atom,
    cancellation: Vec<usize>,
}

/// Taylor endpoint subtraction implements the meromorphic continuation exactly.
/// Boundary terms remain embedded in the original cube so correlated Laurent
/// coefficients are evaluated together at one point. No numerical term summing.
pub(super) fn subtract(
    terms: Vec<MappedTerm>,
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
) -> Result<(Atom, usize, Vec<Vec<usize>>), GenerationError> {
    let mut pieces = terms
        .into_iter()
        .map(|term| Piece {
            powers: term.powers.into_iter().map(Some).collect(),
            prefactor: term.prefactor,
            regular: term.regular,
            cancellation: vec![0; parameters.len()],
        })
        .collect::<Vec<_>>();
    for (axis, parameter) in parameters.iter().enumerate() {
        let x = Atom::var(*parameter);
        let mut next = Vec::new();
        for mut piece in pieces {
            let mut power = piece.powers[axis].as_ref().unwrap().clone();
            let endpoint = endpoints::admit(&power, regulator, options.max_subtractions_per_axis)?;
            if endpoint.constant > -1 {
                next.push(piece);
                continue;
            }
            let slope = endpoint.slope;
            let mut count = endpoint.subtractions;
            if options.subtraction == SubtractionStrategy::IntegrateByParts && !slope.is_zero() {
                for _ in 1..count {
                    let denominator = (&power + Atom::one()).expand();
                    let boundary = piece
                        .regular
                        .replace(Pattern::Literal(x.clone()))
                        .with(Atom::one());
                    if !boundary.is_zero() {
                        let mut boundary_piece = Piece {
                            powers: piece.powers.clone(),
                            prefactor: piece.prefactor.clone(),
                            regular: boundary / &denominator,
                            cancellation: piece.cancellation.clone(),
                        };
                        boundary_piece.powers[axis] = None;
                        next.push(boundary_piece);
                    }
                    piece.regular = -piece.regular.derivative(*parameter) / &denominator;
                    power = denominator;
                    piece.powers[axis] = Some(power.clone());
                    if next.len() > options.max_subtraction_terms {
                        return Err(GenerationError::ResourceLimit("integration-by-parts terms"));
                    }
                }
                count = 1;
            }
            let mut derivative = piece.regular.clone();
            let mut taylor = Atom::Zero;
            for degree in 0..count {
                let coefficient = derivative
                    .replace(Pattern::Literal(x.clone()))
                    .with(Atom::Zero);
                if !coefficient.is_zero() {
                    let denominator = (&power + Atom::num(degree + 1)).expand();
                    if denominator.is_zero() || slope.is_zero() {
                        return Err(GenerationError::UnregulatedEndpoint {
                            parameter: *parameter,
                        });
                    }
                    taylor += &coefficient * x.pow(Atom::num(degree));
                    let mut powers = piece.powers.clone();
                    powers[axis] = None;
                    next.push(Piece {
                        powers,
                        prefactor: piece.prefactor.clone(),
                        regular: coefficient / denominator,
                        cancellation: piece.cancellation.clone(),
                    });
                }
                if degree + 1 < count {
                    derivative = derivative.derivative(*parameter) / Atom::num(degree + 1);
                }
            }
            // Keep the residual powers and Gamma prefactor factored. Expanding
            // a Taylor difference destroys the compact direct-kernel DAG and
            // forces the later Laurent pass to rebuild it term by term.
            if !taylor.is_zero() {
                piece.cancellation[axis] = count;
            }
            // The last derivative already needed for Taylor is enough to
            // prove exactness when it no longer depends on this coordinate.
            // This prunes factored polynomial identities without expansion or
            // computing an unused next derivative.
            let remainder = if !derivative.contains(x.as_view()) {
                Atom::Zero
            } else {
                piece.regular - taylor
            };
            if !remainder.is_zero() {
                next.push(Piece {
                    regular: remainder,
                    ..piece
                });
            }
            if next.len() > options.max_subtraction_terms {
                return Err(GenerationError::ResourceLimit("Taylor subtraction terms"));
            }
        }
        pieces = next;
    }
    let count = pieces.len();
    let cancellation_terms = pieces
        .iter()
        .map(|piece| piece.cancellation.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut groups = BTreeMap::<Atom, Atom>::new();
    for piece in pieces {
        let regular = piece.regular
            * parameters
                .iter()
                .zip(piece.powers)
                .filter_map(|(parameter, power)| {
                    power.map(|power| Atom::var(*parameter).pow(power))
                })
                .product::<Atom>();
        *groups.entry(piece.prefactor).or_insert(Atom::Zero) += regular;
    }
    let density = groups
        .into_iter()
        .map(|(prefactor, regular)| prefactor * regular)
        .sum();
    Ok((density, count, cancellation_terms))
}
