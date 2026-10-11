//! Original coefficient presentations retained across successive ambient maps.
//! These owners do not identify local centers across distinct ambient charts.
use super::super::*;
use super::{induced::*, support::*};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub(crate) enum PresentationOrigin {
    Initial(Arc<InducedChildChart>),
    Transition {
        owner: Arc<super::presentation_transition::EmbeddedTransition>,
        open: usize,
    },
    Incidence(Arc<super::incidence::IncidenceDrop>),
}
#[derive(Clone, Debug)]
pub struct EmbeddedPresentation {
    pub(crate) origin: PresentationOrigin,
    pub(crate) chart: Arc<RelativeRecursiveChart>,
    pub(crate) companion: Arc<RelativeMarkedTransform>,
    pub(crate) support: Arc<StrictContactSupport>,
    pub(crate) open: Arc<StrictSupportOpen>,
    pub(crate) coefficient: Arc<MarkedIdeal>,
    pub(crate) incidence_sum: Arc<MarkedIdeal>,
    pub(crate) history: Arc<ResolutionHistory>,
    pub(crate) old: Vec<BoundaryId>,
    pub(crate) incidence: usize,
}
impl EmbeddedPresentation {
    pub fn from_induced(source: Arc<InducedChildChart>, b: &mut Budget) -> Result<Arc<Self>> {
        let q = source.support();
        if !Arc::ptr_eq(q.coefficient().open(), q.incidence_sum().open())
            || !Arc::ptr_eq(
                q.incidence_sum().open().frame(),
                source.history().ledger().frame(),
            )
            || !Arc::ptr_eq(q.incidence_sum().support(), source.ambient().support())
            || !source
                .history()
                .same_root(source.origin().initial_history())
        {
            return Err(Error::Invalid("initial carried presentation owner"));
        }
        let old = source
            .origin()
            .center()
            .coefficient()
            .coefficient()
            .active_old()
            .to_vec();
        b.reserve_slots(old.len())?;
        Ok(Arc::new(Self {
            chart: source.ambient().chart().clone(),
            companion: source.ambient().companion().clone(),
            support: source.ambient().support().clone(),
            open: q.incidence_sum().open().clone(),
            coefficient: q.coefficient().target().clone(),
            incidence_sum: q.incidence_sum().target().clone(),
            history: source.history().clone(),
            incidence: source.origin().center().old_incidence(),
            old,
            origin: PresentationOrigin::Initial(source),
        }))
    }
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn companion(&self) -> &Arc<RelativeMarkedTransform> {
        &self.companion
    }
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn open(&self) -> &Arc<StrictSupportOpen> {
        &self.open
    }
    pub fn coefficient(&self) -> &Arc<MarkedIdeal> {
        &self.coefficient
    }
    pub fn incidence_sum(&self) -> &Arc<MarkedIdeal> {
        &self.incidence_sum
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn old_ids(&self) -> &[BoundaryId] {
        &self.old
    }
    pub fn old_incidence(&self) -> usize {
        self.incidence
    }
    pub fn original_cycle(&self) -> &Arc<EmbeddedChildCycle> {
        match &self.origin {
            PresentationOrigin::Initial(o) => o.origin(),
            PresentationOrigin::Transition { owner, .. } => owner.source().original_cycle(),
            PresentationOrigin::Incidence(owner) => owner.source().original_cycle(),
        }
    }
    pub fn previous_transition(
        &self,
    ) -> Option<(
        &Arc<super::presentation_transition::EmbeddedTransition>,
        usize,
    )> {
        match &self.origin {
            PresentationOrigin::Transition { owner, open } => Some((owner, *open)),
            _ => None,
        }
    }
}
