use super::*;
use crate::generation::{GenerationMode, GenerationProgress};

impl<W: Write + Seek> Work<W> {
    pub(super) fn advance(
        &mut self,
        width: usize,
        geometry_dispatch: Option<&mut fastsecdec_sectors::GeometryDispatch<'_>>,
        dispatch: Option<&mut RecipeFamilyDispatch<'_>>,
        snapshot: &mut RecipeFamilySnapshot,
        observer: &mut impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
        pause: &mut bool,
    ) -> Result<(Option<RecipeFamilyOutput<W>>, usize), RecipeFamilySessionError> {
        if matches!(
            self.stage,
            Stage::Source(_) | Stage::Map(_) | Stage::Formula(_) | Stage::Sector(_)
        ) {
            return self
                .advance_jobs(width, dispatch, snapshot, observer, pause)
                .map(|count| (None, count));
        }
        match self.stage {
            Stage::Prepare => {
                let input = self.input.take().unwrap();
                let mut progress = |event: &crate::generation::GenerationProgress| {
                    observe_generation(snapshot, observer, pause, self.options.max_order, event)
                };
                let prepared = if let Some(dispatch) = geometry_dispatch {
                    native::prepare_recipes_with_runtime_and_dispatch(
                        &input.integrand,
                        &self.options,
                        self.family.recipes(),
                        &input.runtime,
                        &input.constraints,
                        &self.staging,
                        dispatch,
                        &mut progress,
                    )?
                } else {
                    native::prepare_recipes_with_runtime(
                        &input.integrand,
                        &self.options,
                        self.family.recipes(),
                        &input.runtime,
                        &input.constraints,
                        &self.staging,
                        &mut progress,
                    )?
                };
                self.archive = Some(ProgramArchiveWriter::new(
                    self.storage.take().unwrap(),
                    prepared.source_identity.clone(),
                    self.family.recipes().iter().copied(),
                )?);
                if let Some(recipe) = self.resident_recipe {
                    self.resident = Some(ProgramResidentAssembly::new(
                        prepared.source_identity.clone(),
                        recipe,
                    )?);
                }
                snapshot.generation.sectors = prepared.recipes[0].charts.len();
                let empty = prepared.recipes[0].charts.is_empty();
                self.prepared = Some(Arc::new(prepared));
                self.stage = if empty {
                    Stage::BeginRecipe(0)
                } else {
                    Stage::Source(0)
                };
            }
            Stage::BeginRecipe(index) => {
                snapshot.recipe = Some(self.family.recipes()[index]);
                snapshot.generation.stage = GenerationStage::Mapping;
                snapshot.generation.detail = "Applying recipe to shared chart sources".into();
                snapshot.generation.coefficient_expansion = None;
                snapshot.generation.formula_preparation = None;
                self.active = Some(RecipeWork {
                    index,
                    charts: vec![],
                    representatives: BTreeMap::new(),
                    assignments: vec![],
                    formula_sources: vec![],
                    formulas: vec![],
                    plan: None,
                });
                self.stage = if self.sources.is_empty() {
                    Stage::Plan
                } else {
                    Stage::Map(0)
                };
            }
            Stage::Symmetry(index) => {
                let active = self.active.as_mut().unwrap();
                let preparation = &self.prepared.as_ref().unwrap().recipes[active.index];
                let chart = &active.charts[index];
                let identity = || native::SymmetryAssignment {
                    program_recipe: preparation.program_recipe,
                    source_id: preparation.source.blake3.clone(),
                    source: chart.index,
                    representative: chart.index,
                    permutation: (0..chart.dimension).collect(),
                };
                snapshot.generation.stage = GenerationStage::Symmetry;
                snapshot.generation.completed = index;
                snapshot.generation.total = Some(active.charts.len());
                let assignment = if preparation.mode == GenerationMode::Symbolic {
                    let key = chart.symmetry_key.as_ref().ok_or_else(|| {
                        RecipeFamilySessionError::State(
                            "symbolic chart lacks its native symmetry bucket".into(),
                        )
                    })?;
                    let candidates = active.representatives.entry(key.clone()).or_default();
                    let assignment = if candidates.is_empty() {
                        identity()
                    } else {
                        native::compare_symmetry(
                            &self.staging,
                            preparation,
                            chart,
                            candidates,
                            |event| {
                                observe_generation(
                                    snapshot,
                                    observer,
                                    pause,
                                    self.options.max_order,
                                    event,
                                )
                            },
                        )?
                    };
                    if assignment.representative == chart.index {
                        candidates.push(chart.clone());
                    }
                    assignment
                } else {
                    identity()
                };
                active.assignments.push(assignment);
                if index + 1 == active.charts.len() {
                    let mut formulas = BTreeMap::new();
                    if preparation.mode == GenerationMode::NumericalDual {
                        for chart in &active.charts {
                            if let Some(key) = &chart.formula_key {
                                formulas.entry(key.clone()).or_insert_with(|| chart.clone());
                            }
                        }
                    }
                    active.formula_sources = formulas.into_values().collect();
                    active.representatives.clear();
                    self.stage = if active.formula_sources.is_empty() {
                        Stage::Plan
                    } else {
                        Stage::Formula(0)
                    };
                } else {
                    self.stage = Stage::Symmetry(index + 1);
                }
            }
            Stage::Plan => {
                let active = self.active.as_mut().unwrap();
                let preparation = &self.prepared.as_ref().unwrap().recipes[active.index];
                let plan = native::finish_preparation(
                    preparation,
                    std::mem::take(&mut active.charts),
                    std::mem::take(&mut active.assignments),
                    std::mem::take(&mut active.formulas),
                )?;
                active.formula_sources.clear();
                snapshot.generation.kernels = plan.sectors.len();
                active.plan = Some(Arc::new(plan));
                self.stage = Stage::Sector(0);
            }
            Stage::FinishRecipe => {
                let index = self.active.take().unwrap().index;
                snapshot.completed_recipes += 1;
                self.stage = if index + 1 == self.family.recipes().len() {
                    Stage::Finish
                } else {
                    Stage::BeginRecipe(index + 1)
                };
            }
            Stage::Finish => {
                let (writer, catalogue) = self.archive.take().unwrap().finish()?;
                let resident = self
                    .resident
                    .take()
                    .map(|resident| resident.finish(&catalogue))
                    .transpose()?;
                self.sources.clear();
                self.prepared = None;
                self.stage = Stage::Complete;
                snapshot.generation.stage = GenerationStage::Complete;
                snapshot.generation.detail = "All requested recipes persisted".into();
                return Ok((
                    Some(RecipeFamilyOutput {
                        writer,
                        catalogue,
                        family: self.family.clone(),
                        resident,
                    }),
                    1,
                ));
            }
            Stage::Source(_)
            | Stage::Map(_)
            | Stage::Formula(_)
            | Stage::Sector(_)
            | Stage::Complete
            | Stage::Failed => unreachable!("handled by step or batch"),
        }
        Ok((None, 1))
    }
}

fn observe_generation(
    snapshot: &mut RecipeFamilySnapshot,
    observer: &mut impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
    pause: &mut bool,
    order: i32,
    event: &GenerationProgress,
) -> ControlFlow<()> {
    snapshot.generation.observe_generation(order, event);
    *pause |= observer(snapshot).is_break();
    ControlFlow::Continue(())
}
