//! Cooperative, file-backed family generation with caller-owned storage.
mod jobs;
mod stages;
pub use jobs::{
    RecipeFamilyCompletion, RecipeFamilyDispatch, RecipeFamilyJob, RecipeFamilyJobId,
    RecipeFamilyJobProgress, RecipeFamilyJobStage,
};
#[cfg(test)]
mod tests;
use super::{RecipeFamily, RecipeFamilyError};
use crate::{
    generation::{GenerationOptions, GenerationSessionState, streaming as native},
    kernel::{
        CompilationSettings, KernelError, KernelSet, PrecisionPolicy, RuntimeMassConstraint,
        indexed::{
            ProgramArchiveCatalogue, ProgramArchiveWriter, ProgramRecipe, ProgramResidentAssembly,
        },
    },
    parametric::ParametricIntegrand,
    status::{GenerationSnapshot, GenerationStage},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Seek, Write},
    ops::ControlFlow,
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use symbolica::atom::Symbol;

#[derive(Debug, thiserror::Error)]
pub enum RecipeFamilySessionError {
    #[error(transparent)]
    Request(#[from] RecipeFamilyError),
    #[error(transparent)]
    Generation(#[from] native::StreamingError),
    #[error(transparent)]
    Kernel(#[from] KernelError),
    #[error("recipe-family session: {0}")]
    State(String),
}

/// Frozen observations, separate from the live native owners and source IDs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecipeFamilySnapshot {
    pub recipe: Option<ProgramRecipe>,
    pub completed_recipes: usize,
    pub total_recipes: usize,
    pub completed_units: usize,
    pub prepared_sources: usize,
    pub persisted_units: usize,
    pub generation: GenerationSnapshot,
}

/// Available only after every requested recipe has been completely encoded into
/// caller-owned storage. Flushing/fsync/atomic publication remain caller-owned.
pub struct RecipeFamilyOutput<W> {
    pub writer: W,
    pub catalogue: ProgramArchiveCatalogue,
    pub family: RecipeFamily,
    pub resident: Option<KernelSet>,
}

/// Synchronous recipe-family generation without an executor or worker pool.
///
/// The caller owns `staging` and must keep it available across steps. Source
/// geometry and each monomial-extracted chart are prepared once; symmetry and
/// reusable formulas are local to each recipe. Sequential stepping retains one
/// transient unit; caller dispatch retains at most its bounded batch of completed
/// units. Nonresident units are released before the next batch. The optional
/// resident recipe retains its existing evaluators and only that recipe's
/// portable bytes, independently of the artifact default.
///
/// A step counts indivisible native units, not milliseconds. Observer `Break`
/// pauses after the successful current unit or joined batch. Errors are terminal, and no
/// partial archive is exposed as a completed output. For ordinary singleton
/// work the existing `GenerationSession` remains the direct, unstaged route.
pub struct RecipeFamilySession<W> {
    work: Work<W>,
    snapshot: RecipeFamilySnapshot,
    result: Option<RecipeFamilyOutput<W>>,
}

struct Input {
    integrand: ParametricIntegrand,
    runtime: Vec<Symbol>,
    constraints: Vec<RuntimeMassConstraint>,
}
#[derive(Clone, Copy)]
enum Stage {
    Prepare,
    Source(usize),
    BeginRecipe(usize),
    Map(usize),
    Symmetry(usize),
    Formula(usize),
    Plan,
    Sector(usize),
    FinishRecipe,
    Finish,
    Complete,
    Failed,
}
struct RecipeWork {
    index: usize,
    charts: Vec<native::DiscoveredSector>,
    representatives: BTreeMap<String, Vec<native::DiscoveredSector>>,
    assignments: Vec<native::SymmetryAssignment>,
    formula_sources: Vec<native::DiscoveredSector>,
    formulas: Vec<native::FormulaRecord>,
    plan: Option<Arc<native::PreparedGeneration>>,
}
struct Work<W> {
    stage: Stage,
    input: Option<Input>,
    options: GenerationOptions,
    family: RecipeFamily,
    resident_recipe: Option<ProgramRecipe>,
    resident: Option<ProgramResidentAssembly>,
    precision: PrecisionPolicy,
    compilation: CompilationSettings,
    staging: PathBuf,
    storage: Option<W>,
    archive: Option<ProgramArchiveWriter<W>>,
    prepared: Option<Arc<native::PreparedRecipeSet>>,
    sources: Vec<native::PreparedChartSource>,
    active: Option<RecipeWork>,
}

impl<W: Write + Seek> RecipeFamilySession<W> {
    /// Construction stores inputs; the first step performs native admission and
    /// preparation. No staging file, evaluator or background work is created.
    pub fn new(
        input: ParametricIntegrand,
        options: GenerationOptions,
        family: RecipeFamily,
        staging: PathBuf,
        writer: W,
    ) -> Self {
        let total_recipes = family.recipes().len();
        Self {
            work: Work {
                stage: Stage::Prepare,
                input: Some(Input {
                    integrand: input,
                    runtime: vec![],
                    constraints: vec![],
                }),
                options,
                family,
                resident_recipe: None,
                resident: None,
                precision: PrecisionPolicy::default(),
                compilation: CompilationSettings::default(),
                staging,
                storage: Some(writer),
                archive: None,
                prepared: None,
                sources: vec![],
                active: None,
            },
            snapshot: RecipeFamilySnapshot {
                recipe: None,
                completed_recipes: 0,
                total_recipes,
                completed_units: 0,
                prepared_sources: 0,
                persisted_units: 0,
                generation: GenerationSnapshot {
                    stage: GenerationStage::Input,
                    completed: 0,
                    total: None,
                    sectors: 0,
                    kernels: 0,
                    elapsed_seconds: 0.0,
                    timings: Default::default(),
                    coefficient_expansion: None,
                    formula_preparation: None,
                    detail: "Ready to prepare recipe family".into(),
                },
            },
            result: None,
        }
    }
    fn configuring(&self) -> Result<(), RecipeFamilySessionError> {
        if self.snapshot.completed_units != 0 || !matches!(self.work.stage, Stage::Prepare) {
            return Err(RecipeFamilySessionError::State(
                "settings cannot change after the first step".into(),
            ));
        }
        Ok(())
    }
    pub fn with_resident_recipe(
        mut self,
        recipe: Option<ProgramRecipe>,
    ) -> Result<Self, RecipeFamilySessionError> {
        self.configuring()?;
        self.work.family.validate_resident(recipe)?;
        self.work.resident_recipe = recipe;
        Ok(self)
    }
    pub fn with_runtime_inputs(
        mut self,
        parameters: Vec<Symbol>,
        constraints: Vec<RuntimeMassConstraint>,
    ) -> Result<Self, RecipeFamilySessionError> {
        self.configuring()?;
        let input = self.work.input.as_mut().unwrap();
        input.runtime = parameters;
        input.constraints = constraints;
        Ok(self)
    }
    pub fn with_evaluator(
        mut self,
        precision: PrecisionPolicy,
        compilation: CompilationSettings,
    ) -> Result<Self, RecipeFamilySessionError> {
        self.configuring()?;
        precision.validate()?;
        compilation.validate()?;
        self.work.precision = precision;
        self.work.compilation = compilation;
        Ok(self)
    }
    pub fn snapshot(&self) -> &RecipeFamilySnapshot {
        &self.snapshot
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.work.stage, Stage::Complete)
    }
    pub fn take_result(&mut self) -> Option<RecipeFamilyOutput<W>> {
        self.result.take()
    }

    pub fn step(
        &mut self,
        max_units: usize,
        observer: impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
    ) -> Result<GenerationSessionState, RecipeFamilySessionError> {
        self.step_inner(max_units, 1, None, None, observer)
    }

    /// Schedule bounded independent native units on the caller's executor.
    /// Completion order is arbitrary; admission and publication are canonical.
    /// Observer pause joins the current batch before returning. At most
    /// `width` completed units coexist with the requested resident owner.
    pub fn step_with_dispatch(
        &mut self,
        max_units: usize,
        width: usize,
        geometry_dispatch: &mut fastsecdec_sectors::GeometryDispatch<'_>,
        dispatch: &mut RecipeFamilyDispatch<'_>,
        observer: impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
    ) -> Result<GenerationSessionState, RecipeFamilySessionError> {
        self.step_inner(
            max_units,
            width,
            Some(geometry_dispatch),
            Some(dispatch),
            observer,
        )
    }

    fn step_inner(
        &mut self,
        max_units: usize,
        width: usize,
        mut geometry_dispatch: Option<&mut fastsecdec_sectors::GeometryDispatch<'_>>,
        mut dispatch: Option<&mut RecipeFamilyDispatch<'_>>,
        mut observer: impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
    ) -> Result<GenerationSessionState, RecipeFamilySessionError> {
        if max_units == 0 || width == 0 {
            return Err(RecipeFamilySessionError::State(
                "step requires positive units and dispatch width".into(),
            ));
        }
        if matches!(self.work.stage, Stage::Failed) {
            return Err(RecipeFamilySessionError::State(
                "failed session cannot resume".into(),
            ));
        }
        let started = Instant::now();
        let previous_seconds = self.snapshot.generation.elapsed_seconds;
        let mut live_observer = |snapshot: &RecipeFamilySnapshot| {
            let mut visible = snapshot.clone();
            visible.generation.elapsed_seconds = previous_seconds + started.elapsed().as_secs_f64();
            visible.generation.timings.total_seconds = visible.generation.elapsed_seconds;
            observer(&visible)
        };
        let mut pause = false;
        let mut units = 0;
        while units < max_units {
            if self.is_complete() {
                break;
            }
            let outcome = self.work.advance(
                width.min(max_units - units),
                geometry_dispatch.as_deref_mut(),
                dispatch.as_deref_mut(),
                &mut self.snapshot,
                &mut live_observer,
                &mut pause,
            );
            match outcome {
                Ok((output, consumed)) => {
                    units += consumed;
                    self.snapshot.completed_units += consumed;
                    if let Some(output) = output {
                        self.result = Some(output);
                    }
                }
                Err(error) => {
                    self.work.stage = Stage::Failed;
                    self.snapshot.generation.elapsed_seconds += started.elapsed().as_secs_f64();
                    self.snapshot.generation.timings.total_seconds =
                        self.snapshot.generation.elapsed_seconds;
                    self.snapshot.generation.detail = format!("Generation failed: {error}");
                    return Err(error);
                }
            }
            pause |= live_observer(&self.snapshot).is_break();
            if pause {
                break;
            }
        }
        self.snapshot.generation.elapsed_seconds += started.elapsed().as_secs_f64();
        self.snapshot.generation.timings.total_seconds = self.snapshot.generation.elapsed_seconds;
        Ok(if self.is_complete() {
            GenerationSessionState::Complete
        } else if pause {
            GenerationSessionState::Paused
        } else {
            GenerationSessionState::Pending
        })
    }
}
