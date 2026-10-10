//! Exact source witness for a permutation-equivariant dynamic contour.
//!
//! The native density and explicitly designated branch factors are compared
//! before compact functions are introduced. A hash or function name is never
//! evidence of equality. Retaining declarations separately matters when their
//! powers are zero or source terms cancel.
use super::*;
use crate::{
    contour::continued_power, kernel::indexed::ProgramRecipe, parametric::FactorSemantics,
};
use bincode::{Decode, Encode};
use symbolica::state::StateMap;

#[derive(Clone, Debug, Encode, Decode)]
#[bincode(decode_context = "StateMap")]
pub(crate) struct SourceWitness {
    #[bincode(with_serde)]
    recipe: ProgramRecipe,
    parameters: Vec<Symbol>,
    density: Atom,
    causal: Atom,
    positive: Vec<Atom>,
}

impl SourceWitness {
    pub(in crate::generation) fn new(
        parameters: &[Symbol],
        terms: &[super::super::mapping::PreparedTerm],
        recipe: ProgramRecipe,
    ) -> Result<Option<Self>, GenerationError> {
        if !matches!(
            recipe,
            ProgramRecipe::DynamicPolynomialV1 | ProgramRecipe::DynamicSignAwareV1
        ) || terms.is_empty()
        {
            return Ok(None);
        }
        // Reuse the map's declaration admission; factor order and signs must
        // be identical to the actual envelope construction.
        let (causal, positive) = super::super::mapping::declared_factors(terms)?;
        let Some(causal) = causal else {
            return Ok(None);
        };
        let density = terms
            .iter()
            .map(|term| {
                &term.prefactor
                    * parameters
                        .iter()
                        .zip(&term.powers)
                        .map(|(parameter, power)| {
                            continued_power(&Atom::var(*parameter), power, false)
                        })
                        .product::<Atom>()
                    * term
                        .residuals
                        .iter()
                        .filter(|(_, exponent, _)| !exponent.is_zero())
                        .map(|(factor, exponent, semantics)| {
                            continued_power(factor, exponent, *semantics == FactorSemantics::Causal)
                        })
                        .product::<Atom>()
            })
            .sum();
        Ok(Some(Self {
            recipe,
            parameters: parameters.to_vec(),
            density,
            causal,
            positive,
        }))
    }

    pub(in crate::generation) fn symbols(&self) -> impl Iterator<Item = Symbol> + '_ {
        self.parameters.iter().copied().chain(
            std::iter::once(&self.density)
                .chain(std::iter::once(&self.causal))
                .chain(&self.positive)
                .flat_map(|atom| atom.get_all_symbols(true)),
        )
    }

    pub(in crate::generation) fn prepare(
        &self,
        parameters: &[Symbol],
    ) -> Result<PreparedDensity, GenerationError> {
        if parameters != self.parameters {
            return Err(GenerationError::Invariant(
                "source symmetry witness parameter mismatch".into(),
            ));
        }
        let recipe = match self.recipe {
            ProgramRecipe::DynamicPolynomialV1 => 1,
            ProgramRecipe::DynamicSignAwareV1 => 2,
            _ => {
                return Err(GenerationError::Invariant(
                    "non-dynamic source symmetry witness".into(),
                ));
            }
        };
        let witness = std::iter::once(Atom::num(recipe))
            .chain(std::iter::once(self.causal.clone()))
            .chain(self.positive.iter().cloned())
            .collect::<Vec<_>>();
        Ok(PreparedDensity {
            parameters: parameters.to_vec(),
            density: self.density.clone(),
            canonical: incidence_graph_with_witness(&self.density, &witness, parameters)?,
            witness,
        })
    }
}

#[cfg(test)]
mod tests;
