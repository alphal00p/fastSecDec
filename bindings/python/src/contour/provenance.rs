//! Immutable native checkpoint proof views; no inferred sampling seed.
use fastsecdec::{
    kernel::KernelSet,
    status::{ContourCheckpointProvenance, ContourPilotProvenance},
};
use pyo3::{prelude::*, types::PyModule};

pub(crate) fn checkpoint_provenance(
    kernels: &KernelSet,
    previous: Option<&ContourCheckpointProvenance>,
) -> Option<ContourCheckpointProvenance> {
    let settings = kernels.contour_settings()?;
    let pilot = kernels
        .contour_validation_report()
        .filter(|report| report.pilot_complete && report.accepted_pilot_points != 0)
        .map(|report| {
            ContourPilotProvenance::from_report(
                "caller-supplied-contour-pilot-v1",
                kernels.content_id(),
                None::<u64>,
                &report,
            )
        });
    Some(ContourCheckpointProvenance::update(
        previous,
        &settings.validation,
        pilot,
    ))
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourPilotProvenance",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourPilotProvenance {
    pub(crate) inner: ContourPilotProvenance,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourPilotProvenance {
    #[getter]
    fn protocol(&self) -> &str {
        &self.inner.protocol
    }
    #[getter]
    fn kernel_content_id(&self) -> &str {
        &self.inner.kernel_content_id
    }
    #[getter]
    fn seed(&self) -> Option<u64> {
        self.inner.seed
    }
    #[getter]
    fn policy(&self) -> &'static str {
        super::policy_name(self.inner.policy)
    }
    #[getter]
    fn complete(&self) -> bool {
        self.inner.complete
    }
    #[getter]
    fn required_charts(&self) -> Vec<usize> {
        self.inner.required_charts.clone()
    }
    #[getter]
    fn validated_charts(&self) -> Vec<usize> {
        self.inner.validated_charts.clone()
    }
    #[getter]
    fn sampled_points(&self) -> usize {
        self.inner.sampled_points
    }
    #[getter]
    fn checked_arguments(&self) -> usize {
        self.inner.checked_arguments
    }
    #[getter]
    fn maximum_bits(&self) -> u32 {
        self.inner.maximum_bits
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native contour pilot provenance")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "ContourCheckpointProvenance",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyContourCheckpointProvenance {
    pub(crate) inner: ContourCheckpointProvenance,
}
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourCheckpointProvenance {
    #[getter]
    fn validation(&self) -> &'static str {
        super::policy_name(self.inner.validation.policy)
    }
    #[getter]
    fn pilot_points(&self) -> usize {
        self.inner.validation.pilot_points
    }
    #[getter]
    fn pilots(&self) -> Vec<PyContourPilotProvenance> {
        self.inner
            .pilots
            .iter()
            .cloned()
            .map(|inner| PyContourPilotProvenance { inner })
            .collect()
    }
    fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("native contour checkpoint provenance")
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyContourPilotProvenance>()?;
    module.add_class::<PyContourCheckpointProvenance>()?;
    Ok(())
}
