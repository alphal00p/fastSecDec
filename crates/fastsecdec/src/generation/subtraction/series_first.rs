//! Bounded, test-only series-first composition experiment. Native Symbolica
//! owns every series operation and truncation bound; no inferred pole budget or
//! coefficient convolution is used here.
mod named;
mod replay;
mod tests;

use super::{MappedTerm, endpoint_power};
use crate::generation::{GenerationError, GenerationOptions, SubtractionStrategy};
use std::{collections::BTreeMap, time::Instant};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::atom::AtomField,
    id::Pattern,
    poly::series::{Series, SeriesDepth},
};

type NativeSeries = Series<AtomField>;

struct Piece {
    powers: Vec<Option<Atom>>,
    prefactor: Atom,
    regular: NativeSeries,
}

#[derive(Debug)]
pub(crate) struct Attempt {
    pub route: &'static str,
    pub width: i64,
    pub absolute_bound: String,
    pub pieces: usize,
    pub seconds: f64,
}

struct Expansion {
    regulator: Symbol,
    width: i64,
    cache: BTreeMap<Atom, NativeSeries>,
}

impl Expansion {
    fn series(&mut self, expression: &Atom) -> Result<NativeSeries, GenerationError> {
        if let Some(series) = self.cache.get(expression) {
            return Ok(series.clone());
        }
        let series = expression
            .series(self.regulator, 0, SeriesDepth::relative(self.width))
            .map_err(|error| GenerationError::Series(error.to_string()))?;
        self.cache.insert(expression.clone(), series.clone());
        Ok(series)
    }
}

/// The first pass establishes a bound through native arithmetic. If it does not
/// cover the requested order, increase the common input width by the observed
/// deficit and re-run. The bound is checked again, never assumed to improve by
/// that amount. A finite iteration/width cap makes failure explicit.
pub(crate) fn expand(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
) -> Result<(BTreeMap<i32, Atom>, Vec<Attempt>), GenerationError> {
    expand_with_names(terms, parameters, regulator, options, None)
}

fn expand_with_names(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    names: Option<&named::Coefficients>,
) -> Result<(BTreeMap<i32, Atom>, Vec<Attempt>), GenerationError> {
    // Preserve production's exact unregulated-endpoint admission. This rare
    // path may require epsilon-dependent exact boundary derivatives; a zero
    // *truncated* series cannot establish their all-order vanishing.
    let mut needs_exact_admission = false;
    for term in terms {
        for power in &term.powers {
            let (constant, slope) = endpoint_power(power, regulator)?;
            needs_exact_admission |= constant <= -1 && slope.is_zero();
        }
    }
    let exact_fallback = if needs_exact_admission {
        let (expression, count, _) = super::subtract(
            terms
                .iter()
                .map(|term| MappedTerm {
                    powers: term.powers.clone(),
                    prefactor: term.prefactor.clone(),
                    regular: term.regular.clone(),
                })
                .collect(),
            parameters,
            regulator,
            options,
        )?;
        Some((expression, count))
    } else {
        None
    };
    let mut width = 1i64;
    let mut attempts = Vec::new();
    for _ in 0..12 {
        let started = Instant::now();
        let (series, pieces) = if let Some((expression, count)) = &exact_fallback {
            (
                expression
                    .series(regulator, 0, SeriesDepth::relative(width))
                    .map_err(|error| GenerationError::Series(error.to_string()))?,
                *count,
            )
        } else {
            compose(terms, parameters, regulator, options, width, names)?
        };
        let bound = series.absolute_order();
        attempts.push(Attempt {
            route: if exact_fallback.is_some() {
                "exact_unregulated_fallback"
            } else {
                "series_first"
            },
            width,
            absolute_bound: bound.to_string(),
            pieces,
            seconds: started.elapsed().as_secs_f64(),
        });
        if bound > options.max_order {
            return Ok((
                coefficients(&series, regulator, options.max_order)?,
                attempts,
            ));
        }
        let deficit =
            (symbolica::domains::rational::Rational::from(i64::from(options.max_order) + 1)
                - bound)
                .ceil()
                .to_i64()
                .ok_or(GenerationError::ResourceLimit("series-first precision"))?;
        width = width
            .checked_add(deficit.max(1))
            .filter(|width| *width <= 128)
            .ok_or(GenerationError::ResourceLimit("series-first precision"))?;
    }
    Err(GenerationError::ResourceLimit(
        "series-first precision attempts",
    ))
}

fn compose(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    width: i64,
    names: Option<&named::Coefficients>,
) -> Result<(NativeSeries, usize), GenerationError> {
    let mut expansion = Expansion {
        regulator,
        width,
        cache: BTreeMap::new(),
    };
    let mut pieces = terms
        .iter()
        .map(|term| {
            Ok(Piece {
                powers: term.powers.iter().cloned().map(Some).collect(),
                prefactor: term.prefactor.clone(),
                regular: {
                    let regular = expansion.series(&term.regular)?;
                    match names {
                        Some(names) => names.wrap_series(&regular),
                        None => regular,
                    }
                },
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;
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
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(GenerationError::ResourceLimit("Taylor subtraction degree"))?;
            if count > options.max_subtractions_per_axis {
                return Err(GenerationError::ResourceLimit("Taylor subtraction degree"));
            }
            if options.subtraction == SubtractionStrategy::IntegrateByParts && !slope.is_zero() {
                for _ in 1..count {
                    let denominator = (&power + Atom::one()).expand();
                    let inverse = expansion.series(&(Atom::one() / &denominator))?;
                    let boundary = piece.regular.map_coeff(|coefficient| {
                        coefficient
                            .replace(Pattern::Literal(x.clone()))
                            .with(Atom::one())
                    });
                    let mut powers = piece.powers.clone();
                    powers[axis] = None;
                    next.push(Piece {
                        powers,
                        prefactor: piece.prefactor.clone(),
                        regular: scale(&boundary, &inverse, names),
                    });
                    piece.regular = scale(
                        &piece
                            .regular
                            .map_coeff(|coefficient| -coefficient.derivative(*parameter)),
                        &inverse,
                        names,
                    );
                    power = denominator;
                    piece.powers[axis] = Some(power.clone());
                }
                count = 1;
            }
            let mut derivative = piece.regular.clone();
            // map_coeff preserves the absolute remainder bound. Series::zero
            // instead resets the shift, so it is deliberately not used here.
            let mut taylor = piece.regular.map_coeff(|_| Atom::Zero);
            for degree in 0..count {
                let coefficient = derivative
                    .map_coeff(|value| value.replace(Pattern::Literal(x.clone())).with(Atom::Zero));
                if slope.is_zero() {
                    return Err(GenerationError::Invariant(
                        "unregulated exponent escaped exact prototype fallback".into(),
                    ));
                } else {
                    let denominator = (&power + Atom::num(degree + 1)).expand();
                    if denominator.is_zero() {
                        return Err(GenerationError::UnregulatedEndpoint {
                            parameter: *parameter,
                        });
                    }
                    taylor =
                        &taylor + &coefficient.map_coeff(|value| value * x.pow(Atom::num(degree)));
                    let inverse = expansion.series(&(Atom::one() / denominator))?;
                    let mut powers = piece.powers.clone();
                    powers[axis] = None;
                    next.push(Piece {
                        powers,
                        prefactor: piece.prefactor.clone(),
                        regular: scale(&coefficient, &inverse, names),
                    });
                }
                if degree + 1 < count {
                    derivative = derivative
                        .map_coeff(|value| value.derivative(*parameter) / Atom::num(degree + 1));
                }
            }
            // Retain even a known-zero remainder: its native truncation bound
            // still limits what can be concluded after later singular weights.
            piece.regular = &piece.regular - &taylor;
            next.push(piece);
            if next.len() > options.max_subtraction_terms {
                return Err(GenerationError::ResourceLimit(
                    "series-first subtraction terms",
                ));
            }
        }
        pieces = next;
    }
    let count = pieces.len();
    let mut groups = BTreeMap::<Atom, NativeSeries>::new();
    for piece in pieces {
        let mut regular = piece.regular;
        for (parameter, power) in parameters.iter().zip(piece.powers) {
            if let Some(power) = power {
                regular = scale(
                    &regular,
                    &expansion.series(&Atom::var(*parameter).pow(power))?,
                    names,
                );
            }
        }
        if let Some(group) = groups.get_mut(&piece.prefactor) {
            *group = &*group + &regular;
        } else {
            groups.insert(piece.prefactor, regular);
        }
    }
    let mut result = None;
    for (prefactor, regular) in groups {
        let term = scale(&regular, &expansion.series(&prefactor)?, names);
        result = Some(match result {
            Some(sum) => &sum + &term,
            None => term,
        });
    }
    Ok((
        match result {
            Some(sum) => sum,
            None => expansion.series(&Atom::Zero)?,
        },
        count,
    ))
}

/// The only products involving formal regular coefficients have an independent
/// scalar series. No inverse or nonlinear series operation sees those names.
fn scale(
    regular: &NativeSeries,
    scalar: &NativeSeries,
    names: Option<&named::Coefficients>,
) -> NativeSeries {
    if let Some(names) = names {
        for (_, coefficient) in scalar.terms() {
            names.assert_independent(coefficient);
        }
    }
    regular * scalar
}

fn coefficients(
    series: &NativeSeries,
    regulator: Symbol,
    maximum: i32,
) -> Result<BTreeMap<i32, Atom>, GenerationError> {
    if series.absolute_order() <= maximum {
        return Err(GenerationError::Invariant(
            "series-first output lacks absolute coverage".into(),
        ));
    }
    let mut coefficients = BTreeMap::new();
    for (order, coefficient) in series.terms() {
        if coefficient.is_zero() {
            continue;
        }
        if !order.is_integer() {
            return Err(GenerationError::FractionalLaurent(order.to_string()));
        }
        let order = order
            .numerator()
            .to_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| GenerationError::FractionalLaurent(order.to_string()))?;
        if order <= maximum {
            if coefficient.contains(Atom::var(regulator).as_view()) {
                return Err(GenerationError::Invariant(
                    "series-first coefficient contains regulator".into(),
                ));
            }
            coefficients.insert(order, coefficient.clone());
        }
    }
    Ok(coefficients)
}
