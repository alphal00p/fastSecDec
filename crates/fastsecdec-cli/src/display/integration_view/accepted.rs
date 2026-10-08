//! A display-only record of the last native completed production allocation.
//! It never supplies session statistics, stopping decisions or checkpoints.
use super::{Cached, is_qmc, number};
use fastsecdec::{
    integration::{IntegrationObservation, VectorEstimate},
    results::ResultScope,
    status::{IntegrationMethod, IntegrationStage, UncertaintyStatus},
};

#[derive(Clone)]
pub(in crate::display) struct CompletedAllocation {
    pub estimate: VectorEstimate,
    method: IntegrationMethod,
    scope: ResultScope,
    sector_ids: Vec<u64>,
}
impl CompletedAllocation {
    pub(in crate::display) fn update(
        saved: &mut Option<Self>,
        observation: &IntegrationObservation,
        scope: &ResultScope,
    ) {
        if saved
            .as_ref()
            .is_some_and(|previous| !previous.matches(observation, scope))
        {
            *saved = None;
        }
        if observation.snapshot.stage == IntegrationStage::Production
            && let Some(estimate) = observation.contributions.total.as_ref()
            && estimate.production_complete
        {
            *saved = Some(Self {
                estimate: estimate.clone(),
                method: observation.snapshot.method,
                scope: scope.clone(),
                sector_ids: observation
                    .contributions
                    .sectors
                    .iter()
                    .map(|s| s.progress.id)
                    .collect(),
            });
        }
    }

    fn matches(&self, observation: &IntegrationObservation, scope: &ResultScope) -> bool {
        self.method == observation.snapshot.method
            && self.scope == *scope
            && self.sector_ids.iter().copied().eq(observation
                .contributions
                .sectors
                .iter()
                .map(|s| s.progress.id))
            && self.estimate.orders == observation.contributions.orders
            && self.estimate.components == observation.contributions.components
    }
}

fn failed(data: &Cached) -> bool {
    matches!(
        data.observation.contributions.uncertainty,
        UncertaintyStatus::StatisticalFailure { .. }
    ) || data.live.as_ref().is_some_and(|live| {
        live.total.status == fastsecdec::integration::LiveStatus::NumericalRange
    })
}

pub(super) fn previous(data: &Cached) -> Option<&VectorEstimate> {
    if data.observation.contributions.sectors.is_empty()
        || data
            .observation
            .contributions
            .total
            .as_ref()
            .is_some_and(|e| e.production_complete)
        || failed(data)
    {
        return None;
    }
    data.previous_completed
        .as_ref()
        .filter(|previous| previous.matches(&data.observation, &data.scope))
        .map(|previous| &previous.estimate)
}

pub(super) fn label(data: &Cached) -> &'static str {
    if previous(data).is_some() {
        "Previous allocation"
    } else if matches!(
        data.observation.contributions.uncertainty,
        UncertaintyStatus::Exact
    ) {
        "Exact"
    } else if is_qmc(data) {
        "Accepted shifts"
    } else {
        "Accepted batches"
    }
}

pub(super) fn waiting(data: &Cached) -> &'static str {
    if failed(data) {
        return "Stats failed";
    }
    match data.observation.contributions.uncertainty {
        UncertaintyStatus::PilotOnly => "Pilot only",
        UncertaintyStatus::StatisticalFailure { .. } => "Stats failed",
        _ if is_qmc(data) => "Await shifts",
        _ => "Await batches",
    }
}

pub(super) fn value(data: &Cached, index: Option<usize>) -> String {
    let Some(index) = index else {
        return "—".into();
    };
    let estimate = previous(data).or(data.observation.contributions.total.as_ref());
    estimate
        .and_then(|e| {
            e.mean
                .get(index)
                .map(|v| (*v, e.standard_error.get(index).copied()))
        })
        .map_or_else(
            || waiting(data).into(),
            |(value, error)| number::uncertainty(value, error),
        )
}

pub(super) fn explanation(data: &Cached) -> String {
    if previous(data).is_some() {
        format!(
            "Previous completed production allocation; current: {}",
            waiting(data)
        )
    } else if data.observation.contributions.total.is_some() {
        "Accepted complete statistical units in the current allocation".into()
    } else if failed(data) {
        "Accepted statistics failed; no earlier estimate substitutes for this failure".into()
    } else {
        match data.observation.contributions.uncertainty {
            UncertaintyStatus::PilotOnly => {
                "Pilot adapts sampling; no production estimate is accepted yet"
            }
            _ if is_qmc(data) => "Await at least two complete shifted lattices per sector",
            _ => "Await at least two complete batches with full sector coverage",
        }
        .into()
    }
}
