use std::{rc::Rc, time::Instant};

use fastsecdec::{
    kernel::KernelSet,
    status::{GenerationSnapshot, GenerationStage, GenerationTimings},
};
use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict},
};
use symbolica::api::python::PythonExpression;

use super::{
    error,
    inspection::PyEvaluatorStatistics,
    mc::{PyHavanaDiscreteSession, PyHavanaDiscreteSettings},
    session::{PyQmcSession, PyQmcSettings},
    status::PyGenerationSnapshot,
};

/// Compiled native evaluator owners. Each session owns one lazy execution context.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "Kernels",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyKernels {
    pub(crate) inner: Rc<KernelSet>,
    pub(crate) status: GenerationSnapshot,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyKernels {
    /// Bind an explicit complete physical point on independently owned native evaluators.
    #[pyo3(signature=(values, *, stability=None, contour=None))]
    fn with_parameters(
        &self,
        py: Python<'_>,
        values: &Bound<'_, PyDict>,
        stability: Option<&crate::settings::PyStabilitySettings>,
        contour: Option<&crate::contour::PyContourSettings>,
    ) -> PyResult<Self> {
        let mut point = std::collections::BTreeMap::new();
        for (key, value) in values.iter() {
            let key = key.extract::<PyRef<'_, PythonExpression>>()?;
            point.insert(
                crate::input::symbol(py, &key, "parameter key")?,
                value.extract::<f64>()?,
            );
        }
        let mut inner = self
            .inner
            .try_clone()
            .map_err(|e| error::native(py, "parameters", e))?;
        match contour {
            Some(contour) => inner.bind_parameters_with_contour(&point, &contour.inner),
            None => inner.bind_parameters(&point),
        }
        .map_err(|e| error::native(py, "parameters", e))?;
        if let Some(settings) = stability {
            inner
                .set_stability_settings(&settings.inner)
                .map_err(|e| error::native(py, "stability", e))?;
        }
        Ok(Self {
            inner: Rc::new(inner),
            status: self.status.clone(),
        })
    }
    #[pyo3(signature=(settings))]
    fn with_stability(
        &self,
        py: Python<'_>,
        settings: &crate::settings::PyStabilitySettings,
    ) -> PyResult<Self> {
        let mut inner = self
            .inner
            .try_clone()
            .map_err(|e| error::native(py, "stability", e))?;
        inner
            .set_stability_settings(&settings.inner)
            .map_err(|e| error::native(py, "stability", e))?;
        Ok(Self {
            inner: Rc::new(inner),
            status: self.status.clone(),
        })
    }
    #[getter]
    fn runtime_parameters(&self) -> Vec<PythonExpression> {
        self.inner
            .runtime_parameters()
            .iter()
            .filter(|symbol| !fastsecdec::contour::is_contour_parameter(**symbol))
            .map(|s| PythonExpression {
                expr: fastsecdec::Atom::var(*s),
            })
            .collect()
    }
    #[getter]
    fn parameters_bound(&self) -> bool {
        self.inner.parameters_bound()
    }
    #[getter]
    fn compilation_settings(&self) -> crate::settings::PyCompilationSettings {
        crate::settings::PyCompilationSettings {
            inner: *self.inner.compilation_settings(),
        }
    }
    #[getter]
    fn stability_settings(&self) -> crate::settings::PyStabilitySettings {
        crate::settings::PyStabilitySettings {
            inner: self.inner.stability_settings().clone(),
        }
    }
    /// Persist the library's native portable program/metadata codec, excluding machine code.
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .artifact_bytes()
            .map_err(|e| error::native(py, "artifact", e))?;
        Ok(PyBytes::new(py, bytes))
    }

    /// Validate native programs and construct evaluators for the current host backend.
    #[staticmethod]
    fn from_bytes(py: Python<'_>, artifact: &Bound<'_, PyBytes>) -> PyResult<Self> {
        py.check_signals()?;
        let started = Instant::now();
        let inner = KernelSet::from_bytes(artifact.as_bytes())
            .map_err(|e| error::native(py, "artifact", e))?;
        crate::citations::mark_generation();
        py.check_signals()?;
        let count = inner.sectors().len();
        let elapsed = started.elapsed().as_secs_f64();
        let status = GenerationSnapshot {
            stage: GenerationStage::Complete,
            completed: count,
            total: Some(count),
            sectors: count,
            kernels: count,
            elapsed_seconds: elapsed,
            timings: GenerationTimings {
                compilation_seconds: elapsed,
                total_seconds: elapsed,
                ..GenerationTimings::default()
            },
            coefficient_expansion: None,
            formula_preparation: None,
            detail: "Native artifact loaded; original generation timings unavailable".into(),
        };
        Ok(Self {
            inner: Rc::new(inner),
            status,
        })
    }

    #[getter]
    fn content_id(&self) -> &str {
        self.inner.content_id()
    }
    #[getter]
    fn orders(&self) -> Vec<i32> {
        self.inner.orders().to_vec()
    }
    #[getter]
    fn components(&self) -> Vec<&'static str> {
        self.inner
            .components()
            .iter()
            .map(|c| match c {
                fastsecdec::status::CoefficientComponent::Real => "real",
                fastsecdec::status::CoefficientComponent::Imag => "imag",
            })
            .collect()
    }
    #[getter]
    fn sector_count(&self) -> usize {
        self.inner.sectors().len()
    }
    /// Actual complete-vector evaluator records, aligned with native kernel sectors.
    #[getter]
    fn sector_statistics(&self) -> Vec<PyEvaluatorStatistics> {
        self.inner
            .sectors()
            .iter()
            .map(|sector| PyEvaluatorStatistics {
                inner: sector.statistics().clone(),
            })
            .collect()
    }
    #[getter]
    fn exact_coefficients(&self) -> Vec<f64> {
        self.inner.exact_coefficients().to_vec()
    }
    #[getter]
    fn backend(&self) -> &'static str {
        if self.inner.compilation_settings().backend.is_eager() {
            "symbolica_interpreter"
        } else {
            "symjit_o2"
        }
    }
    fn snapshot(&self) -> PyGenerationSnapshot {
        PyGenerationSnapshot {
            inner: self.status.clone(),
        }
    }

    #[pyo3(signature = (settings=None))]
    fn session(&self, py: Python<'_>, settings: Option<&PyQmcSettings>) -> PyResult<PyQmcSession> {
        PyQmcSession::new(
            py,
            self.inner.clone(),
            settings.map(|s| s.inner.clone()).unwrap_or_default(),
        )
    }

    /// Restore complete accepted packages and replay state against these exact kernels.
    fn restore(&self, py: Python<'_>, checkpoint: &Bound<'_, PyBytes>) -> PyResult<PyQmcSession> {
        PyQmcSession::restore_native(py, self.inner.clone(), checkpoint.as_bytes())
    }

    /// Native sector-importance Havana, with a caller-controlled pilot or frozen production.
    #[pyo3(signature = (settings=None, *, pilot=false, sector_probabilities=None))]
    fn mc_session(
        &self,
        py: Python<'_>,
        settings: Option<&PyHavanaDiscreteSettings>,
        pilot: bool,
        sector_probabilities: Option<Vec<f64>>,
    ) -> PyResult<PyHavanaDiscreteSession> {
        PyHavanaDiscreteSession::new(
            py,
            self.inner.clone(),
            settings.map(|s| s.inner.clone()).unwrap_or_default(),
            pilot,
            sector_probabilities.as_deref(),
        )
    }

    /// Restore only a native frozen-production discrete Havana checkpoint.
    fn restore_mc(
        &self,
        py: Python<'_>,
        checkpoint: &Bound<'_, PyBytes>,
    ) -> PyResult<PyHavanaDiscreteSession> {
        PyHavanaDiscreteSession::restore_native(py, self.inner.clone(), checkpoint.as_bytes())
    }
}
