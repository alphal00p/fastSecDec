use super::{ExactProgram, compilation};
use crate::kernel::{CompilationSettings, KernelError};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::Dualizer,
};

/// Shared compilation of unmapped source factors and their native jet lowering.
/// The caller owns this cache; there is no pool or process-global program state.
#[derive(Default)]
pub(crate) struct SourcePrograms(Mutex<Vec<Source>>);
type JetKey = (Vec<Vec<usize>>, Vec<(usize, usize)>);

struct Source {
    polynomial: Atom,
    inputs: Vec<Symbol>,
    settings: CompilationSettings,
    exact: Arc<ExactProgram>,
    jets: BTreeMap<JetKey, Arc<ExactProgram>>,
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
        let mut sources = self
            .0
            .lock()
            .map_err(|_| compilation("source evaluator cache lock poisoned"))?;
        let index = if let Some(index) = sources.iter().position(|source| {
            source.polynomial == *polynomial
                && source.inputs == inputs
                && source.settings == settings
        }) {
            index
        } else {
            let params = inputs.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
            let exact = polynomial
                .evaluator(&params)
                .optimization_settings(settings.native())
                .build()
                .map_err(compilation)?;
            sources.push(Source {
                polynomial: polynomial.clone(),
                inputs: inputs.to_vec(),
                settings,
                exact: Arc::new(exact),
                jets: BTreeMap::new(),
            });
            sources.len() - 1
        };
        let source = &mut sources[index];
        let key = (shape.to_vec(), zeros.to_vec());
        if let Some(jets) = source.jets.get(&key) {
            return Ok(jets.clone());
        }
        let jets = Arc::new(
            source
                .exact
                .as_ref()
                .clone()
                .vectorize(&Dualizer::new(
                    HyperDual::<Complex<Rational>>::new(shape.to_vec()),
                    zeros.to_vec(),
                ))
                .map_err(compilation)?,
        );
        source.jets.insert(key, jets.clone());
        Ok(jets)
    }
}
