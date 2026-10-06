use fastsecdec::status::DiscreteSectorAllocation;
use pyo3::prelude::*;

/// Selection probability in the native discrete grid and global points per batch.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "DiscreteSectorAllocation"
)]
pub(crate) struct PyDiscreteSectorAllocation {
    pub(super) inner: DiscreteSectorAllocation,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyDiscreteSectorAllocation {
    #[getter]
    fn probability(&self) -> f64 {
        self.inner.probability
    }
    #[getter]
    fn points_per_batch(&self) -> u64 {
        self.inner.points_per_batch
    }
}
