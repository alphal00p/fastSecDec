//! Lazy views of the native deformation recipe; no algebra runs on inspection.
use std::rc::Rc;

use fastsecdec::{generation::GenerationMetadata, kernel::KernelSet};
use pyo3::prelude::*;
use symbolica::{api::python::PythonExpression, atom::Atom};

use super::{Owner, expression};

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(
    frozen,
    unsendable,
    module = "symbolica.community.hepkit.sector_decomposition",
    name = "ContourRecipe"
)]
pub(crate) struct PyContourRecipe {
    owner: ContourOwner,
    chart: usize,
}

enum ContourOwner {
    Generated(Owner),
    Compiled(Rc<KernelSet>),
}

impl ContourOwner {
    fn metadata(&self) -> &GenerationMetadata {
        match self {
            Self::Generated(owner) => owner.metadata(),
            Self::Compiled(owner) => owner
                .generation_metadata()
                .expect("the immutable compiled owner retained generation metadata"),
        }
    }
}

impl PyContourRecipe {
    pub(super) fn new(owner: Owner, chart: usize) -> Self {
        Self {
            owner: ContourOwner::Generated(owner),
            chart,
        }
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
    /// Original source-chart ordinal, retained through recipe selection and reload.
    #[getter]
    fn source_index(&self) -> usize {
        self.owner.metadata().charts()[self.chart].source_index()
    }

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

    /// Number of retained coefficient bodies; inspecting this count performs
    /// no expression materialization or evaluator construction.
    #[getter]
    fn function_definition_count(&self) -> usize {
        self.native().function_definitions().entries().len()
    }

    /// Native (formal function call, body) definitions for the compact map.
    /// Returned Symbolica expressions are independent views. Symbolic derivative
    /// calls in the map refer to derivatives of these same full-source bodies;
    /// no derivatives or substitutions are evaluated by this getter.
    #[getter]
    fn function_definitions(&self) -> Vec<(PythonExpression, PythonExpression)> {
        self.native()
            .function_definitions()
            .entries()
            .iter()
            .map(|definition| {
                let call = definition
                    .function()
                    .call_args(definition.parameters().iter().copied().map(Atom::var));
                (expression(&call), expression(definition.body()))
            })
            .collect()
    }

    /// Checked restrictions of the same full-sector map, as (axis, endpoint) pairs.
    /// An empty restriction denotes the interior. Returned lists are owned copies.
    #[getter]
    fn validation_faces(&self) -> Vec<Vec<(usize, u8)>> {
        self.native().validation_faces().to_vec()
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::kernels::PyKernels {
    /// Inspect already retained contour maps, including compact native bodies.
    /// None denotes missing historical metadata; an empty list denotes no maps.
    /// The views retain this native owner without cloning charts, rebuilding
    /// expressions or constructing evaluators.
    #[getter]
    fn contour_recipes(&self) -> Option<Vec<PyContourRecipe>> {
        self.inner.generation_metadata().map(|metadata| {
            metadata
                .charts()
                .iter()
                .enumerate()
                .filter_map(|(chart, value)| {
                    value.contour().map(|_| PyContourRecipe {
                        owner: ContourOwner::Compiled(self.inner.clone()),
                        chart,
                    })
                })
                .collect()
        })
    }
}
