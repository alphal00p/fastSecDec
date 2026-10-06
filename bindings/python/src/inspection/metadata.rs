use pyo3::prelude::*;

use super::{
    Owner,
    domain::PyDomainAssessment,
    geometry::{PyCoordinateMap, PySectorMap},
    pre_subtraction::PyPreSubtractionMetadata,
};

/// Retained domain and chart associations; no decomposition is repeated by inspection.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "GenerationMetadata"
)]
pub(crate) struct PyGenerationMetadata {
    pub(super) owner: Owner,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGenerationMetadata {
    #[getter]
    fn domain(&self) -> PyDomainAssessment {
        PyDomainAssessment {
            owner: self.owner.clone(),
        }
    }
    #[getter]
    fn charts(&self) -> Vec<PyChart> {
        (0..self.owner.metadata().charts().len())
            .map(|index| PyChart {
                owner: self.owner.clone(),
                index,
            })
            .collect()
    }
}

/// Original chart ordinals and their retained native representative/kernel association.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "Chart"
)]
pub(crate) struct PyChart {
    owner: Owner,
    index: usize,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyChart {
    #[getter]
    fn source_index(&self) -> usize {
        self.owner.metadata().charts()[self.index].source_index()
    }
    #[getter]
    fn representative(&self) -> usize {
        self.owner.metadata().charts()[self.index].representative()
    }
    /// Permutes this chart's target coordinates into its representative's coordinates.
    #[getter]
    fn representative_permutation(&self) -> Vec<usize> {
        self.owner.metadata().charts()[self.index]
            .representative_permutation()
            .to_vec()
    }
    /// None means no numerical kernel at the requested orders: exact, cancelled,
    /// or truncated. No separate per-chart exact coefficients are retained.
    #[getter]
    fn kernel_sector(&self) -> Option<usize> {
        self.owner.metadata().charts()[self.index].kernel_sector()
    }
    #[getter]
    fn coordinates(&self) -> PyCoordinateMap {
        PyCoordinateMap {
            owner: self.owner.clone(),
            chart: self.index,
        }
    }
    #[getter]
    fn geometry(&self) -> PySectorMap {
        PySectorMap::chart(self.owner.clone(), self.index)
    }
    /// Source-chart endpoint powers before symmetry, multiplicity and subtraction.
    /// None denotes an older artifact without this retained record.
    #[getter]
    fn pre_subtraction(&self) -> Option<PyPreSubtractionMetadata> {
        self.owner.metadata().charts()[self.index]
            .pre_subtraction()
            .map(|_| PyPreSubtractionMetadata::new(self.owner.clone(), self.index))
    }
}
