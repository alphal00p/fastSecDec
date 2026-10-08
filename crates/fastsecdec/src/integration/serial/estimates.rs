use super::state::*;
use crate::integration::{
    AccuracyTarget, IntegrationError, Result, VectorEstimate, estimate::precise_sum,
};
use numerica::domains::float::{DoubleFloat, RealLike};

impl SerialSession {
    pub(super) fn sector_estimate(&self, s: &SectorState) -> Result<Option<VectorEstimate>> {
        if s.pilot || s.moments.count() < 2 {
            return Ok(None);
        }
        Ok(Some(
            s.moments
                .estimate(&self.problem, s.moments.count() >= s.target)?,
        ))
    }
    pub fn estimate(&self) -> Result<VectorEstimate> {
        self.aggregate(false)
    }
    /// Running complete-replica statistics of current allocations. This view
    /// does not make an unfinished replacement allocation convergence evidence.
    pub fn live_estimate(&self) -> Result<VectorEstimate> {
        self.aggregate(true)
    }
    fn aggregate(&self, live: bool) -> Result<VectorEstimate> {
        let selected = self
            .sectors
            .iter()
            .map(|s| {
                // Previous complete evidence remains authoritative until a new
                // allocation completes. It may certify a global stop while a
                // faster sector has already started another refinement epoch.
                if !live
                    && (s.pilot || s.moments.count() < s.target)
                    && let Some(previous) = &s.previous
                {
                    Ok((&previous.moments, true))
                } else if !s.pilot && s.moments.count() >= 2 {
                    Ok((&s.moments, s.moments.count() >= s.target))
                } else {
                    Err(IntegrationError::Unavailable(
                        "serial production coverage is incomplete".into(),
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let n = self.problem.orders.len();
        let mut means: Vec<_> = self
            .problem
            .exact_coefficients
            .iter()
            .map(|&v| DoubleFloat::from(v))
            .collect();
        let mut covariance = vec![DoubleFloat::from(0.); n * n];
        for (moments, _) in &selected {
            moments.validate(n)?;
            for (sum, mean) in means.iter_mut().zip(moments.centered_mean()) {
                *sum += mean;
            }
            for (sum, value) in covariance.iter_mut().zip(moments.centered_covariance()?) {
                *sum += value;
            }
        }
        let mean = means.into_iter().map(|v| v.to_f64()).collect();
        let covariance_of_mean: Vec<_> = covariance.into_iter().map(|v| v.to_f64()).collect();
        let value = VectorEstimate {
            orders: self.problem.orders.clone(),
            components: self.problem.components.clone(),
            mean,
            standard_error: (0..n)
                .map(|j| covariance_of_mean[j * n + j].sqrt())
                .collect(),
            covariance_of_mean,
            production_complete: selected.iter().all(|(_, complete)| *complete),
        };
        value.validate()?;
        Ok(value)
    }
    pub fn meets_target(&self) -> Result<bool> {
        match self.estimate() {
            Ok(v) => v.meets_target(self.settings.target, self.settings.tolerance),
            Err(IntegrationError::Unavailable(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    pub fn first_coverage_complete(&self) -> bool {
        self.sectors.iter().all(|s| s.first_visit)
    }
    pub fn exhausted(&self) -> bool {
        self.pending.is_empty() && self.sectors.iter().all(|s| self.exhausted_sector(s))
    }
    pub(super) fn priority(&self, s: &SectorState, total: Option<&VectorEstimate>) -> Result<f64> {
        let Some(value) = self
            .sector_estimate(s)?
            .or_else(|| s.previous.as_ref().map(|v| v.estimate.clone()))
        else {
            return Ok(f64::INFINITY);
        };
        let n = value.orders.len();
        match self.settings.target {
            AccuracyTarget::LaurentOrder(order) => precise_sum(
                (0..n)
                    .filter(|&j| value.orders[j] == order)
                    .map(|j| value.covariance_of_mean[j * n + j]),
            ),
            AccuracyTarget::AllComponents => {
                let mut priority = 0.0_f64;
                for j in 0..n {
                    let scale = self.settings.tolerance.absolute.max(
                        self.settings.tolerance.relative * total.map_or(0., |v| v.mean[j].abs()),
                    );
                    let variance = value.covariance_of_mean[j * n + j];
                    let ratio = if scale == 0. {
                        if variance == 0. { 0. } else { f64::INFINITY }
                    } else {
                        variance / scale / scale
                    };
                    priority = priority.max(ratio);
                }
                Ok(priority)
            }
        }
    }
    pub fn snapshot(&self) -> Result<SerialSnapshot> {
        let total = match self.estimate() {
            Ok(v) => Some(v),
            Err(IntegrationError::Unavailable(_)) => None,
            Err(e) => return Err(e),
        };
        let live_total = match self.live_estimate() {
            Ok(v) => Some(v),
            Err(IntegrationError::Unavailable(_)) => None,
            Err(e) => return Err(e),
        };
        let sectors = self
            .sectors
            .iter()
            .map(|s| {
                let priority = self.priority(s, total.as_ref())?;
                Ok(SerialSectorSnapshot {
                    id: s.id,
                    dimension: s.dimension,
                    pilot: s.pilot,
                    epoch: s.epoch,
                    round: s.round,
                    points_per_replica: s.points,
                    replicas: s.moments.count(),
                    target_replicas: s.target,
                    in_flight: self.pending_count(s.id),
                    first_visit_complete: s.first_visit,
                    allocation_complete: !s.pilot && s.moments.count() >= s.target,
                    exhausted: self.exhausted_sector(s),
                    worker_seconds: s.worker_seconds,
                    accepted_points: s.accepted_points,
                    current: self.sector_estimate(s)?,
                    previous: s.previous.as_ref().map(|v| v.estimate.clone()),
                    priority: priority.is_finite().then_some(priority),
                })
            })
            .collect::<Result<_>>()?;
        Ok(SerialSnapshot {
            sectors,
            total,
            live_total,
            first_coverage_complete: self.first_coverage_complete(),
            exhausted: self.exhausted(),
            in_flight: self.pending.len(),
        })
    }
}
