use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::AliasedAtom};

use super::{Owner, expression};

/// A retained Symbolica alias root and definitions, without restoring a full expression.
#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "CompactCoefficient"
)]
pub(crate) struct PyCompactCoefficient {
    pub(super) owner: Owner,
    pub(super) sector: usize,
    pub(super) coefficient: usize,
}

impl PyCompactCoefficient {
    fn native(&self) -> &AliasedAtom {
        &self.owner.sectors()[self.sector].aliased_coefficients()[self.coefficient]
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyCompactCoefficient {
    /// Signed epsilon order of this possibly complex generated coefficient.
    /// Compiled kernels may split it into separate real/imaginary components.
    #[getter]
    fn order(&self) -> i32 {
        self.owner.orders()[self.coefficient]
    }
    #[getter]
    fn root(&self) -> PythonExpression {
        expression(self.native().get_root())
    }
    /// Explicitly restore this selected coefficient with Symbolica's alias owner.
    /// This can be expensive; passive views retain the compact root and aliases.
    fn expression(&self) -> PythonExpression {
        PythonExpression {
            expr: self.native().clone().into_inner(),
        }
    }
    /// Stored definitions for this coefficient; definitions may be shared with others.
    #[getter]
    fn alias_count(&self) -> usize {
        self.native().get_aliases().len()
    }
    /// Alias/definition pairs in native Atom order. Definitions may reference aliases.
    /// Read only for selected detail; this does not inline or expand any expression.
    #[getter]
    fn aliases(&self) -> Vec<(PythonExpression, PythonExpression)> {
        let mut pairs: Vec<_> = self.native().get_aliases().iter().collect();
        pairs.sort_unstable_by_key(|(left, _)| *left);
        pairs
            .into_iter()
            .map(|(alias, definition)| (expression(alias), expression(definition)))
            .collect()
    }
}
