//! Endpoint operations on native Series with named regular coefficients.
//! Keep scalar weights separate: no inverse or nonlinear Series operation may
//! consume the formal coefficient names.

use super::*;
use crate::generation::SubtractionStrategy;
use symbolica::{id::Pattern, poly::series::SeriesDepth};

struct Piece {
    powers: Vec<Option<Atom>>,
    prefactor: Atom,
    regular: NativeSeries,
}

struct Expansion {
    regulator: Symbol,
    width: i64,
    cache: BTreeMap<Atom, NativeSeries>,
}

impl Expansion {
    fn series(
        &mut self,
        expression: &Atom,
        progress: Progress,
        poll: &mut impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<NativeSeries, GenerationError> {
        notify(poll, progress.clone())?;
        if let Some(series) = self.cache.get(expression) {
            return Ok(series.clone());
        }
        let series = expression
            .series(self.regulator, 0, SeriesDepth::relative(self.width))
            .map_err(|error| GenerationError::Series(error.to_string()))?;
        notify(poll, progress)?;
        self.cache.insert(expression.clone(), series.clone());
        Ok(series)
    }

    fn scalar(
        &mut self,
        expression: &Atom,
        kind: ScalarKind,
        poll: &mut impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<NativeSeries, GenerationError> {
        self.series(expression, Progress::ScalarSeries { kind }, poll)
    }
}

fn push(pieces: &mut Vec<Piece>, piece: Piece, maximum: usize) -> Result<(), GenerationError> {
    let count = pieces
        .len()
        .checked_add(1)
        .ok_or(GenerationError::ResourceLimit(
            "coefficient-first subtraction terms",
        ))?;
    if count > maximum {
        return Err(GenerationError::ResourceLimit(
            "coefficient-first subtraction terms",
        ));
    }
    pieces.push(piece);
    Ok(())
}

pub(super) fn compose(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    width: i64,
    names: &mut requests::Requests,
    poll: &mut impl FnMut(Progress) -> ControlFlow<()>,
) -> Result<(NativeSeries, usize), GenerationError> {
    let mut expansion = Expansion {
        regulator,
        width,
        cache: BTreeMap::new(),
    };
    let mut pieces = Vec::new();
    for (index, term) in terms.iter().enumerate() {
        let regular =
            expansion.series(&term.regular, Progress::RegularSeries { term: index }, poll)?;
        let regular =
            names.wrap_series(&regular, &mut |progress| poll(Progress::Requests(progress)))?;
        push(
            &mut pieces,
            Piece {
                powers: term.powers.iter().cloned().map(Some).collect(),
                prefactor: term.prefactor.clone(),
                regular,
            },
            options.max_subtraction_terms,
        )?;
    }
    for (axis, parameter) in parameters.iter().enumerate() {
        let x = Atom::var(*parameter);
        let mut next = Vec::new();
        for (piece_index, mut piece) in pieces.into_iter().enumerate() {
            notify(
                poll,
                Progress::Endpoint {
                    axis,
                    piece: piece_index,
                },
            )?;
            let mut power = piece
                .powers
                .get(axis)
                .and_then(Option::as_ref)
                .ok_or_else(|| GenerationError::Invariant("missing active endpoint power".into()))?
                .clone();
            let endpoint = endpoints::admit(&power, regulator, options.max_subtractions_per_axis)?;
            if endpoint.constant > -1 {
                push(&mut next, piece, options.max_subtraction_terms)?;
                continue;
            }
            let slope = endpoint.slope;
            let mut count = endpoint.subtractions;
            if slope.is_zero() {
                return Err(GenerationError::Invariant(
                    "unregulated endpoint escaped exact physical fallback".into(),
                ));
            }
            if options.subtraction == SubtractionStrategy::IntegrateByParts {
                for degree in 1..count {
                    notify(poll, Progress::EndpointDegree { axis, degree })?;
                    let denominator = (&power + Atom::one()).expand();
                    let inverse = expansion.scalar(
                        &(Atom::one() / &denominator),
                        ScalarKind::EndpointInverse,
                        poll,
                    )?;
                    let boundary = piece.regular.map_coeff(|coefficient| {
                        coefficient
                            .replace(Pattern::Literal(x.clone()))
                            .with(Atom::one())
                    });
                    let mut powers = piece.powers.clone();
                    powers[axis] = None;
                    // A truncated zero is not an all-order zero certificate.
                    push(
                        &mut next,
                        Piece {
                            powers,
                            prefactor: piece.prefactor.clone(),
                            regular: scale(&boundary, &inverse, names)?,
                        },
                        options.max_subtraction_terms,
                    )?;
                    piece.regular = scale(
                        &piece
                            .regular
                            .map_coeff(|coefficient| -coefficient.derivative(*parameter)),
                        &inverse,
                        names,
                    )?;
                    power = denominator;
                    piece.powers[axis] = Some(power.clone());
                    notify(poll, Progress::EndpointDegree { axis, degree })?;
                }
                count = 1;
            }
            let mut derivative = piece.regular.clone();
            // Native map_coeff retains the absolute remainder. Constructing a
            // fresh Series::zero would discard that unknown-tail information.
            let mut taylor = piece.regular.map_coeff(|_| Atom::Zero);
            for degree in 0..count {
                notify(poll, Progress::EndpointDegree { axis, degree })?;
                let coefficient = derivative
                    .map_coeff(|value| value.replace(Pattern::Literal(x.clone())).with(Atom::Zero));
                let denominator = (&power + Atom::num(degree + 1)).expand();
                if denominator.is_zero() {
                    return Err(GenerationError::UnregulatedEndpoint {
                        parameter: *parameter,
                    });
                }
                taylor = &taylor + &coefficient.map_coeff(|value| value * x.pow(Atom::num(degree)));
                let inverse = expansion.scalar(
                    &(Atom::one() / denominator),
                    ScalarKind::EndpointInverse,
                    poll,
                )?;
                let mut powers = piece.powers.clone();
                powers[axis] = None;
                push(
                    &mut next,
                    Piece {
                        powers,
                        prefactor: piece.prefactor.clone(),
                        regular: scale(&coefficient, &inverse, names)?,
                    },
                    options.max_subtraction_terms,
                )?;
                if degree + 1 < count {
                    derivative = derivative
                        .map_coeff(|value| value.derivative(*parameter) / Atom::num(degree + 1));
                }
                notify(poll, Progress::EndpointDegree { axis, degree })?;
            }
            piece.regular = &piece.regular - &taylor;
            push(&mut next, piece, options.max_subtraction_terms)?;
            notify(
                poll,
                Progress::Endpoint {
                    axis,
                    piece: piece_index,
                },
            )?;
        }
        pieces = next;
    }
    let count = pieces.len();
    let mut groups = BTreeMap::<Atom, NativeSeries>::new();
    for (piece_index, piece) in pieces.into_iter().enumerate() {
        notify(poll, Progress::ComposePiece { piece: piece_index })?;
        let mut regular = piece.regular;
        for (parameter, power) in parameters.iter().zip(piece.powers) {
            if let Some(power) = power {
                let scalar = expansion.scalar(
                    &Atom::var(*parameter).pow(power),
                    ScalarKind::CoordinateWeight,
                    poll,
                )?;
                regular = scale(&regular, &scalar, names)?;
                notify(poll, Progress::ComposePiece { piece: piece_index })?;
            }
        }
        if let Some(group) = groups.get_mut(&piece.prefactor) {
            *group = &*group + &regular;
        } else {
            groups.insert(piece.prefactor, regular);
        }
        notify(poll, Progress::ComposePiece { piece: piece_index })?;
    }
    let mut result = None;
    for (index, (prefactor, regular)) in groups.into_iter().enumerate() {
        notify(poll, Progress::ComposeGroup { group: index })?;
        let scalar = expansion.scalar(&prefactor, ScalarKind::Prefactor, poll)?;
        let term = scale(&regular, &scalar, names)?;
        result = Some(match result {
            Some(sum) => &sum + &term,
            None => term,
        });
        notify(poll, Progress::ComposeGroup { group: index })?;
    }
    Ok((
        match result {
            Some(series) => series,
            // Relative-depth native zero retains a covering remainder; rely on
            // that native bound exactly as for every nonempty composition.
            None => expansion.series(&Atom::Zero, Progress::EmptySeries, poll)?,
        },
        count,
    ))
}

fn scale(
    regular: &NativeSeries,
    scalar: &NativeSeries,
    names: &requests::Requests,
) -> Result<NativeSeries, GenerationError> {
    for (_, coefficient) in scalar.terms() {
        names.check_scalar(coefficient)?;
    }
    Ok(regular * scalar)
}
