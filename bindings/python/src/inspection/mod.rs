//! Immutable views of retained generation records; inspection never reruns algebra.
mod coefficient;
mod domain;
mod geometry;
mod metadata;
mod pre_subtraction;
mod sector;
mod statistics;

use std::sync::Arc;

use fastsecdec::generation::GeneratedIntegral;
use fastsecdec_sectors::ParametricDomain;
use pyo3::{prelude::*, types::PyModule};
use symbolica::{api::python::PythonExpression, atom::Atom};

pub(crate) use metadata::PyGenerationMetadata;
pub(crate) use sector::PyGeneratedSector;
pub(crate) use statistics::PyEvaluatorStatistics;

type Owner = Arc<GeneratedIntegral>;

pub(crate) fn sectors(owner: Owner) -> Vec<PyGeneratedSector> {
    (0..owner.sectors().len())
        .map(|index| PyGeneratedSector {
            owner: owner.clone(),
            index,
        })
        .collect()
}

pub(crate) fn metadata(owner: Owner) -> PyGenerationMetadata {
    PyGenerationMetadata { owner }
}

fn expression(atom: &Atom) -> PythonExpression {
    PythonExpression { expr: atom.clone() }
}

fn domain_name(domain: ParametricDomain) -> &'static str {
    match domain {
        ParametricDomain::ProjectiveSimplex => "projective_simplex",
        ParametricDomain::UnitCube => "unit_cube",
        ParametricDomain::PositiveOrthant => "positive_orthant",
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyGeneratedSector>()?;
    module.add_class::<coefficient::PyCompactCoefficient>()?;
    module.add_class::<PyGenerationMetadata>()?;
    module.add_class::<metadata::PyChart>()?;
    module.add_class::<domain::PyDomainAssessment>()?;
    module.add_class::<domain::PyFactorAssessment>()?;
    module.add_class::<geometry::PyCoordinateMap>()?;
    module.add_class::<geometry::PySectorMap>()?;
    module.add_class::<pre_subtraction::PyPreSubtractionMetadata>()?;
    module.add_class::<pre_subtraction::PyPreSubtractionTerm>()?;
    module.add_class::<pre_subtraction::PyEndpointPower>()?;
    module.add_class::<statistics::PyEvaluatorStatistics>()?;
    module.add_class::<statistics::PyEvaluatorOperations>()?;
    Ok(())
}
