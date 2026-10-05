use crate::{
    Decomposition, DecompositionOptions, DecompositionPhase, DecompositionProgress,
    ParametricDomain, PolynomialSupport, SectorError, decompose,
};
use std::{collections::VecDeque, ops::ControlFlow, sync::Arc};

mod dispatch;
pub use dispatch::GeometryDispatch;

/// A complete native geometry result and whether this request reused an entry.
#[derive(Clone, Debug)]
pub struct GeometryCacheOutcome {
    pub decomposition: Arc<Decomposition>,
    /// False for a fresh computation, including when caching is disabled.
    pub reused: bool,
}

/// Caller-owned reuse of complete, exact sector decompositions.
///
/// Capacity counts entries, **not bytes**. The oldest inserted entry is evicted
/// after a successful miss when full; a hit does not change insertion order.
/// Zero capacity disables retention. Returned shared results remain valid after
/// eviction or [`Self::clear`]. There is no global state, worker pool or disk
/// format, and the ordinary [`decompose`] function remains cache-free.
///
/// Keys retain the input domain, ordered native supports (including coordinate
/// order and monomial translations), and all three resource limits. Thus a warm
/// result cannot bypass a tighter limit or misassign positional valuations.
#[derive(Debug)]
pub struct GeometryCache {
    capacity: usize,
    entries: VecDeque<Entry>,
}

#[derive(Debug)]
struct Entry {
    domain: ParametricDomain,
    supports: Vec<PolynomialSupport>,
    limits: [usize; 3],
    decomposition: Arc<Decomposition>,
    completion: DecompositionProgress,
}

impl GeometryCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            capacity: max_entries,
            entries: VecDeque::new(),
        }
    }

    /// Maximum retained entry count, not a memory-size bound.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Reuse a completed entry or invoke the existing native decomposition.
    ///
    /// Misses forward native progress unchanged and insert only after successful
    /// completion, including acceptance of the final `Complete` callback. Hits
    /// invoke the callback once with the retained native `Complete` report; its
    /// fields describe that completed result, not work performed by this lookup.
    /// The returned `reused` flag distinguishes the two paths. Cancellation on a
    /// hit returns [`SectorError::Cancelled`] without removing the valid entry.
    /// Failed or cancelled misses neither insert nor evict any entry.
    pub fn decompose(
        &mut self,
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        options: &DecompositionOptions,
        mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
    ) -> Result<GeometryCacheOutcome, SectorError> {
        let limits = [
            options.max_support_pairs,
            options.max_rays,
            options.max_sectors,
        ];
        if let Some(hit) = self.lookup(domain, supports, limits, &mut progress)? {
            return Ok(hit);
        }

        let mut completion = None;
        let result = decompose(domain, supports, options, |status| {
            let decision = progress(status);
            if decision.is_continue() && status.phase == DecompositionPhase::Complete {
                completion = Some(status.clone());
            }
            decision
        })?;
        self.store(domain, supports, limits, result, completion)
    }

    fn lookup(
        &self,
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        limits: [usize; 3],
        progress: &mut impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
    ) -> Result<Option<GeometryCacheOutcome>, SectorError> {
        let Some(entry) = self.entries.iter().find(|entry| {
            entry.domain == domain && entry.supports == supports && entry.limits == limits
        }) else {
            return Ok(None);
        };
        if progress(&entry.completion).is_break() {
            return Err(SectorError::Cancelled);
        }
        Ok(Some(GeometryCacheOutcome {
            decomposition: Arc::clone(&entry.decomposition),
            reused: true,
        }))
    }

    // Only native serial completion or work::finish can reach this boundary.
    // There is deliberately no public insertion of caller-created maps.
    fn store(
        &mut self,
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        limits: [usize; 3],
        result: Decomposition,
        completion: Option<DecompositionProgress>,
    ) -> Result<GeometryCacheOutcome, SectorError> {
        let completion = completion.ok_or_else(|| {
            SectorError::Geometry("decomposition returned without accepted completion".into())
        })?;
        let decomposition = Arc::new(result);
        if self.capacity != 0 {
            if self.entries.len() == self.capacity {
                self.entries.pop_front();
            }
            self.entries.push_back(Entry {
                domain,
                supports: supports.to_vec(),
                limits,
                decomposition: Arc::clone(&decomposition),
                completion,
            });
        }
        Ok(GeometryCacheOutcome {
            decomposition,
            reused: false,
        })
    }
}
