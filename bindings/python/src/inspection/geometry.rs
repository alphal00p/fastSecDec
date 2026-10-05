use fastsecdec_sectors::SectorMap;
use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::Atom, domains::integer::Integer};

use super::{Owner, domain_name, expression};

/// Native density pullback before endpoint subtraction, including its positive measure.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "CoordinateMap"
)]
pub(crate) struct PyCoordinateMap {
    pub(super) owner: Owner,
    pub(super) chart: usize,
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyCoordinateMap {
    #[getter]
    fn source_parameters(&self) -> Vec<PythonExpression> {
        self.owner.metadata().charts()[self.chart]
            .coordinates()
            .source_parameters()
            .iter()
            .map(|symbol| PythonExpression {
                expr: Atom::var(*symbol),
            })
            .collect()
    }
    #[getter]
    fn target_parameters(&self) -> Vec<PythonExpression> {
        self.owner.metadata().charts()[self.chart]
            .coordinates()
            .target_parameters()
            .iter()
            .map(|symbol| PythonExpression {
                expr: Atom::var(*symbol),
            })
            .collect()
    }
    #[getter]
    fn images(&self) -> Vec<PythonExpression> {
        self.owner.metadata().charts()[self.chart]
            .coordinates()
            .images()
            .iter()
            .map(expression)
            .collect()
    }
    /// Positive real density factor, not an oriented/complex determinant.
    /// A projective map is gauge-fixed, not a normalized simplex parameterization.
    #[getter]
    fn measure_jacobian(&self) -> PythonExpression {
        expression(
            self.owner.metadata().charts()[self.chart]
                .coordinates()
                .measure_jacobian(),
        )
    }
    #[getter]
    fn source_domain(&self) -> &'static str {
        domain_name(
            self.owner.metadata().charts()[self.chart]
                .coordinates()
                .source_domain(),
        )
    }
    #[getter]
    fn projective_fixed_parameter(&self) -> Option<usize> {
        self.owner.metadata().charts()[self.chart]
            .coordinates()
            .projective_fixed_parameter()
    }
}

enum Location {
    Sector(usize),
    Chart(usize),
}

/// The retained exact integer geometry, with native Numerica-to-Python integer transport.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.fastsecdec",
    name = "SectorMap"
)]
pub(crate) struct PySectorMap {
    owner: Owner,
    location: Location,
}

impl PySectorMap {
    pub(super) fn sector(owner: Owner, index: usize) -> Self {
        Self {
            owner,
            location: Location::Sector(index),
        }
    }
    pub(super) fn chart(owner: Owner, index: usize) -> Self {
        Self {
            owner,
            location: Location::Chart(index),
        }
    }
    fn native(&self) -> &SectorMap {
        match self.location {
            Location::Sector(index) => self.owner.sectors()[index].map(),
            Location::Chart(index) => self.owner.metadata().charts()[index].geometry(),
        }
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PySectorMap {
    #[getter]
    fn source_dimension(&self) -> usize {
        self.native().source_dimension()
    }
    #[getter]
    fn dimension(&self) -> usize {
        self.native().dimension()
    }
    #[getter]
    fn fixed_parameter(&self) -> Option<usize> {
        self.native().fixed_parameter
    }
    #[getter]
    fn exponent_matrix(&self) -> Vec<Vec<Integer>> {
        self.native().exponent_matrix.clone()
    }
    #[getter]
    fn determinant(&self) -> Integer {
        self.native().determinant.clone()
    }
    #[getter]
    fn jacobian_powers(&self) -> Vec<Integer> {
        self.native().jacobian_powers.clone()
    }
    /// Indices follow deduplicated support order, not uniquely named U/F factors.
    #[getter]
    fn factor_valuations(&self) -> Vec<Vec<Integer>> {
        self.native().factor_valuations.clone()
    }
}
