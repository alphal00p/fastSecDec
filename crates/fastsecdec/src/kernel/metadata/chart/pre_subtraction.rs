use super::super::{StoredAtom, atom, invalid, symbols};
use crate::{
    generation::{EndpointPower, PreSubtractionMetadata, PreSubtractionTerm},
    kernel::KernelError,
};
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore, Symbol};
use symbolica::state::StateMap;

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
pub(super) struct PortablePreSubtraction {
    version: u32,
    regulator: String,
    terms: Vec<PortableTerm>,
}
#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
struct PortableTerm {
    prefactor: StoredAtom,
    powers: Vec<StoredAtom>,
    regular_expression_bytes: usize,
}
impl PortablePreSubtraction {
    pub(super) fn visit_atoms(&self, visit: &mut impl FnMut(&Atom)) {
        for term in &self.terms {
            visit(&term.prefactor.0);
            for power in &term.powers {
                visit(&power.0);
            }
        }
    }

    pub(super) fn from_native(value: &PreSubtractionMetadata) -> Self {
        Self {
            version: value.version(),
            regulator: Atom::var(value.regulator()).to_canonical_string(),
            terms: value
                .terms()
                .iter()
                .map(|term| PortableTerm {
                    prefactor: term.prefactor().into(),
                    powers: term
                        .powers()
                        .iter()
                        .map(|power| StoredAtom::from(power.exponent()))
                        .collect(),
                    regular_expression_bytes: term.regular_expression_bytes(),
                })
                .collect(),
        }
    }
    pub(super) fn into_native(
        self,
        source: &[Symbol],
        target: &[Symbol],
    ) -> Result<PreSubtractionMetadata, KernelError> {
        if self.version != 1 {
            return Err(invalid("unsupported pre-subtraction metadata version"));
        }
        let regulator = symbols(vec![self.regulator])?[0];
        if source.contains(&regulator) || target.contains(&regulator) {
            return Err(invalid(
                "pre-subtraction regulator overlaps chart coordinates",
            ));
        }
        let terms = self
            .terms
            .into_iter()
            .map(|term| {
                if term.powers.len() != target.len() {
                    return Err(invalid("pre-subtraction power dimension differs"));
                }
                let prefactor = atom(term.prefactor)?;
                let symbols = prefactor.get_all_symbols(true);
                if prefactor.is_zero()
                    || source
                        .iter()
                        .chain(target)
                        .any(|parameter| symbols.contains(parameter))
                {
                    return Err(invalid(
                        "pre-subtraction prefactor is zero or coordinate dependent",
                    ));
                }
                let powers = term
                    .powers
                    .into_iter()
                    .map(|power| {
                        EndpointPower::admit(atom(power)?, regulator, usize::MAX)
                            .map_err(|_| invalid("invalid pre-subtraction endpoint power"))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(PreSubtractionTerm {
                    prefactor,
                    powers,
                    regular_expression_bytes: term.regular_expression_bytes,
                })
            })
            .collect::<Result<_, KernelError>>()?;
        Ok(PreSubtractionMetadata { regulator, terms })
    }
}
