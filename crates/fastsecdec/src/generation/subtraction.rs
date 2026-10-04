use super::{GenerationError, GenerationOptions, SubtractionStrategy, mapping::MappedTerm};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::Coefficient,
    domains::rational::Rational,
    id::Pattern,
};

struct Piece {
    powers: Vec<Option<Atom>>,
    prefactor: Atom,
    regular: Atom,
    cancellation: Vec<usize>,
}

pub(super) fn rational(expression: &Atom) -> Option<Rational> {
    if expression.is_zero() {
        return Some(Rational::from(0));
    }
    let AtomView::Num(number) = expression.as_view() else {
        return None;
    };
    match number.get_coeff_view().to_owned() {
        Coefficient::Complex(value) if value.im.is_zero() => Some(value.re),
        _ => None,
    }
}

fn endpoint_power(
    expression: &Atom,
    regulator: Symbol,
) -> Result<(Rational, Rational), GenerationError> {
    let epsilon = Atom::var(regulator);
    let constant = expression
        .replace(Pattern::Literal(epsilon.clone()))
        .with(Atom::Zero)
        .expand();
    let slope = expression.derivative(regulator).expand();
    if rational(&slope).is_none()
        || !(expression - &constant - &slope * epsilon)
            .expand()
            .is_zero()
    {
        return Err(GenerationError::EndpointExponent(expression.clone()));
    }
    rational(&constant)
        .map(|a| (a, rational(&slope).unwrap()))
        .ok_or_else(|| GenerationError::EndpointExponent(expression.clone()))
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
            let (constant, slope) = endpoint_power(&power, regulator)?;
            if constant > -1 {
                next.push(piece);
                continue;
            }
            let mut count = (-constant)
                .floor()
                .to_i64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or(GenerationError::ResourceLimit("Taylor subtraction degree"))?;
            if count > options.max_subtractions_per_axis {
                return Err(GenerationError::ResourceLimit("Taylor subtraction degree"));
            }
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
