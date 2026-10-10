//! One complete chart discovers all branch factors before choosing its map.
use super::MappedTerm;
use crate::{
    contour::{ContourMetadata, SmoothContourMap},
    generation::GenerationError,
    generation::symmetry::SourceWitness,
    kernel::ProgramRecipe,
    parametric::FactorSemantics,
};
use symbolica::atom::{Atom, Symbol};
pub(super) mod program;

pub(in crate::generation) struct PreparedTerm {
    pub powers: Vec<Atom>,
    pub prefactor: Atom,
    pub residuals: Vec<(Atom, Atom, FactorSemantics)>,
}

type DeformedChart = (
    Vec<MappedTerm>,
    Option<ContourMetadata>,
    Option<SourceWitness>,
);

pub(super) fn deform_with(
    parameters: &[Symbol],
    terms: Vec<PreparedTerm>,
    recipe: ProgramRecipe,
    build: impl FnOnce(&[Symbol], Atom, &[Atom]) -> Result<SmoothContourMap, GenerationError>,
) -> Result<DeformedChart, GenerationError> {
    let (causal, positive) = declared_factors(&terms)?;
    let Some(causal) = causal else {
        return Ok((Vec::new(), None, None));
    };
    let mut map = build(parameters, causal, &positive)?;
    let witness =
        if recipe != ProgramRecipe::FixedV1 || !map.metadata().function_definitions().is_empty() {
            SourceWitness::new(parameters, &terms, recipe)?
        } else {
            None
        };
    let mapped = terms
        .into_iter()
        .map(|term| {
            let regular = map.smooth_density(&term.powers, &term.residuals);
            MappedTerm {
                powers: term.powers,
                prefactor: term.prefactor,
                regular,
            }
        })
        .collect();
    Ok((mapped, Some(map.metadata().clone()), witness))
}

pub(in crate::generation) fn declared_factors(
    terms: &[PreparedTerm],
) -> Result<(Option<Atom>, Vec<Atom>), GenerationError> {
    let mut causal = None;
    let mut positive = Vec::new();
    for term in terms {
        let mut factors = term
            .residuals
            .iter()
            .filter(|(_, _, role)| *role == FactorSemantics::Causal);
        let Some((f, _, _)) = factors.next() else {
            return Err(GenerationError::Contour(
                "each term requires an explicitly designated F polynomial".into(),
            ));
        };
        if factors.next().is_some() {
            return Err(GenerationError::Contour(
                "each term requires exactly one residual F polynomial".into(),
            ));
        }
        if causal.as_ref().is_some_and(|previous| previous != f) {
            return Err(GenerationError::Contour(
                "terms in one contour chart must share the same residual F polynomial".into(),
            ));
        }
        causal = Some(f.clone());
        for (factor, _, role) in &term.residuals {
            if *role == FactorSemantics::Positive && !positive.contains(factor) {
                positive.push(factor.clone());
            }
        }
    }
    Ok((causal, positive))
}

#[cfg(test)]
mod tests;
