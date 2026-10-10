//! Bounded native units; the caller schedules them and the session admits results.
use super::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeFamilyJobStage {
    Source,
    Mapping,
    Formula,
    Sector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeFamilyJobId {
    pub stage: RecipeFamilyJobStage,
    pub recipe: Option<ProgramRecipe>,
    pub index: usize,
}

pub enum RecipeFamilyJobProgress<'a> {
    Generation(&'a crate::generation::GenerationProgress),
    Compilation(&'a crate::kernel::CompilationProgress),
}

pub type RecipeFamilyDispatch<'a> = dyn FnMut(
        &mut dyn ExactSizeIterator<Item = RecipeFamilyJob>,
    ) -> Result<Vec<RecipeFamilyCompletion>, RecipeFamilySessionError>
    + 'a;

pub struct RecipeFamilyJob {
    owner: Arc<()>,
    id: RecipeFamilyJobId,
    staging: PathBuf,
    work: JobWork,
}

pub struct RecipeFamilyCompletion {
    owner: Arc<()>,
    id: RecipeFamilyJobId,
    output: JobOutput,
}

enum JobWork {
    Source(Arc<native::PreparedRecipeSet>),
    Map(
        Arc<native::PreparedRecipeSet>,
        usize,
        native::PreparedChartSource,
    ),
    Formula(
        Arc<native::PreparedRecipeSet>,
        usize,
        native::DiscoveredSector,
    ),
    Sector(
        Arc<native::PreparedGeneration>,
        PrecisionPolicy,
        CompilationSettings,
    ),
}

enum JobOutput {
    Source(native::PreparedChartSource),
    Map(native::DiscoveredSector),
    Formula(native::FormulaRecord),
    Sector(Box<KernelSet>, Vec<usize>),
}

impl RecipeFamilyJob {
    pub fn id(&self) -> RecipeFamilyJobId {
        self.id
    }

    /// Execute the existing native source/generation/compiler operation once.
    /// The caller must join every issued job before returning its completions.
    pub fn run(
        self,
        mut observe: impl FnMut(RecipeFamilyJobProgress<'_>) -> ControlFlow<()>,
    ) -> Result<RecipeFamilyCompletion, RecipeFamilySessionError> {
        let output = match self.work {
            JobWork::Source(prepared) => JobOutput::Source(native::prepare_chart_source(
                &self.staging,
                &prepared,
                &prepared.recipes[0].charts[self.id.index],
                |event| observe(RecipeFamilyJobProgress::Generation(event)),
            )?),
            JobWork::Map(prepared, recipe, source) => JobOutput::Map(native::discover_prepared(
                &self.staging,
                &prepared.recipes[recipe],
                &source,
                |event| observe(RecipeFamilyJobProgress::Generation(event)),
            )?),
            JobWork::Formula(prepared, recipe, chart) => JobOutput::Formula(native::build_formula(
                &self.staging,
                &prepared.recipes[recipe],
                &chart,
                |event| observe(RecipeFamilyJobProgress::Generation(event)),
            )?),
            JobWork::Sector(plan, precision, compilation) => {
                let unit = native::generate_sector(
                    &self.staging,
                    &plan.sectors[self.id.index],
                    |event| observe(RecipeFamilyJobProgress::Generation(event)),
                )?;
                let kernels = unit
                    .generated
                    .compile_with_settings_parameters_and_progress(
                        precision,
                        &unit.runtime_parameters,
                        compilation,
                        |event| observe(RecipeFamilyJobProgress::Compilation(event)),
                    )?
                    .with_runtime_mass_constraints(unit.runtime_mass_constraints)?;
                JobOutput::Sector(Box::new(kernels), unit.source_indices)
            }
        };
        Ok(RecipeFamilyCompletion {
            owner: self.owner,
            id: self.id,
            output,
        })
    }
}

impl<W: Write + Seek> Work<W> {
    pub(super) fn advance_jobs(
        &mut self,
        width: usize,
        dispatch: Option<&mut RecipeFamilyDispatch<'_>>,
        snapshot: &mut RecipeFamilySnapshot,
        observer: &mut impl FnMut(&RecipeFamilySnapshot) -> ControlFlow<()>,
        pause: &mut bool,
    ) -> Result<usize, RecipeFamilySessionError> {
        let (stage, first, total, recipe) = match self.stage {
            Stage::Source(index) => (
                RecipeFamilyJobStage::Source,
                index,
                self.prepared.as_ref().unwrap().recipes[0].charts.len(),
                None,
            ),
            Stage::Map(index) => (
                RecipeFamilyJobStage::Mapping,
                index,
                self.sources.len(),
                Some(self.active.as_ref().unwrap().index),
            ),
            Stage::Formula(index) => (
                RecipeFamilyJobStage::Formula,
                index,
                self.active.as_ref().unwrap().formula_sources.len(),
                Some(self.active.as_ref().unwrap().index),
            ),
            Stage::Sector(index) => (
                RecipeFamilyJobStage::Sector,
                index,
                self.active
                    .as_ref()
                    .unwrap()
                    .plan
                    .as_ref()
                    .unwrap()
                    .sectors
                    .len(),
                Some(self.active.as_ref().unwrap().index),
            ),
            _ => unreachable!("only independent stages issue jobs"),
        };
        let count = width.min(total - first);
        snapshot.generation.stage = match stage {
            RecipeFamilyJobStage::Source | RecipeFamilyJobStage::Mapping => {
                GenerationStage::Mapping
            }
            RecipeFamilyJobStage::Formula => GenerationStage::FormulaPreparation,
            RecipeFamilyJobStage::Sector => GenerationStage::Compilation,
        };
        snapshot.generation.completed = first;
        snapshot.generation.total = Some(total);
        let owner = Arc::new(());
        let mut jobs = (first..first + count).map(|index| {
            let work = match stage {
                RecipeFamilyJobStage::Source => {
                    JobWork::Source(self.prepared.as_ref().unwrap().clone())
                }
                RecipeFamilyJobStage::Mapping => JobWork::Map(
                    self.prepared.as_ref().unwrap().clone(),
                    recipe.unwrap(),
                    self.sources[index].clone(),
                ),
                RecipeFamilyJobStage::Formula => JobWork::Formula(
                    self.prepared.as_ref().unwrap().clone(),
                    recipe.unwrap(),
                    self.active.as_ref().unwrap().formula_sources[index].clone(),
                ),
                RecipeFamilyJobStage::Sector => JobWork::Sector(
                    self.active.as_ref().unwrap().plan.as_ref().unwrap().clone(),
                    self.precision.clone(),
                    self.compilation,
                ),
            };
            RecipeFamilyJob {
                owner: owner.clone(),
                id: RecipeFamilyJobId {
                    stage,
                    recipe: recipe.map(|r| self.family.recipes()[r]),
                    index,
                },
                staging: self.staging.clone(),
                work,
            }
        });
        let mut completed = if let Some(dispatch) = dispatch {
            dispatch(&mut jobs)?
        } else {
            jobs.by_ref()
                .map(|job| {
                    job.run(|event| {
                        match event {
                            RecipeFamilyJobProgress::Generation(event) => snapshot
                                .generation
                                .observe_generation(self.options.max_order, event),
                            RecipeFamilyJobProgress::Compilation(event) => {
                                snapshot.generation.observe_compilation(event)
                            }
                        }
                        *pause |= observer(snapshot).is_break();
                        ControlFlow::Continue(())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        // Admit the complete batch before modifying any archive or stage state.
        if jobs.len() != 0 || completed.len() != count {
            return Err(RecipeFamilySessionError::State(
                "incomplete family dispatch batch".into(),
            ));
        }
        completed.sort_by_key(|result| result.id.index);
        for (offset, result) in completed.iter().enumerate() {
            let expected = RecipeFamilyJobId {
                stage,
                recipe: recipe.map(|r| self.family.recipes()[r]),
                index: first + offset,
            };
            if !Arc::ptr_eq(&owner, &result.owner) || result.id != expected {
                return Err(RecipeFamilySessionError::State(
                    "foreign or duplicate family completion".into(),
                ));
            }
        }
        drop(jobs);
        for result in completed {
            match result.output {
                JobOutput::Source(source) => self.sources.push(source),
                JobOutput::Map(chart) => self.active.as_mut().unwrap().charts.push(chart),
                JobOutput::Formula(formula) => self.active.as_mut().unwrap().formulas.push(formula),
                JobOutput::Sector(kernels, sources) => {
                    let recipe = result.id.recipe.unwrap();
                    if self.resident_recipe == Some(recipe) {
                        self.resident.as_mut().unwrap().append_unit(
                            self.archive.as_mut().unwrap(),
                            *kernels,
                            &sources,
                        )?;
                    } else {
                        self.archive
                            .as_mut()
                            .unwrap()
                            .append_unit(recipe, &kernels, &sources)?;
                    }
                    snapshot.persisted_units += 1;
                }
            }
        }
        snapshot.prepared_sources = self.sources.len();
        let next = first + count;
        snapshot.generation.completed = next;
        self.stage = match stage {
            RecipeFamilyJobStage::Source => {
                if next == total {
                    Stage::BeginRecipe(0)
                } else {
                    Stage::Source(next)
                }
            }
            RecipeFamilyJobStage::Mapping => {
                if next == total {
                    Stage::Symmetry(0)
                } else {
                    Stage::Map(next)
                }
            }
            RecipeFamilyJobStage::Formula => {
                if next == total {
                    Stage::Plan
                } else {
                    Stage::Formula(next)
                }
            }
            RecipeFamilyJobStage::Sector => {
                if next == total {
                    Stage::FinishRecipe
                } else {
                    Stage::Sector(next)
                }
            }
        };
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricTerm, PolynomialFactor,
    };
    use std::io::Cursor;
    use symbolica::{atom::Atom, parse, symbol};

    #[test]
    fn duplicate_completion_ids_are_refused_before_archive_mutation() {
        let input = ParametricIntegrand::new(
            vec![symbol!("duplicate_batch::x"), symbol!("duplicate_batch::y")],
            symbol!("duplicate_batch::eps"),
            ParametricDomain::ProjectiveSimplex,
            vec![ParametricTerm::new(
                Atom::one(),
                vec![Atom::Zero; 2],
                vec![
                    PolynomialFactor::new(
                        parse!("duplicate_batch::x+2*duplicate_batch::y"),
                        Atom::num(-2),
                        FactorRole::Singularity,
                    )
                    .with_semantics(FactorSemantics::Causal),
                ],
            )],
        )
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let mut session = RecipeFamilySession::new(
            input,
            Default::default(),
            RecipeFamily::contour(),
            directory.path().into(),
            Cursor::new(Vec::new()),
        );
        session.step(1, |_| ControlFlow::Continue(())).unwrap();
        let error = session
            .step_with_dispatch(
                2,
                2,
                &mut |_| unreachable!(),
                &mut |jobs| {
                    let mut completed = jobs
                        .map(|job| job.run(|_| ControlFlow::Continue(())))
                        .collect::<Result<Vec<_>, _>>()?;
                    assert_eq!(completed.len(), 2);
                    // Mutation models a broken dispatcher without manufacturing a
                    // numerical result: both payloads were genuinely executed.
                    completed[1].id = completed[0].id;
                    Ok(completed)
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap_err();
        assert!(error.to_string().contains("foreign or duplicate"));
        assert_eq!(session.snapshot().persisted_units, 0);
        assert!(session.take_result().is_none());
        assert!(session.step(1, |_| ControlFlow::Continue(())).is_err());
    }
}
