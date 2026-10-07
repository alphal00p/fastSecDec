use super::*;
use crate::integration::{
    LiveEstimate, LiveObservation, LiveSector, LiveSource, LiveStatus, QmcReturn,
};

fn replicas(means: &[Vec<f64>], points: u64) -> LiveEstimate {
    match means.len() {
        0 => LiveEstimate::empty(points, 0),
        1 => LiveEstimate {
            mean: Some(means[0].clone()),
            standard_error: None,
            points,
            replicas: 1,
            status: LiveStatus::MeanOnly,
        },
        count => match QmcEstimate::from_shift_means(means) {
            Ok(estimate) => LiveEstimate {
                mean: Some(estimate.mean),
                standard_error: Some(estimate.standard_error),
                points,
                replicas: count,
                status: LiveStatus::Available,
            },
            Err(_) => LiveEstimate::range(points, count),
        },
    }
}

impl QmcSession {
    /// Means only from complete gap-free lattices; uncertainty only from at
    /// least two independent shifts. Sector rows use their own coverage; the
    /// democratic total uses common shift identities across every sector.
    pub fn live_observation(&self) -> Result<LiveObservation> {
        let mut sectors = Vec::with_capacity(self.runs.len());
        for (spec, run) in self.problem.sectors.iter().zip(&self.runs) {
            let count = run.accumulator.complete_shift_ids().len();
            let points = count as u64 * run.accumulator.plan().rule().points();
            let value = match run.accumulator.shift_estimates() {
                Ok(rows) => replicas(
                    &rows.into_iter().map(|row| row.mean).collect::<Vec<_>>(),
                    points,
                ),
                Err(_) => LiveEstimate::range(points, count),
            };
            sectors.push(LiveSector {
                id: spec.id,
                estimate: value,
            });
        }
        let mut total = if self.runs.is_empty() {
            LiveEstimate {
                mean: Some(vec![0.0; self.problem.orders.len()]),
                standard_error: Some(vec![0.0; self.problem.orders.len()]),
                points: 0,
                replicas: 0,
                status: LiveStatus::Available,
            }
        } else if self.method == IntegrationMethod::DemocraticQmc {
            let common = self.common_shift_rows()?;
            let points = common.len() as u64 * self.settings.points * self.runs.len() as u64;
            if common.len() >= 2 {
                // Reuse the authoritative centered sector/shift reduction.
                match self.estimate() {
                    Ok(estimate) => {
                        return Ok(LiveObservation {
                            source: LiveSource::CompleteLattices,
                            stage: self.stage,
                            orders: self.problem.orders.clone(),
                            components: self.problem.components.clone(),
                            sectors,
                            total: LiveEstimate {
                                mean: Some(estimate.mean),
                                standard_error: Some(estimate.standard_error),
                                points,
                                replicas: common.len(),
                                status: LiveStatus::Available,
                            },
                        });
                    }
                    Err(error) if error.is_statistical_range() => {
                        LiveEstimate::range(points, common.len())
                    }
                    Err(error) => return Err(error),
                }
            } else if let Some(rows) = common.first() {
                LiveEstimate {
                    mean: Some(
                        (0..self.problem.orders.len())
                            .map(|j| precise_sum(rows.iter().map(|row| row.mean[j])))
                            .collect::<Result<_>>()?,
                    ),
                    standard_error: None,
                    points,
                    replicas: 1,
                    status: LiveStatus::MeanOnly,
                }
            } else {
                LiveEstimate::empty(0, 0)
            }
        } else if sectors.iter().all(|row| row.estimate.mean.is_some()) {
            let points = sectors.iter().map(|row| row.estimate.points).sum();
            let count = sectors
                .iter()
                .map(|row| row.estimate.replicas)
                .min()
                .unwrap_or(0);
            let mean = (0..self.problem.orders.len())
                .map(|j| {
                    precise_sum(
                        sectors
                            .iter()
                            .map(|row| row.estimate.mean.as_ref().unwrap()[j]),
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let available = sectors
                .iter()
                .all(|row| row.estimate.standard_error.is_some());
            let error = available.then(|| {
                (0..self.problem.orders.len())
                    .map(|j| {
                        sectors.iter().fold(0.0_f64, |s, row| {
                            s.hypot(row.estimate.standard_error.as_ref().unwrap()[j])
                        })
                    })
                    .collect()
            });
            LiveEstimate {
                mean: Some(mean),
                standard_error: error,
                points,
                replicas: count,
                status: if available {
                    LiveStatus::Available
                } else {
                    LiveStatus::MeanOnly
                },
            }
        } else {
            LiveEstimate::empty(0, 0)
        };
        if let Some(mean) = &mut total.mean {
            for (value, exact) in mean.iter_mut().zip(&self.problem.exact_coefficients) {
                *value = precise_sum([*value, *exact])?;
            }
        }
        Ok(LiveObservation {
            source: LiveSource::CompleteLattices,
            stage: self.stage,
            orders: self.problem.orders.clone(),
            components: self.problem.components.clone(),
            sectors,
            total,
        })
    }

    /// Observe complete validated returns held by a caller's ordered admission
    /// queue, without changing accepted coverage, replay, or checkpoints. Uses
    /// native session admission on a temporary statistics-only copy.
    pub fn live_observation_with_pending(&self, pending: &[QmcReturn]) -> Result<LiveObservation> {
        if pending.is_empty() {
            return self.live_observation();
        }
        let mut observed = self.clone();
        let mut pending = pending.iter().collect::<Vec<_>>();
        pending.sort_by_key(|value| (value.task().sector_id(), value.task().work().start()));
        for value in pending {
            observed.submit(value.clone())?;
        }
        observed.live_observation()
    }
}
