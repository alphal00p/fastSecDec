use crate::status::PyGenerationSnapshot;
use fastsecdec::generation::RecipeFamilySnapshot;
use pyo3::prelude::*;

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "RecipeFamilySnapshot",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyRecipeFamilySnapshot {
    pub(super) inner: RecipeFamilySnapshot,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyRecipeFamilySnapshot {
    #[getter]
    fn recipe(&self) -> Option<&'static str> {
        self.inner.recipe.map(super::label)
    }
    #[getter]
    fn completed_recipes(&self) -> usize {
        self.inner.completed_recipes
    }
    #[getter]
    fn total_recipes(&self) -> usize {
        self.inner.total_recipes
    }
    /// Completed native units; initial graph parametrization is reported separately.
    #[getter]
    fn completed_units(&self) -> usize {
        self.inner.completed_units
    }
    #[getter]
    fn prepared_sources(&self) -> usize {
        self.inner.prepared_sources
    }
    #[getter]
    fn persisted_units(&self) -> usize {
        self.inner.persisted_units
    }
    #[getter]
    fn generation(&self) -> PyGenerationSnapshot {
        PyGenerationSnapshot {
            inner: self.inner.generation.clone(),
        }
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native family snapshot is serializable")
    }
    fn __repr__(&self) -> String {
        format!(
            "RecipeFamilySnapshot(recipe={:?}, completed_recipes={}, total_recipes={}, stage='{}')",
            self.recipe(),
            self.inner.completed_recipes,
            self.inner.total_recipes,
            self.inner.generation.stage.label()
        )
    }
}
