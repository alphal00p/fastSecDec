//! Immutable views over optional native observations; no sampling or algebra.
use std::rc::Rc;

use fastsecdec::contour::{
    ContourDiagnosticRange, ContourDiagnosticsMode, ContourRuntimeReport, ContourRuntimeWork,
};
use pyo3::{prelude::*, types::PyModule};

use crate::{error, kernels::PyKernels};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourDiagnosticRange",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourDiagnosticRange {
    inner: ContourDiagnosticRange,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourDiagnosticRange {
    #[getter]
    fn count(&self) -> u64 {
        self.inner.count
    }
    #[getter]
    fn unavailable(&self) -> u64 {
        self.inner.unavailable
    }
    #[getter]
    fn minimum(&self) -> Option<f64> {
        self.inner.minimum
    }
    #[getter]
    fn maximum(&self) -> Option<f64> {
        self.inner.maximum
    }
    /// Approximate positive centres, never certified enclosures. Missing
    /// observations include values below display resolution, not exact zeros.
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native finite diagnostic range")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourRuntimeWork",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourRuntimeWork {
    inner: ContourRuntimeWork,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourRuntimeWork {
    #[getter]
    fn callback_calls(&self) -> u64 {
        self.inner.callback_calls
    }
    #[getter]
    fn callback_failures(&self) -> u64 {
        self.inner.callback_failures
    }
    #[getter]
    fn solver_calls(&self) -> u64 {
        self.inner.solver_calls
    }
    #[getter]
    fn solver_failures(&self) -> u64 {
        self.inner.solver_failures
    }
    #[getter]
    fn solver_iterations(&self) -> u64 {
        self.inner.solver_iterations
    }
    #[getter]
    fn maximum_solver_iterations(&self) -> u64 {
        self.inner.maximum_solver_iterations
    }
    #[getter]
    fn closed_form_calls(&self) -> u64 {
        self.inner.closed_form_calls
    }
    #[getter]
    fn maximum_bits(&self) -> u32 {
        self.inner.maximum_bits
    }
    #[getter]
    fn strength(&self) -> PyContourDiagnosticRange {
        PyContourDiagnosticRange {
            inner: self.inner.strength.clone(),
        }
    }
    #[getter]
    fn normalized_displacement(&self) -> PyContourDiagnosticRange {
        PyContourDiagnosticRange {
            inner: self.inner.normalized_displacement.clone(),
        }
    }
    #[getter]
    fn physical_displacement(&self) -> PyContourDiagnosticRange {
        PyContourDiagnosticRange {
            inner: self.inner.physical_displacement.clone(),
        }
    }
    /// Complete native operational record, including discarded attempts and
    /// precision retries. These counts are not accepted sampling statistics.
    /// Iterations before a failed root solve are unavailable.
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native finite runtime work")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourRuntimeReport",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourRuntimeReport {
    inner: ContourRuntimeReport,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourRuntimeReport {
    #[getter]
    fn evaluation(&self) -> PyContourRuntimeWork {
        PyContourRuntimeWork {
            inner: self.inner.evaluation.clone(),
        }
    }
    #[getter]
    fn conditioning(&self) -> PyContourRuntimeWork {
        PyContourRuntimeWork {
            inner: self.inner.conditioning.clone(),
        }
    }
    #[getter]
    fn preparation(&self) -> PyContourRuntimeWork {
        PyContourRuntimeWork {
            inner: self.inner.preparation.clone(),
        }
    }
    #[getter]
    fn exact(&self) -> PyContourRuntimeWork {
        PyContourRuntimeWork {
            inner: self.inner.exact.clone(),
        }
    }
    #[getter]
    fn pilot(&self) -> PyContourRuntimeWork {
        PyContourRuntimeWork {
            inner: self.inner.pilot.clone(),
        }
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native finite runtime report")
    }
}

/// Native integration phases, each retaining its operational work categories.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourRuntimeDiagnostics",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourRuntimeDiagnostics {
    pub(crate) inner: fastsecdec::status::ContourRuntimeDiagnostics,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourRuntimeDiagnostics {
    #[getter]
    fn adaptation(&self) -> PyContourRuntimeReport {
        PyContourRuntimeReport {
            inner: self.inner.adaptation.clone(),
        }
    }
    #[getter]
    fn production(&self) -> PyContourRuntimeReport {
        PyContourRuntimeReport {
            inner: self.inner.production.clone(),
        }
    }
    /// Actual work, including retries and discarded batches. These counters
    /// do not measure statistically accepted samples or certify causality.
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native finite integration diagnostics")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyKernels {
    #[getter]
    fn contour_diagnostics_mode(&self) -> &'static str {
        match self.inner.contour_diagnostics_mode() {
            ContourDiagnosticsMode::Disabled => "disabled",
            ContourDiagnosticsMode::Aggregate => "aggregate",
        }
    }
    /// Copy the native owner with optional observations. This does not change
    /// contour mathematics, checking policy, pilot evidence or sampling streams.
    /// Enabling observations may restore evaluators from their saved programs.
    #[pyo3(signature=(mode="aggregate"))]
    fn with_contour_diagnostics(&self, py: Python<'_>, mode: &str) -> PyResult<Self> {
        let mode: ContourDiagnosticsMode =
            serde_json::from_value(serde_json::Value::String(mode.into())).map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "contour diagnostics must be 'disabled' or 'aggregate'",
                )
            })?;
        let mut inner = self
            .inner
            .try_clone()
            .map_err(|e| error::native(py, "contour diagnostics", e))?;
        inner
            .set_contour_diagnostics(mode)
            .map_err(|e| error::native(py, "contour diagnostics", e))?;
        Ok(Self {
            inner: Rc::new(inner),
            status: self.status.clone(),
        })
    }
    /// Snapshot this owner's actual work. Independent sessions have their own
    /// counters. None means observations were not recorded, not zero work.
    fn contour_runtime_report(&self, py: Python<'_>) -> PyResult<Option<PyContourRuntimeReport>> {
        self.inner
            .contour_runtime_report()
            .map(|report| report.map(|inner| PyContourRuntimeReport { inner }))
            .map_err(|e| error::native(py, "contour diagnostics", e))
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyContourDiagnosticRange>()?;
    module.add_class::<PyContourRuntimeWork>()?;
    module.add_class::<PyContourRuntimeReport>()?;
    module.add_class::<PyContourRuntimeDiagnostics>()
}
