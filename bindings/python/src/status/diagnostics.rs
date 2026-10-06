use fastsecdec::status::EvaluationDiagnostics;
use pyo3::prelude::*;

/// Native caller-aggregated evaluation counters, including failed attempts.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    from_py_object,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "EvaluationDiagnostics"
)]
#[derive(Clone)]
pub(crate) struct PyEvaluationDiagnostics {
    pub(super) inner: EvaluationDiagnostics,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyEvaluationDiagnostics {
    #[getter]
    fn evaluations(&self) -> u64 {
        self.inner.evaluations
    }

    #[getter]
    fn conditioning_checks(&self) -> u64 {
        self.inner.conditioning_checks
    }

    #[getter]
    fn rescues(&self) -> u64 {
        self.inner.rescues
    }

    #[getter]
    fn max_precision_bits(&self) -> u32 {
        self.inner.max_precision_bits
    }

    #[getter]
    fn failures(&self) -> u64 {
        self.inner.failures
    }

    #[getter]
    fn weighted_checks(&self) -> u64 {
        self.inner.weighted_checks
    }

    #[getter]
    fn additional_replays(&self) -> u64 {
        self.inner.additional_replays
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}
