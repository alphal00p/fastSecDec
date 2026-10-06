use std::{collections::BTreeMap, sync::Arc};

use fastsecdec::{Atom, EdgeId, Symbol, input::GraphIntegral};
use feynkit_py::{PyFeynmanDiagram, PyKinematics};
use pyo3::{prelude::*, types::PyDict};
use symbolica::{api::python::PythonExpression, atom::AtomView};

use super::error;

/// Copy a complete native diagram, replacing only the supplied expressions.
/// Native numerator validation and all graph/model ownership are preserved.
#[cfg_attr(
    feature = "python_stubgen",
    pyo3_stub_gen::derive::gen_stub_pyfunction(
        module = "symbolica.community.hepkit.sector_decomposition"
    )
)]
#[pyfunction]
#[pyo3(signature = (diagram, *, numerator=None, projector=None, overall_factor=None))]
pub(crate) fn with_diagram_expressions(
    py: Python<'_>,
    diagram: &PyFeynmanDiagram,
    numerator: Option<&PythonExpression>,
    projector: Option<&PythonExpression>,
    overall_factor: Option<&PythonExpression>,
) -> PyResult<PyFeynmanDiagram> {
    let mut copied = diagram.as_diagram()?.clone();
    if let Some(value) = numerator {
        copied = copied
            .with_numerator(value.expr.clone())
            .map_err(|e| error::native(py, "input", e))?;
    }
    if let Some(value) = projector {
        copied = copied.with_projector(value.expr.clone());
    }
    if let Some(value) = overall_factor {
        copied = copied.with_overall_factor(value.expr.clone());
    }
    Ok(copied.into())
}

/// Native diagram, assumptions and explicit normalized loop measure.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    name = "Integral",
    module = "symbolica.community.hepkit.sector_decomposition",
    frozen
)]
pub(crate) struct PyIntegral {
    pub(crate) graph: GraphIntegral,
    pub(crate) regulator: Symbol,
    pub(crate) dimension: Atom,
}

pub(crate) fn symbol(py: Python<'_>, value: &PythonExpression, name: &str) -> PyResult<Symbol> {
    match value.expr.as_view() {
        AtomView::Var(value) => Ok(value.get_symbol()),
        _ => Err(error::native(
            py,
            "input",
            format!("{name} must be a Symbolica symbol"),
        )),
    }
}

/// Transport Python expressions only; native input owners validate and apply
/// the complete scalar point consistently to their physics data.
pub(crate) fn scalar_bindings(
    py: Python<'_>,
    values: Option<&Bound<'_, PyDict>>,
) -> PyResult<BTreeMap<Symbol, Atom>> {
    let mut bindings = BTreeMap::new();
    if let Some(values) = values {
        for (key, value) in values.iter() {
            let key = key.extract::<PyRef<'_, PythonExpression>>()?;
            let value = value.extract::<PyRef<'_, PythonExpression>>()?;
            bindings.insert(symbol(py, &key, "scalar binding key")?, value.expr.clone());
        }
    }
    Ok(bindings)
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyIntegral {
    /// Kinematics retains its symbolic tensor dimension; dimension defaults to 4-2*regulator.
    #[new]
    #[pyo3(signature = (diagram, kinematics, *, regulator, dimension=None, powers=None, scalar_values=None, auxiliary_momenta=None, measure_multiplier=None))]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        py: Python<'_>,
        diagram: &PyFeynmanDiagram,
        kinematics: &PyKinematics,
        regulator: &PythonExpression,
        dimension: Option<&PythonExpression>,
        powers: Option<BTreeMap<usize, u32>>,
        scalar_values: Option<&Bound<'_, PyDict>>,
        auxiliary_momenta: Option<Vec<PythonExpression>>,
        measure_multiplier: Option<&PythonExpression>,
    ) -> PyResult<Self> {
        let regulator = symbol(py, regulator, "regulator")?;
        // Guard partial selected subgraphs before borrowing the native owner.
        let graph = GraphIntegral::new(
            Arc::new(diagram.as_diagram()?.clone()),
            kinematics.as_kinematics(),
        )
        .map_err(|e| error::native(py, "input", e))?;
        let bindings = scalar_bindings(py, scalar_values)?;
        let powers = powers
            .unwrap_or_default()
            .into_iter()
            .map(|(id, power)| (EdgeId(id), power))
            .collect();
        let momenta: Vec<_> = auxiliary_momenta
            .unwrap_or_default()
            .into_iter()
            .map(|v| v.expr)
            .collect();
        let graph = graph
            .with_auxiliary_external_momenta(&momenta)
            .and_then(|g| g.with_scalar_values(&bindings))
            .and_then(|g| g.with_powers(&powers))
            .map_err(|e| error::native(py, "input", e))?
            .with_measure_multiplier(measure_multiplier.map_or_else(Atom::one, |v| v.expr.clone()));
        Ok(Self {
            graph,
            regulator,
            dimension: dimension.map_or_else(
                || Atom::num(4) - Atom::num(2) * Atom::var(regulator),
                |v| v.expr.clone(),
            ),
        })
    }

    #[getter]
    fn regulator(&self) -> PythonExpression {
        PythonExpression {
            expr: Atom::var(self.regulator),
        }
    }
    #[getter]
    fn dimension(&self) -> PythonExpression {
        PythonExpression {
            expr: self.dimension.clone(),
        }
    }
    #[getter]
    fn powers(&self) -> Vec<(usize, u32)> {
        self.graph
            .propagator_edges()
            .iter()
            .zip(self.graph.powers())
            .map(|(e, p)| (e.0, *p))
            .collect()
    }
}
