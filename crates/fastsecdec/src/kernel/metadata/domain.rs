use super::{KernelError, atom, invalid, symbol_strings, symbols};
use crate::{
    generation::{BranchPolicy, DomainAssessment, FactorCertificate, check_factors},
    parametric::{FactorRole, PolynomialFactor},
};
use fastsecdec_sectors::ParametricDomain;
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore};

#[derive(Clone, Copy, Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortableAssessment {
    parameters: Vec<String>,
    domain: PortableDomain,
    branch: BranchPolicy,
    caller_asserted: bool,
    factors: Vec<PortableFactor>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableFactor {
    term_index: usize,
    factor_index: usize,
    polynomial: String,
    exponent: String,
    certificate: FactorCertificate,
}
impl PortableAssessment {
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
                    polynomial: factor.polynomial().to_canonical_string(),
                    exponent: factor.exponent().to_canonical_string(),
                    certificate: factor.certificate(),
                })
                .collect(),
        }
    }
    pub(super) fn into_native(self) -> Result<DomainAssessment, KernelError> {
        let parameters = symbols(self.parameters)?;
        if parameters.is_empty() && matches!(self.domain, PortableDomain::ProjectiveSimplex) {
            return Err(invalid("empty projective source parameter list"));
        }
        let mut indices = std::collections::BTreeSet::new();
        let mut expected = Vec::new();
        let factors = self
            .factors
            .into_iter()
            .map(|factor| {
                if !indices.insert((factor.term_index, factor.factor_index)) {
                    return Err(invalid("duplicate factor assessment"));
                }
                expected.push(factor.certificate);
                let polynomial = atom(factor.polynomial)?;
                let exponent = atom(factor.exponent)?;
                if parameters
                    .iter()
                    .any(|symbol| exponent.contains(Atom::var(*symbol).as_view()))
                {
                    return Err(invalid("factor exponent depends on source coordinates"));
                }
                let factor_value =
                    PolynomialFactor::new(polynomial, exponent, FactorRole::Singularity);
                factor_value
                    .support(&parameters)
                    .map_err(|error| invalid(&error.to_string()))?;
                Ok((factor.term_index, factor.factor_index, factor_value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let assessment = check_factors(
            &parameters,
            self.domain.into_native(),
            &factors,
            self.caller_asserted,
        )
        .map_err(|error| invalid(&error.to_string()))?;
        if assessment.branch_policy() != self.branch
            || assessment
                .factors()
                .iter()
                .map(|factor| factor.certificate())
                .collect::<Vec<_>>()
                != expected
        {
            return Err(invalid(
                "domain factor certificate does not match native revalidation",
            ));
        }
        Ok(assessment)
    }
}
