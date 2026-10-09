//! Lazy views of the native deformation recipe; no algebra runs on inspection.
use pyo3::prelude::*;
use symbolica::api::python::PythonExpression;

use super::{Owner, expression};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "ContourRecipe"
)]
pub(crate) struct PyContourRecipe {
    owner: Owner,
    chart: usize,
}

impl PyContourRecipe {
    pub(super) fn new(owner: Owner, chart: usize) -> Self {
        Self { owner, chart }
    }

    fn native(&self) -> &fastsecdec::contour::ContourMetadata {
        self.owner.metadata().charts()[self.chart]
            .contour()
            .expect("the immutable owner retained this contour recipe")
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyContourRecipe {
    #[getter]
    fn version(&self) -> u32 {
        self.native().version()
    }

    /// Residual F, after sector mapping and endpoint monomial extraction.
    #[getter]
    fn causal_polynomial(&self) -> PythonExpression {
        expression(self.native().causal_polynomial())
    }

    /// Residual positive factors whose analytic branches must be preserved.
    #[getter]
    fn positive_polynomials(&self) -> Vec<PythonExpression> {
        self.native()
            .positive_polynomials()
            .iter()
            .map(expression)
            .collect()
    }

    /// Complex coordinates z(x), before the outer real periodization.
    #[getter]
    fn images(&self) -> Vec<PythonExpression> {
        self.native().images().iter().map(expression).collect()
    }

    /// Analytic z_i/x_i ratios, well-defined also on the coordinate faces.
    #[getter]
    fn ratios(&self) -> Vec<PythonExpression> {
        self.native().ratios().iter().map(expression).collect()
    }

    /// Native symbolic determinant; excludes the outer periodization Jacobian.
    #[getter]
    fn jacobian(&self) -> PythonExpression {
        expression(self.native().jacobian())
    }

    /// Checked restrictions of the same full-sector map, as (axis, endpoint) pairs.
    /// An empty restriction denotes the interior. Returned lists are owned copies.
    #[getter]
    fn validation_faces(&self) -> Vec<Vec<(usize, u8)>> {
        self.native().validation_faces().to_vec()
    }
}
