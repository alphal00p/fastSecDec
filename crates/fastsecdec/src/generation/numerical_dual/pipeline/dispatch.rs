//! Caller-owned formula jobs and deterministic phase admission.
use super::*;

/// Opaque jobs use the same primitives as the cooperative pipeline. The caller
/// executor may reorder completions; the generation owner admits them by ID.
pub(in crate::generation) enum Task {
    Discover {
        context: Arc<Context>,
        map: SectorMap,
        index: usize,
        total: usize,
        supports: SupportCache,
    },
    Formula {
        context: Arc<Context>,
        key: Key,
        index: usize,
        counts: Counts,
    },
    Instantiate {
        context: Arc<Context>,
        chart: Box<DiscoveredChart>,
        recipe: Option<Arc<Recipe>>,
        total: usize,
        supports: SupportCache,
    },
}
pub(in crate::generation) enum TaskResult {
    Discovered(Box<DiscoveredChart>),
    Formula(Arc<Recipe>),
    Instantiated(Box<PreparedChart>),
}
impl Task {
    pub(in crate::generation) fn run(
        self,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<TaskResult, GenerationError> {
        match self {
            Self::Discover {
                context,
                map,
                index,
                total,
                mut supports,
            } => chart::discover(&context, map, index, total, &mut supports, progress)
                .map(|chart| TaskResult::Discovered(Box::new(chart))),
            Self::Formula {
                context,
                key,
                index,
                counts,
            } => formula::build(&key, &context, index, counts, progress).map(TaskResult::Formula),
            Self::Instantiate {
                context,
                chart,
                recipe,
                total,
                mut supports,
            } => chart::instantiate(&context, *chart, recipe, total, &mut supports, progress)
                .map(|chart| TaskResult::Instantiated(Box::new(chart))),
        }
    }
}

pub(in crate::generation::numerical_dual) fn dispatched(
    context: Arc<Context>,
    maps: Vec<SectorMap>,
    supports: &SupportCache,
    dispatch: &mut SymbolicDispatch<'_>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Vec<PreparedChart>, GenerationError> {
    let total = maps.len();
    emit(
        progress,
        GenerationProgress::NumericalMapping { sector: 0, total },
    )?;
    let started = Instant::now();
    let jobs = maps
        .into_iter()
        .enumerate()
        .map(|(index, map)| Task::Discover {
            context: context.clone(),
            map,
            index,
            total,
            supports: supports.clone(),
        })
        .collect();
    let charts =
        crate::generation::work::numerical_dual_tasks(jobs, SymbolicStage::Mapping, dispatch)?
            .into_iter()
            .map(|result| match result {
                TaskResult::Discovered(chart) => Ok(*chart),
                _ => Err(GenerationError::Invariant(
                    "wrong dual discovery completion".into(),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
    let plan = Plan::new(charts);
    emit(
        progress,
        GenerationProgress::NumericalMapping {
            sector: total,
            total,
        },
    )?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    emit(progress, plan.counts.event(0))?;
    let started = Instant::now();
    let jobs = plan
        .keys
        .into_iter()
        .enumerate()
        .map(|(index, key)| Task::Formula {
            context: context.clone(),
            key,
            index,
            counts: plan.counts,
        })
        .collect();
    let recipes = crate::generation::work::numerical_dual_tasks(
        jobs,
        SymbolicStage::FormulaPreparation,
        dispatch,
    )?
    .into_iter()
    .map(|result| match result {
        TaskResult::Formula(recipe) => Ok(recipe),
        _ => Err(GenerationError::Invariant(
            "wrong dual formula completion".into(),
        )),
    })
    .collect::<Result<Vec<_>, _>>()?;
    emit(progress, plan.counts.event(recipes.len()))?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::FormulaPreparation,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    emit(
        progress,
        GenerationProgress::FormulaInstantiation { sector: 0, total },
    )?;
    let started = Instant::now();
    let jobs = plan
        .charts
        .into_iter()
        .map(|usage| Task::Instantiate {
            context: context.clone(),
            chart: Box::new(usage.chart),
            recipe: usage.formula.map(|id| recipes[id].clone()),
            total,
            supports: supports.clone(),
        })
        .collect();
    let charts =
        crate::generation::work::numerical_dual_tasks(jobs, SymbolicStage::Coefficients, dispatch)?
            .into_iter()
            .map(|result| match result {
                TaskResult::Instantiated(chart) => Ok(*chart),
                _ => Err(GenerationError::Invariant(
                    "wrong dual instantiation completion".into(),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
    emit(
        progress,
        GenerationProgress::FormulaInstantiation {
            sector: charts.len(),
            total,
        },
    )?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::CoefficientExpansion,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    Ok(charts)
}
