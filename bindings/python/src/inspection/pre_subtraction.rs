//! Borrowed views of native source-chart endpoint admission, with no reconstruction.
use fastsecdec::generation::{EndpointPower, PreSubtractionMetadata, PreSubtractionTerm};
use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::Atom};

use super::{Owner, expression};

/// Mapped density before symmetry, multiplicity, endpoint subtraction or expansion.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "PreSubtractionMetadata"
)]
pub(crate) struct PyPreSubtractionMetadata {
    owner: Owner,
    chart: usize,
}

impl PyPreSubtractionMetadata {
    pub(super) fn new(owner: Owner, chart: usize) -> Self {
        Self { owner, chart }
    }
    fn native(&self) -> &PreSubtractionMetadata {
        self.owner.metadata().charts()[self.chart]
            .pre_subtraction()
            .expect("view constructed only for a retained native record")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyPreSubtractionMetadata {
    #[getter]
    fn version(&self) -> u32 {
        self.native().version()
    }
    #[getter]
    fn regulator(&self) -> PythonExpression {
        expression(&Atom::var(self.native().regulator()))
    }
    #[getter]
    fn terms(&self) -> Vec<PyPreSubtractionTerm> {
        (0..self.native().terms().len())
            .map(|term| PyPreSubtractionTerm {
                owner: self.owner.clone(),
                chart: self.chart,
                term,
            })
            .collect()
    }
}

/// One mapped term; its potentially large regular body is never materialized.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "PreSubtractionTerm"
)]
pub(crate) struct PyPreSubtractionTerm {
    owner: Owner,
    chart: usize,
    term: usize,
}

impl PyPreSubtractionTerm {
    fn native(&self) -> &PreSubtractionTerm {
        &self.owner.metadata().charts()[self.chart]
            .pre_subtraction()
            .expect("retained native record")
            .terms()[self.term]
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyPreSubtractionTerm {
    #[getter]
    fn prefactor(&self) -> PythonExpression {
        expression(self.native().prefactor())
    }
    /// Native Atom storage at mapping, not an expanded term or evaluator count.
    #[getter]
    fn regular_expression_bytes(&self) -> usize {
        self.native().regular_expression_bytes()
    }
    /// Ordered like this source chart's coordinate target_parameters.
    #[getter]
    fn powers(&self) -> Vec<PyEndpointPower> {
        (0..self.native().powers().len())
            .map(|axis| PyEndpointPower {
                owner: self.owner.clone(),
                chart: self.chart,
                term: self.term,
                axis,
            })
            .collect()
    }
}

/// Exact b + c epsilon power and required Taylor coefficients; not a pole count.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "EndpointPower"
)]
pub(crate) struct PyEndpointPower {
    owner: Owner,
    chart: usize,
    term: usize,
    axis: usize,
}

impl PyEndpointPower {
    fn native(&self) -> &EndpointPower {
        &self.owner.metadata().charts()[self.chart]
            .pre_subtraction()
            .expect("retained native record")
            .terms()[self.term]
            .powers()[self.axis]
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyEndpointPower {
    #[getter]
    fn parameter(&self) -> PythonExpression {
        expression(&Atom::var(
            self.owner.metadata().charts()[self.chart]
                .coordinates()
                .target_parameters()[self.axis],
        ))
    }
    #[getter]
    fn exponent(&self) -> PythonExpression {
        expression(self.native().exponent())
    }
    #[getter]
    fn constant(&self) -> PythonExpression {
        expression(&Atom::num(self.native().constant().clone()))
    }
    #[getter]
    fn slope(&self) -> PythonExpression {
        expression(&Atom::num(self.native().slope().clone()))
    }
    /// Required endpoint Taylor coefficients; vanishing boundary terms may cancel.
    #[getter]
    fn subtraction_count(&self) -> usize {
        self.native().subtraction_count()
    }
}
