//! Opaque symbolic jobs; the caller owns threads, cancellation and scheduling.
//! Workers invoke the existing Symbolica mapping and Laurent pipeline unchanged.
use super::program::ProgramData;
use super::{
    CoordinateMap, GenerationError, GenerationEvent, GenerationOptions, GenerationPhase,
    GenerationProgress, PreSubtractionMetadata, coefficients, context::emit, laurent, mapping,
    support::SupportCache, symmetry,
};
use crate::parametric::ParametricIntegrand;
use fastsecdec_sectors::SectorMap;
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc, time::Instant};
use symbolica::atom::Symbol;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolicStage {
    Mapping,
    FormulaPreparation,
    Symmetry,
    Coefficients,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolicJobId {
    pub stage: SymbolicStage,
    pub index: usize,
}

pub type SymbolicDispatch<'a> = dyn FnMut(
        &mut dyn ExactSizeIterator<Item = SymbolicJob>,
    ) -> Result<Vec<SymbolicCompletion>, GenerationError>
    + 'a;

pub struct SymbolicJob {
    owner: Arc<()>,
    id: SymbolicJobId,
    work: Work,
}
pub struct SymbolicCompletion {
    owner: Arc<()>,
    id: SymbolicJobId,
    result: Output,
}
enum Work {
    NumericalDual(super::numerical_dual::pipeline::Task),
    Mapping {
        input: Arc<ParametricIntegrand>,
        options: Arc<GenerationOptions>,
        map: SectorMap,
        parameters: Vec<Symbol>,
        supports: SupportCache,
    },
    Symmetry {
        chart: Box<MappedChart>,
        total: usize,
    },
    Coefficients {
        options: Arc<GenerationOptions>,
        regulator: Symbol,
        total: usize,
        representative: usize,
        map: SectorMap,
        parameters: Vec<Symbol>,
        mapped: Vec<mapping::MappedTerm>,
        multiplicity: usize,
        program: ProgramData,
    },
}
enum Output {
    NumericalDual(super::numerical_dual::pipeline::TaskResult),
    Mapping(Box<MappedChart>),
    Symmetry(Box<PreparedChart>),
    Coefficients(Box<ExpandedChart>),
}
pub(super) struct MappedChart {
    pub program: ProgramData,
    pub map: SectorMap,
    pub parameters: Vec<Symbol>,
    pub coordinates: CoordinateMap,
    pub mapped: Vec<mapping::MappedTerm>,
    pub pre_subtraction: Option<PreSubtractionMetadata>,
    pub contour: Option<crate::contour::ContourMetadata>,
}
pub(super) struct PreparedChart {
    pub chart: MappedChart,
    pub symmetry: symmetry::PreparedDensity,
}
type ExpandedChart = (
    usize,
    SectorMap,
    Vec<Symbol>,
    usize,
    coefficients::Output,
    ProgramData,
);
type Representatives = BTreeMap<
    usize,
    (
        SectorMap,
        Vec<Symbol>,
        Vec<mapping::MappedTerm>,
        usize,
        ProgramData,
    ),
>;

impl SymbolicJob {
    pub fn id(&self) -> SymbolicJobId {
        self.id
    }
    /// Execute one native job; no worker pool is constructed by the library.
    pub fn run(
        self,
        mut progress: impl FnMut(&GenerationProgress) -> ControlFlow<()>,
    ) -> Result<SymbolicCompletion, GenerationError> {
        let mut observe = |event: &GenerationEvent| match event {
            GenerationEvent::Progress(status) => progress(status),
            GenerationEvent::GeometryReuse(_) => ControlFlow::Continue(()),
        };
        let result = (|| match self.work {
            Work::NumericalDual(task) => task.run(&mut observe).map(Output::NumericalDual),
            Work::Mapping {
                input,
                options,
                map,
                parameters,
                mut supports,
            } => {
                emit(
                    &mut observe,
                    GenerationProgress::Factorization {
                        sector: self.id.index,
                        total: 0,
                    },
                )?;
                map_chart(
                    &input,
                    &options,
                    map,
                    parameters,
                    &mut supports,
                    &mut observe,
                )
                .map(|chart| Output::Mapping(Box::new(chart)))
            }
            Work::Symmetry { chart, total } => {
                prepare_symmetry(self.id.index, *chart, total, &mut observe)
                    .map(|prepared| Output::Symmetry(Box::new(prepared)))
            }
            Work::Coefficients {
                options,
                regulator,
                total,
                representative,
                map,
                parameters,
                mapped,
                multiplicity,
                program,
            } => {
                let mut templates = laurent::TemplateCache::default();
                #[cfg(test)]
                {
                    laurent::profiling::context(self.id.index, representative, multiplicity);
                    laurent::profiling::before_subtraction(
                        &mapped,
                        &parameters,
                        regulator,
                        options.max_order,
                    )?;
                }
                let output = coefficients::expand(
                    mapped,
                    coefficients::Representative {
                        parameters: &parameters,
                        regulator,
                        index: self.id.index,
                        total,
                    },
                    &options,
                    &mut templates,
                    &mut observe,
                )?;
                emit(
                    &mut observe,
                    GenerationProgress::PhaseTiming {
                        phase: output.phase,
                        seconds: output.phase_started.elapsed().as_secs_f64(),
                    },
                )?;
                Ok(Output::Coefficients(Box::new((
                    representative,
                    map,
                    parameters,
                    multiplicity,
                    output,
                    program,
                ))))
            }
        })();
        Ok(SymbolicCompletion {
            owner: self.owner,
            id: self.id,
            result: result?,
        })
    }
}

pub(super) fn numerical_dual_tasks(
    tasks: Vec<super::numerical_dual::pipeline::Task>,
    stage: SymbolicStage,
    dispatch: &mut SymbolicDispatch<'_>,
) -> Result<Vec<super::numerical_dual::pipeline::TaskResult>, GenerationError> {
    // Empty phases have known zero work and do not require an executor call.
    if tasks.is_empty() {
        return Ok(Vec::new());
    }
    let owner = Arc::new(());
    let total = tasks.len();
    let mut jobs = tasks
        .into_iter()
        .enumerate()
        .map(|(index, task)| SymbolicJob {
            owner: owner.clone(),
            id: SymbolicJobId { stage, index },
            work: Work::NumericalDual(task),
        });
    admit(&owner, stage, total, dispatch(&mut jobs)?)?
        .into_iter()
        .map(|output| match output {
            Output::NumericalDual(result) => Ok(result),
            _ => Err(GenerationError::Invariant(
                "wrong numerical dual completion kind".into(),
            )),
        })
        .collect()
}

pub(super) fn map_chart(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    map: SectorMap,
    parameters: Vec<Symbol>,
    supports: &mut SupportCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<MappedChart, GenerationError> {
    map_chart_with(
        input,
        options,
        map,
        parameters,
        progress,
        |map, coordinates| {
            mapping::map_terms_with_contour(
                input,
                map,
                coordinates,
                supports,
                options.program_recipe,
                true,
            )
        },
    )
}

pub(super) fn map_prepared_chart(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    map: SectorMap,
    parameters: Vec<Symbol>,
    terms: Vec<mapping::PreparedTerm>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<MappedChart, GenerationError> {
    map_chart_with(
        input,
        options,
        map,
        parameters,
        progress,
        |_, coordinates| {
            mapping::apply_prepared(
                coordinates.target_parameters(),
                terms,
                options.program_recipe,
                true,
            )
        },
    )
}

fn map_chart_with(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    map: SectorMap,
    parameters: Vec<Symbol>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    build: impl FnOnce(&SectorMap, &CoordinateMap) -> Result<mapping::MappedOutput, GenerationError>,
) -> Result<MappedChart, GenerationError> {
    let started = Instant::now();
    let coordinates = mapping::coordinates(input, &map, &parameters);
    let (mapped, mut contour, program) = build(&map, &coordinates)?;
    let pre_subtraction = Some(PreSubtractionMetadata::capture(
        &mapped,
        input.regulator(),
        options.max_subtractions_per_axis,
    )?);
    if let Some(contour) = &mut contour {
        contour.record_subtraction_faces(pre_subtraction.as_ref().unwrap(), options.subtraction);
    }
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    Ok(MappedChart {
        program,
        map,
        parameters,
        coordinates,
        mapped,
        pre_subtraction,
        contour,
    })
}

fn admit(
    owner: &Arc<()>,
    stage: SymbolicStage,
    total: usize,
    mut completions: Vec<SymbolicCompletion>,
) -> Result<Vec<Output>, GenerationError> {
    if completions.len() != total
        || completions
            .iter()
            .any(|value| !Arc::ptr_eq(owner, &value.owner) || value.id.stage != stage)
    {
        return Err(GenerationError::Invariant(
            "symbolic dispatcher returned foreign or incomplete work".into(),
        ));
    }
    completions.sort_by_key(|value| value.id.index);
    let mut output = Vec::with_capacity(total);
    for (index, completion) in completions.into_iter().enumerate() {
        if completion.id.index != index {
            return Err(GenerationError::Invariant(
                "symbolic dispatcher duplicated or omitted work".into(),
            ));
        }
        let result = completion.result;
        output.push(result);
    }
    Ok(output)
}

pub(super) fn map_dispatched(
    input: &ParametricIntegrand,
    options: &GenerationOptions,
    maps: Vec<SectorMap>,
    parameters: &[Symbol],
    supports: &SupportCache,
    dispatch: &mut SymbolicDispatch<'_>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Vec<MappedChart>, GenerationError> {
    let owner = Arc::new(());
    let input = Arc::new(input.clone());
    let options = Arc::new(options.clone());
    let total = maps.len();
    let mut jobs = maps
        .into_iter()
        .enumerate()
        .map(|(index, map)| SymbolicJob {
            owner: Arc::clone(&owner),
            id: SymbolicJobId {
                stage: SymbolicStage::Mapping,
                index,
            },
            work: Work::Mapping {
                input: Arc::clone(&input),
                options: Arc::clone(&options),
                map,
                parameters: parameters.to_vec(),
                supports: supports.clone(),
            },
        });
    let started = Instant::now();
    let completed = dispatch(&mut jobs)?;
    let completed = admit(&owner, SymbolicStage::Mapping, total, completed)?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Mapping,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    completed
        .into_iter()
        .map(|output| match output {
            Output::Mapping(chart) => Ok(*chart),
            _ => Err(GenerationError::Invariant(
                "wrong symbolic completion kind".into(),
            )),
        })
        .collect()
}

pub(super) fn prepare_symmetry(
    index: usize,
    mut chart: MappedChart,
    total: usize,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<PreparedChart, GenerationError> {
    chart.program.remap(&[index])?;
    let symmetry = symmetry::prepare_program(
        index,
        &chart.parameters,
        &chart.mapped,
        &chart.program,
        || {
            emit(
                progress,
                GenerationProgress::SymmetryPreparation {
                    sector: index,
                    total,
                },
            )
        },
    )?;
    Ok(PreparedChart { chart, symmetry })
}

pub(super) fn symmetry_dispatched(
    charts: Vec<MappedChart>,
    dispatch: &mut SymbolicDispatch<'_>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Vec<PreparedChart>, GenerationError> {
    let owner = Arc::new(());
    let total = charts.len();
    let mut jobs = charts
        .into_iter()
        .enumerate()
        .map(|(index, chart)| SymbolicJob {
            owner: Arc::clone(&owner),
            id: SymbolicJobId {
                stage: SymbolicStage::Symmetry,
                index,
            },
            work: Work::Symmetry {
                chart: Box::new(chart),
                total,
            },
        });
    let started = Instant::now();
    let completed = dispatch(&mut jobs)?;
    let completed = admit(&owner, SymbolicStage::Symmetry, total, completed)?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Symmetry,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    completed
        .into_iter()
        .map(|output| match output {
            Output::Symmetry(chart) => Ok(*chart),
            _ => Err(GenerationError::Invariant(
                "wrong symbolic completion kind".into(),
            )),
        })
        .collect()
}

pub(super) fn expand_dispatched(
    representatives: Representatives,
    regulator: Symbol,
    options: &GenerationOptions,
    dispatch: &mut SymbolicDispatch<'_>,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Vec<ExpandedChart>, GenerationError> {
    let owner = Arc::new(());
    let options = Arc::new(options.clone());
    let total = representatives.len();
    let mut jobs = representatives.into_iter().enumerate().map(
        |(index, (representative, (map, parameters, mapped, multiplicity, program)))| SymbolicJob {
            owner: Arc::clone(&owner),
            id: SymbolicJobId {
                stage: SymbolicStage::Coefficients,
                index,
            },
            work: Work::Coefficients {
                options: Arc::clone(&options),
                regulator,
                total,
                representative,
                map,
                parameters,
                mapped,
                multiplicity,
                program,
            },
        },
    );
    let started = Instant::now();
    let completed = dispatch(&mut jobs)?;
    let completed = admit(&owner, SymbolicStage::Coefficients, total, completed)?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::CoefficientExpansion,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    completed
        .into_iter()
        .map(|output| match output {
            Output::Coefficients(chart) => Ok(*chart),
            _ => Err(GenerationError::Invariant(
                "wrong symbolic completion kind".into(),
            )),
        })
        .collect()
}

#[cfg(test)]
mod formula_admission_tests {
    use super::super::numerical_dual::{pipeline::TaskResult, subtraction::Recipe};
    use super::*;

    fn completion(owner: &Arc<()>, stage: SymbolicStage, index: usize) -> SymbolicCompletion {
        SymbolicCompletion {
            owner: owner.clone(),
            id: SymbolicJobId { stage, index },
            result: Output::NumericalDual(TaskResult::Formula(Arc::new(Recipe {
                coefficients: BTreeMap::new(),
                requests: Vec::new(),
            }))),
        }
    }

    #[test]
    fn formula_admission_requires_exact_owner_stage_and_index_coverage() {
        let owner = Arc::new(());
        let stage = SymbolicStage::FormulaPreparation;
        assert!(
            admit(
                &owner,
                stage,
                2,
                vec![completion(&owner, stage, 1), completion(&owner, stage, 0)]
            )
            .is_ok()
        );
        for rejected in [
            vec![completion(&owner, stage, 0)],
            vec![completion(&owner, stage, 0), completion(&owner, stage, 0)],
            vec![
                completion(&owner, stage, 0),
                completion(&owner, SymbolicStage::Coefficients, 1),
            ],
            vec![
                completion(&owner, stage, 0),
                completion(&Arc::new(()), stage, 1),
            ],
            vec![completion(&owner, stage, 0), completion(&owner, stage, 2)],
        ] {
            assert!(matches!(
                admit(&owner, stage, 2, rejected),
                Err(GenerationError::Invariant(_))
            ));
        }
    }
}
