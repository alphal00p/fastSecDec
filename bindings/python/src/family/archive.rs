//! Native archive ownership, selective loading and explicit export.
use crate::{error, kernels::PyKernels};
use fastsecdec::{
    generation::{RecipeFamilyOutput, RecipeFamilySnapshot},
    kernel::{
        KernelLoadOptions,
        indexed::{ProgramArchiveCatalogue, ProgramArchiveReader, ProgramRecipe},
    },
    status::{GenerationSnapshot, GenerationStage},
};
use pyo3::{prelude::*, types::PyBytes};
use std::{
    fs::File,
    io::{self, Cursor, Write},
    ops::ControlFlow,
    path::{Path, PathBuf},
    rc::Rc,
};
use tempfile::TempDir;

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "RecipeArchive",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable
)]
pub(crate) struct PyRecipeArchive {
    // Retain the storage independently of the generation session. Selected native
    // kernels own decoded programs/metadata and do not borrow this directory.
    _storage: Rc<TempDir>,
    path: PathBuf,
    catalogue: ProgramArchiveCatalogue,
    default_recipe: Option<ProgramRecipe>,
    resident: Option<(ProgramRecipe, Py<PyKernels>)>,
    status: GenerationSnapshot,
}

impl PyRecipeArchive {
    pub(super) fn generated(
        py: Python<'_>,
        storage: Rc<TempDir>,
        path: PathBuf,
        output: RecipeFamilyOutput<File>,
        snapshot: &RecipeFamilySnapshot,
    ) -> PyResult<Self> {
        output
            .writer
            .sync_all()
            .map_err(|e| error::native(py, "archive", e))?;
        let resident = output
            .resident
            .map(|kernels| {
                let recipe = kernels.program_recipe();
                let mut status = snapshot.generation.clone();
                status.sectors = kernels.sectors().len();
                status.kernels = kernels.sectors().len();
                status.detail = format!(
                    "Resident {} recipe; generation timings cover the complete family",
                    super::label(recipe)
                );
                Py::new(
                    py,
                    PyKernels {
                        inner: Rc::new(kernels),
                        status,
                    },
                )
                .map(|owner| (recipe, owner))
            })
            .transpose()?;
        Ok(Self {
            _storage: storage,
            path,
            catalogue: output.catalogue,
            default_recipe: Some(output.family.default_recipe()),
            resident,
            status: snapshot.generation.clone(),
        })
    }

    fn import(
        py: Python<'_>,
        mut input: impl io::Read,
        default_recipe: Option<&str>,
    ) -> PyResult<Self> {
        let default_recipe = default_recipe.map(super::archive_recipe).transpose()?;
        let storage = Rc::new(
            tempfile::Builder::new()
                .prefix("fastsecdec-family-")
                .tempdir()
                .map_err(|e| error::native(py, "archive", e))?,
        );
        let path = storage.path().join("integral.fsd");
        let mut output = File::create(&path).map_err(|e| error::native(py, "archive", e))?;
        io::copy(&mut input, &mut output).map_err(|e| error::native(py, "archive", e))?;
        output
            .flush()
            .map_err(|e| error::native(py, "archive", e))?;
        let reader = ProgramArchiveReader::from_reader(
            File::open(&path).map_err(|e| error::native(py, "archive", e))?,
            KernelLoadOptions::default(),
        )
        .map_err(|e| error::native(py, "archive", e))?;
        let catalogue = reader.catalogue().clone();
        if let Some(recipe) = default_recipe {
            catalogue
                .recipe(recipe)
                .map_err(|e| error::native(py, "archive", e))?;
        }
        Ok(Self {
            _storage: storage,
            path,
            catalogue,
            default_recipe,
            resident: None,
            status: GenerationSnapshot {
                stage: GenerationStage::Complete,
                completed: 0,
                total: None,
                sectors: 0,
                kernels: 0,
                elapsed_seconds: 0.0,
                timings: Default::default(),
                coefficient_expansion: None,
                formula_preparation: None,
                detail: "Native archive directory loaded; select a recipe".into(),
            },
        })
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyRecipeArchive {
    #[getter]
    fn recipes(&self) -> Vec<&'static str> {
        self.catalogue
            .recipes
            .iter()
            .map(|r| super::label(r.recipe))
            .collect()
    }
    /// The generation request's default, or an explicit import override. Native
    /// binary archives do not encode a default, so imports otherwise return None.
    #[getter]
    fn default_recipe(&self) -> Option<&'static str> {
        self.default_recipe.map(super::label)
    }
    #[getter]
    fn resident_recipe(&self) -> Option<&'static str> {
        self.resident
            .as_ref()
            .map(|(recipe, _)| super::label(*recipe))
    }
    #[getter]
    fn content_id(&self) -> &str {
        &self.catalogue.content_id
    }
    #[getter]
    fn source_identity(&self) -> Option<&str> {
        self.catalogue.source_identity.as_deref()
    }

    /// Reuse the retained resident object or load only the requested native recipe.
    /// Selection does not bind kinematics or bypass dynamic numerical admission.
    fn select(&self, py: Python<'_>, recipe: &str) -> PyResult<Py<PyKernels>> {
        py.check_signals()?;
        let recipe = super::archive_recipe(recipe)?;
        self.catalogue
            .recipe(recipe)
            .map_err(|e| error::native(py, "archive selection", e))?;
        if let Some((resident, kernels)) = &self.resident
            && *resident == recipe
        {
            return Ok(kernels.clone_ref(py));
        }
        let mut reader = ProgramArchiveReader::from_reader(
            File::open(&self.path).map_err(|e| error::native(py, "archive selection", e))?,
            KernelLoadOptions::default(),
        )
        .map_err(|e| error::native(py, "archive selection", e))?;
        let mut interruption = None;
        let selected = reader.select(recipe).and_then(|mut selected| {
            selected.load_all_with_progress(&mut |_| match py.check_signals() {
                Ok(()) => ControlFlow::Continue(()),
                Err(error) => {
                    interruption = Some(error);
                    ControlFlow::Break(())
                }
            })
        });
        if let Some(error) = interruption {
            return Err(error);
        }
        let kernels = selected.map_err(|e| error::native(py, "archive selection", e))?;
        py.check_signals()?;
        let mut status = self.status.clone();
        status.sectors = kernels.sectors().len();
        status.kernels = kernels.sectors().len();
        status.detail = format!(
            "Selected {} recipe from native archive",
            super::label(recipe)
        );
        Py::new(
            py,
            PyKernels {
                inner: Rc::new(kernels),
                status,
            },
        )
    }

    /// Explicitly copy the complete native binary archive into Python bytes.
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = std::fs::read(&self.path).map_err(|e| error::native(py, "archive", e))?;
        Ok(PyBytes::new(py, &bytes))
    }
    /// Import a native binary archive. No programs are decoded until select().
    #[staticmethod]
    #[pyo3(signature=(data, *, default_recipe=None))]
    fn from_bytes(
        py: Python<'_>,
        data: &Bound<'_, PyBytes>,
        default_recipe: Option<&str>,
    ) -> PyResult<Self> {
        Self::import(py, Cursor::new(data.as_bytes()), default_recipe)
    }
    /// Import native binary bytes, not the CLI JSON manifest. Retains an independent
    /// temporary copy, so replacing the original file cannot invalidate this owner.
    #[staticmethod]
    #[pyo3(signature=(path, *, default_recipe=None))]
    fn load(py: Python<'_>, path: PathBuf, default_recipe: Option<&str>) -> PyResult<Self> {
        Self::import(
            py,
            File::open(path).map_err(|e| error::native(py, "archive", e))?,
            default_recipe,
        )
    }
    /// Atomically replace a standalone native binary archive. This is distinct
    /// from the CLI's manifest and immutable generation-specific data publication.
    fn save(&self, py: Python<'_>, path: PathBuf) -> PyResult<()> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| error::native(py, "archive export", e))?;
        let mut source =
            File::open(&self.path).map_err(|e| error::native(py, "archive export", e))?;
        io::copy(&mut source, &mut temporary)
            .map_err(|e| error::native(py, "archive export", e))?;
        temporary
            .as_file_mut()
            .flush()
            .map_err(|e| error::native(py, "archive export", e))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|e| error::native(py, "archive export", e))?;
        temporary
            .persist(path)
            .map_err(|e| error::native(py, "archive export", e))?;
        Ok(())
    }
    fn catalogue_json(&self) -> String {
        serde_json::to_string(&self.catalogue).expect("native catalogue is serializable")
    }
    fn __repr__(&self) -> String {
        format!(
            "RecipeArchive(recipes={:?}, default_recipe={:?}, resident_recipe={:?})",
            self.recipes(),
            self.default_recipe(),
            self.resident_recipe()
        )
    }
}
