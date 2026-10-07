//! Conservative weighted numerator valuations through native leading Series.
#[cfg(test)]
mod tests;
use super::super::GenerationError;
use fastsecdec_sectors::SectorMap;
use std::{collections::BTreeMap, sync::Mutex};
use symbolica::{
    atom::{Atom, AtomCore, Symbol, SymbolBuilder},
    domains::integer::Integer,
    id::{Pattern, Replacement},
    poly::series::SeriesDepth,
    wrap_symbol,
};

/// Generation-owned cache, independent of factor exponent and sector numbering.
/// A leading coefficient is deliberately not expanded or statistically tested:
/// unresolved cancellation can only make this returned bound smaller.
#[derive(Debug)]
pub(in crate::generation) struct ValuationCache {
    parameters: Vec<Symbol>,
    entries: Mutex<BTreeMap<(Atom, Vec<Integer>), i32>>,
}
impl ValuationCache {
    pub(in crate::generation) fn new(parameters: &[Symbol]) -> Self {
        Self {
            parameters: parameters.to_vec(),
            entries: Mutex::default(),
        }
    }
    pub(super) fn for_map(
        &self,
        polynomial: &Atom,
        map: &SectorMap,
    ) -> Result<Vec<i32>, GenerationError> {
        if map.exponent_matrix.len() != self.parameters.len()
            || map
                .exponent_matrix
                .iter()
                .any(|row| row.len() != map.dimension() || row.iter().any(|power| power < &0))
        {
            return Err(GenerationError::Invariant(
                "weighted numerator valuation requires a nonnegative map of matching dimension"
                    .into(),
            ));
        }
        let mut entries = self.entries.lock().map_err(|_| {
            GenerationError::Invariant("numerator valuation cache lock poisoned".into())
        })?;
        (0..map.dimension())
            .map(|axis| {
                let weights = map
                    .exponent_matrix
                    .iter()
                    .map(|row| row[axis].clone())
                    .collect::<Vec<_>>();
                let key = (polynomial.clone(), weights);
                if let Some(value) = entries.get(&key) {
                    return Ok(*value);
                }
                let value = self.bound(&key.0, &key.1)?;
                entries.insert(key, value);
                Ok(value)
            })
            .collect()
    }
    fn bound(&self, polynomial: &Atom, weights: &[Integer]) -> Result<i32, GenerationError> {
        if weights.iter().all(|weight| weight == &0) {
            return Ok(0);
        }
        let variables = self
            .parameters
            .iter()
            .map(|symbol| Atom::var(*symbol))
            .collect::<Vec<_>>();
        let Some(indeterminates) = polynomial.is_polynomial(true, false) else {
            return Ok(0);
        };
        if !indeterminates.iter().all(|indeterminate| {
            variables.iter().any(|v| *indeterminate == v.as_view())
                || variables
                    .iter()
                    .all(|v| !indeterminate.contains(v.as_view()))
        }) {
            return Ok(0);
        }
        let Ok(scale) =
            SymbolBuilder::new(wrap_symbol!("fastsecdec::numerical_dual::valuation_scale")).build()
        else {
            return Ok(0);
        };
        if !scale.is_exportable()
            || polynomial.contains_symbol(scale)
            || self.parameters.contains(&scale)
        {
            return Ok(0);
        }
        let scaled =
            polynomial.replace_multiple(variables.iter().zip(weights).map(|(variable, weight)| {
                Replacement::new(
                    Pattern::Literal(variable.clone()),
                    Pattern::Literal(variable * Atom::var(scale).pow(Atom::num(weight.clone()))),
                )
            }));
        // Native AtomCore::series constructs AtomField with statistical_zero_test
        // false. The admission above excludes negative parameter-dependent powers,
        // the only native polynomial Series branch that enables a sampled check.
        let Ok(series) = scaled.series(scale, 0, SeriesDepth::relative(1)) else {
            return Ok(0);
        };
        if series.get_field().statistical_zero_test {
            return Err(GenerationError::Invariant(
                "weighted valuation unexpectedly used statistical coefficient tests".into(),
            ));
        }
        let degree = series.get_trailing_exponent();
        if !degree.is_integer() || degree < 0 {
            return Err(GenerationError::Invariant(
                "polynomial weighted valuation is not a nonnegative integer".into(),
            ));
        }
        degree
            .numerator()
            .to_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or(GenerationError::ResourceLimit(
                "numerical dual weighted numerator valuation",
            ))
    }
}
