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
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::{EvaluatorComposer, ExpressionEvaluator},
};

pub(crate) use cache::SourcePrograms;
type ExactProgram = ExpressionEvaluator<Complex<Rational>>;

fn compilation(error: impl std::fmt::Display) -> KernelError {
    KernelError::Compilation(error.to_string())
}

// Preflight the actual maintained HyperDual constructor, which owns the dense
// multiplication table. The byte bound is conservative; it is not a count cap.
fn hyperdual(shape: Vec<Vec<usize>>) -> Result<HyperDual<Complex<Rational>>, KernelError> {
    symbolica::utils::resources::checkpoint()?;
    let count = shape.len() as u64;
    let table_bytes = count
        .saturating_mul(count)
        .saturating_mul(std::mem::size_of::<(usize, usize, usize)>() as u64);
    let value_bytes = count.saturating_mul(std::mem::size_of::<Complex<Rational>>() as u64);
    symbolica::utils::resources::check_allocation(table_bytes.saturating_add(value_bytes))?;
    let dual = HyperDual::new(shape);
    symbolica::utils::resources::checkpoint()?;
    Ok(dual)
}

pub(crate) fn build(
    sector: &DualSector,
    runtime: &[Symbol],
    settings: CompilationSettings,
) -> Result<ExactProgram, KernelError> {
    symbolica::utils::resources::checkpoint()?;
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
    let (mut composer, mut slots) = EvaluatorComposer::new_with_inputs(inputs.len());
    let mut builder = jets::Requests::new(sector, runtime, settings, slots.clone())?;
    for request in &sector.recipe.requests {
        symbolica::utils::resources::checkpoint()?;
        let value = builder.append(request, &mut composer)?;
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
    symbolica::utils::resources::checkpoint()?;
    let recipe = recipe.build().map_err(compilation)?;
    symbolica::utils::resources::checkpoint()?;
    let outputs = composer.add(&recipe, &slots).map_err(compilation)?;
    let result = composer
        .finish(&outputs, settings.native())
        .map_err(compilation)?;
    symbolica::utils::resources::checkpoint()?;
    Ok(result)
}
