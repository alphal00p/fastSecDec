use super::{ExactProgram, compilation};
use crate::kernel::{CompilationSettings, KernelError};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::Dualizer,
};

/// Shared compilation of unmapped source factors and their native jet lowering.
/// The caller owns this cache; there is no pool or process-global program state.
#[derive(Default)]
pub(crate) struct SourcePrograms(Mutex<Vec<Arc<Source>>>, Mutex<Vec<JacobianEntry>>);
mod jacobian;

type JetKey = (Vec<Vec<usize>>, Vec<(usize, usize)>);
type ProgramCell = OnceLock<Result<Arc<ExactProgram>, String>>;
type JacobianEntry = (JacobianKey, Arc<ProgramCell>);

#[derive(PartialEq, Eq)]
struct JacobianKey {
    inputs: Vec<Symbol>,
    settings: CompilationSettings,
    definitions: Option<Arc<crate::contour::ContourDefinitions>>,
    plan: Arc<crate::contour::ContourJacobianPlan>,
}

#[cfg(test)]
mod tests;

struct Source {
    polynomial: Atom,
    inputs: Vec<Symbol>,
    settings: CompilationSettings,
    definitions: Option<Arc<crate::contour::ContourDefinitions>>,
    jacobian: Option<Arc<crate::contour::ContourJacobianPlan>>,
    jacobian_program: Option<Arc<ProgramCell>>,
    exact: ProgramCell,
    jets: Mutex<BTreeMap<JetKey, Arc<ProgramCell>>>,
}
impl std::fmt::Debug for SourcePrograms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourcePrograms").finish_non_exhaustive()
    }
}
impl SourcePrograms {
    pub(super) fn jets(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        shape: &[Vec<usize>],
        zeros: &[(usize, usize)],
        settings: CompilationSettings,
    ) -> Result<Arc<ExactProgram>, KernelError> {
        self.source(polynomial, inputs, settings, None, None)?
            .jets(shape, zeros)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn jets_with_definitions(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        shape: &[Vec<usize>],
        zeros: &[(usize, usize)],
        settings: CompilationSettings,
        definitions: &Arc<crate::contour::ContourDefinitions>,
        jacobian: Option<Arc<crate::contour::ContourJacobianPlan>>,
    ) -> Result<Arc<ExactProgram>, KernelError> {
        let definitions = (!definitions.is_empty()).then(|| definitions.clone());
        self.source(polynomial, inputs, settings, definitions, jacobian)?
            .jets(shape, zeros)
    }

    fn source(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        settings: CompilationSettings,
        definitions: Option<Arc<crate::contour::ContourDefinitions>>,
        jacobian: Option<Arc<crate::contour::ContourJacobianPlan>>,
    ) -> Result<Arc<Source>, KernelError> {
        let mut sources = self
            .0
            .lock()
            .map_err(|_| compilation("source evaluator cache lock poisoned"))?;
        if let Some(source) = sources.iter().find(|source| {
            source.polynomial == *polynomial
                && source.inputs == inputs
                && source.settings == settings
                && source.definitions == definitions
                && source.jacobian == jacobian
        }) {
            return Ok(source.clone());
        }
        let jacobian_program = if let Some(plan) = &jacobian {
            let key = JacobianKey {
                inputs: inputs.to_vec(),
                settings,
                definitions: definitions.clone(),
                plan: plan.clone(),
            };
            let mut prefixes = self
                .1
                .lock()
                .map_err(|_| compilation("Jacobian evaluator cache lock poisoned"))?;
            if let Some((_, cell)) = prefixes.iter().find(|(existing, _)| existing == &key) {
                Some(cell.clone())
            } else {
                let cell = Arc::new(ProgramCell::new());
                prefixes.push((key, cell.clone()));
                Some(cell)
            }
        } else {
            None
        };
        let source = Arc::new(Source {
            polynomial: polynomial.clone(),
            inputs: inputs.to_vec(),
            settings,
            definitions,
            jacobian,
            jacobian_program,
            exact: OnceLock::new(),
            jets: Mutex::default(),
        });
        sources.push(source.clone());
        Ok(source)
    }
}

impl Source {
    fn jets(
        &self,
        shape: &[Vec<usize>],
        zeros: &[(usize, usize)],
    ) -> Result<Arc<ExactProgram>, KernelError> {
        // Index locks only publish cells. Native builds execute after their
        // guards have been dropped, so unrelated source/jet keys can progress.
        let exact = program(&self.exact, || {
            let params = self
                .inputs
                .iter()
                .map(|s| Atom::var(*s))
                .collect::<Vec<_>>();
            if let Some(plan) = &self.jacobian
                && self.polynomial.contains(plan.jacobian.as_view())
            {
                // The image derivatives and determinant depend on the full
                // face-local plan, not on this density body's polynomial.
                // Build outside both index locks and retain only caller-owned IR.
                let prefix = program(
                    self.jacobian_program
                        .as_ref()
                        .ok_or("missing Jacobian evaluator cache cell")?,
                    || jacobian::build_prefix(self, plan),
                )
                .map_err(|error| error.to_string())?;
                jacobian::append_body(self, plan, &prefix)
            } else {
                jacobian::build_outputs(self, std::slice::from_ref(&self.polynomial), &params)
            }
        })?;
        let cell = {
            let mut jets = self
                .jets
                .lock()
                .map_err(|_| compilation("source jet cache lock poisoned"))?;
            jets.entry((shape.to_vec(), zeros.to_vec()))
                .or_default()
                .clone()
        };
        program(&cell, || {
            exact
                .as_ref()
                .clone()
                .vectorize(&Dualizer::new(
                    HyperDual::<Complex<Rational>>::new(shape.to_vec()),
                    zeros.to_vec(),
                ))
                .map_err(|error| error.to_string())
        })
    }
}

fn program(
    cell: &ProgramCell,
    build: impl FnOnce() -> Result<ExactProgram, String>,
) -> Result<Arc<ExactProgram>, KernelError> {
    // Native construction failures are immutable for this exact key. A panic
    // propagates without poisoning OnceLock; caller job handling owns unwind.
    cell.get_or_init(|| build().map(Arc::new))
        .as_ref()
        .map(Arc::clone)
        .map_err(compilation)
}
