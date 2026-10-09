//! Caller-driven discovery, unique-formula construction and chart assembly.
mod dispatch;
pub(super) use dispatch::dispatched;
pub(in crate::generation) use dispatch::{Task, TaskResult};
#[cfg(test)]
mod tests;
use super::{
    PreparedChart,
    chart::{self, DiscoveredChart},
    formula::{self, Counts, Key},
    subtraction::Recipe,
};
use crate::{
    generation::{
        GenerationError, GenerationEvent, GenerationOptions, GenerationPhase, GenerationProgress,
        SymbolicDispatch, SymbolicStage, context::emit, support::SupportCache,
    },
    parametric::ParametricIntegrand,
};
use fastsecdec_sectors::SectorMap;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ops::ControlFlow,
    sync::Arc,
    time::Instant,
};
use symbolica::atom::{Atom, AtomCore, Symbol};

pub(in crate::generation) struct Context {
    pub input: ParametricIntegrand,
    pub options: GenerationOptions,
    pub parameters: Vec<Symbol>,
    pub reserved_symbols: Vec<Symbol>,
    pub programs: Arc<super::native::SourcePrograms>,
    pub valuations: Arc<super::ValuationCache>,
    pub eligible: bool,
}
impl Context {
    pub(in crate::generation) fn prepare(
        input: &ParametricIntegrand,
        options: &GenerationOptions,
        parameters: Vec<Symbol>,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<Self, GenerationError> {
        let started = Instant::now();
        let context = Self::new(input, options, parameters);
        emit(
            progress,
            GenerationProgress::PhaseTiming {
                phase: GenerationPhase::Mapping,
                seconds: started.elapsed().as_secs_f64(),
            },
        )?;
        Ok(context)
    }
    pub(in crate::generation) fn new(
        input: &ParametricIntegrand,
        options: &GenerationOptions,
        parameters: Vec<Symbol>,
    ) -> Self {
        let epsilon = Atom::var(input.regulator());
        let polynomial_in_epsilon = |value: &Atom| {
            !value.contains(epsilon.as_view())
                || value.is_polynomial(true, false).is_some_and(|variables| {
                    variables.iter().all(|variable| {
                        *variable == epsilon.as_view() || !variable.contains(epsilon.as_view())
                    })
                })
        };
        let eligible = !parameters.is_empty()
            && !input
                .terms()
                .iter()
                .flat_map(|term| term.factors())
                .any(|factor| {
                    (crate::generation::domain::is_singular(factor)
                        && factor.polynomial().contains(epsilon.as_view()))
                        || !polynomial_in_epsilon(factor.polynomial())
                        || !polynomial_in_epsilon(factor.exponent())
                });
        // Reserve symbols from every source field, even when the complete
        // density cancels them. Formula skeletons intentionally omit bodies.
        let mut reserved_symbols = input.parameters().iter().copied().collect::<BTreeSet<_>>();
        reserved_symbols.insert(input.regulator());
        reserved_symbols.extend(
            options
                .program_recipe
                .recipe_parameters()
                .iter()
                .map(|name| symbolica::symbol!(*name)),
        );
        for term in input.terms() {
            for expression in std::iter::once(term.prefactor())
                .chain(term.monomial_powers())
                .chain(
                    term.factors()
                        .iter()
                        .flat_map(|factor| [factor.polynomial(), factor.exponent()]),
                )
            {
                reserved_symbols.extend(expression.get_all_symbols(true));
            }
        }
        Self {
            input: input.clone(),
            options: options.clone(),
            parameters,
            reserved_symbols: reserved_symbols.into_iter().collect(),
            programs: Default::default(),
            valuations: Arc::new(super::ValuationCache::new(input.parameters())),
            eligible,
        }
    }
}

struct Use {
    chart: DiscoveredChart,
    formula: Option<usize>,
}
struct Plan {
    charts: VecDeque<Use>,
    keys: VecDeque<Key>,
    counts: Counts,
}
impl Plan {
    fn new(charts: Vec<DiscoveredChart>) -> Self {
        let mut keys = charts
            .iter()
            .filter_map(|chart| chart.key.clone())
            .map(|key| (key, 0))
            .collect::<BTreeMap<_, _>>();
        for (index, (_, id)) in keys.iter_mut().enumerate() {
            *id = index;
        }
        let sectors = charts.iter().filter(|chart| chart.key.is_some()).count();
        let charts = charts
            .into_iter()
            .map(|chart| {
                let formula = chart.key.as_ref().map(|key| keys[key]);
                Use { chart, formula }
            })
            .collect();
        Self {
            charts,
            counts: Counts {
                total: keys.len(),
                sectors,
            },
            keys: keys.into_keys().collect(),
        }
    }
}

enum Stage {
    Discover {
        maps: VecDeque<SectorMap>,
        charts: Vec<DiscoveredChart>,
    },
    Formulas {
        plan: Plan,
        recipes: Vec<Arc<Recipe>>,
    },
    Instantiate {
        charts: VecDeque<Use>,
        recipes: Vec<Arc<Recipe>>,
    },
    Complete,
}

/// Retains every completed discovery, formula and chart across caller pauses.
pub(in crate::generation) struct Pipeline {
    context: Arc<Context>,
    total: usize,
    stage: Stage,
    completed: Vec<PreparedChart>,
}
impl Pipeline {
    pub(in crate::generation) fn new(context: Arc<Context>, maps: Vec<SectorMap>) -> Self {
        Self {
            context,
            total: maps.len(),
            stage: Stage::Discover {
                maps: maps.into(),
                charts: Vec::new(),
            },
            completed: Vec::new(),
        }
    }
    pub(in crate::generation) fn is_complete(&self) -> bool {
        matches!(self.stage, Stage::Complete)
    }
    pub(in crate::generation) fn completed_charts(&self) -> usize {
        self.completed.len()
    }
    pub(in crate::generation) fn take_result(&mut self) -> Vec<PreparedChart> {
        std::mem::take(&mut self.completed)
    }
    pub(in crate::generation) fn step(
        &mut self,
        supports: &mut SupportCache,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<(), GenerationError> {
        let started = Instant::now();
        let stage = std::mem::replace(&mut self.stage, Stage::Complete);
        let (next, phase) = match stage {
            Stage::Discover {
                mut maps,
                mut charts,
            } => {
                if let Some(map) = maps.pop_front() {
                    charts.push(chart::discover(
                        &self.context,
                        map,
                        charts.len(),
                        self.total,
                        supports,
                        progress,
                    )?);
                    emit(
                        progress,
                        GenerationProgress::NumericalMapping {
                            sector: charts.len(),
                            total: self.total,
                        },
                    )?;
                    (Stage::Discover { maps, charts }, GenerationPhase::Mapping)
                } else {
                    let plan = Plan::new(charts);
                    emit(progress, plan.counts.event(0))?;
                    (
                        Stage::Formulas {
                            plan,
                            recipes: Vec::new(),
                        },
                        GenerationPhase::Mapping,
                    )
                }
            }
            Stage::Formulas {
                mut plan,
                mut recipes,
            } => {
                if let Some(key) = plan.keys.pop_front() {
                    recipes.push(formula::build(
                        &key,
                        &self.context,
                        recipes.len(),
                        plan.counts,
                        progress,
                    )?);
                    emit(progress, plan.counts.event(recipes.len()))?;
                    (
                        Stage::Formulas { plan, recipes },
                        GenerationPhase::FormulaPreparation,
                    )
                } else {
                    emit(
                        progress,
                        GenerationProgress::FormulaInstantiation {
                            sector: 0,
                            total: self.total,
                        },
                    )?;
                    (
                        Stage::Instantiate {
                            charts: plan.charts,
                            recipes,
                        },
                        GenerationPhase::FormulaPreparation,
                    )
                }
            }
            Stage::Instantiate {
                mut charts,
                recipes,
            } => {
                if let Some(usage) = charts.pop_front() {
                    let recipe = usage.formula.map(|id| recipes[id].clone());
                    self.completed.push(chart::instantiate(
                        &self.context,
                        usage.chart,
                        recipe,
                        self.total,
                        supports,
                        progress,
                    )?);
                    emit(
                        progress,
                        GenerationProgress::FormulaInstantiation {
                            sector: self.completed.len(),
                            total: self.total,
                        },
                    )?;
                    (
                        Stage::Instantiate { charts, recipes },
                        GenerationPhase::CoefficientExpansion,
                    )
                } else {
                    (Stage::Complete, GenerationPhase::CoefficientExpansion)
                }
            }
            Stage::Complete => return Ok(()),
        };
        self.stage = next;
        emit(
            progress,
            GenerationProgress::PhaseTiming {
                phase,
                seconds: started.elapsed().as_secs_f64(),
            },
        )
    }
}
