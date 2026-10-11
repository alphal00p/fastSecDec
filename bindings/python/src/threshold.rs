//! Native-only caller-owned threshold preparation and incremental compilation.
//! No graph conversion, mathematical admission, estimator or worker pool here.
mod session;
mod work;
use crate::{error, input::PyIntegral, settings::PyCompilationSettings};
use fastsecdec::threshold::{gcad, generation::PreparationOptions};
use pyo3::prelude::*;

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ThresholdSettings",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyThresholdSettings {
    limits: gcad::Limits,
    solver: gcad::SolverOptions,
    transport_bytes: u64,
}
impl Default for PyThresholdSettings {
    fn default() -> Self {
        Self {
            limits: gcad::GcadRequest::default_limits(),
            solver: Default::default(),
            transport_bytes: 1 << 30,
        }
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyThresholdSettings {
    /// Native GCAD JSON schemas, without a second settings implementation.
    /// limits.workers must remain one; schedule processes in the caller.
    /// Byte limits cover transport, not native decoded/compiler RSS.
    #[new]
    #[pyo3(signature=(*, gcad_limits_json=None, solver_options_json=None, maximum_transport_bytes=1_073_741_824))]
    fn new(
        py: Python<'_>,
        gcad_limits_json: Option<&str>,
        solver_options_json: Option<&str>,
        maximum_transport_bytes: u64,
    ) -> PyResult<Self> {
        if maximum_transport_bytes == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "maximum_transport_bytes must be positive",
            ));
        }
        let mut defaults = serde_json::to_value(gcad::GcadRequest::default_limits())
            .map_err(|e| error::native(py, "threshold settings", e))?;
        if let Some(json) = gcad_limits_json {
            let overrides: serde_json::Value = serde_json::from_str(json)
                .map_err(|e| error::native(py, "threshold settings", e))?;
            let values = overrides.as_object().ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err("gcad_limits_json must be an object")
            })?;
            defaults
                .as_object_mut()
                .expect("native limits object")
                .extend(values.clone());
        }
        let limits: gcad::Limits = serde_json::from_value(defaults)
            .map_err(|e| error::native(py, "threshold settings", e))?;
        if limits.workers != 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "native GCAD requires workers=1; schedule processes in the caller",
            ));
        }
        Ok(Self {
            limits,
            solver: solver_options_json
                .map(serde_json::from_str)
                .transpose()
                .map_err(|e| error::native(py, "threshold settings", e))?
                .unwrap_or_default(),
            transport_bytes: maximum_transport_bytes,
        })
    }
    fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&serde_json::json!({"gcad_limits":self.limits,"solver_options":self.solver,"maximum_transport_bytes":self.transport_bytes}))
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}
#[pymethods]
impl PyIntegral {
    /// Inert native work. prepare() performs parametrization/verified preparation;
    /// compile_next() performs compilation. Neither action starts integration.
    /// Initially supports fixed inputs with one admitted rational integration axis.
    #[pyo3(signature=(max_order=0, *, coefficient_expansion="coefficient_series", subtraction="integrate_by_parts", compilation_settings=None, threshold_settings=None))]
    pub(crate) fn threshold_generation_session(
        &self,
        py: Python<'_>,
        max_order: i32,
        coefficient_expansion: &str,
        subtraction: &str,
        compilation_settings: Option<&PyCompilationSettings>,
        threshold_settings: Option<&PyThresholdSettings>,
    ) -> PyResult<session::PyThresholdGenerationSession> {
        let mut options =
            PreparationOptions::new(symbolica::symbol!("fastsecdec::hepkit::threshold_unit"));
        options.generation = crate::generation::options(
            max_order,
            coefficient_expansion,
            "symbolic",
            subtraction,
            false,
            "symbolic",
        )?;
        options.compilation = compilation_settings.cloned().unwrap_or_default().inner;
        options
            .compilation
            .validate()
            .map_err(|e| error::native(py, "threshold compilation", e))?;
        if options.compilation.contour_jacobian != fastsecdec::contour::ContourJacobian::Symbolic {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "threshold generation does not use a contour Jacobian",
            ));
        }
        let settings = threshold_settings.cloned().unwrap_or_default();
        options.threshold.gcad_limits = settings.limits;
        options.threshold.solver = settings.solver;
        Ok(session::PyThresholdGenerationSession::new(
            self.clone(),
            options,
            settings.transport_bytes,
        ))
    }
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyThresholdSettings>()?;
    module.add_class::<session::PyThresholdGenerationSession>()?;
    Ok(())
}

#[cfg(feature = "python_stubgen")]
pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_methods_from_python! {
        r#"
import typing
import symbolica.community.hepkit.sector_decomposition

class PyIntegral:
    def threshold_generation_session(self, max_order: int = 0, *,
        coefficient_expansion: str = "coefficient_series",
        subtraction: str = "integrate_by_parts",
        compilation_settings: typing.Optional[symbolica.community.hepkit.sector_decomposition.CompilationSettings] = None,
        threshold_settings: typing.Optional[symbolica.community.hepkit.sector_decomposition.ThresholdSettings] = None,
    ) -> symbolica.community.hepkit.sector_decomposition.ThresholdGenerationSession:
        """Create inert native threshold work; preparation and compilation are explicit."""
"#
    }
}
