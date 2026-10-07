//! Observation adapter around Numerica's scalar statistics. Accepted vector
//! covariance continues to come exclusively from complete native batch returns.
use super::{
    IntegrationError, IntegrationProblem, LiveEstimate, LiveObservation, LiveSector, LiveSource,
    LiveStatus, Result,
};
use crate::status::IntegrationStage;
use numerica::{
    domains::float::{DoubleFloat, Real, RealLike},
    numerical_integration::StatisticsAccumulator,
};

#[derive(Clone, Debug)]
struct Moments {
    count: u64,
    origin: Vec<DoubleFloat>,
    offset: Vec<DoubleFloat>,
    error: Vec<DoubleFloat>,
}

pub(crate) struct CenteredAccumulator {
    values: Vec<StatisticsAccumulator<DoubleFloat>>,
    origin: Vec<f64>,
    count: u64,
}
impl CenteredAccumulator {
    pub(crate) fn new(outputs: usize) -> Self {
        Self {
            values: vec![StatisticsAccumulator::new(); outputs],
            origin: vec![0.0; outputs],
            count: 0,
        }
    }
    pub(crate) fn add(&mut self, values: &[f64]) {
        if self.count == 0 {
            self.origin.copy_from_slice(values);
        }
        for ((acc, origin), value) in self.values.iter_mut().zip(&self.origin).zip(values) {
            acc.add_sample(DoubleFloat::from(*value) - DoubleFloat::from(*origin), None);
        }
        self.count += 1;
    }
    fn moments(&self) -> Moments {
        let (offset, error) = self
            .values
            .iter()
            .map(|acc| {
                if self.count < 2 {
                    (DoubleFloat::from(0.0), DoubleFloat::from(0.0))
                } else {
                    let (mean, error, _) = acc.get_live_estimate();
                    (mean, error)
                }
            })
            .unzip();
        Moments {
            count: self.count,
            origin: self.origin.iter().map(|v| DoubleFloat::from(*v)).collect(),
            offset,
            error,
        }
    }
    pub(crate) fn mean(mut self, global_count: usize) -> Vec<f64> {
        if self.count == 0 {
            return vec![0.0; self.origin.len()];
        }
        self.values
            .iter_mut()
            .zip(self.origin)
            .map(|(acc, origin)| {
                acc.update_iter(false);
                ((DoubleFloat::from(origin) + acc.avg) * DoubleFloat::from(self.count as f64)
                    / DoubleFloat::from(global_count as f64))
                .to_f64()
            })
            .collect()
    }
    pub(crate) fn conditional_mean(mut self) -> Vec<f64> {
        self.values
            .iter_mut()
            .zip(self.origin)
            .map(|(acc, origin)| {
                acc.update_iter(false);
                (DoubleFloat::from(origin) + acc.avg).to_f64()
            })
            .collect()
    }
}

/// Borrowed native accumulators. Creating an owned snapshot is optional and
/// controlled by the caller's observation cadence, never by a library thread.
pub struct McLiveView<'a> {
    pub(crate) batch: u32,
    pub(crate) sector_id: Option<u64>,
    pub(crate) points: u64,
    pub(crate) total: Option<&'a CenteredAccumulator>,
    pub(crate) sectors: &'a [CenteredAccumulator],
    pub(crate) last_point_timing: Option<(u64, f64)>,
    pub(crate) sector_ids: &'a [u64],
}
impl McLiveView<'_> {
    /// Completed point effort attributed by the native discrete sampler.
    /// This excludes the observer callback and exists even for discarded prefixes.
    pub fn last_point_timing(&self) -> Option<(u64, f64)> {
        self.last_point_timing
    }
    pub fn points(&self) -> u64 {
        self.points
    }
    pub fn snapshot(&self) -> McLiveBatch {
        McLiveBatch {
            batch: self.batch,
            sector_id: self.sector_id,
            points: self.points,
            total: self.total.map(CenteredAccumulator::moments),
            sectors: self
                .sector_ids
                .iter()
                .copied()
                .zip(self.sectors.iter().map(CenteredAccumulator::moments))
                .collect(),
        }
    }
}
/// Cumulative prefix, replaced by task identity rather than added repeatedly.
#[derive(Clone, Debug)]
pub struct McLiveBatch {
    pub batch: u32,
    pub sector_id: Option<u64>,
    pub points: u64,
    total: Option<Moments>,
    sectors: Vec<(u64, Moments)>,
}

fn pooled(rows: &[&Moments], global: Option<u64>) -> Result<LiveEstimate> {
    let count = rows.iter().try_fold(0u64, |sum, row| {
        sum.checked_add(row.count)
            .ok_or(IntegrationError::NumericRange)
    })?;
    if count == 0 {
        return Ok(LiveEstimate::empty(0, 0));
    }
    let n = global.unwrap_or(count);
    if n < count {
        return Err(IntegrationError::Invalid(
            "live sector count exceeds total draws".into(),
        ));
    }
    let rows = rows
        .iter()
        .copied()
        .filter(|r| r.count != 0)
        .collect::<Vec<_>>();
    let anchor = &rows[0].origin;
    let mut means = Vec::with_capacity(anchor.len());
    let mut errors = Vec::with_capacity(anchor.len());
    let dd_count = DoubleFloat::from(count as f64);
    let dd_n = DoubleFloat::from(n as f64);
    for (j, a) in anchor.iter().enumerate() {
        let center = rows.iter().fold(DoubleFloat::from(0.0), |sum, row| {
            sum + (DoubleFloat::from(row.count as f64) / dd_count)
                * ((row.origin[j] - *a) + row.offset[j])
        });
        let conditional = *a + center;
        let mean = (DoubleFloat::from(count as f64) / dd_n) * conditional;
        let mut error = DoubleFloat::from(0.0);
        if n >= 2 {
            let denominator = dd_n * DoubleFloat::from((n - 1) as f64);
            for row in &rows {
                let k = DoubleFloat::from(row.count as f64);
                let within = (k * DoubleFloat::from(row.count.saturating_sub(1) as f64)
                    / denominator)
                    .sqrt()
                    * row.error[j];
                let between =
                    (k / denominator).sqrt() * ((row.origin[j] - *a) + row.offset[j] - center);
                error = error.hypot(&within).hypot(&between);
            }
            // Add the implicit zero draws without allocating or iterating them.
            if n != count {
                let zero_selection =
                    (dd_count / dd_n * DoubleFloat::from((n - count) as f64) / denominator).sqrt()
                        * conditional;
                error = error.hypot(&zero_selection);
            }
        }
        let (mean, error) = (mean.to_f64(), error.to_f64());
        if !mean.is_finite() || !error.is_finite() {
            return Err(IntegrationError::NumericRange);
        }
        means.push(mean);
        errors.push(error);
    }
    Ok(LiveEstimate {
        mean: Some(means),
        standard_error: (n >= 2).then_some(errors),
        points: count,
        replicas: 0,
        status: if n >= 2 {
            LiveStatus::Available
        } else {
            LiveStatus::MeanOnly
        },
    })
}

/// Pool only one immutable sampling epoch. This is an observational scalar
/// reduction, never a convergence estimate or a checkpoint admission operation.
pub fn mc_live_observation(
    problem: &IntegrationProblem,
    stage: IntegrationStage,
    source: LiveSource,
    batches: &[McLiveBatch],
    discrete: bool,
) -> Result<LiveObservation> {
    problem.validate()?;
    let count = batches.iter().try_fold(0u64, |sum, row| {
        sum.checked_add(row.points)
            .ok_or(IntegrationError::NumericRange)
    })?;
    let mut seen = std::collections::BTreeSet::new();
    if batches.iter().any(|b| !seen.insert((b.sector_id, b.batch))) {
        return Err(IntegrationError::Invalid(
            "duplicate live task observation".into(),
        ));
    }
    let sectors = problem
        .sectors
        .iter()
        .map(|spec| {
            let rows = batches
                .iter()
                .flat_map(|batch| &batch.sectors)
                .filter_map(|(id, row)| (*id == spec.id).then_some(row))
                .collect::<Vec<_>>();
            let points = rows.iter().map(|row| row.count).sum();
            let estimate = pooled(&rows, discrete.then_some(count))
                .unwrap_or_else(|_| LiveEstimate::range(points, 0));
            LiveSector {
                id: spec.id,
                estimate,
            }
        })
        .collect::<Vec<_>>();
    let mut total = if discrete {
        let rows = batches
            .iter()
            .filter_map(|batch| batch.total.as_ref())
            .collect::<Vec<_>>();
        pooled(&rows, None).unwrap_or_else(|_| LiveEstimate::range(count, 0))
    } else {
        let mut value = LiveEstimate::empty(count, 0);
        if sectors.iter().all(|s| s.estimate.mean.is_some()) {
            let mut means = Vec::new();
            let mut errors = Vec::new();
            for j in 0..problem.orders.len() {
                means.push(super::estimate::precise_sum(
                    sectors.iter().map(|s| s.estimate.mean.as_ref().unwrap()[j]),
                )?);
                errors.push(
                    sectors
                        .iter()
                        .filter_map(|s| s.estimate.standard_error.as_ref())
                        .fold(0.0_f64, |sum, e| sum.hypot(e[j])),
                );
            }
            let available = sectors.iter().all(|s| s.estimate.standard_error.is_some());
            value.mean = Some(means);
            value.standard_error = available.then_some(errors);
            value.status = if available {
                LiveStatus::Available
            } else {
                LiveStatus::MeanOnly
            };
        }
        value
    };
    if problem.sectors.is_empty() {
        total = LiveEstimate {
            mean: Some(vec![0.0; problem.orders.len()]),
            standard_error: Some(vec![0.0; problem.orders.len()]),
            points: 0,
            replicas: 0,
            status: LiveStatus::Available,
        };
    }
    if let Some(mean) = &mut total.mean {
        for (v, exact) in mean.iter_mut().zip(&problem.exact_coefficients) {
            *v = super::estimate::precise_sum([*v, *exact])?;
        }
    }
    Ok(LiveObservation {
        source,
        stage,
        orders: problem.orders.clone(),
        components: problem.components.clone(),
        sectors,
        total,
    })
}
