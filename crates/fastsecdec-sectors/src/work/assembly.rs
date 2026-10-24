use super::*;
use crate::{
    decompose::{add_count, initial_status},
    stages::trivial_map,
};
use std::mem;
use std::ops::ControlFlow;

fn work_error(error: GeometryWorkError) -> SectorError {
    SectorError::Work(error)
}

fn collect(
    owner: &Owner,
    count: usize,
    index: impl Fn(GeometryJobId) -> Option<usize>,
    id: impl Fn(usize) -> GeometryJobId,
    completed: impl IntoIterator<Item = GeometryCompletion>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<GeometryCompletion>, SectorError> {
    resources::checkpoint()?;
    let bytes = count
        .checked_mul(mem::size_of::<Option<GeometryCompletion>>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(SectorError::CountOverflow {
            resource: "geometry completion slots",
        })?;
    resources::check_allocation(bytes)?;
    let mut slots: Vec<Option<GeometryCompletion>> =
        std::iter::repeat_with(|| None).take(count).collect();
    let mut stopped = cancelled();
    for result in completed {
        let same = match (owner, &result.owner) {
            (Owner::Charts(a), Owner::Charts(b)) => Arc::ptr_eq(a, b),
            (Owner::Cones(a), Owner::Cones(b)) => Arc::ptr_eq(a, b),
            _ => return Err(work_error(GeometryWorkError::WrongStage)),
        };
        if !same {
            return Err(work_error(GeometryWorkError::ForeignPlan));
        }
        let position =
            index(result.id).ok_or_else(|| work_error(GeometryWorkError::UnknownJob(result.id)))?;
        if slots[position].is_some() {
            return Err(work_error(GeometryWorkError::DuplicateJob(result.id)));
        }
        stopped |= matches!(result.error(), Some(SectorError::Cancelled)) || cancelled();
        slots[position] = Some(result);
        if !stopped {
            resources::checkpoint()?;
        }
    }
    if stopped || cancelled() {
        return Err(SectorError::Cancelled);
    }
    if let Some(first) = slots.iter().position(Option::is_none) {
        return Err(work_error(GeometryWorkError::MissingJobs {
            first: id(first),
            count: slots.iter().filter(|s| s.is_none()).count(),
        }));
    }
    resources::checkpoint()?;
    Ok(slots.into_iter().map(Option::unwrap).collect())
}

impl GeometryPlan {
    /// Admit exactly one completion per chart, in any arrival order.
    /// Native chart errors are retained for canonical final resolution. Invalid
    /// provenance/coverage is rejected; observed cancellation returns Cancelled,
    /// including when other jobs were not dispatched. No partial fan escapes.
    pub fn prepare(
        self,
        completed: impl IntoIterator<Item = GeometryCompletion>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<PreparedGeometry, SectorError> {
        let budget = self.inner.budget.clone();
        budget.run(|| {
            let count = self.inner.fixed.len();
            let results = collect(
                &Owner::Charts(Arc::clone(&self.inner)),
                count,
                |id| match id {
                    GeometryJobId::Chart { chart } if chart < count => Some(chart),
                    _ => None,
                },
                |chart| GeometryJobId::Chart { chart },
                completed,
                &mut cancelled,
            )?;
            let charts = results
                .into_iter()
                .map(|record| {
                    let Payload::Chart(result) = record.payload else {
                        unreachable!("private chart payload")
                    };
                    ChartCompletion {
                        result,
                        status: record.status,
                    }
                })
                .collect::<Vec<_>>();
            let mut offsets = vec![0usize];
            let mut total = 0;
            for chart in &charts {
                if cancelled() {
                    return Err(SectorError::Cancelled);
                }
                let Ok(chart) = &chart.result else { break };
                resources::checkpoint()?;
                add_count(&mut total, chart.candidates.len(), "cone jobs")?;
                offsets.push(total);
            }
            resources::checkpoint()?;
            Ok(PreparedGeometry {
                inner: Arc::new(PreparedData {
                    plan: self.inner,
                    charts,
                    offsets,
                }),
            })
        })
    }
}

impl PreparedGeometry {
    /// Validate all required cone work and merge in native chart/candidate/
    /// simplex order. Global limits are checked on each successful map prefix
    /// before a later conversion error. Only the final observer sees Complete;
    /// it may cancel the return. No cache insertion or partial fan API exists.
    pub fn finish(
        self,
        completed: impl IntoIterator<Item = GeometryCompletion>,
        mut cancelled: impl FnMut() -> bool,
        mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
    ) -> Result<Decomposition, SectorError> {
        let budget = self.inner.plan.budget.clone();
        budget.run(|| {
            let results = collect(
                &Owner::Cones(Arc::clone(&self.inner)),
                self.inner.count(),
                |id| self.inner.index(id),
                |index| self.inner.id(index),
                completed,
                &mut cancelled,
            )?;
            let mut results = results.into_iter();
            let plan = &self.inner.plan;
            let mut output = Decomposition {
                domain: plan.domain,
                sectors: Vec::new(),
                candidate_vertices: 0,
                geometric_vertices: 0,
            };
            let mut status = initial_status(0, 0);
            for chart in &self.inner.charts {
                if cancelled() {
                    return Err(SectorError::Cancelled);
                }
                let chart_data = chart.result.as_ref().map_err(Clone::clone)?;
                status = chart.status.clone();
                if chart_data.supports[0].dimension() == 0 {
                    // Serial's zero-dimensional check precedes the trivial append.
                    if let Some(limit) = plan.options.max_sectors
                        && output.sectors.len() >= limit
                    {
                        return Err(SectorError::ResourceLimit {
                            resource: "sectors",
                            limit,
                        });
                    }
                    output.sectors.push(trivial_map(chart_data));
                    resources::checkpoint()?;
                    continue;
                }
                add_count(
                    &mut output.candidate_vertices,
                    chart_data.candidates.len(),
                    "candidate vertices",
                )?;
                for _ in &chart_data.candidates {
                    if cancelled() {
                        return Err(SectorError::Cancelled);
                    }
                    resources::checkpoint()?;
                    let record = results.next().expect("validated canonical coverage");
                    let Payload::Cone {
                        maps,
                        geometric,
                        error,
                    } = record.payload
                    else {
                        unreachable!("private cone payload")
                    };
                    // Keep native conversion prefixes even if the cone later fails.
                    for map in maps {
                        if cancelled() {
                            return Err(SectorError::Cancelled);
                        }
                        output.sectors.push(map);
                        if let Some(limit) = plan.options.max_sectors
                            && output.sectors.len() > limit
                        {
                            return Err(SectorError::ResourceLimit {
                                resource: "sectors",
                                limit,
                            });
                        }
                    }
                    if let Some(error) = error {
                        return Err(error);
                    }
                    resources::checkpoint()?;
                    add_count(
                        &mut output.geometric_vertices,
                        usize::from(geometric),
                        "geometric vertices",
                    )?;
                    status = record.status;
                }
                if output.sectors.is_empty() {
                    return Err(SectorError::Geometry(
                        "normal fan produced no full-dimensional sectors".into(),
                    ));
                }
            }
            if cancelled() {
                return Err(SectorError::Cancelled);
            }
            status.phase = DecompositionPhase::Complete;
            status.sectors = output.sectors.len();
            if progress(&status).is_break() {
                return Err(SectorError::Cancelled);
            }
            resources::checkpoint()?;
            Ok(output)
        })
    }
}
