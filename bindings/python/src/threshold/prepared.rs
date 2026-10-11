//! Prepared threshold view over the existing retained native generation owner.
use super::session::PyThresholdGenerationSession;
use crate::{
    error, family::archive::PyRecipeArchive, kernels::PyKernels, settings::PyCompilationSettings,
    status::PyGenerationSnapshot,
};
use pyo3::{prelude::*, types::PyString};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "PreparedThreshold",
    module = "symbolica.community.hepkit.sector_decomposition",
    unsendable
)]
pub(crate) struct PyPreparedThreshold {
    session: Py<PyThresholdGenerationSession>,
    kernels: Option<Py<PyKernels>>,
}
impl PyPreparedThreshold {
    pub(super) fn from_prepared(
        py: Python<'_>,
        session: PyThresholdGenerationSession,
    ) -> PyResult<Self> {
        if !session.prepared() {
            return Err(error::cancelled(py, "generation"));
        }
        Ok(Self {
            session: Py::new(py, session)?,
            kernels: None,
        })
    }
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyPreparedThreshold {
    /// Compile with the immutable settings selected during preparation, then load
    /// the native recipe. No numerical integration is started. Cancellation keeps
    /// completed records in generation_session(), so this action can be retried.
    #[pyo3(signature=(*,observer=None,progress=Some(Python::attach(|py|PyString::new(py,"auto").into_any().unbind()))))]
    fn compile(
        &mut self,
        py: Python<'_>,
        observer: Option<Py<PyAny>>,
        progress: Option<Py<PyAny>>,
    ) -> PyResult<Py<PyKernels>> {
        if let Some(kernels) = &self.kernels {
            return Ok(kernels.clone_ref(py));
        }
        let archive = {
            let mut session = self.session.try_borrow_mut(py)?;
            session.compile_remaining(py, observer, progress)?;
            session
                .result(py)
                .ok_or_else(|| error::cancelled(py, "compilation"))?
        };
        let kernels = archive.try_borrow(py)?.select(py, "threshold")?;
        self.kernels = Some(kernels.clone_ref(py));
        Ok(kernels)
    }
    /// The identical retained owner; calling this does no native work.
    fn generation_session(&self, py: Python<'_>) -> Py<PyThresholdGenerationSession> {
        self.session.clone_ref(py)
    }
    fn snapshot(&self, py: Python<'_>) -> PyResult<PyGenerationSnapshot> {
        Ok(self.session.try_borrow(py)?.snapshot())
    }
    fn preparation_receipt_json(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.session.try_borrow(py)?.preparation_receipt_json()
    }
    #[getter]
    fn compilation_settings(&self, py: Python<'_>) -> PyResult<PyCompilationSettings> {
        Ok(self.session.try_borrow(py)?.compilation_settings())
    }
    #[getter]
    fn archive(&self, py: Python<'_>) -> PyResult<Option<Py<PyRecipeArchive>>> {
        Ok(self.session.try_borrow(py)?.result(py))
    }
    /// All native records have been published to the archive. This does not
    /// imply a resident Kernels object has been loaded; compile() retries that
    /// separate load without discarding the accepted archive.
    #[getter]
    fn complete(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.session.try_borrow(py)?.complete())
    }
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "PreparedThreshold(archive_complete={}, scope='admitted rational fixed fiber')",
            self.complete(py)?
        ))
    }
}
