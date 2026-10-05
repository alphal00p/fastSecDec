//! Immutable views of native snapshots, with no reconstructed statistics.
mod diagnostics;
mod generation;
mod integration;

pub(crate) use generation::PyGenerationSnapshot;
pub(crate) use integration::PyIntegrationSnapshot;
use pyo3::{prelude::*, types::PyModule};

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<generation::PyGenerationSnapshot>()?;
    module.add_class::<generation::PyGenerationTimings>()?;
    module.add_class::<generation::PyCoefficientExpansionSnapshot>()?;
    module.add_class::<generation::PyCoefficientRequestCounts>()?;
    module.add_class::<integration::PyIntegrationSnapshot>()?;
    module.add_class::<integration::PySectorSnapshot>()?;
    module.add_class::<integration::PyVectorEstimate>()?;
    module.add_class::<diagnostics::PyEvaluationDiagnostics>()?;
    Ok(())
}
