use fastsecdec::integration::{mc::HavanaSettings, mc_discrete::HavanaDiscreteSettings};
use pyo3::prelude::*;

/// Native global-batch allocation and discrete/continuous Havana grid settings.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "HavanaDiscreteSettings",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen,
    from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyHavanaDiscreteSettings {
    pub(crate) inner: HavanaDiscreteSettings,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyHavanaDiscreteSettings {
    #[new]
    #[pyo3(signature = (*, points_per_batch=4096, batches=64, seed=0, bins=32, minimum_probability_density=0.01, maximum_sector_probability_ratio=100.0))]
    #[allow(clippy::too_many_arguments)]
    fn py_new(
        py: Python<'_>,
        points_per_batch: usize,
        batches: u32,
        seed: u64,
        bins: usize,
        minimum_probability_density: f64,
        maximum_sector_probability_ratio: f64,
    ) -> PyResult<Self> {
        let inner = HavanaDiscreteSettings {
            batch: HavanaSettings {
                points_per_batch,
                batches,
                seed,
                bins,
                minimum_probability_density,
            },
            maximum_sector_probability_ratio,
        };
        inner
            .validate()
            .map_err(|e| crate::error::native(py, "configuration", e))?;
        Ok(Self { inner })
    }
    #[getter]
    fn points_per_batch(&self) -> usize {
        self.inner.batch.points_per_batch
    }
    #[getter]
    fn batches(&self) -> u32 {
        self.inner.batch.batches
    }
    #[getter]
    fn seed(&self) -> u64 {
        self.inner.batch.seed
    }
    #[getter]
    fn bins(&self) -> usize {
        self.inner.batch.bins
    }
    #[getter]
    fn minimum_probability_density(&self) -> f64 {
        self.inner.batch.minimum_probability_density
    }
    #[getter]
    fn maximum_sector_probability_ratio(&self) -> f64 {
        self.inner.maximum_sector_probability_ratio
    }
}
