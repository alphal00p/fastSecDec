use super::{GenerationError, GenerationOptions, mapping::MappedTerm};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    coefficient::Coefficient,
    domains::rational::Rational,
};

struct Piece {
    powers: Vec<Option<Atom>>,
    regular: Atom,
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
        .replace(epsilon.to_pattern())
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
) -> Result<(Atom, usize, usize), GenerationError> {
    let mut cancellation_degree = 0;
    let mut pieces = terms
        .into_iter()
        .map(|term| Piece {
            powers: term.powers.into_iter().map(Some).collect(),
            regular: term.regular,
        })
        .collect::<Vec<_>>();
    for (axis, parameter) in parameters.iter().enumerate() {
        let mut axis_degree = 0;
        let x = Atom::var(*parameter);
        let mut next = Vec::new();
        for piece in pieces {
            let power = piece.powers[axis].as_ref().unwrap();
            let (constant, slope) = endpoint_power(power, regulator)?;
            if constant > -1 {
                next.push(piece);
                continue;
            }
            let count = (-constant)
                .floor()
                .to_i64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or(GenerationError::ResourceLimit("Taylor subtraction degree"))?;
            if count > options.max_subtractions_per_axis {
                return Err(GenerationError::ResourceLimit("Taylor subtraction degree"));
            }
            axis_degree = axis_degree.max(count);
            let mut derivative = piece.regular.clone();
            let mut taylor = Atom::Zero;
            for degree in 0..count {
                let coefficient = derivative.replace(x.to_pattern()).with(Atom::Zero);
                if !coefficient.is_zero() {
                    let denominator = (power + Atom::num(degree + 1)).expand();
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
                        regular: coefficient / denominator,
                    });
                }
                derivative = derivative.derivative(*parameter) / Atom::num(degree + 1);
            }
            // Keep the residual powers and Gamma prefactor factored. Expanding
            // a Taylor difference destroys the compact direct-kernel DAG and
            // forces the later Laurent pass to rebuild it term by term.
            let remainder = piece.regular - taylor;
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
        cancellation_degree += axis_degree;
    }
    let count = pieces.len();
    let density = pieces
        .into_iter()
        .map(|piece| {
            piece.regular
                * parameters
                    .iter()
                    .zip(piece.powers)
                    .filter_map(|(parameter, power)| {
                        power.map(|power| Atom::var(*parameter).pow(power))
                    })
                    .product::<Atom>()
        })
        .sum();
    Ok((density, count, cancellation_degree))
}
