use super::*;
use crate::{
    decompose::initial_status,
    stages::{decompose_candidate, prepare_chart, validate_input},
    types::Monitor,
};
use std::ops::ControlFlow;

impl GeometryPlan {
    /// Use the same dimension/homogeneity admission as the serial API.
    pub fn new(
        domain: ParametricDomain,
        supports: Vec<PolynomialSupport>,
        options: DecompositionOptions,
    ) -> Result<Self, SectorError> {
        let dimension = validate_input(domain, &supports)?;
        let fixed = if domain == ParametricDomain::ProjectiveSimplex {
            (0..dimension).map(Some).collect()
        } else {
            vec![None]
        };
        Ok(Self {
            inner: Arc::new(PlanData {
                domain,
                supports,
                options,
                fixed,
            }),
        })
    }

    /// Lazy native chart order. Yielded jobs own their shared input references.
    pub fn charts(&self) -> impl ExactSizeIterator<Item = GeometryJob> + '_ {
        (0..self.inner.fixed.len()).map(|chart| GeometryJob {
            owner: Owner::Charts(Arc::clone(&self.inner)),
            id: GeometryJobId::Chart { chart },
        })
    }
}

impl PreparedGeometry {
    /// Lazy native chart/candidate order. Later charts after a preparation
    /// failure need no work; the error is resolved by ordered final merge.
    pub fn cones(&self) -> impl ExactSizeIterator<Item = GeometryJob> + '_ {
        (0..self.inner.count()).map(|index| GeometryJob {
            owner: Owner::Cones(Arc::clone(&self.inner)),
            id: self.inner.id(index),
        })
    }
}

impl GeometryJob {
    pub fn id(&self) -> GeometryJobId {
        self.id
    }

    /// Run existing exact native operations without creating threads.
    ///
    /// The callback belongs to this invocation. Capture [`Self::id`] to tag
    /// observations; `status.sectors` counts local job work, not global accepted
    /// maps. Jobs emit no `Complete` event. A caller-owned atomic cancellation
    /// flag may be read here; only the caller coordinates its UI observer.
    /// Cancellation is cooperative at existing native arithmetic boundaries.
    pub fn run(
        &self,
        mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
    ) -> GeometryCompletion {
        let (payload, status) = match (&self.owner, self.id) {
            (Owner::Charts(plan), GeometryJobId::Chart { chart }) => {
                let fixed = plan.fixed[chart];
                let mut monitor = Monitor {
                    callback: &mut progress,
                    options: &plan.options,
                    status: initial_status(chart, 0),
                };
                let result = monitor.emit().and_then(|()| {
                    let supports = plan
                        .supports
                        .iter()
                        .map(|s| fixed.map_or_else(|| s.clone(), |axis| s.without_axis(axis)))
                        .collect();
                    prepare_chart(
                        supports,
                        fixed,
                        plan.domain != ParametricDomain::PositiveOrthant,
                        &mut monitor,
                    )
                });
                (Payload::Chart(result), monitor.status)
            }
            (Owner::Cones(prepared), GeometryJobId::Cone { chart, candidate }) => {
                let native = prepared.charts[chart]
                    .result
                    .as_ref()
                    .expect("private job requires successful chart");
                let mut status = prepared.charts[chart].status.clone();
                status.sectors = 0;
                let mut monitor = Monitor {
                    callback: &mut progress,
                    options: &prepared.plan.options,
                    status,
                };
                let mut maps = Vec::new();
                let result = decompose_candidate(native, candidate, &mut maps, &mut monitor);
                let (geometric, error) = match result {
                    Ok(g) => (g, None),
                    Err(e) => (false, Some(e)),
                };
                (
                    Payload::Cone {
                        maps,
                        geometric,
                        error,
                    },
                    monitor.status,
                )
            }
            _ => unreachable!("job fields are private and stage-bound"),
        };
        GeometryCompletion {
            owner: self.owner.clone(),
            id: self.id,
            payload,
            status,
        }
    }
}
