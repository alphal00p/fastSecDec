//! One retained native owner from parameterization through eager compilation.
use crate::{
    error,
    generation::PyGeneratedIntegral,
    input::{PyIntegral, RuntimeInputs},
    kernels::PyKernels,
    settings::PyCompilationSettings,
    status::PyGenerationSnapshot,
};
use fastsecdec::{
    generation::{GenerationOptions, GenerationSession},
    kernel::CompilationSession,
    status::{GenerationSnapshot, GenerationStage, GenerationTimings},
};
use pyo3::prelude::*;
use std::{ops::ControlFlow, rc::Rc, sync::Arc, time::Instant};
use symbolica::api::python::PythonExpression;

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "GenerationSession",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable
)]
pub(crate) struct PyGenerationSession {
    input: Option<PyIntegral>,
    options: GenerationOptions,
    settings: fastsecdec::kernel::CompilationSettings,
    generation: Option<GenerationSession>,
    compilation: Option<CompilationSession>,
    generated: Option<PyGeneratedIntegral>,
    kernels: Option<PyKernels>,
    runtime: RuntimeInputs,
    status: GenerationSnapshot,
    failed: Option<String>,
    elapsed: f64,
    parametrization_seconds: f64,
}

#[pymethods]
impl PyIntegral {
    /// Create inert retained work. Only step() performs parameterization, generation or compilation.
    #[pyo3(signature=(max_order=0, *, coefficient_expansion="coefficient_series", mode="symbolic", subtraction="taylor", contour=false, compilation_settings=None, runtime_parameters=None))]
    #[allow(clippy::too_many_arguments)]
    fn generation_session(
        &self,
        py: Python<'_>,
        max_order: i32,
        coefficient_expansion: &str,
        mode: &str,
        subtraction: &str,
        contour: bool,
        compilation_settings: Option<&PyCompilationSettings>,
        runtime_parameters: Option<Vec<PythonExpression>>,
    ) -> PyResult<PyGenerationSession> {
        let options = crate::generation::options(
            max_order,
            coefficient_expansion,
            mode,
            subtraction,
            contour,
        )?;
        let settings = compilation_settings.cloned().unwrap_or_default().inner;
        settings
            .validate()
            .map_err(|e| error::native(py, "compilation settings", e))?;
        let mut input = self.clone();
        if let Some(parameters) = runtime_parameters {
            input.runtime_parameters = parameters
                .iter()
                .map(|p| crate::input::symbol(py, p, "runtime parameter"))
                .collect::<PyResult<_>>()?;
        }
        Ok(PyGenerationSession {
            input: Some(input),
            options,
            settings,
            generation: None,
            compilation: None,
            generated: None,
            kernels: None,
            runtime: Default::default(),
            failed: None,
            elapsed: 0.0,
            parametrization_seconds: 0.0,
            status: GenerationSnapshot {
                stage: GenerationStage::Input,
                completed: 0,
                total: None,
                sectors: 0,
                kernels: 0,
                elapsed_seconds: 0.0,
                timings: GenerationTimings::default(),
                coefficient_expansion: None,
                formula_preparation: None,
                detail: "Ready; call step to generate".into(),
            },
        })
    }
}

fn observe(
    py: Python<'_>,
    callback: Option<&Py<PyAny>>,
    snapshot: &GenerationSnapshot,
) -> PyResult<bool> {
    py.check_signals()?;
    if let Some(callback) = callback {
        let value = callback.call1(
            py,
            (PyGenerationSnapshot {
                inner: snapshot.clone(),
            },),
        )?;
        if !value.is_none(py) {
            return value.extract::<bool>(py);
        }
    }
    Ok(true)
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGenerationSession {
    #[getter]
    fn mode(&self) -> &'static str {
        self.options.mode.name()
    }
    #[getter]
    fn subtraction(&self) -> &'static str {
        self.options.subtraction.name()
    }
    #[getter]
    fn contour(&self) -> bool {
        self.options.contour
    }

    /// Run at most max_units indivisible native units on this caller's thread.
    /// False observers and KeyboardInterrupt pause; already completed work is retained.
    /// Numerical/algebra errors are terminal and expose no partial kernels.
    #[pyo3(signature=(max_units=1, *, observer=None))]
    fn step(
        &mut self,
        py: Python<'_>,
        max_units: usize,
        observer: Option<Py<PyAny>>,
    ) -> PyResult<PyGenerationSnapshot> {
        if max_units == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "max_units must be positive",
            ));
        }
        if let Some(reason) = &self.failed {
            return Err(error::native(py, "generation", reason));
        }
        if self.kernels.is_some() {
            return Ok(self.snapshot());
        }
        let started = Instant::now();
        let mut paused = false;
        let result = (|| {
            for _ in 0..max_units {
                if !observe(py, observer.as_ref(), &self.status)? {
                    break;
                }
                if let Some(input) = &self.input {
                    crate::citations::mark_generation();
                    self.status.stage = GenerationStage::Parametrization;
                    self.status.detail = "Parametrizing the native diagram".into();
                    let timer = Instant::now();
                    let (input, runtime) = match input.parametrize(py) {
                        Ok(value) => value,
                        Err(e) => {
                            self.failed = Some(e.to_string());
                            return Err(e);
                        }
                    };
                    self.parametrization_seconds = timer.elapsed().as_secs_f64();
                    self.runtime = runtime;
                    self.generation = Some(GenerationSession::new(input, self.options.clone()));
                    self.input = None;
                } else if let Some(generation) = &mut self.generation {
                    let mut callback_error = None;
                    let status = &mut self.status;
                    let max_order = self.options.max_order;
                    let result = generation.step(1, |event| {
                        status.observe_generation(max_order, event);
                        if callback_error.is_some() || paused {
                            return ControlFlow::Break(());
                        }
                        match observe(py, observer.as_ref(), status) {
                            Ok(true) => ControlFlow::Continue(()),
                            Ok(false) => {
                                paused = true;
                                ControlFlow::Break(())
                            }
                            Err(e) => {
                                callback_error = Some(e);
                                ControlFlow::Break(())
                            }
                        }
                    });
                    self.status = generation.snapshot().clone();
                    if let Err(e) = result {
                        self.failed = Some(e.to_string());
                        return Err(error::native(py, "generation", e));
                    }
                    if let Some(generated) = generation.take_result() {
                        let generated = Arc::new(generated);
                        self.compilation = Some(
                            CompilationSession::new(
                                generated.clone(),
                                self.runtime.parameters.clone(),
                                Default::default(),
                                self.settings,
                            )
                            .map_err(|e| {
                                self.failed = Some(e.to_string());
                                error::native(py, "compilation", e)
                            })?,
                        );
                        self.generated = Some(PyGeneratedIntegral {
                            inner: generated,
                            status: self.status.clone(),
                            runtime: self.runtime.clone(),
                            mode: self.options.mode,
                            subtraction: self.options.subtraction,
                        });
                        self.generation = None;
                    }
                    if let Some(e) = callback_error {
                        return Err(e);
                    }
                } else if let Some(compilation) = &mut self.compilation {
                    let mut callback_error = None;
                    let status = &mut self.status;
                    let result = compilation.step(1, |event| {
                        status.observe_compilation(event);
                        if callback_error.is_some() || paused {
                            return ControlFlow::Break(());
                        }
                        match observe(py, observer.as_ref(), status) {
                            Ok(true) => ControlFlow::Continue(()),
                            Ok(false) => {
                                paused = true;
                                ControlFlow::Break(())
                            }
                            Err(e) => {
                                callback_error = Some(e);
                                ControlFlow::Break(())
                            }
                        }
                    });
                    self.status.observe_compilation(&compilation.snapshot());
                    if let Err(e) = result {
                        self.failed = Some(e.to_string());
                        return Err(error::native(py, "compilation", e));
                    }
                    if let Some(kernels) = compilation.take_result() {
                        let kernels = kernels
                            .with_runtime_mass_constraints(self.runtime.masses.clone())
                            .map_err(|e| {
                                self.failed = Some(e.to_string());
                                error::native(py, "compilation", e)
                            })?;
                        self.status.stage = GenerationStage::Complete;
                        self.status.detail =
                            "Kernels ready; supply any declared runtime inputs before integration"
                                .into();
                        self.kernels = Some(PyKernels {
                            inner: Rc::new(kernels),
                            status: self.status.clone(),
                        });
                        self.compilation = None;
                    }
                    if let Some(e) = callback_error {
                        return Err(e);
                    }
                }
                self.status.elapsed_seconds = self.elapsed + started.elapsed().as_secs_f64();
                if paused
                    || !observe(py, observer.as_ref(), &self.status)?
                    || self.kernels.is_some()
                {
                    break;
                }
            }
            Ok(())
        })();
        self.elapsed += started.elapsed().as_secs_f64();
        self.status.elapsed_seconds = self.elapsed;
        self.status.timings.parametrization_seconds = self.parametrization_seconds;
        self.status.timings.total_seconds = self.elapsed;
        if let Some(kernels) = &mut self.kernels {
            kernels.status = self.status.clone();
        }
        result?;
        Ok(self.snapshot())
    }
    #[getter]
    fn complete(&self) -> bool {
        self.kernels.is_some()
    }
    #[getter]
    fn failed(&self) -> Option<String> {
        self.failed.clone()
    }
    #[getter]
    fn generated(&self) -> Option<PyGeneratedIntegral> {
        self.generated.clone()
    }
    #[getter]
    fn kernels(&self) -> Option<PyKernels> {
        self.kernels.clone()
    }
    fn snapshot(&self) -> PyGenerationSnapshot {
        PyGenerationSnapshot {
            inner: self.status.clone(),
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "GenerationSession(stage='{}', complete={}, sectors={}, kernels={})",
            self.status.stage.label(),
            self.complete(),
            self.status.sectors,
            self.status.kernels
        )
    }
}

#[cfg(feature = "python_stubgen")]
pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_methods_from_python! {
        r#"
import typing
import symbolica
import symbolica.community.hepkit.sector_decomposition

class PyIntegral:
    def generation_session(self, max_order: int = 0, *, coefficient_expansion: str = "coefficient_series",
        mode: str = "symbolic", subtraction: str = "taylor", contour: bool = False,
        compilation_settings: typing.Optional[symbolica.community.hepkit.sector_decomposition.CompilationSettings] = None,
        runtime_parameters: typing.Optional[list[symbolica.Expression]] = None,
    ) -> symbolica.community.hepkit.sector_decomposition.GenerationSession:
        """Create retained caller-stepped generation and compilation; construction performs no symbolic work."""
"#
    }
}
