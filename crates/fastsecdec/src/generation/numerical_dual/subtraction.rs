//! Formal endpoint recipes, reusing the production native Series composer.
//!
//! The regular functions are intentionally opaque. No mapped polynomial or
//! symbolic derivative of an actual regular function is constructed here.
pub(super) mod formal;
#[cfg(test)]
mod tests;

use super::super::{
    GenerationError, GenerationOptions, coefficient_first, laurent::TemplateCache,
    mapping::MappedTerm, subtraction::endpoints,
};
use std::{collections::BTreeMap, ops::ControlFlow};
use symbolica::atom::{AliasedAtom, Atom, AtomCore, Symbol};

/// The argument at which one axis of a regular function is evaluated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Coordinate {
    Variable(usize),
    Zero,
    One,
}

/// One normalized multivariate Taylor coefficient of a regular function.
///
/// The regulator is centered at zero. Coordinate axes are centered at
/// `coordinates`. The value supplied for `placeholder` is the coefficient of
/// epsilon^epsilon_order times the coordinate increments to `derivatives`:
/// derivatives divided by the product of ALL factorials, exactly as native
/// HyperDual stores them. The scalar recipe owns any factorial compensation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    pub placeholder: Symbol,
    pub term: usize,
    pub epsilon_order: usize,
    pub derivatives: Vec<usize>,
    pub coordinates: Vec<Coordinate>,
}

#[derive(Clone, Debug)]
pub(crate) struct Recipe {
    pub coefficients: BTreeMap<i32, AliasedAtom>,
    pub requests: Vec<Request>,
}

/// An opaque function cannot establish that an unregulated face vanishes.
/// The caller must obtain an exact proof from its native residual owner.
#[derive(Debug, thiserror::Error)]
pub(crate) enum RecipeError {
    #[error(transparent)]
    Generation(#[from] GenerationError),
    #[error("regular term {term}, coordinate {axis} needs exact unregulated endpoint admission")]
    NeedsExactAdmission { term: usize, axis: usize },
}

/// Compose the complete Laurent vector using the existing Taylor/IBP and
/// Laurent-coverage implementation. `regular` is used only to reserve symbols;
/// the caller retains the actual regular functions separately for native jets.
pub(super) fn expand(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
    reserved_symbols: &[Symbol],
    poll: &mut impl FnMut(coefficient_first::Progress) -> ControlFlow<()>,
) -> Result<Recipe, RecipeError> {
    // The production composer likewise performs exact physical admission before
    // its formal Series route. An unknown function must not become a false
    // unregulated-divergence diagnosis, nor can sampled zeros admit it.
    for (term, mapped) in terms.iter().enumerate() {
        if mapped.powers.len() != parameters.len() {
            return Err(
                GenerationError::Invariant("dual endpoint dimension mismatch".into()).into(),
            );
        }
        for (axis, power) in mapped.powers.iter().enumerate() {
            if poll(coefficient_first::Progress::Admission { term, axis }).is_break() {
                return Err(GenerationError::Cancelled.into());
            }
            let (constant, slope) = endpoints::endpoint_power(power, regulator)?;
            if constant <= -1 && slope.is_zero() {
                return Err(RecipeError::NeedsExactAdmission { term, axis });
            }
        }
    }
    let mut functions = formal::Functions::new(terms, parameters, regulator, reserved_symbols)?;
    let formal_terms = terms
        .iter()
        .enumerate()
        .map(|(term, mapped)| {
            Ok(MappedTerm {
                powers: mapped.powers.clone(),
                prefactor: mapped.prefactor.clone(),
                regular: functions.function(term)?,
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;
    let limits = coefficient_first::Limits {
        max_attempts: options.coefficient_expansion.max_series_attempts,
        max_relative_width: options.coefficient_expansion.max_relative_width,
        requests: coefficient_first::RequestLimits {
            max_unique_requests: options.coefficient_expansion.max_unique_requests,
        },
    };
    let result = coefficient_first::expand(
        &formal_terms,
        parameters,
        regulator,
        options,
        limits,
        &mut TemplateCache::default(),
        poll,
    )?;
    let recipe = functions.lower(
        result.coefficients,
        options.coefficient_expansion.max_unique_requests,
        &mut || {
            if poll(coefficient_first::Progress::Requests(
                coefficient_first::RequestProgress {
                    stage: coefficient_first::RequestStage::LowerRequests,
                    counts: result.requests,
                },
            ))
            .is_break()
            {
                Err(GenerationError::Cancelled)
            } else {
                Ok(())
            }
        },
    )?;
    // Both the exact roots and their flat native aliases must be regulator free.
    let epsilon = Atom::var(regulator);
    for coefficient in recipe.coefficients.values() {
        if coefficient.get_root().contains(epsilon.as_view())
            || coefficient
                .get_aliases()
                .values()
                .any(|value| value.contains(epsilon.as_view()))
        {
            return Err(GenerationError::Invariant("dual recipe retained regulator".into()).into());
        }
    }
    Ok(recipe)
}
