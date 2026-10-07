//! Read-only native estimate provenance, covariance and provisional coverage.
use super::integration::{PyIntegrationSnapshot, PyVectorEstimate};
use fastsecdec::{
    integration::{
        IntegrationObservation, LiveEstimate, LiveObservation, LiveSector, SectorContribution,
    },
    status::CoefficientComponent,
};
use pyo3::prelude::*;

fn components(values: &[CoefficientComponent]) -> Vec<&'static str> {
    values
        .iter()
        .map(|v| match v {
            CoefficientComponent::Real => "real",
            CoefficientComponent::Imag => "imag",
        })
        .collect()
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "IntegrationObservation",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyIntegrationObservation {
    pub(crate) inner: IntegrationObservation,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyIntegrationObservation {
    #[getter]
    fn snapshot(&self) -> PyIntegrationSnapshot {
        PyIntegrationSnapshot {
            inner: self.inner.snapshot.clone(),
        }
    }
    #[getter]
    fn total(&self) -> Option<PyVectorEstimate> {
        self.inner
            .contributions
            .total
            .clone()
            .map(|inner| PyVectorEstimate { inner })
    }
    #[getter]
    fn sectors(&self) -> Vec<PySectorContribution> {
        self.inner
            .contributions
            .sectors
            .iter()
            .cloned()
            .map(|inner| PySectorContribution { inner })
            .collect()
    }
    #[getter]
    fn exact_coefficients(&self) -> Vec<f64> {
        self.inner.contributions.exact_coefficients.clone()
    }
    #[getter]
    fn orders(&self) -> Vec<i32> {
        self.inner.contributions.orders.clone()
    }
    #[getter]
    fn components(&self) -> Vec<&'static str> {
        components(&self.inner.contributions.components)
    }
    #[getter]
    fn replica_relation(&self) -> &'static str {
        match self.inner.contributions.replica_relation {
            fastsecdec::integration::ReplicaRelation::SharedAcrossSectors => {
                "shared_across_sectors"
            }
            fastsecdec::integration::ReplicaRelation::IndependentAcrossSectors => {
                "independent_across_sectors"
            }
        }
    }
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
    fn __str__(&self) -> String {
        self.inner.snapshot.to_string()
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "SectorContribution",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PySectorContribution {
    inner: SectorContribution,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PySectorContribution {
    #[getter]
    fn id(&self) -> u64 {
        self.inner.progress.id
    }
    #[getter]
    fn completed_points(&self) -> u64 {
        self.inner.progress.completed_points
    }
    #[getter]
    fn planned_points(&self) -> Option<u64> {
        self.inner.progress.planned_points
    }
    #[getter]
    fn used_points(&self) -> u64 {
        self.inner.used_points
    }
    #[getter]
    fn used_replicas(&self) -> usize {
        self.inner.used_replicas
    }
    #[getter]
    fn estimate(&self) -> Option<PyVectorEstimate> {
        self.inner
            .estimate
            .clone()
            .map(|inner| PyVectorEstimate { inner })
    }
    #[getter]
    fn uncertainty(&self) -> String {
        format!("{:?}", self.inner.uncertainty)
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "LiveEstimate",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLiveEstimate {
    inner: LiveEstimate,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyLiveEstimate {
    #[getter]
    fn mean(&self) -> Option<Vec<f64>> {
        self.inner.mean.clone()
    }
    #[getter]
    fn standard_error(&self) -> Option<Vec<f64>> {
        self.inner.standard_error.clone()
    }
    #[getter]
    fn points(&self) -> u64 {
        self.inner.points
    }
    #[getter]
    fn replicas(&self) -> usize {
        self.inner.replicas
    }
    #[getter]
    fn status(&self) -> &'static str {
        use fastsecdec::integration::LiveStatus::*;
        match self.inner.status {
            NotSampled => "not_sampled",
            MeanOnly => "mean_only",
            Available => "available",
            NumericalRange => "numerical_range",
        }
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "LiveSector",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLiveSector {
    inner: LiveSector,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyLiveSector {
    #[getter]
    fn id(&self) -> u64 {
        self.inner.id
    }
    #[getter]
    fn estimate(&self) -> PyLiveEstimate {
        PyLiveEstimate {
            inner: self.inner.estimate.clone(),
        }
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "LiveObservation",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLiveObservation {
    pub(crate) inner: LiveObservation,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyLiveObservation {
    #[getter]
    fn source(&self) -> &'static str {
        use fastsecdec::integration::LiveSource::*;
        match self.inner.source {
            CurrentIteration => "current_iteration",
            SinceResume => "since_resume",
            CompleteLattices => "complete_lattices",
        }
    }
    #[getter]
    fn stage(&self) -> &'static str {
        match self.inner.stage {
            fastsecdec::status::IntegrationStage::Pilot => "pilot",
            fastsecdec::status::IntegrationStage::Production => "production",
        }
    }
    #[getter]
    fn total(&self) -> PyLiveEstimate {
        PyLiveEstimate {
            inner: self.inner.total.clone(),
        }
    }
    #[getter]
    fn sectors(&self) -> Vec<PyLiveSector> {
        self.inner
            .sectors
            .iter()
            .cloned()
            .map(|inner| PyLiveSector { inner })
            .collect()
    }
    #[getter]
    fn orders(&self) -> Vec<i32> {
        self.inner.orders.clone()
    }
    #[getter]
    fn components(&self) -> Vec<&'static str> {
        components(&self.inner.components)
    }
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
    fn __repr__(&self) -> String {
        format!(
            "LiveObservation(source='{}', points={}, status={:?})",
            self.source(),
            self.inner.total.points,
            self.inner.total.status
        )
    }
}
