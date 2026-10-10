//! Thin ownership of native, caller-stepped recipe-family generation.
mod archive;
mod session;
mod snapshot;

use crate::{error, input::PyIntegral, settings::PyCompilationSettings};
use fastsecdec::{generation::RecipeFamily, kernel::indexed::ProgramRecipe};
use pyo3::prelude::*;
use symbolica::api::python::PythonExpression;

pub(crate) fn recipe(value: &str) -> PyResult<ProgramRecipe> {
    match value {
        "off" => Ok(ProgramRecipe::UndeformedV1),
        "fixed" => Ok(ProgramRecipe::FixedV1),
        "polynomial" => Ok(ProgramRecipe::DynamicPolynomialV1),
        "sign_aware" => Ok(ProgramRecipe::DynamicSignAwareV1),
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "recipe must be off, fixed, polynomial, or sign_aware",
        )),
    }
}

fn label(recipe: ProgramRecipe) -> &'static str {
    match recipe {
        ProgramRecipe::UndeformedV1 => "off",
        ProgramRecipe::FixedV1 => "fixed",
        ProgramRecipe::DynamicPolynomialV1 => "polynomial",
        ProgramRecipe::DynamicSignAwareV1 => "sign_aware",
        ProgramRecipe::ThresholdV1 => "threshold",
    }
}

#[pymethods]
impl PyIntegral {
    /// Create inert caller-stepped work sharing native source preparation across recipes.
    /// Recipe availability and dynamic numerical admission are distinct.
    #[pyo3(signature=(recipes, max_order=0, *, default_recipe="off", resident_recipe=None, coefficient_expansion="coefficient_series", mode="symbolic", subtraction="taylor", contour_jacobian="symbolic", compilation_settings=None, runtime_parameters=None))]
    #[allow(clippy::too_many_arguments)]
    fn generation_family_session(
        &self,
        py: Python<'_>,
        recipes: Vec<String>,
        max_order: i32,
        default_recipe: &str,
        resident_recipe: Option<&str>,
        coefficient_expansion: &str,
        mode: &str,
        subtraction: &str,
        contour_jacobian: &str,
        compilation_settings: Option<&PyCompilationSettings>,
        runtime_parameters: Option<Vec<PythonExpression>>,
    ) -> PyResult<session::PyRecipeFamilySession> {
        let family = RecipeFamily::new(
            recipes
                .iter()
                .map(|r| recipe(r))
                .collect::<PyResult<Vec<_>>>()?,
            recipe(default_recipe)?,
        )
        .map_err(|e| error::native(py, "generation family", e))?;
        let resident = resident_recipe.map(recipe).transpose()?;
        family
            .validate_resident(resident)
            .map_err(|e| error::native(py, "generation family", e))?;
        let options = crate::generation::options(
            max_order,
            coefficient_expansion,
            mode,
            subtraction,
            false,
            contour_jacobian,
        )?;
        let settings = compilation_settings
            .cloned()
            .unwrap_or_default()
            .inner
            .resolve_contour_jacobian(options.contour_jacobian)
            .map_err(|e| error::native(py, "compilation settings", e))?;
        settings
            .validate()
            .map_err(|e| error::native(py, "compilation settings", e))?;
        let mut input = self.clone();
        if let Some(parameters) = runtime_parameters {
            input.runtime_parameters = parameters
                .iter()
                .map(|p| crate::input::symbol(py, p, "runtime parameter"))
                .collect::<PyResult<_>>()?;
        }
        Ok(session::PyRecipeFamilySession::new(
            input, options, family, resident, settings,
        ))
    }
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<session::PyRecipeFamilySession>()?;
    module.add_class::<archive::PyRecipeArchive>()?;
    module.add_class::<snapshot::PyRecipeFamilySnapshot>()?;
    Ok(())
}

#[cfg(feature = "python_stubgen")]
pyo3_stub_gen::inventory::submit! {
    pyo3_stub_gen::derive::gen_methods_from_python! {
        r#"
import typing
import symbolica
import symbolica.community.hepkit.sector_decomposition

class PyIntegral:
    def generation_family_session(self, recipes: list[str], max_order: int = 0, *,
        default_recipe: str = "off", resident_recipe: typing.Optional[str] = None,
        coefficient_expansion: str = "coefficient_series", mode: str = "symbolic",
        subtraction: str = "taylor", contour_jacobian: str = "symbolic",
        compilation_settings: typing.Optional[symbolica.community.hepkit.sector_decomposition.CompilationSettings] = None,
        runtime_parameters: typing.Optional[list[symbolica.Expression]] = None,
    ) -> symbolica.community.hepkit.sector_decomposition.RecipeFamilySession:
        """Create inert caller-stepped native family generation with shared source preparation."""
"#
    }
}
