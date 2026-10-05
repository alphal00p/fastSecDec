//! Two native geometry stages, scheduled entirely by the caller.
mod assembly;
mod job;
#[cfg(test)]
mod tests;

use crate::{stages::ChartData, *};
use std::sync::Arc;

/// Work identity within one in-process geometry plan, not a sector content ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeometryJobId {
    Chart { chart: usize },
    Cone { chart: usize, candidate: usize },
}

/// Invalid work cannot produce a partial or completed decomposition.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GeometryWorkError {
    #[error("geometry completion belongs to another plan")]
    ForeignPlan,
    #[error("geometry completion belongs to the wrong stage")]
    WrongStage,
    #[error("unknown geometry job {0:?}")]
    UnknownJob(GeometryJobId),
    #[error("duplicate geometry job {0:?}")]
    DuplicateJob(GeometryJobId),
    #[error("missing {count} geometry jobs, beginning with {first:?}")]
    MissingJobs { first: GeometryJobId, count: usize },
}

/// Validated native geometry input. Owns no pool, Atom or physical certificate.
///
/// Enumerate [`Self::charts`], execute jobs on the caller's workers, then pass
/// their private completions to [`Self::prepare`]. Equal-input plans remain
/// distinct: completions may only be returned to their originating plan.
pub struct GeometryPlan {
    inner: Arc<PlanData>,
}

/// Prepared native chart facets shared by candidate-cone jobs.
///
/// This is not a decomposition. Only [`Self::finish`] can return the fully
/// covered fan. Chart preparation errors are retained until canonical merge,
/// so a later chart error cannot mask an earlier cone/sector-limit error.
/// All prepared facets and out-of-order completions can consume more storage
/// than the serial driver; bounded in-flight jobs are not a byte-memory bound.
pub struct PreparedGeometry {
    inner: Arc<PreparedData>,
}

/// Immutable, cheaply cloned native work. The caller chooses its executor.
/// Re-executing a cloned job is allowed; accepting its result twice is not.
#[derive(Clone)]
pub struct GeometryJob {
    owner: Owner,
    id: GeometryJobId,
}

/// Native result and private provenance. No caller-created maps are admitted.
/// A failed cone privately retains its successful map prefix so final merge
/// checks cumulative limits before returning a later map-conversion error.
/// That prefix can never be extracted as a public partial decomposition.
pub struct GeometryCompletion {
    owner: Owner,
    id: GeometryJobId,
    payload: Payload,
    status: DecompositionProgress,
}

struct PlanData {
    domain: ParametricDomain,
    supports: Vec<PolynomialSupport>,
    options: DecompositionOptions,
    fixed: Vec<Option<usize>>,
}

struct PreparedData {
    plan: Arc<PlanData>,
    charts: Vec<ChartCompletion>,
    // Prefix offsets for successful charts preceding the earliest failed one.
    // Zero-dimensional charts have no cone jobs.
    offsets: Vec<usize>,
}

struct ChartCompletion {
    result: Result<ChartData, SectorError>,
    status: DecompositionProgress,
}

#[derive(Clone)]
enum Owner {
    Charts(Arc<PlanData>),
    Cones(Arc<PreparedData>),
}

enum Payload {
    Chart(Result<ChartData, SectorError>),
    Cone {
        maps: Vec<SectorMap>,
        geometric: bool,
        error: Option<SectorError>,
    },
}

impl GeometryCompletion {
    pub fn id(&self) -> GeometryJobId {
        self.id
    }

    /// Inspect/log every observed error before consuming completions. A later
    /// canonical merge may instead return an earlier cumulative sector limit.
    pub fn error(&self) -> Option<&SectorError> {
        match &self.payload {
            Payload::Chart(result) => result.as_ref().err(),
            Payload::Cone { error, .. } => error.as_ref(),
        }
    }
}

impl PreparedData {
    fn count(&self) -> usize {
        *self.offsets.last().unwrap()
    }

    fn id(&self, index: usize) -> GeometryJobId {
        let chart = self.offsets.partition_point(|offset| *offset <= index) - 1;
        GeometryJobId::Cone {
            chart,
            candidate: index - self.offsets[chart],
        }
    }

    fn index(&self, id: GeometryJobId) -> Option<usize> {
        let GeometryJobId::Cone { chart, candidate } = id else {
            return None;
        };
        let start = *self.offsets.get(chart)?;
        let end = *self.offsets.get(chart.checked_add(1)?)?;
        (candidate < end - start).then(|| start + candidate)
    }
}
