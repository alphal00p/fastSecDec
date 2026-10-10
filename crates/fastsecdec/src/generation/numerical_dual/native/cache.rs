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
pub(crate) struct SourcePrograms(Mutex<Vec<Arc<Source>>>);
type JetKey = (Vec<Vec<usize>>, Vec<(usize, usize)>);
type ProgramCell = OnceLock<Result<Arc<ExactProgram>, String>>;

#[cfg(test)]
mod tests;

struct Source {
    polynomial: Atom,
    inputs: Vec<Symbol>,
    settings: CompilationSettings,
    definitions: Option<Arc<crate::contour::ContourDefinitions>>,
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
        self.source(polynomial, inputs, settings, None)?
            .jets(shape, zeros)
    }

    pub(super) fn jets_with_definitions(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        shape: &[Vec<usize>],
        zeros: &[(usize, usize)],
        settings: CompilationSettings,
        definitions: &Arc<crate::contour::ContourDefinitions>,
    ) -> Result<Arc<ExactProgram>, KernelError> {
        let definitions = (!definitions.is_empty()).then(|| definitions.clone());
        self.source(polynomial, inputs, settings, definitions)?
            .jets(shape, zeros)
    }

    fn source(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        settings: CompilationSettings,
        definitions: Option<Arc<crate::contour::ContourDefinitions>>,
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
        }) {
            return Ok(source.clone());
        }
        let source = Arc::new(Source {
            polynomial: polynomial.clone(),
            inputs: inputs.to_vec(),
            settings,
            definitions,
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
            let builder = self
                .polynomial
                .evaluator(&params)
                .optimization_settings(self.settings.native());
            let builder = if let Some(definitions) = &self.definitions {
                builder.function_map(definitions.function_map([&self.polynomial])?)
            } else {
                builder
            };
            builder.build().map_err(|error| error.to_string())
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
