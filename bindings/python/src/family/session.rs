use super::{archive::PyRecipeArchive, snapshot::PyRecipeFamilySnapshot};
use crate::{error, input::PyIntegral};
use fastsecdec::{
    generation::{GenerationOptions, RecipeFamily, RecipeFamilySession, RecipeFamilySnapshot},
    kernel::{CompilationSettings, indexed::ProgramRecipe},
    status::{GenerationSnapshot, GenerationStage},
};
use pyo3::prelude::*;
use std::{fs::File, ops::ControlFlow, path::PathBuf, rc::Rc, time::Instant};
use tempfile::TempDir;

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "RecipeFamilySession",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable
)]
pub(crate) struct PyRecipeFamilySession {
    input: Option<PyIntegral>,
    options: GenerationOptions,
    family: RecipeFamily,
    resident: Option<ProgramRecipe>,
    settings: CompilationSettings,
    native: Option<RecipeFamilySession<File>>,
    storage: Option<Rc<TempDir>>,
    path: Option<PathBuf>,
    result: Option<Py<PyRecipeArchive>>,
    status: RecipeFamilySnapshot,
    failed: Option<String>,
    elapsed: f64,
    parametrization_seconds: f64,
}

impl PyRecipeFamilySession {
    pub(super) fn new(
        input: PyIntegral,
        options: GenerationOptions,
        family: RecipeFamily,
        resident: Option<ProgramRecipe>,
        settings: CompilationSettings,
    ) -> Self {
        Self {
            status: RecipeFamilySnapshot {
                recipe: None,
                completed_recipes: 0,
                total_recipes: family.recipes().len(),
                completed_units: 0,
                prepared_sources: 0,
                persisted_units: 0,
                generation: GenerationSnapshot {
                    stage: GenerationStage::Input,
                    completed: 0,
                    total: None,
                    sectors: 0,
                    kernels: 0,
                    elapsed_seconds: 0.0,
                    timings: Default::default(),
                    coefficient_expansion: None,
                    formula_preparation: None,
                    detail: "Ready; call step to prepare the recipe family".into(),
                },
            },
            input: Some(input),
            options,
            family,
            resident,
            settings,
            native: None,
            storage: None,
            path: None,
            result: None,
            failed: None,
            elapsed: 0.0,
            parametrization_seconds: 0.0,
        }
    }

    fn initialize(&mut self, py: Python<'_>) -> PyResult<()> {
        crate::citations::mark_generation();
        self.status.generation.stage = GenerationStage::Parametrization;
        self.status.generation.detail =
            "Parametrizing the native integral once for all recipes".into();
        let started = Instant::now();
        let (integrand, runtime) = self
            .input
            .as_ref()
            .expect("pending input")
            .parametrize(py)?;
        self.parametrization_seconds = started.elapsed().as_secs_f64();
        let storage = Rc::new(
            tempfile::Builder::new()
                .prefix("fastsecdec-family-")
                .tempdir()
                .map_err(|e| error::native(py, "generation storage", e))?,
        );
        let path = storage.path().join("integral.fsd");
        let writer = File::create(&path).map_err(|e| error::native(py, "generation storage", e))?;
        let native = RecipeFamilySession::new(
            integrand,
            self.options.clone(),
            self.family.clone(),
            storage.path().join("staging"),
            writer,
        )
        .with_resident_recipe(self.resident)
        .and_then(|session| session.with_runtime_inputs(runtime.parameters, runtime.masses))
        .and_then(|session| session.with_evaluator(Default::default(), self.settings))
        .map_err(|e| error::native(py, "generation family", e))?;
        self.native = Some(native);
        self.storage = Some(storage);
        self.path = Some(path);
        self.input = None;
        Ok(())
    }

    fn advance(
        &mut self,
        py: Python<'_>,
        observer: Option<&Py<PyAny>>,
        started: Instant,
    ) -> PyResult<bool> {
        if self.input.is_some() {
            if let Err(error) = self.initialize(py) {
                self.failed = Some(error.to_string());
                return Err(error);
            }
            return Ok(false);
        }
        let mut callback_error = None;
        let mut paused = false;
        let native = self.native.as_mut().expect("active native family");
        let elapsed = self.elapsed;
        let parametrization_seconds = self.parametrization_seconds;
        let outcome = native.step(1, |status| {
            if paused || callback_error.is_some() {
                return ControlFlow::Break(());
            }
            let mut visible = status.clone();
            visible.generation.elapsed_seconds = elapsed + started.elapsed().as_secs_f64();
            visible.generation.timings.total_seconds = visible.generation.elapsed_seconds;
            visible.generation.timings.parametrization_seconds = parametrization_seconds;
            match observe(py, observer, &visible) {
                Ok(true) => ControlFlow::Continue(()),
                Ok(false) => {
                    paused = true;
                    ControlFlow::Break(())
                }
                Err(error) => {
                    callback_error = Some(error);
                    ControlFlow::Break(())
                }
            }
        });
        self.status = native.snapshot().clone();
        self.status.generation.elapsed_seconds = elapsed + started.elapsed().as_secs_f64();
        self.status.generation.timings.total_seconds = self.status.generation.elapsed_seconds;
        self.status.generation.timings.parametrization_seconds = parametrization_seconds;
        if let Err(error) = outcome {
            self.failed = Some(error.to_string());
            return Err(error::native(py, "generation family", error));
        }
        if let Some(output) = native.take_result() {
            let archive = PyRecipeArchive::generated(
                py,
                self.storage.as_ref().expect("active storage").clone(),
                self.path.as_ref().expect("active archive path").clone(),
                output,
                &self.status,
            )
            .and_then(|archive| Py::new(py, archive));
            match archive {
                Ok(archive) => {
                    self.result = Some(archive);
                    self.native = None;
                }
                Err(error) => {
                    self.failed = Some(error.to_string());
                    return Err(error);
                }
            }
        }
        // Pausing callbacks retain all completed native work, including a just
        // completed archive. They do not turn a successful unit into a failure.
        if let Some(error) = callback_error {
            return Err(error);
        }
        Ok(paused)
    }
}

fn observe(
    py: Python<'_>,
    callback: Option<&Py<PyAny>>,
    snapshot: &RecipeFamilySnapshot,
) -> PyResult<bool> {
    py.check_signals()?;
    if let Some(callback) = callback {
        let value = callback.call1(
            py,
            (PyRecipeFamilySnapshot {
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
impl PyRecipeFamilySession {
    #[getter]
    fn contour_jacobian(&self) -> &'static str {
        crate::settings::jacobian_name(self.options.contour_jacobian)
    }

    /// Execute at most max_units caller-owned units; parameterization is the first
    /// unit. False callbacks and KeyboardInterrupt pause without losing completed work.
    #[pyo3(signature=(max_units=1, *, observer=None))]
    fn step(
        &mut self,
        py: Python<'_>,
        max_units: usize,
        observer: Option<Py<PyAny>>,
    ) -> PyResult<PyRecipeFamilySnapshot> {
        if max_units == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "max_units must be positive",
            ));
        }
        if let Some(reason) = &self.failed {
            return Err(error::native(py, "generation family", reason));
        }
        if self.result.is_some() {
            return Ok(self.snapshot());
        }
        let started = Instant::now();
        let outcome: PyResult<()> = (|| {
            for _ in 0..max_units {
                if !observe(py, observer.as_ref(), &self.status)? {
                    break;
                }
                let paused = self.advance(py, observer.as_ref(), started)?;
                self.status.generation.elapsed_seconds =
                    self.elapsed + started.elapsed().as_secs_f64();
                self.status.generation.timings.total_seconds =
                    self.status.generation.elapsed_seconds;
                self.status.generation.timings.parametrization_seconds =
                    self.parametrization_seconds;
                if paused || !observe(py, observer.as_ref(), &self.status)? || self.result.is_some()
                {
                    break;
                }
            }
            Ok(())
        })();
        self.elapsed += started.elapsed().as_secs_f64();
        self.status.generation.elapsed_seconds = self.elapsed;
        self.status.generation.timings.total_seconds = self.elapsed;
        self.status.generation.timings.parametrization_seconds = self.parametrization_seconds;
        outcome?;
        Ok(self.snapshot())
    }
    #[getter]
    fn complete(&self) -> bool {
        self.result.is_some()
    }
    #[getter]
    fn failed(&self) -> Option<String> {
        self.failed.clone()
    }
    #[getter]
    fn recipes(&self) -> Vec<&'static str> {
        self.family
            .recipes()
            .iter()
            .map(|r| super::label(*r))
            .collect()
    }
    #[getter]
    fn default_recipe(&self) -> &'static str {
        super::label(self.family.default_recipe())
    }
    #[getter]
    fn resident_recipe(&self) -> Option<&'static str> {
        self.resident.map(super::label)
    }
    /// The same completed owner on every access; None while work is pending.
    #[getter]
    fn result(&self, py: Python<'_>) -> Option<Py<PyRecipeArchive>> {
        self.result.as_ref().map(|archive| archive.clone_ref(py))
    }
    fn snapshot(&self) -> PyRecipeFamilySnapshot {
        PyRecipeFamilySnapshot {
            inner: self.status.clone(),
        }
    }
    fn __repr__(&self) -> String {
        format!(
            "RecipeFamilySession(recipes={:?}, resident_recipe={:?}, complete={}, stage='{}')",
            self.recipes(),
            self.resident_recipe(),
            self.complete(),
            self.status.generation.stage.label()
        )
    }
}
