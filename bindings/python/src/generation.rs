use std::{ops::ControlFlow, rc::Rc, sync::Arc, time::Instant};

use fastsecdec::{
    generation::{
        CoefficientExpansionMethod, GeneratedIntegral, GenerationMode, GenerationOptions,
        SubtractionStrategy, generate,
    },
    parametric::ParametricIntegrand,
    status::{GenerationSnapshot, GenerationStage, GenerationTimings},
};
use pyo3::{prelude::*, types::PyString};
use symbolica::api::python::PythonExpression;

use super::{
    error,
    input::{PyIntegral, RuntimeInputs},
    kernels::PyKernels,
    progress::GenerationProgress,
    status::PyGenerationSnapshot,
};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "GeneratedIntegral",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyGeneratedIntegral {
    pub(crate) inner: Arc<GeneratedIntegral>,
    pub(crate) status: GenerationSnapshot,
    pub(crate) runtime: RuntimeInputs,
    pub(crate) mode: GenerationMode,
    pub(crate) subtraction: SubtractionStrategy,
}

#[pymethods]
impl PyIntegral {
    /// Synchronous native generation, retaining metadata without compilation.
    /// progress="auto" displays HEPKit progress in a running marimo notebook,
    /// unless observer is supplied. None disables the display; a callable gets
    /// every GenerationSnapshot. Observer runs first; None/True continues,
    /// False from either callback cancels at a native event boundary. Original
    /// callback and KeyboardInterrupt exceptions propagate after UI cleanup.
    #[pyo3(signature = (max_order=0, *, coefficient_expansion="full_expression", mode="symbolic", subtraction="taylor", contour=false, observer=None, progress=Some(Python::attach(|py| PyString::new(py, "auto").into_any().unbind()))))]
    #[pyo3(
        text_signature = "($self, max_order=0, *, coefficient_expansion='full_expression', mode='symbolic', subtraction='taylor', contour=False, observer=None, progress='auto')"
    )]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn generate(
        &self,
        py: Python<'_>,
        max_order: i32,
        coefficient_expansion: &str,
        mode: &str,
        subtraction: &str,
        contour: bool,
        observer: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
    ) -> PyResult<PyGeneratedIntegral> {
        generate_native(
            py,
            options(max_order, coefficient_expansion, mode, subtraction, contour)?,
            observer.as_ref(),
            progress.as_ref(),
            "Parametrizing the native diagram",
            || self.parametrize(py),
        )
    }
}

/// Shared validation for synchronous and retained generation entrypoints.
/// Enum deserialization belongs to the native generation options owner.
pub(crate) fn options(
    max_order: i32,
    coefficient_expansion: &str,
    mode: &str,
    subtraction: &str,
    contour: bool,
) -> PyResult<GenerationOptions> {
    let method = match coefficient_expansion {
        "full_expression" | "physical" => CoefficientExpansionMethod::Physical,
        "coefficient_series" | "native_named" => CoefficientExpansionMethod::NativeNamed,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "coefficient_expansion must be 'full_expression' or 'coefficient_series'",
            ));
        }
    };
    let mode = serde_json::from_value(serde_json::Value::String(mode.into())).map_err(|_| {
        pyo3::exceptions::PyValueError::new_err("mode must be 'symbolic' or 'numerical_dual'")
    })?;
    let subtraction = serde_json::from_value(serde_json::Value::String(subtraction.into()))
        .map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(
                "subtraction must be 'taylor' or 'integrate_by_parts'",
            )
        })?;
    let mut options = GenerationOptions {
        max_order,
        mode,
        subtraction,
        program_recipe: if contour {
            fastsecdec::kernel::ProgramRecipe::FixedV1
        } else {
            fastsecdec::kernel::ProgramRecipe::UndeformedV1
        },
        ..Default::default()
    };
    options.coefficient_expansion.method = method;
    Ok(options)
}

/// One observer, cancellation and native generation path for every input owner.
/// Parameterization is deferred until the initial observer permits it; returning
/// this result neither compiles evaluators nor creates an integration session.
pub(crate) fn generate_native(
    py: Python<'_>,
    options: GenerationOptions,
    observer: Option<&Py<PyAny>>,
    progress: Option<&Py<PyAny>>,
    detail: &str,
    parametrize: impl FnOnce() -> PyResult<(ParametricIntegrand, RuntimeInputs)>,
) -> PyResult<PyGeneratedIntegral> {
    let max_order = options.max_order;
    let mut progress = GenerationProgress::new(py, progress, observer, false)?;
    let result = (|| {
        let started = Instant::now();
        let mut status = GenerationSnapshot {
            stage: GenerationStage::Parametrization,
            completed: 0,
            total: None,
            sectors: 0,
            kernels: 0,
            elapsed_seconds: 0.0,
            timings: GenerationTimings::default(),
            coefficient_expansion: None,
            formula_preparation: None,
            detail: detail.into(),
        };
        if !progress.observe(py, &status)? {
            return Err(error::cancelled(py, "generation"));
        }
        crate::citations::mark_generation();
        let (input, runtime) = parametrize()?;
        status.timings.parametrization_seconds = started.elapsed().as_secs_f64();
        let mut callback_error = None;
        let mut cancelled = false;
        let result = generate(&input, &options, |event| {
            status.observe_generation(max_order, event);
            status.elapsed_seconds = started.elapsed().as_secs_f64();
            match progress.observe(py, &status) {
                Ok(true) => ControlFlow::Continue(()),
                Ok(false) => {
                    cancelled = true;
                    ControlFlow::Break(())
                }
                Err(e) => {
                    callback_error = Some(e);
                    ControlFlow::Break(())
                }
            }
        });
        if let Some(error) = callback_error {
            return Err(error);
        }
        if cancelled {
            return Err(error::cancelled(py, "generation"));
        }
        let inner = result.map_err(|e| error::native(py, "generation", e))?;
        Ok(PyGeneratedIntegral {
            inner: Arc::new(inner),
            status,
            runtime,
            mode: options.mode,
            subtraction: options.subtraction,
        })
    })();
    progress.finish(py, result)
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGeneratedIntegral {
    /// Whether the retained native chart recipes include contour deformation.
    #[getter]
    fn contour_capable(&self) -> bool {
        self.inner
            .metadata()
            .charts()
            .iter()
            .any(|chart| chart.contour().is_some())
    }
    /// Requested native generation lane, retained without recomputing sectors.
    #[getter]
    fn mode(&self) -> &'static str {
        self.mode.name()
    }
    #[getter]
    fn subtraction(&self) -> &'static str {
        self.subtraction.name()
    }
    #[getter]
    fn runtime_parameters(&self) -> Vec<PythonExpression> {
        self.runtime
            .parameters
            .iter()
            .map(|s| PythonExpression {
                expr: fastsecdec::Atom::var(*s),
            })
            .collect()
    }
    /// Model-card defaults are human metadata, never automatically bound.
    #[getter]
    fn model_parameter_defaults(&self) -> std::collections::BTreeMap<String, f64> {
        self.runtime.defaults.clone()
    }
    /// Explicitly inspectable defaults keyed by the actual native runtime symbols.
    #[getter]
    fn runtime_parameter_defaults<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let result = pyo3::types::PyDict::new(py);
        for symbol in &self.runtime.parameters {
            if let Some(value) = self.runtime.defaults.get(symbol.get_name()) {
                result.set_item(
                    PythonExpression {
                        expr: fastsecdec::Atom::var(*symbol),
                    },
                    *value,
                )?;
            }
        }
        Ok(result)
    }
    /// Immutable native sector views; coefficients remain compact until requested.
    #[getter]
    fn sectors(&self) -> Vec<crate::inspection::PyGeneratedSector> {
        crate::inspection::sectors(self.inner.clone())
    }

    /// Retained native domain and chart metadata, without evaluating the integral.
    #[getter]
    fn metadata(&self) -> crate::inspection::PyGenerationMetadata {
        crate::inspection::metadata(self.inner.clone())
    }

    #[getter]
    fn orders(&self) -> Vec<i32> {
        self.inner.orders().to_vec()
    }
    #[getter]
    fn sector_count(&self) -> usize {
        self.inner.sectors().len()
    }
    #[getter]
    fn exact_coefficients(&self) -> Vec<PythonExpression> {
        self.inner
            .exact_coefficients()
            .iter()
            .cloned()
            .map(|expr| PythonExpression { expr })
            .collect()
    }
    fn snapshot(&self) -> PyGenerationSnapshot {
        PyGenerationSnapshot {
            inner: self.status.clone(),
        }
    }
}

#[pymethods]
impl PyGeneratedIntegral {
    /// Build the native O2 or portable interpreted evaluator. Progress follows
    /// generate: auto marimo display (suppressed with observer), None, or a full
    /// snapshot callback. False cancels; original exceptions survive cleanup.
    #[pyo3(signature = (*, backend="eager", settings=None, observer=None, progress=Some(Python::attach(|py| PyString::new(py, "auto").into_any().unbind()))))]
    #[pyo3(
        text_signature = "($self, *, backend='eager', settings=None, observer=None, progress='auto')"
    )]
    fn compile(
        &self,
        py: Python<'_>,
        backend: &str,
        settings: Option<&crate::settings::PyCompilationSettings>,
        observer: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
    ) -> PyResult<PyKernels> {
        let settings =
            settings
                .map(|v| v.inner)
                .unwrap_or(fastsecdec::kernel::CompilationSettings {
                    backend: crate::settings::backend(backend)?,
                    ..Default::default()
                });
        let mut progress = GenerationProgress::new(py, progress.as_ref(), observer.as_ref(), true)?;
        let result = (|| {
            let started = Instant::now();
            let mut status = self.status.clone();
            let offset = status.elapsed_seconds;
            let mut callback_error = None;
            let mut cancelled = false;
            let result = self.inner.compile_with_settings_parameters_and_progress(
                Default::default(),
                &self.runtime.parameters,
                settings,
                |event| {
                    status.observe_compilation(event);
                    status.elapsed_seconds = offset + started.elapsed().as_secs_f64();
                    match progress.observe(py, &status) {
                        Ok(true) => ControlFlow::Continue(()),
                        Ok(false) => {
                            cancelled = true;
                            ControlFlow::Break(())
                        }
                        Err(e) => {
                            callback_error = Some(e);
                            ControlFlow::Break(())
                        }
                    }
                },
            );
            if let Some(error) = callback_error {
                return Err(error);
            }
            if cancelled {
                return Err(error::cancelled(py, "compilation"));
            }
            let inner = result
                .and_then(|k| k.with_runtime_mass_constraints(self.runtime.masses.clone()))
                .map_err(|e| error::native(py, "compilation", e))?;
            status.stage = GenerationStage::Complete;
            status.timings.total_seconds = offset + started.elapsed().as_secs_f64();
            status.detail = "Kernels ready".into();
            status.elapsed_seconds = offset + started.elapsed().as_secs_f64();
            if !progress.observe(py, &status)? {
                return Err(error::cancelled(py, "compilation"));
            }
            Ok(PyKernels {
                inner: Rc::new(inner),
                status,
            })
        })();
        progress.finish(py, result)
    }
}

// Separate supported declaration metadata avoids conditional helper attributes
// inside the runtime PyO3 methods. It changes no argument conversion or work.
#[cfg(feature = "python_stubgen")]
pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_methods_from_python! {
        r#"
import typing
import symbolica.community.hepkit.sector_decomposition

class PyIntegral:
    def generate(self, max_order: int = 0, *, coefficient_expansion: str = "full_expression",
                 mode: str = "symbolic", subtraction: str = "taylor", contour: bool = False,
                 observer: typing.Optional[typing.Callable[[symbolica.community.hepkit.sector_decomposition.GenerationSnapshot], typing.Optional[bool]]] = None,
                 progress: typing.Union[typing.Literal["auto"], typing.Callable[[symbolica.community.hepkit.sector_decomposition.GenerationSnapshot], typing.Optional[bool]], None] = "auto",
                 ) -> symbolica.community.hepkit.sector_decomposition.GeneratedIntegral:
        """Generate native sectors without compilation or numerical integration.

        Auto progress uses HEPKit's marimo presenter unless observer is supplied;
        None disables it. Callables receive all snapshots, observer first.
        None/True continues, False cancels; original exceptions propagate.
        """
"#
    }
}

#[cfg(feature = "python_stubgen")]
pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_methods_from_python! {
        r#"
import typing
import symbolica.community.hepkit.sector_decomposition

class PyGeneratedIntegral:
    def compile(self, *, backend: str = "eager",
                settings: typing.Optional[symbolica.community.hepkit.sector_decomposition.CompilationSettings] = None,
                observer: typing.Optional[typing.Callable[[symbolica.community.hepkit.sector_decomposition.GenerationSnapshot], typing.Optional[bool]]] = None,
                progress: typing.Union[typing.Literal["auto"], typing.Callable[[symbolica.community.hepkit.sector_decomposition.GenerationSnapshot], typing.Optional[bool]], None] = "auto",
                ) -> symbolica.community.hepkit.sector_decomposition.Kernels:
        """Compile native evaluators without creating or advancing a session.

        Auto progress uses HEPKit's marimo presenter unless observer is supplied;
        None disables it. Callables receive all snapshots, observer first.
        None/True continues, False cancels; original exceptions propagate.
        """
"#
    }
}
