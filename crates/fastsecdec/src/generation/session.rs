//! Cooperative generation using retained native work, without an executor.
mod stages;
use super::{
    ChartRecord, DomainAssessment, GeneratedIntegral, GenerationError, GenerationEvent,
    GenerationOptions, GenerationPhase, GenerationProgress, assembly::Assembly, coefficients,
    domain, laurent, mapping::MappedTerm, support::SupportCache, symmetry, work,
};
use crate::{
    parametric::ParametricIntegrand,
    status::{GenerationSnapshot, GenerationStage},
};
use fastsecdec_sectors::{
    GeometryCompletion, GeometryJob, GeometryPlan, PolynomialSupport, PreparedGeometry, SectorMap,
};
use std::{
    collections::{BTreeMap, VecDeque},
    ops::ControlFlow,
    time::Instant,
};
use symbolica::atom::Symbol;

/// A paused owner can resume by calling [`GenerationSession::step`] again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationSessionState {
    Pending,
    Paused,
    Complete,
}

type Representative = (
    SectorMap,
    Vec<Symbol>,
    Vec<MappedTerm>,
    usize,
    super::program::ProgramData,
);
type Representatives = BTreeMap<usize, Representative>;

enum Stage {
    NumericalDual(Box<super::numerical_dual::pipeline::Pipeline>),
    Admission,
    GeometryCharts {
        plan: GeometryPlan,
        jobs: VecDeque<GeometryJob>,
        completed: Vec<GeometryCompletion>,
    },
    GeometryPrepare {
        plan: GeometryPlan,
        completed: Vec<GeometryCompletion>,
    },
    GeometryCones {
        prepared: PreparedGeometry,
        jobs: VecDeque<GeometryJob>,
        completed: Vec<GeometryCompletion>,
    },
    GeometryFinish {
        prepared: PreparedGeometry,
        completed: Vec<GeometryCompletion>,
    },
    Mapping {
        maps: VecDeque<SectorMap>,
        parameters: Vec<Symbol>,
    },
    SymmetryPrepare {
        chart: Box<work::MappedChart>,
        maps: VecDeque<SectorMap>,
        parameters: Vec<Symbol>,
    },
    SymmetryRegister {
        prepared: Box<work::PreparedChart>,
        maps: VecDeque<SectorMap>,
        parameters: Vec<Symbol>,
    },
    Coefficients {
        remaining: std::collections::btree_map::IntoIter<usize, Representative>,
        index: usize,
        total: usize,
    },
    Finish,
    Complete,
    Failed,
}

/// Caller-driven generation with no hidden threads, pool or integration work.
///
/// Construction only stores validated input objects. Each step executes at most
/// the requested number of native units: geometry chart/cone work or merge,
/// mapping, symmetry preparation/admission, one representative's complete
/// coefficient expansion, or final assembly. A native algebra call is atomic:
/// this is a work-unit bound, not a wall-time guarantee. Completed coefficient
/// expansions and all earlier native work stay owned across pauses.
///
/// Returning `Break` from an observer requests a pause *after* the current unit.
/// It never discards successful native work. Errors are terminal and never
/// expose a partial generated integral. Use the ordinary generation entry when
/// aborting inside existing native progress boundaries is preferable to resume.
pub struct GenerationSession {
    input: ParametricIntegrand,
    options: GenerationOptions,
    stage: Stage,
    domain: Option<DomainAssessment>,
    supports: SupportCache,
    registry: symmetry::SymmetryRegistry,
    representatives: Representatives,
    charts: Vec<ChartRecord>,
    chart_count: usize,
    source_scope: Option<super::GenerationSourceScope>,
    templates: laurent::TemplateCache,
    assembly: Option<Assembly>,
    result: Option<GeneratedIntegral>,
    snapshot: GenerationSnapshot,
    completed_units: usize,
    completed_representatives: usize,
}

impl GenerationSession {
    pub fn new(input: ParametricIntegrand, options: GenerationOptions) -> Self {
        let supports = SupportCache::new(input.parameters());
        let assembly = Some(Assembly::new(options.max_order));
        Self {
            input,
            options,
            stage: Stage::Admission,
            domain: None,
            supports,
            registry: Default::default(),
            representatives: BTreeMap::new(),
            charts: Vec::new(),
            chart_count: 0,
            source_scope: None,
            templates: Default::default(),
            assembly,
            result: None,
            snapshot: GenerationSnapshot {
                stage: GenerationStage::Input,
                completed: 0,
                total: None,
                sectors: 0,
                kernels: 0,
                elapsed_seconds: 0.0,
                timings: Default::default(),
                coefficient_expansion: None,
                formula_preparation: None,
                detail: "Ready to generate".into(),
            },
            completed_units: 0,
            completed_representatives: 0,
        }
    }
    /// Elapsed and total timing count active step calls; paused wall time is excluded.
    pub fn snapshot(&self) -> &GenerationSnapshot {
        &self.snapshot
    }
    pub fn completed_units(&self) -> usize {
        self.completed_units
    }
    pub fn completed_representatives(&self) -> usize {
        self.completed_representatives
    }
    pub fn is_complete(&self) -> bool {
        matches!(self.stage, Stage::Complete)
    }
    /// Consume the completed scientific result at most once.
    pub fn take_result(&mut self) -> Option<GeneratedIntegral> {
        self.result.take()
    }

    pub fn step(
        &mut self,
        max_units: usize,
        mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
    ) -> Result<GenerationSessionState, GenerationError> {
        if max_units == 0 {
            return Err(GenerationError::Invariant(
                "generation step requires at least one work unit".into(),
            ));
        }
        if matches!(self.stage, Stage::Failed) {
            return Err(GenerationError::Invariant(
                "generation session failed; create a new session to retry".into(),
            ));
        }
        let started = Instant::now();
        let mut pause = false;
        for _ in 0..max_units {
            if self.is_complete() {
                break;
            }
            let mut snapshot = self.snapshot.clone();
            let max_order = self.options.max_order;
            let outcome = self.advance(&mut |event: &GenerationEvent| {
                if let GenerationEvent::Progress(status) = event {
                    snapshot.observe_generation(max_order, status);
                    pause |= progress(status).is_break();
                }
                // A resumable unit must retain its completed native result.
                ControlFlow::Continue(())
            });
            self.snapshot = snapshot;
            if let Err(error) = outcome {
                self.stage = Stage::Failed;
                self.snapshot.elapsed_seconds += started.elapsed().as_secs_f64();
                self.snapshot.timings.total_seconds = self.snapshot.elapsed_seconds;
                self.snapshot.detail = format!("Generation failed: {error}");
                return Err(error);
            }
            self.completed_units += 1;
            if pause {
                break;
            }
        }
        self.snapshot.elapsed_seconds += started.elapsed().as_secs_f64();
        self.snapshot.timings.total_seconds = self.snapshot.elapsed_seconds;
        Ok(if self.is_complete() {
            GenerationSessionState::Complete
        } else if pause {
            GenerationSessionState::Paused
        } else {
            GenerationSessionState::Pending
        })
    }
}
