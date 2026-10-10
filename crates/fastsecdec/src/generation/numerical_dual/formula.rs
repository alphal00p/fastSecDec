//! Generation-scoped exact formula signatures. Native Atoms own equality;
//! no polynomial expansion, hashing certificate or termwise factorization.
use super::subtraction::{self, Recipe, RecipeError};
use crate::generation::{
    GenerationError, GenerationEvent, GenerationOptions, GenerationProgress, context::emit,
    mapping::MappedTerm, subtraction::endpoints,
};
use std::{ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::rational::Rational,
};

/// Only compared within one generation's shared regulator, ordered coordinates,
/// and reserved-symbol context. Ordered terms preserve whole-chart cancellation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::generation) struct Key {
    dimension: usize,
    terms: Vec<(Vec<(Rational, Rational)>, Atom)>,
    max_order: i32,
    strategy: &'static str,
    max_subtractions_per_axis: usize,
    max_subtraction_terms: usize,
    method: &'static str,
    initial_relative_width: i64,
    max_series_attempts: Option<usize>,
    max_relative_width: Option<i64>,
    max_unique_requests: Option<usize>,
}

impl Key {
    pub(in crate::generation) fn discover(
        terms: &[MappedTerm],
        dimension: usize,
        regulator: Symbol,
        options: &GenerationOptions,
    ) -> Result<Option<Self>, GenerationError> {
        let mut signature = Vec::with_capacity(terms.len());
        for term in terms {
            if term.powers.len() != dimension {
                return Err(GenerationError::Invariant(
                    "dual endpoint dimension mismatch".into(),
                ));
            }
            let mut powers = Vec::with_capacity(dimension);
            for power in &term.powers {
                let (constant, slope) = endpoints::endpoint_power(power, regulator)?;
                if constant <= -1 && slope.is_zero() {
                    return Ok(None);
                }
                powers.push((constant, slope));
            }
            signature.push((powers, term.prefactor.clone()));
        }
        Ok(Some(Self {
            dimension,
            terms: signature,
            max_order: options.max_order,
            strategy: options.subtraction.name(),
            max_subtractions_per_axis: options.max_subtractions_per_axis,
            max_subtraction_terms: options.max_subtraction_terms,
            method: options.coefficient_expansion.method.name(),
            initial_relative_width: options.coefficient_expansion.initial_relative_width,
            max_series_attempts: options.coefficient_expansion.max_series_attempts,
            max_relative_width: options.coefficient_expansion.max_relative_width,
            max_unique_requests: options.coefficient_expansion.max_unique_requests,
        }))
    }

    /// The regular bodies are supplied later by each chart's native evaluators.
    /// All their symbols have already been reserved once in the shared context.
    pub(in crate::generation) fn terms(&self, regulator: Symbol) -> Vec<MappedTerm> {
        self.terms
            .iter()
            .map(|(powers, prefactor)| MappedTerm {
                powers: powers
                    .iter()
                    .map(|(constant, slope)| {
                        Atom::num(constant.clone())
                            + Atom::num(slope.clone()) * Atom::var(regulator)
                    })
                    .collect(),
                prefactor: prefactor.clone(),
                regular: Atom::one(),
            })
            .collect()
    }
}

impl Key {
    /// Portable lookup key; users must still compare the restored native key.
    pub(in crate::generation) fn lookup_key(&self) -> String {
        let terms = self
            .terms
            .iter()
            .map(|(powers, prefactor)| {
                (
                    powers
                        .iter()
                        .map(|(a, b)| (a.to_string(), b.to_string()))
                        .collect::<Vec<_>>(),
                    prefactor.to_canonical_string(),
                )
            })
            .collect::<Vec<_>>();
        let legacy = (
            self.dimension,
            terms,
            self.max_order,
            self.strategy,
            self.max_subtractions_per_axis,
            self.max_subtraction_terms,
            self.method,
            self.max_series_attempts,
            self.max_relative_width,
            self.max_unique_requests,
        );
        // Default callers retain the historical portable key bytes. Explicit
        // wider starts remain distinct because attempt caps can change success.
        let bytes = if self.initial_relative_width == 1 {
            serde_json::to_vec(&legacy)
        } else {
            serde_json::to_vec(&(legacy, self.initial_relative_width))
        }
        .expect("formula key contains native canonical strings and integers");
        blake3::hash(&bytes).to_hex().to_string()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::generation) struct Counts {
    pub total: usize,
    pub sectors: usize,
}
impl Counts {
    pub(super) fn event(self, completed: usize) -> GenerationProgress {
        GenerationProgress::FormulaPreparation {
            completed,
            total: self.total,
            sectors: self.sectors,
            reused: self.sectors - self.total,
        }
    }
}

pub(in crate::generation) fn build(
    key: &Key,
    context: &super::pipeline::Context,
    index: usize,
    counts: Counts,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Arc<Recipe>, GenerationError> {
    let mut poll = |_| match emit(progress, counts.event(index)) {
        Ok(()) => ControlFlow::Continue(()),
        Err(_) => ControlFlow::Break(()),
    };
    match subtraction::expand(
        &key.terms(context.input.regulator()),
        &context.parameters,
        context.input.regulator(),
        &context.options,
        &context.reserved_symbols,
        &mut poll,
    ) {
        Ok(recipe) => Ok(Arc::new(recipe)),
        Err(RecipeError::Generation(error)) => Err(error),
        Err(RecipeError::NeedsExactAdmission { .. }) => Err(GenerationError::Invariant(
            "discovered dual formula unexpectedly requires exact admission".into(),
        )),
    }
}

#[cfg(test)]
mod tests;
