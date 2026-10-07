//! Native work-unit transitions; completed owners move between stages.
use super::*;

impl GenerationSession {
    pub(super) fn advance(
        &mut self,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<(), GenerationError> {
        let stage = std::mem::replace(&mut self.stage, Stage::Failed);
        self.stage = match stage {
            Stage::NumericalDual {
                mut maps,
                parameters,
                programs,
                valuations,
                mut completed,
            } => {
                if let Some(map) = maps.pop_front() {
                    completed.push(super::super::numerical_dual::prepare(
                        super::super::numerical_dual::Sources {
                            input: &self.input,
                            options: &self.options,
                            programs: &programs,
                            valuations: &valuations,
                        },
                        map,
                        parameters.clone(),
                        completed.len(),
                        self.chart_count,
                        &mut self.supports,
                        progress,
                    )?);
                    self.completed_representatives += 1;
                    Stage::NumericalDual {
                        maps,
                        parameters,
                        programs,
                        valuations,
                        completed,
                    }
                } else {
                    let result = super::super::numerical_dual::finish(
                        self.domain.take().expect("admitted domain"),
                        completed,
                        self.options.max_order,
                    );
                    let _ = progress(
                        &GenerationProgress::Complete {
                            sectors: result.sectors().len(),
                            orders: result.orders().to_vec(),
                        }
                        .into(),
                    );
                    self.result = Some(result);
                    Stage::Complete
                }
            }
            Stage::Admission => {
                let started = Instant::now();
                self.domain = Some(domain::check(
                    &self.input,
                    self.options.assume_no_threshold,
                )?);
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Domain,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                if self.input.terms().is_empty() {
                    Stage::Finish
                } else {
                    let mut supports = Vec::new();
                    for term in self.input.terms() {
                        for factor in term.factors() {
                            if domain::is_singular(factor) {
                                let support = self.supports.get(factor)?;
                                if !supports.contains(support) {
                                    supports.push(support.clone());
                                }
                            }
                        }
                    }
                    if supports.is_empty() {
                        supports.push(PolynomialSupport::new(vec![vec![
                            0;
                            self.input
                                .parameters()
                                .len()
                        ]])?);
                    }
                    let plan = GeometryPlan::new(
                        self.input.domain(),
                        supports,
                        self.options.decomposition.clone(),
                    )?;
                    let jobs = plan.charts().collect();
                    Stage::GeometryCharts {
                        plan,
                        jobs,
                        completed: Vec::new(),
                    }
                }
            }
            Stage::GeometryCharts {
                plan,
                mut jobs,
                mut completed,
            } => {
                let started = Instant::now();
                if let Some(job) = jobs.pop_front() {
                    completed.push(job.run(|status| {
                        let _ = progress(&GenerationProgress::Decomposition(status.clone()).into());
                        ControlFlow::Continue(())
                    }));
                }
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Geometry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                if jobs.is_empty() {
                    Stage::GeometryPrepare { plan, completed }
                } else {
                    Stage::GeometryCharts {
                        plan,
                        jobs,
                        completed,
                    }
                }
            }
            Stage::GeometryPrepare { plan, completed } => {
                let started = Instant::now();
                let prepared = plan.prepare(completed, || false)?;
                let jobs = prepared.cones().collect();
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Geometry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                Stage::GeometryCones {
                    prepared,
                    jobs,
                    completed: Vec::new(),
                }
            }
            Stage::GeometryCones {
                prepared,
                mut jobs,
                mut completed,
            } => {
                let started = Instant::now();
                if let Some(job) = jobs.pop_front() {
                    completed.push(job.run(|status| {
                        let _ = progress(&GenerationProgress::Decomposition(status.clone()).into());
                        ControlFlow::Continue(())
                    }));
                }
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Geometry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                if jobs.is_empty() {
                    Stage::GeometryFinish {
                        prepared,
                        completed,
                    }
                } else {
                    Stage::GeometryCones {
                        prepared,
                        jobs,
                        completed,
                    }
                }
            }
            Stage::GeometryFinish {
                prepared,
                completed,
            } => {
                let started = Instant::now();
                let result = prepared.finish(
                    completed,
                    || false,
                    |status| {
                        let _ = progress(&GenerationProgress::Decomposition(status.clone()).into());
                        ControlFlow::Continue(())
                    },
                )?;
                self.chart_count = result.sectors.len();
                let dimension = result.sectors.first().map_or(0, SectorMap::dimension);
                let parameters = super::super::mapping::target_parameters(&self.input, dimension);
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Geometry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                if self.options.mode == super::super::GenerationMode::NumericalDual {
                    Stage::NumericalDual {
                        maps: result.sectors.into(),
                        parameters,
                        programs: Default::default(),
                        valuations: std::sync::Arc::new(
                            super::super::numerical_dual::ValuationCache::new(
                                self.input.parameters(),
                            ),
                        ),
                        completed: Vec::new(),
                    }
                } else {
                    Stage::Mapping {
                        maps: result.sectors.into(),
                        parameters,
                    }
                }
            }
            Stage::Mapping {
                mut maps,
                parameters,
            } => {
                if let Some(map) = maps.pop_front() {
                    let _ = progress(
                        &GenerationProgress::Factorization {
                            sector: self.charts.len(),
                            total: self.chart_count,
                        }
                        .into(),
                    );
                    let chart = work::map_chart(
                        &self.input,
                        &self.options,
                        map,
                        parameters.clone(),
                        &mut self.supports,
                        progress,
                    )?;
                    Stage::SymmetryPrepare {
                        chart: Box::new(chart),
                        maps,
                        parameters,
                    }
                } else {
                    let total = self.representatives.len();
                    Stage::Coefficients {
                        remaining: std::mem::take(&mut self.representatives).into_iter(),
                        index: 0,
                        total,
                    }
                }
            }
            Stage::SymmetryPrepare {
                chart,
                maps,
                parameters,
            } => {
                let started = Instant::now();
                let prepared =
                    work::prepare_symmetry(self.charts.len(), *chart, self.chart_count, progress)?;
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Symmetry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                Stage::SymmetryRegister {
                    prepared: Box::new(prepared),
                    maps,
                    parameters,
                }
            }
            Stage::SymmetryRegister {
                prepared,
                maps,
                parameters: targets,
            } => {
                let started = Instant::now();
                let index = self.charts.len();
                let _ = progress(
                    &GenerationProgress::Symmetry {
                        completed: index,
                        total: self.chart_count,
                    }
                    .into(),
                );
                let work::PreparedChart { chart, symmetry } = *prepared;
                let work::MappedChart {
                    map,
                    parameters,
                    coordinates,
                    mapped,
                    pre_subtraction,
                } = chart;
                let matched = self.registry.register_prepared(index, symmetry)?;
                self.charts.push(ChartRecord {
                    source_index: index,
                    representative: matched.representative,
                    representative_permutation: matched.permutation,
                    kernel_sector: None,
                    coordinates,
                    geometry: map.clone(),
                    pre_subtraction,
                });
                if matched.representative == index {
                    self.representatives
                        .insert(index, (map, parameters, mapped, 1));
                } else {
                    self.representatives
                        .get_mut(&matched.representative)
                        .ok_or_else(|| {
                            GenerationError::Invariant("missing symmetry representative".into())
                        })?
                        .3 += 1;
                }
                let _ = progress(
                    &GenerationProgress::PhaseTiming {
                        phase: GenerationPhase::Symmetry,
                        seconds: started.elapsed().as_secs_f64(),
                    }
                    .into(),
                );
                let _ = progress(
                    &GenerationProgress::Symmetry {
                        completed: index + 1,
                        total: self.chart_count,
                    }
                    .into(),
                );
                Stage::Mapping {
                    maps,
                    parameters: targets,
                }
            }
            Stage::Coefficients {
                mut remaining,
                index,
                total,
            } => {
                if let Some((representative, (map, parameters, mapped, multiplicity))) =
                    remaining.next()
                {
                    let output = coefficients::expand(
                        mapped,
                        coefficients::Representative {
                            parameters: &parameters,
                            regulator: self.input.regulator(),
                            index,
                            total,
                        },
                        &self.options,
                        &mut self.templates,
                        progress,
                    )?;
                    let _ = progress(
                        &GenerationProgress::PhaseTiming {
                            phase: output.phase,
                            seconds: output.phase_started.elapsed().as_secs_f64(),
                        }
                        .into(),
                    );
                    self.assembly
                        .as_mut()
                        .expect("assembly retained until finish")
                        .push(representative, map, parameters, multiplicity, output);
                    self.completed_representatives += 1;
                    Stage::Coefficients {
                        remaining,
                        index: index + 1,
                        total,
                    }
                } else {
                    Stage::Finish
                }
            }
            Stage::Finish => {
                let result = self
                    .assembly
                    .take()
                    .expect("assembly retained until finish")
                    .finish(
                        self.domain.take().expect("admitted domain"),
                        std::mem::take(&mut self.charts),
                        self.options.max_order,
                    );
                let _ = progress(
                    &GenerationProgress::Complete {
                        sectors: result.sectors().len(),
                        orders: result.orders().to_vec(),
                    }
                    .into(),
                );
                self.result = Some(result);
                Stage::Complete
            }
            Stage::Complete => Stage::Complete,
            Stage::Failed => {
                return Err(GenerationError::Invariant(
                    "failed generation session".into(),
                ));
            }
        };
        Ok(())
    }
}
