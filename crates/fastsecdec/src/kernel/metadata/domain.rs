use super::{KernelError, StoredAtom, atom, invalid, symbol_strings, symbols};
use crate::{
    generation::{BranchPolicy, DomainAssessment, FactorAssessment, FactorCertificate},
    parametric::{FactorRole, PolynomialFactor},
};
use fastsecdec_sectors::ParametricDomain;
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore};
use symbolica::state::StateMap;

#[derive(Clone, Copy, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
pub(super) enum PortableDomain {
    ProjectiveSimplex,
    UnitCube,
    PositiveOrthant,
}
impl PortableDomain {
    pub(super) fn from_native(value: ParametricDomain) -> Self {
        match value {
            ParametricDomain::ProjectiveSimplex => Self::ProjectiveSimplex,
            ParametricDomain::UnitCube => Self::UnitCube,
            ParametricDomain::PositiveOrthant => Self::PositiveOrthant,
        }
    }
    pub(super) fn into_native(self) -> ParametricDomain {
        match self {
            Self::ProjectiveSimplex => ParametricDomain::ProjectiveSimplex,
            Self::UnitCube => ParametricDomain::UnitCube,
            Self::PositiveOrthant => ParametricDomain::PositiveOrthant,
        }
    }
}
#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
pub(super) struct PortableAssessment {
    parameters: Vec<String>,
    domain: PortableDomain,
    #[bincode(with_serde)]
    branch: BranchPolicy,
    caller_asserted: bool,
    factors: Vec<PortableFactor>,
}
#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "StateMap")]
#[serde(deny_unknown_fields)]
struct PortableFactor {
    term_index: usize,
    factor_index: usize,
    polynomial: StoredAtom,
    exponent: StoredAtom,
    #[bincode(with_serde)]
    certificate: FactorCertificate,
}
impl PortableAssessment {
    pub(super) fn visit_atoms(&self, visit: &mut impl FnMut(&Atom)) {
        for factor in &self.factors {
            visit(&factor.polynomial.0);
            visit(&factor.exponent.0);
        }
    }

    pub(super) fn from_native(value: &DomainAssessment) -> Self {
        Self {
            parameters: symbol_strings(value.parameters()),
            domain: PortableDomain::from_native(value.domain()),
            branch: value.branch_policy(),
            caller_asserted: value.caller_asserted(),
            factors: value
                .factors()
                .iter()
                .map(|factor| PortableFactor {
                    term_index: factor.term_index(),
                    factor_index: factor.factor_index(),
                    polynomial: factor.polynomial().into(),
                    exponent: factor.exponent().into(),
                    certificate: factor.certificate(),
                })
                .collect(),
        }
    }
    pub(super) fn into_native(self, validate: bool) -> Result<DomainAssessment, KernelError> {
        let parameters = symbols(self.parameters)?;
        if parameters.is_empty() && matches!(self.domain, PortableDomain::ProjectiveSimplex) {
            return Err(invalid("empty projective source parameter list"));
        }
        let mut indices = std::collections::BTreeSet::new();
        let factors = self
            .factors
            .into_iter()
            .map(|factor| {
                if !indices.insert((factor.term_index, factor.factor_index)) {
                    return Err(invalid("duplicate factor assessment"));
                }
                let polynomial = atom(factor.polynomial)?;
                let exponent = atom(factor.exponent)?;
                if parameters
                    .iter()
                    .any(|symbol| exponent.contains(Atom::var(*symbol).as_view()))
                {
                    return Err(invalid("factor exponent depends on source coordinates"));
                }
                if validate {
                    let factor_value = PolynomialFactor::new(
                        polynomial.clone(),
                        exponent.clone(),
                        FactorRole::Singularity,
                    );
                    factor_value
                        .support(&parameters)
                        .map_err(|error| invalid(error.to_string()))?;
                }
                Ok(FactorAssessment {
                    term_index: factor.term_index,
                    factor_index: factor.factor_index,
                    polynomial,
                    exponent,
                    certificate: factor.certificate,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Retain historical labels without re-running threshold certification.
        // Absence of thresholds is the caller's responsibility.
        Ok(DomainAssessment {
            parameters,
            domain: self.domain.into_native(),
            branch: self.branch,
            caller_asserted: self.caller_asserted,
            factors,
        })
    }
}
