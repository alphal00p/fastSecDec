use fastsecdec::generation::{BranchPolicy, FactorCertificate};
use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::Atom};

use super::{Owner, domain_name, expression};

/// The native domain and threshold policy, including historical assessments.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "DomainAssessment"
)]
pub(crate) struct PyDomainAssessment {
    pub(super) owner: Owner,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDomainAssessment {
    #[getter]
    fn domain(&self) -> &'static str {
        domain_name(self.owner.metadata().domain_assessment().domain())
    }
    #[getter]
    fn branch_policy(&self) -> &'static str {
        match self.owner.metadata().domain_assessment().branch_policy() {
            BranchPolicy::NoThresholdReal => "no_threshold_real",
            BranchPolicy::UserResponsible => "user_responsible",
        }
    }
    #[getter]
    fn caller_asserted(&self) -> bool {
        self.owner.metadata().domain_assessment().caller_asserted()
    }
    #[getter]
    fn relies_on_assertion(&self) -> bool {
        self.owner
            .metadata()
            .domain_assessment()
            .relies_on_assertion()
    }
    #[getter]
    fn parameters(&self) -> Vec<PythonExpression> {
        self.owner
            .metadata()
            .domain_assessment()
            .parameters()
            .iter()
            .map(|symbol| PythonExpression {
                expr: Atom::var(*symbol),
            })
            .collect()
    }
    #[getter]
    fn factors(&self) -> Vec<PyFactorAssessment> {
        (0..self.owner.metadata().domain_assessment().factors().len())
            .map(|index| PyFactorAssessment {
                owner: self.owner.clone(),
                index,
            })
            .collect()
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "FactorAssessment"
)]
pub(crate) struct PyFactorAssessment {
    owner: Owner,
    index: usize,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyFactorAssessment {
    #[getter]
    fn term_index(&self) -> usize {
        self.owner.metadata().domain_assessment().factors()[self.index].term_index()
    }
    #[getter]
    fn factor_index(&self) -> usize {
        self.owner.metadata().domain_assessment().factors()[self.index].factor_index()
    }
    #[getter]
    fn polynomial(&self) -> PythonExpression {
        expression(self.owner.metadata().domain_assessment().factors()[self.index].polynomial())
    }
    #[getter]
    fn exponent(&self) -> PythonExpression {
        expression(self.owner.metadata().domain_assessment().factors()[self.index].exponent())
    }
    #[getter]
    fn certificate(&self) -> &'static str {
        match self.owner.metadata().domain_assessment().factors()[self.index].certificate() {
            FactorCertificate::UncheckedUserResponsibility => "unchecked_user_responsibility",
            FactorCertificate::PositiveCoefficients => "positive_coefficients",
            FactorCertificate::NegativeCoefficientsIntegerPower => {
                "negative_coefficients_integer_power"
            }
            FactorCertificate::ExplicitInteriorAssertion => "explicit_interior_assertion",
        }
    }
}
