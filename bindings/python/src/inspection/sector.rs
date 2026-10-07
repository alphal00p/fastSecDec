use fastsecdec::generation::ConditioningBasis;
use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::Atom};

use super::{Owner, coefficient::PyCompactCoefficient, geometry::PySectorMap};

/// A numerical sector at its native kernel-association index, with shared ownership.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "GeneratedSector"
)]
pub(crate) struct PyGeneratedSector {
    pub(super) owner: Owner,
    pub(super) index: usize,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGeneratedSector {
    /// Native numerical-sector index used by Chart.kernel_sector.
    #[getter]
    fn index(&self) -> usize {
        self.index
    }
    /// Actual route used for this sector, including explicit local symbolic fallback.
    #[getter]
    fn generation_mode(&self) -> &'static str {
        self.owner.sectors()[self.index].generation_mode().name()
    }
    #[getter]
    fn dimension(&self) -> usize {
        self.owner.sectors()[self.index].dimension()
    }
    #[getter]
    fn parameters(&self) -> Vec<PythonExpression> {
        self.owner.sectors()[self.index]
            .parameters()
            .iter()
            .map(|symbol| PythonExpression {
                expr: Atom::var(*symbol),
            })
            .collect()
    }
    #[getter]
    fn coefficient_count(&self) -> usize {
        self.owner.sectors()[self.index]
            .aliased_coefficients()
            .len()
    }
    /// Stored alias definitions per coefficient, not a unique count across coefficients.
    #[getter]
    fn alias_counts(&self) -> Vec<usize> {
        self.owner.sectors()[self.index]
            .aliased_coefficients()
            .iter()
            .map(|coefficient| coefficient.get_aliases().len())
            .collect()
    }
    #[getter]
    fn cancellation_degree(&self) -> usize {
        self.owner.sectors()[self.index].cancellation_degree()
    }
    /// Provenance of conditioning profiles, not a floating-point error certificate.
    #[getter]
    fn conditioning_basis(&self) -> &'static str {
        match self.owner.sectors()[self.index].conditioning_basis() {
            ConditioningBasis::RetainedRemainders => "retained_remainders",
            ConditioningBasis::MappedEndpointBound => "mapped_endpoint_bound",
        }
    }
    #[getter]
    fn cancellation_terms(&self) -> Vec<Vec<usize>> {
        self.owner.sectors()[self.index]
            .cancellation_terms()
            .to_vec()
    }
    #[getter]
    fn map(&self) -> PySectorMap {
        PySectorMap::sector(self.owner.clone(), self.index)
    }
    /// Native complex coefficient roots/definitions aligned with GeneratedIntegral.orders.
    /// This creates cheap views; no coefficients are materialized or expanded.
    #[getter]
    fn aliased_coefficients(&self) -> Vec<PyCompactCoefficient> {
        (0..self.coefficient_count())
            .map(|coefficient| PyCompactCoefficient {
                owner: self.owner.clone(),
                sector: self.index,
                coefficient,
            })
            .collect()
    }
}
