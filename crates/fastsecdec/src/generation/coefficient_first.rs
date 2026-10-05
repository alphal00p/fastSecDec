//! Private coefficient-first extraction for the explicit native named route.
//! Native Series owns all arithmetic.

mod compose;
mod requests;
#[cfg(test)]
mod tests;

use super::{
    GenerationError, GenerationOptions,
    laurent::{self, TemplateCache},
    mapping::MappedTerm,
    subtraction::{self, endpoints},
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, Symbol},
    domains::{atom::AtomField, rational::Rational},
    poly::series::Series,
};

type NativeSeries = Series<AtomField>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RequestLimits {
    pub max_unique_requests: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RequestCounts {
    pub source_bodies: usize,
    pub unique_requests: usize,
    pub cached_partials: usize,
    pub aliases: usize,
    pub interleaved_requests: usize,
    pub fallback_requests: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestStage {
    ReserveSymbols,
    NameCoefficients,
    LowerRequests,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RequestProgress {
    pub stage: RequestStage,
    pub counts: RequestCounts,
}

pub(super) struct LoweredRequests {
    pub coefficients: BTreeMap<i32, AliasedAtom>,
    pub counts: RequestCounts,
}

/// Explicit caller caps, independent of endpoint degree/piece limits. A request
/// cap limits distinct derivative/face tuples, not native intermediate memory.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Limits {
    pub max_attempts: Option<usize>,
    pub max_relative_width: Option<i64>,
    pub requests: RequestLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScalarKind {
    EndpointInverse,
    CoordinateWeight,
    Prefactor,
}

/// Private work boundaries. Only the future generation coordinator owns public
/// progress variants, physical contribution counts and integral completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Admission {
        term: usize,
        axis: usize,
    },
    Attempt {
        number: usize,
        width: i64,
    },
    RegularSeries {
        term: usize,
    },
    ScalarSeries {
        kind: ScalarKind,
    },
    Endpoint {
        axis: usize,
        piece: usize,
    },
    EndpointDegree {
        axis: usize,
        degree: usize,
    },
    ComposePiece {
        piece: usize,
    },
    ComposeGroup {
        group: usize,
    },
    Coverage {
        attempt: usize,
        width: i64,
        absolute_order: Rational,
        formal_pieces: usize,
    },
    Requests(RequestProgress),
    PhysicalFallback,
    EmptySeries,
}

#[derive(Debug)]
pub(super) enum Route {
    Native {
        attempts: usize,
        relative_width: i64,
        absolute_order: Rational,
        formal_pieces: usize,
    },
    /// Production Laurent expansion checks its own native coverage. Do not
    /// invent a Series cutoff when that existing API does not return one.
    PhysicalFallback {
        pieces: usize,
        cancellation_terms: Vec<Vec<usize>>,
    },
}

pub(super) struct Completed {
    pub coefficients: BTreeMap<i32, AliasedAtom>,
    pub route: Route,
    pub requests: RequestCounts,
}

fn notify(
    poll: &mut impl FnMut(Progress) -> ControlFlow<()>,
    progress: Progress,
) -> Result<(), GenerationError> {
    match poll(progress) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(()) => Err(GenerationError::Cancelled),
    }
}

pub(super) fn expand(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    limits: Limits,
    fallback_cache: &mut TemplateCache,
    poll: &mut impl FnMut(Progress) -> ControlFlow<()>,
) -> Result<Completed, GenerationError> {
    if terms.len() > options.max_subtraction_terms {
        return Err(GenerationError::ResourceLimit(
            "coefficient-first subtraction terms",
        ));
    }
    let mut needs_exact_admission = false;
    for (term_index, term) in terms.iter().enumerate() {
        if term.powers.len() != parameters.len() {
            return Err(GenerationError::Invariant(
                "mapped endpoint dimension mismatch".into(),
            ));
        }
        for (axis, power) in term.powers.iter().enumerate() {
            notify(
                poll,
                Progress::Admission {
                    term: term_index,
                    axis,
                },
            )?;
            let (constant, slope) = endpoints::endpoint_power(power, regulator)?;
            notify(
                poll,
                Progress::Admission {
                    term: term_index,
                    axis,
                },
            )?;
            needs_exact_admission |= constant <= -1 && slope.is_zero();
        }
    }
    if needs_exact_admission {
        notify(poll, Progress::PhysicalFallback)?;
        let (expression, pieces, cancellation_terms) = subtraction::subtract(
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
        notify(poll, Progress::PhysicalFallback)?;
        let coefficients = laurent::expand(
            &expression,
            parameters,
            regulator,
            options.max_order,
            fallback_cache,
        )?;
        notify(poll, Progress::PhysicalFallback)?;
        return Ok(Completed {
            coefficients,
            route: Route::PhysicalFallback {
                pieces,
                cancellation_terms,
            },
            requests: RequestCounts::default(),
        });
    }

    // Enforce caller degree caps before any regular Series work. Keep this
    // after the fallback decision: the existing physical route owns its
    // derivative-dependent pruning and error precedence.
    for (term_index, term) in terms.iter().enumerate() {
        for (axis, power) in term.powers.iter().enumerate() {
            notify(
                poll,
                Progress::Admission {
                    term: term_index,
                    axis,
                },
            )?;
            endpoints::admit(power, regulator, options.max_subtractions_per_axis)?;
        }
    }

    let mut width = 1i64;
    let mut attempts = 0usize;
    loop {
        if limits
            .max_attempts
            .is_some_and(|maximum| attempts >= maximum)
        {
            return Err(GenerationError::ResourceLimit(
                "coefficient-first series attempts",
            ));
        }
        if limits
            .max_relative_width
            .is_some_and(|maximum| width > maximum)
        {
            return Err(GenerationError::ResourceLimit(
                "coefficient-first series width",
            ));
        }
        attempts = attempts
            .checked_add(1)
            .ok_or(GenerationError::ResourceLimit(
                "coefficient-first series attempts",
            ))?;
        notify(
            poll,
            Progress::Attempt {
                number: attempts,
                width,
            },
        )?;
        // Every width starts afresh; failed-width body/partial caches cannot
        // retain obsolete coefficients or contribute names to another attempt.
        let mut names = requests::Requests::new(
            terms,
            parameters,
            regulator,
            limits.requests,
            &mut |progress| poll(Progress::Requests(progress)),
        )?;
        let (series, pieces) = compose::compose(
            terms, parameters, regulator, options, width, &mut names, poll,
        )?;
        let bound = series.absolute_order();
        notify(
            poll,
            Progress::Coverage {
                attempt: attempts,
                width,
                absolute_order: bound.clone(),
                formal_pieces: pieces,
            },
        )?;
        if bound > options.max_order {
            let coefficients = coefficients(&series, regulator, options.max_order)?;
            let lowered = names.lower(coefficients, &mut |progress| {
                poll(Progress::Requests(progress))
            })?;
            return Ok(Completed {
                coefficients: lowered.coefficients,
                route: Route::Native {
                    attempts,
                    relative_width: width,
                    absolute_order: bound,
                    formal_pieces: pieces,
                },
                requests: lowered.counts,
            });
        }
        let deficit = (Rational::from(i64::from(options.max_order) + 1) - bound)
            .ceil()
            .to_i64()
            .ok_or(GenerationError::ResourceLimit(
                "coefficient-first series width",
            ))?;
        width = width
            .checked_add(deficit.max(1))
            .ok_or(GenerationError::ResourceLimit(
                "coefficient-first series width",
            ))?;
    }
}

fn coefficients(
    series: &NativeSeries,
    regulator: Symbol,
    maximum: i32,
) -> Result<BTreeMap<i32, Atom>, GenerationError> {
    if series.absolute_order() <= maximum {
        return Err(GenerationError::Invariant(
            "coefficient-first output lacks absolute coverage".into(),
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
                    "coefficient-first coefficient contains regulator".into(),
                ));
            }
            coefficients.insert(order, coefficient.clone());
        }
    }
    Ok(coefficients)
}
