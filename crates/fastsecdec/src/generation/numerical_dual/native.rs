//! Compose native scalar IR for numerical maps and normalized endpoint jets.
//!
//! Original polynomial Atoms are compiled without a sector substitution. Native
//! Dualizer owns Taylor arithmetic; EvaluatorComposer connects its scalar IR.
mod cache;
mod jets;

use super::DualSector;
use crate::kernel::{CompilationSettings, KernelError};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{EvaluatorComposer, ExpressionEvaluator, Slot},
};

pub(crate) use super::subtraction::Coordinate;
pub(crate) use cache::SourcePrograms;
type ExactProgram = ExpressionEvaluator<Complex<Rational>>;
type Lowering<'a> = dyn FnMut(&Atom, &[Coordinate]) -> Result<Atom, KernelError> + 'a;

fn compilation(error: impl std::fmt::Display) -> KernelError {
    KernelError::Compilation(error.to_string())
}

pub(crate) fn build(
    sector: &DualSector,
    runtime: &[Symbol],
    settings: CompilationSettings,
) -> Result<ExactProgram, KernelError> {
    build_inner(sector, runtime, settings, None)
}

/// Lower callback metadata only after the complete mathematical subtraction
/// recipe exists, before native jets differentiate each selected smooth body.
///
/// The lowerer receives the unchanged full-sector coordinate projection. It
/// must preserve the mathematical expression and return a deterministic body
/// for that projection; it can be called for multiple derivative requests on
/// the same face. Native source-program caching includes the returned Atom.
/// Opaque undeformed source programs do not require this hook.
pub(crate) fn build_with_lowering(
    sector: &DualSector,
    runtime: &[Symbol],
    settings: CompilationSettings,
    lower: &mut Lowering<'_>,
) -> Result<ExactProgram, KernelError> {
    build_inner(sector, runtime, settings, Some(lower))
}

fn build_inner(
    sector: &DualSector,
    runtime: &[Symbol],
    settings: CompilationSettings,
    mut lower: Option<&mut Lowering<'_>>,
) -> Result<ExactProgram, KernelError> {
    settings.validate()?;
    let mut inputs = sector
        .parameters
        .iter()
        .chain(runtime)
        .copied()
        .collect::<Vec<_>>();
    let mut unique = std::collections::BTreeSet::new();
    if inputs.iter().any(|symbol| !unique.insert(*symbol)) || unique.contains(&sector.regulator) {
        return Err(compilation(
            "duplicate numerical-dual coordinate/runtime/regulator input",
        ));
    }
    if sector
        .map
        .exponent_matrix
        .iter()
        .flatten()
        .any(|power| power < &0)
    {
        return Err(compilation(
            "numerical-dual jets require nonnegative monomial maps",
        ));
    }
    let mut composer = EvaluatorComposer::new(inputs.len());
    let mut slots = (0..inputs.len()).map(Slot::Param).collect::<Vec<_>>();
    let mut builder = jets::Requests::new(sector, runtime, settings)?;
    for request in &sector.recipe.requests {
        let value = builder.append(request, &mut composer, &mut lower)?;
        inputs.push(request.placeholder);
        slots.push(value);
    }
    let coefficients = sector
        .orders
        .iter()
        .map(|order| {
            sector
                .recipe
                .coefficients
                .get(order)
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let roots = coefficients
        .iter()
        .map(|coefficient| coefficient.get_root())
        .collect::<Vec<_>>();
    let input_atoms = inputs
        .iter()
        .map(|symbol| Atom::var(*symbol))
        .collect::<Vec<_>>();
    let mut recipe =
        Atom::evaluator_multiple(&roots, &input_atoms).optimization_settings(settings.native());
    if let Some(aliases) = coefficients
        .iter()
        .map(|coefficient| coefficient.get_aliases())
        .find(|aliases| !aliases.is_empty())
    {
        if coefficients.iter().any(|coefficient| {
            !coefficient.get_aliases().is_empty() && coefficient.get_aliases() != aliases
        }) {
            return Err(compilation(
                "numerical-dual recipe has inconsistent native aliases",
            ));
        }
        let mut ordered = aliases.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.0.cmp(b.0));
        recipe = recipe
            .add_aliases(ordered.into_iter().map(|(a, b)| (a.clone(), b.clone())))
            .map_err(compilation)?;
    }
    let recipe = recipe.build().map_err(compilation)?;
    let outputs = composer.append(&recipe, &slots).map_err(compilation)?;
    composer
        .finish(&outputs, settings.native())
        .map_err(compilation)
}
