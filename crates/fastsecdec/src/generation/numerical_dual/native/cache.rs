use super::{ExactProgram, compilation};
use crate::kernel::{CompilationSettings, KernelError};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    evaluate::Dualizer,
};

/// Shared compilation of unmapped source factors and their native jet lowering.
/// The caller owns this cache; there is no pool or process-global program state.
#[derive(Default)]
pub(crate) struct SourcePrograms(Mutex<Vec<Arc<Source>>>);
type JetKey = (Vec<Vec<usize>>, Vec<(usize, usize)>);
#[derive(Default)]
struct ProgramCell {
    value: OnceLock<Result<Arc<ExactProgram>, String>>,
    building: Mutex<()>,
}

#[cfg(test)]
mod tests;

struct Source {
    polynomial: Atom,
    inputs: Vec<Symbol>,
    settings: CompilationSettings,
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
        symbolica::utils::resources::checkpoint()?;
        let program = self
            .source(polynomial, inputs, settings)?
            .jets(shape, zeros)?;
        symbolica::utils::resources::checkpoint()?;
        Ok(program)
    }

    fn source(
        &self,
        polynomial: &Atom,
        inputs: &[Symbol],
        settings: CompilationSettings,
    ) -> Result<Arc<Source>, KernelError> {
        let mut sources = self
            .0
            .lock()
            .map_err(|_| compilation("source evaluator cache lock poisoned"))?;
        if let Some(source) = sources.iter().find(|source| {
            source.polynomial == *polynomial
                && source.inputs == inputs
                && source.settings == settings
        }) {
            return Ok(source.clone());
        }
        let source = Arc::new(Source {
            polynomial: polynomial.clone(),
            inputs: inputs.to_vec(),
            settings,
            exact: ProgramCell::default(),
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
            self.polynomial
                .evaluator(&params)
                .optimization_settings(self.settings.native())
                .build()
                .map_err(compilation)
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
                    super::hyperdual(shape.to_vec())?,
                    zeros.to_vec(),
                ))
                .map_err(compilation)
        })
    }
}

fn program(
    cell: &ProgramCell,
    build: impl FnOnce() -> Result<ExactProgram, KernelError>,
) -> Result<Arc<ExactProgram>, KernelError> {
    symbolica::utils::resources::checkpoint()?;
    if cell.value.get().is_none() {
        // One initializer per exact key; unrelated keys retain independent
        // locks. An unwound initializer publishes nothing and can be retried.
        let _initializing = cell
            .building
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        symbolica::utils::resources::checkpoint()?;
        if cell.value.get().is_none() {
            let value = match build() {
                Ok(program) => {
                    let program = Arc::new(program);
                    symbolica::utils::resources::checkpoint()?;
                    Ok(program)
                }
                // Resource state belongs to a request, never to an exact key.
                Err(KernelError::Resource(error)) => return Err(error.into()),
                Err(KernelError::Compilation(error)) => Err(error),
                Err(error) => Err(error.to_string()),
            };
            let _ = cell.value.set(value);
        }
    }
    cell.value
        .get()
        .ok_or_else(|| compilation("native program cell was not initialized"))?
        .as_ref()
        .map(Arc::clone)
        .map_err(compilation)
}
