use super::*;
use crate::{GeometryCompletion, GeometryJob, GeometryPlan};

/// Caller-owned execution of one native geometry stage.
///
/// Jobs are lazy, immutable and stage-bound. The caller chooses its own worker
/// limit, cancellation/progress channels and joins. It returns native opaque
/// completions in any order, never a manufactured decomposition. Job callbacks
/// describe local work; the final cache observer receives global completion.
pub type GeometryDispatch<'a> = dyn FnMut(
        &mut dyn ExactSizeIterator<Item = GeometryJob>,
    ) -> Result<Vec<GeometryCompletion>, SectorError>
    + 'a;

impl GeometryCache {
    /// Reuse a completed entry or dispatch the two existing native work stages.
    ///
    /// Hits emit only retained `Complete` and never dispatch. Misses dispatch
    /// charts and then cones (including an empty native cone stage); this cache
    /// owns plan construction, complete provenance/coverage validation, ordered
    /// merge and insertion. Native worker errors should remain in completions
    /// for canonical resolution. Scheduling errors may be returned directly.
    ///
    /// The caller owns stopping/joining its tasks before returning. Cancellation
    /// is cooperative; workers should observe the same caller token through
    /// their native callbacks. Failed/cancelled work cannot insert or evict an
    /// entry. Cancellation on a hit retains the valid entry. No pool is created,
    /// and entry/in-flight counts are not byte-memory bounds.
    pub fn decompose_with_dispatch(
        &mut self,
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        options: &DecompositionOptions,
        dispatch: &mut GeometryDispatch<'_>,
        mut cancelled: impl FnMut() -> bool,
        mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
    ) -> Result<GeometryCacheOutcome, SectorError> {
        if cancelled() {
            return Err(SectorError::Cancelled);
        }
        let limits = [
            options.max_support_pairs,
            options.max_rays,
            options.max_sectors,
        ];
        if let Some(hit) = self.lookup(domain, supports, limits, &mut progress)? {
            return if cancelled() {
                Err(SectorError::Cancelled)
            } else {
                Ok(hit)
            };
        }

        let plan = GeometryPlan::new(domain, supports.to_vec(), options.clone())?;
        if cancelled() {
            return Err(SectorError::Cancelled);
        }
        let charts = dispatch(&mut plan.charts())?;
        let prepared = plan.prepare(charts, &mut cancelled)?;
        if cancelled() {
            return Err(SectorError::Cancelled);
        }
        let cones = dispatch(&mut prepared.cones())?;
        let mut completion = None;
        let result = prepared.finish(cones, &mut cancelled, |status| {
            let decision = progress(status);
            if decision.is_continue() && status.phase == DecompositionPhase::Complete {
                completion = Some(status.clone());
            }
            decision
        })?;
        if cancelled() {
            return Err(SectorError::Cancelled);
        }
        self.store(domain, supports, limits, result, completion)
    }
}
