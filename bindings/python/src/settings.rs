//! Typed transport of native compilation and stability policy.
use std::collections::BTreeMap;

use fastsecdec::kernel::{CompilationSettings, EvaluatorBackend, StabilityMode, StabilitySettings};
use pyo3::prelude::*;

use crate::error;

pub(crate) fn backend(value: &str) -> PyResult<EvaluatorBackend> {
    match value {
        "eager" => Ok(EvaluatorBackend::Eager),
        "auto" => Ok(EvaluatorBackend::Auto),
        "symjit" => Ok(EvaluatorBackend::Symjit),
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "backend must be eager, auto, or symjit",
        )),
    }
}

pub(crate) fn contour_jacobian(value: &str) -> PyResult<fastsecdec::contour::ContourJacobian> {
    serde_json::from_value(serde_json::Value::String(value.into())).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("contour_jacobian must be symbolic or dual")
    })
}

pub(crate) fn jacobian_name(value: fastsecdec::contour::ContourJacobian) -> &'static str {
    match value {
        fastsecdec::contour::ContourJacobian::Symbolic => "symbolic",
        fastsecdec::contour::ContourJacobian::Dual => "dual",
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "CompilationSettings",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyCompilationSettings {
    pub(crate) inner: CompilationSettings,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyCompilationSettings {
    /// Eager execution is the notebook default on every host. None means unlimited CPE rounds.
    #[new]
    #[pyo3(signature = (*, backend="eager", horner_iterations=10, cpe_rounds=Some(1000), cores=1, max_horner_scheme_variables=500, max_common_pair_cache_entries=1_000_000, max_common_pair_distance=1000, verbose=false, direct_translation=true, contour_jacobian="symbolic"))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        backend: &str,
        horner_iterations: usize,
        cpe_rounds: Option<usize>,
        cores: usize,
        max_horner_scheme_variables: usize,
        max_common_pair_cache_entries: usize,
        max_common_pair_distance: usize,
        verbose: bool,
        direct_translation: bool,
        contour_jacobian: &str,
    ) -> PyResult<Self> {
        let inner = CompilationSettings {
            backend: self::backend(backend)?,
            contour_jacobian: self::contour_jacobian(contour_jacobian)?,
            horner_iterations,
            cpe_rounds,
            cores,
            max_horner_scheme_variables,
            max_common_pair_cache_entries,
            max_common_pair_distance,
            verbose,
            direct_translation,
        };
        inner
            .validate()
            .map_err(|e| error::native(py, "compilation settings", e))?;
        Ok(Self { inner })
    }
    #[getter]
    fn backend(&self) -> &'static str {
        match self.inner.backend {
            EvaluatorBackend::Auto => "auto",
            EvaluatorBackend::Eager => "eager",
            EvaluatorBackend::Symjit => "symjit",
        }
    }
    #[getter]
    fn contour_jacobian(&self) -> &'static str {
        jacobian_name(self.inner.contour_jacobian)
    }
    #[getter]
    fn horner_iterations(&self) -> usize {
        self.inner.horner_iterations
    }
    #[getter]
    fn cpe_rounds(&self) -> Option<usize> {
        self.inner.cpe_rounds
    }
    #[getter]
    fn cores(&self) -> usize {
        self.inner.cores
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native scalar settings")
    }
    fn __repr__(&self) -> String {
        format!(
            "CompilationSettings(backend='{}', contour_jacobian='{}', horner_iterations={}, cpe_rounds={:?}, cores={})",
            self.backend(),
            self.contour_jacobian(),
            self.inner.horner_iterations,
            self.inner.cpe_rounds,
            self.inner.cores
        )
    }
}
impl Default for PyCompilationSettings {
    fn default() -> Self {
        Self {
            inner: CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
        }
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "StabilitySettings",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyStabilitySettings {
    pub(crate) inner: StabilitySettings,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyStabilitySettings {
    #[new]
    #[pyo3(signature = (*, mode="distance", f64_distance=1e-3, double_float_distance=1e-8, large_weight_threshold=Some(0.9), double_float_large_weight_threshold=None, unstable_cutoff=None, f64_power_thresholds=None, double_float_power_thresholds=None, unstable_power_thresholds=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        mode: &str,
        f64_distance: f64,
        double_float_distance: f64,
        large_weight_threshold: Option<f64>,
        double_float_large_weight_threshold: Option<f64>,
        unstable_cutoff: Option<f64>,
        f64_power_thresholds: Option<BTreeMap<String, f64>>,
        double_float_power_thresholds: Option<BTreeMap<String, f64>>,
        unstable_power_thresholds: Option<BTreeMap<String, f64>>,
    ) -> PyResult<Self> {
        let mode = match mode {
            "distance" => StabilityMode::Distance,
            "validated" => StabilityMode::Validated,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "mode must be distance or validated",
                ));
            }
        };
        let mut inner = StabilitySettings {
            mode,
            unstable_cutoff,
            unstable_power_thresholds: unstable_power_thresholds.unwrap_or_default(),
            ..Default::default()
        };
        inner.levels[0].minimum_effective_distance = f64_distance;
        inner.levels[0].escalate_for_large_weight_threshold = large_weight_threshold;
        inner.levels[0].power_thresholds = f64_power_thresholds.unwrap_or_default();
        inner.levels[1].minimum_effective_distance = double_float_distance;
        inner.levels[1].escalate_for_large_weight_threshold = double_float_large_weight_threshold;
        inner.levels[1].power_thresholds = double_float_power_thresholds.unwrap_or_default();
        inner
            .validate()
            .map_err(|e| error::native(py, "stability", e))?;
        Ok(Self { inner })
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, value: &str) -> PyResult<Self> {
        let inner: StabilitySettings =
            serde_json::from_str(value).map_err(|e| error::native(py, "stability", e))?;
        inner
            .validate()
            .map_err(|e| error::native(py, "stability", e))?;
        Ok(Self { inner })
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("validated native stability")
    }
    #[getter]
    fn mode(&self) -> &'static str {
        match self.inner.mode {
            StabilityMode::Distance => "distance",
            StabilityMode::Validated => "validated",
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "StabilitySettings(mode='{}', f64_distance={}, double_float_distance={}, unstable_cutoff={:?})",
            self.mode(),
            self.inner.levels[0].minimum_effective_distance,
            self.inner.levels[1].minimum_effective_distance,
            self.inner.unstable_cutoff
        )
    }
}
