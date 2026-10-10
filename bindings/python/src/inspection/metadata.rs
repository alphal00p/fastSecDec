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
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "GenerationMetadata"
)]
pub(crate) struct PyGenerationMetadata {
    pub(super) owner: Owner,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyGenerationMetadata {
    /// Original decomposition extent for a partial generation; None means full scope.
    #[getter]
    fn original_source_count(&self) -> Option<usize> {
        self.owner
            .metadata()
            .source_scope()
            .map(|scope| scope.selection().original_source_count())
    }
    /// Original decomposition charts selected before symmetry and endpoint subtraction.
    #[getter]
    fn source_sectors(&self) -> Option<Vec<usize>> {
        self.owner
            .metadata()
            .source_scope()
            .map(|scope| scope.selection().source_sectors().to_vec())
    }
    /// Original source ID for each entry of charts, including record-local owners.
    #[getter]
    fn chart_source_sectors(&self) -> Option<Vec<usize>> {
        self.owner
            .metadata()
            .source_scope()
            .map(|scope| scope.chart_source_sectors().to_vec())
    }
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

/// Compact chart ordinals and their retained native representative/kernel association.
/// Partial generations expose original geometry IDs through GenerationMetadata.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
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

    /// Retained native deformation recipe, without regenerating expressions.
    #[getter]
    fn contour(&self) -> Option<super::contour::PyContourRecipe> {
        self.owner.metadata().charts()[self.index]
            .contour()
            .map(|_| super::contour::PyContourRecipe::new(self.owner.clone(), self.index))
    }
}
