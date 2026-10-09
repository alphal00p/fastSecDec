//! Thin settings and caller-owned validation views over the native contour API.
use std::rc::Rc;

use fastsecdec::{
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    kernel::{ContourCheckReport, ContourValidationChart, ContourValidationReport, KernelSet},
};
use pyo3::{prelude::*, types::PyModule};

use crate::{error, kernels::PyKernels};
mod provenance;
pub(crate) use provenance::{PyContourCheckpointProvenance, checkpoint_provenance};

fn policy(value: &str) -> PyResult<ContourValidation> {
    serde_json::from_value(serde_json::Value::String(value.into())).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("validation must be 'always', 'pilot', or 'off'")
    })
}
fn policy_name(value: ContourValidation) -> &'static str {
    match value {
        ContourValidation::Always => "always",
        ContourValidation::Pilot => "pilot",
        ContourValidation::Off => "off",
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourSettings",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourSettings {
    pub(crate) inner: ContourSettings,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourSettings {
    /// Mathematical strength and optional checking policy are separate native settings.
    #[new]
    #[pyo3(signature=(*, mode="off", lambda_value=None, validation="always", pilot_points=256))]
    fn new(
        py: Python<'_>,
        mode: &str,
        lambda_value: Option<f64>,
        validation: &str,
        pilot_points: usize,
    ) -> PyResult<Self> {
        let deformation = match (mode, lambda_value) {
            ("off", None) => ContourMode::Off,
            ("fixed", Some(lambda)) => ContourMode::Fixed { lambda },
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "use mode='off' without a strength, or mode='fixed' with lambda_value",
                ));
            }
        };
        let inner = ContourSettings {
            deformation,
            validation: ContourValidationOptions {
                policy: policy(validation)?,
                pilot_points,
            },
        };
        inner
            .validate()
            .map_err(|e| error::native(py, "contour settings", e))?;
        Ok(Self { inner })
    }
    #[staticmethod]
    #[pyo3(signature=(lambda_value, *, validation="always", pilot_points=256))]
    fn fixed(
        py: Python<'_>,
        lambda_value: f64,
        validation: &str,
        pilot_points: usize,
    ) -> PyResult<Self> {
        Self::new(py, "fixed", Some(lambda_value), validation, pilot_points)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, value: &str) -> PyResult<Self> {
        let inner: ContourSettings =
            serde_json::from_str(value).map_err(|e| error::native(py, "contour settings", e))?;
        inner
            .validate()
            .map_err(|e| error::native(py, "contour settings", e))?;
        Ok(Self { inner })
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("validated native contour settings")
    }
    #[getter]
    fn mode(&self) -> &'static str {
        match self.inner.deformation {
            ContourMode::Off => "off",
            ContourMode::Fixed { .. } => "fixed",
        }
    }
    #[getter]
    fn lambda_value(&self) -> Option<f64> {
        match self.inner.deformation {
            ContourMode::Off => None,
            ContourMode::Fixed { lambda } => Some(lambda),
        }
    }
    #[getter]
    fn validation(&self) -> &'static str {
        policy_name(self.inner.validation.policy)
    }
    #[getter]
    fn pilot_points(&self) -> usize {
        self.inner.validation.pilot_points
    }
    fn __repr__(&self) -> String {
        format!(
            "ContourSettings(mode='{}', lambda_value={:?}, validation='{}', pilot_points={})",
            self.mode(),
            self.lambda_value(),
            self.validation(),
            self.pilot_points()
        )
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourValidationChart",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourValidationChart {
    pub(crate) inner: ContourValidationChart,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourValidationChart {
    #[getter]
    fn chart_index(&self) -> usize {
        self.inner.chart_index
    }
    #[getter]
    fn kernel_sector(&self) -> Option<usize> {
        self.inner.kernel_sector
    }
    #[getter]
    fn dimension(&self) -> usize {
        self.inner.dimension
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourCheckReport",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourCheckReport {
    pub(crate) inner: ContourCheckReport,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourCheckReport {
    #[getter]
    fn chart_index(&self) -> usize {
        self.inner.chart_index
    }
    #[getter]
    fn checked_arguments(&self) -> usize {
        self.inner.checked_arguments
    }
    #[getter]
    fn maximum_bits(&self) -> u32 {
        self.inner.maximum_bits
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native contour check report")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourValidationReport",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourValidationReport {
    pub(crate) inner: ContourValidationReport,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourValidationReport {
    #[getter]
    fn policy(&self) -> &'static str {
        policy_name(self.inner.policy)
    }
    #[getter]
    fn pilot_complete(&self) -> bool {
        self.inner.pilot_complete
    }
    #[getter]
    fn required_charts(&self) -> Vec<usize> {
        self.inner.required_charts.clone()
    }
    #[getter]
    fn validated_charts(&self) -> Vec<usize> {
        self.inner.validated_charts.clone()
    }
    #[getter]
    fn accepted_pilot_points(&self) -> usize {
        self.inner.accepted_pilot_points
    }
    #[getter]
    fn checked_arguments(&self) -> usize {
        self.inner.checked_arguments
    }
    #[getter]
    fn maximum_bits(&self) -> u32 {
        self.inner.maximum_bits
    }
    #[getter]
    fn production_checked_arguments(&self) -> usize {
        self.inner.production_checked_arguments
    }
    #[getter]
    fn production_maximum_bits(&self) -> u32 {
        self.inner.production_maximum_bits
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native contour validation report")
    }
}

impl PyKernels {
    /// Preserve existing sessions' owners when a caller starts independent pilot work.
    fn contour_owner(&mut self, py: Python<'_>) -> PyResult<&mut KernelSet> {
        if Rc::strong_count(&self.inner) != 1 {
            self.inner = Rc::new(
                self.inner
                    .try_clone()
                    .map_err(|e| error::native(py, "contour validation", e))?,
            );
        }
        Rc::get_mut(&mut self.inner).ok_or_else(|| {
            error::native(py, "contour validation", "native evaluator owner is shared")
        })
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyKernels {
    #[getter]
    fn contour_settings(&self) -> Option<PyContourSettings> {
        self.inner
            .contour_settings()
            .cloned()
            .map(|inner| PyContourSettings { inner })
    }
    #[getter]
    fn contour_capable(&self) -> bool {
        self.inner.contour_capable()
    }
    #[getter]
    fn contour_validation_charts(&self) -> Vec<PyContourValidationChart> {
        self.inner
            .contour_validation_charts()
            .into_iter()
            .map(|inner| PyContourValidationChart { inner })
            .collect()
    }
    /// Validate one caller-supplied pilot point and its retained subtraction faces.
    /// Finite fixed-strength pilots are diagnostics, not a global causal certificate.
    /// Pilot points never enter the production integrator or consume its RNG.
    /// Only homotopy=True calls count toward completing the preflight pilot.
    #[pyo3(signature=(chart_index, coordinates, *, homotopy=true))]
    fn validate_contour_point(
        &mut self,
        py: Python<'_>,
        chart_index: usize,
        coordinates: Vec<f64>,
        homotopy: bool,
    ) -> PyResult<PyContourCheckReport> {
        py.check_signals()?;
        let inner = self
            .contour_owner(py)?
            .validate_contour_point(chart_index, &coordinates, homotopy)
            .map_err(|e| error::native(py, "contour validation", e))?;
        py.check_signals()?;
        Ok(PyContourCheckReport { inner })
    }
    /// Finish only the requested chart scope; excluded kernels remain locked.
    #[pyo3(signature=(chart_indices=None))]
    fn finish_contour_pilot(
        &mut self,
        py: Python<'_>,
        chart_indices: Option<Vec<usize>>,
    ) -> PyResult<PyContourValidationReport> {
        let owner = self.contour_owner(py)?;
        let inner = match chart_indices {
            Some(charts) => owner.finish_contour_pilot_for_charts(&charts),
            None => owner.finish_contour_pilot(),
        }
        .map_err(|e| error::native(py, "contour validation", e))?;
        Ok(PyContourValidationReport { inner })
    }
    /// Snapshot this owner's evidence; independently running sessions own their counters.
    fn contour_validation_report(&self) -> Option<PyContourValidationReport> {
        self.inner
            .contour_validation_report()
            .map(|inner| PyContourValidationReport { inner })
    }
    /// Copy checking policy without rebinding the physical point or changing numerical identity.
    #[pyo3(signature=(validation, *, pilot_points=256))]
    fn with_contour_validation(
        &self,
        py: Python<'_>,
        validation: &str,
        pilot_points: usize,
    ) -> PyResult<Self> {
        let mut inner = self
            .inner
            .try_clone()
            .map_err(|e| error::native(py, "contour validation", e))?;
        inner
            .set_contour_validation(ContourValidationOptions {
                policy: policy(validation)?,
                pilot_points,
            })
            .map_err(|e| error::native(py, "contour validation", e))?;
        Ok(Self {
            inner: Rc::new(inner),
            status: self.status.clone(),
        })
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyContourSettings>()?;
    module.add_class::<PyContourValidationChart>()?;
    module.add_class::<PyContourCheckReport>()?;
    module.add_class::<PyContourValidationReport>()?;
    module.add_class::<PyContourEvaluationDiagnostics>()?;
    module.add_class::<PyContourCheckCounters>()?;
    provenance::register(module)?;
    Ok(())
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourCheckCounters",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourCheckCounters {
    pub(crate) inner: fastsecdec::status::ContourCheckCounters,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourCheckCounters {
    #[getter]
    fn checked_arguments(&self) -> u64 {
        self.inner.checked_arguments
    }
    #[getter]
    fn maximum_bits(&self) -> u32 {
        self.inner.maximum_bits
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourEvaluationDiagnostics",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourEvaluationDiagnostics {
    pub(crate) inner: fastsecdec::status::ContourEvaluationDiagnostics,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourEvaluationDiagnostics {
    #[getter]
    fn production(&self) -> PyContourCheckCounters {
        PyContourCheckCounters {
            inner: self.inner.production,
        }
    }
    #[getter]
    fn adaptation(&self) -> PyContourCheckCounters {
        PyContourCheckCounters {
            inner: self.inner.adaptation,
        }
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native contour evaluation diagnostics")
    }
}
