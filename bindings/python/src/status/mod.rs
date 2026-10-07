//! Immutable views of native snapshots, with no reconstructed statistics.
pub(crate) mod allocation;
pub(crate) mod diagnostics;
pub(crate) mod generation;
pub(crate) mod integration;
pub(crate) mod observation;
pub(crate) use observation::{PyIntegrationObservation, PyLiveObservation};

pub(crate) use generation::PyGenerationSnapshot;
pub(crate) use integration::PyIntegrationSnapshot;
use pyo3::{prelude::*, types::PyModule};

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<generation::PyGenerationSnapshot>()?;
    module.add_class::<generation::PyGenerationTimings>()?;
    module.add_class::<generation::PyFormulaPreparationSnapshot>()?;
    module.add_class::<generation::PyCoefficientExpansionSnapshot>()?;
    module.add_class::<generation::PyCoefficientRequestCounts>()?;
    module.add_class::<integration::PyIntegrationSnapshot>()?;
    module.add_class::<integration::PySectorSnapshot>()?;
    module.add_class::<integration::PyVectorEstimate>()?;
    module.add_class::<diagnostics::PyEvaluationDiagnostics>()?;
    module.add_class::<diagnostics::PyEvaluatorTiming>()?;
    module.add_class::<allocation::PyDiscreteSectorAllocation>()?;
    module.add_class::<observation::PyIntegrationObservation>()?;
    module.add_class::<observation::PySectorContribution>()?;
    module.add_class::<observation::PyLiveObservation>()?;
    module.add_class::<observation::PyLiveSector>()?;
    module.add_class::<observation::PyLiveEstimate>()?;
    Ok(())
}
