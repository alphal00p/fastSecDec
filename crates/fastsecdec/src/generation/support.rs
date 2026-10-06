//! Reuse exact source supports within one generation, before chart mapping.

use crate::parametric::{ParametricError, PolynomialFactor};
use fastsecdec_sectors::PolynomialSupport;
use std::collections::{BTreeMap, btree_map::Entry};
use symbolica::atom::{Atom, Symbol};

/// The ordered parameter vector is fixed for this owner. Entries contain only
/// source-polynomial support, independent of a factor's role/exponent or chart.
/// No support is collected before an existing consumer actually requests it.
#[derive(Clone)]
pub(super) struct SupportCache {
    parameters: Vec<Symbol>,
    entries: BTreeMap<Atom, PolynomialSupport>,
    #[cfg(test)]
    requests: usize,
    #[cfg(test)]
    misses: usize,
    #[cfg(test)]
    uncached: Option<PolynomialSupport>,
}

impl SupportCache {
    pub(super) fn new(parameters: &[Symbol]) -> Self {
        Self {
            parameters: parameters.to_vec(),
            entries: BTreeMap::new(),
            #[cfg(test)]
            requests: 0,
            #[cfg(test)]
            misses: 0,
            #[cfg(test)]
            uncached: None,
        }
    }

    pub(super) fn get(
        &mut self,
        factor: &PolynomialFactor,
    ) -> Result<&PolynomialSupport, ParametricError> {
        #[cfg(test)]
        {
            self.requests += 1;
            if BYPASS.with(std::cell::Cell::get) {
                self.misses += 1;
                self.uncached = Some(factor.support(&self.parameters)?);
                return Ok(self.uncached.as_ref().unwrap());
            }
        }
        Ok(match self.entries.entry(factor.polynomial().clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                #[cfg(test)]
                {
                    self.misses += 1;
                }
                entry.insert(factor.support(&self.parameters)?)
            }
        })
    }
}

#[cfg(test)]
std::thread_local! {
    static BYPASS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(super) fn profile_bypass(enabled: bool) -> ProfileBypass {
    ProfileBypass(BYPASS.with(|value| value.replace(enabled)))
}

#[cfg(test)]
pub(super) struct ProfileBypass(bool);

#[cfg(test)]
impl Drop for ProfileBypass {
    fn drop(&mut self) {
        BYPASS.with(|value| value.set(self.0));
    }
}

#[cfg(test)]
impl Drop for SupportCache {
    fn drop(&mut self) {
        let rows = self
            .entries
            .values()
            .map(|support| support.exponents().len())
            .sum::<usize>();
        let exponents = rows * self.parameters.len();
        super::mapping::profile::record_cache(super::mapping::profile::SupportCacheProfile {
            enabled: !BYPASS.with(std::cell::Cell::get),
            requests: self.requests,
            misses: self.misses,
            entries: self.entries.len(),
            key_expression_bytes: self
                .entries
                .keys()
                .map(|key| key.as_view().get_byte_size())
                .sum(),
            support_rows: rows,
            integer_entries: exponents,
            integer_storage_bytes: exponents
                * std::mem::size_of::<symbolica::domains::integer::Integer>(),
        });
    }
}
