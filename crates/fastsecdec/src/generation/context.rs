use super::{
    GeneratedIntegral, GenerationError, GenerationOptions, GenerationProgress, GeometryDispatch,
    geometry::GeometrySource,
};
use crate::{parametric::ParametricIntegrand, status::GeometryReuseStatus};
use fastsecdec_sectors::GeometryCache;
use std::ops::ControlFlow;

/// Events for the additive caller-owned generation entry.
///
/// The original [`GenerationProgress`] remains unchanged for cache-free callers.
#[derive(Clone, Debug)]
pub enum GenerationEvent {
    Progress(GenerationProgress),
    GeometryReuse(GeometryReuseStatus),
}

impl From<GenerationProgress> for GenerationEvent {
    fn from(value: GenerationProgress) -> Self {
        Self::Progress(value)
    }
}

/// Caller-owned reuse across independently admitted parametric integrals.
///
/// Only completed exact geometry is retained. Domain assessments, expressions,
/// coefficients, symmetry decisions and Laurent results belong to each call.
/// The context owns no threads, global state or evaluator workers.
#[derive(Debug)]
pub struct GenerationContext {
    geometry: GeometryCache,
}

impl GenerationContext {
    /// Bound retained geometry by entry count, not bytes; zero disables retention.
    pub fn new(geometry_capacity: usize) -> Self {
        Self {
            geometry: GeometryCache::new(geometry_capacity),
        }
    }

    pub fn geometry_cache(&self) -> &GeometryCache {
        &self.geometry
    }

    /// Allows explicit clearing and direct native geometry reuse by the caller.
    pub fn geometry_cache_mut(&mut self) -> &mut GeometryCache {
        &mut self.geometry
    }

    /// Generate using the same native stages as [`super::generate`].
    ///
    /// Every call records its domain before consulting the cache and checks
    /// mapped algebraic invariants afterward. A `GeometryReuse` event reports native
    /// geometry maps before symmetry merging. Empty inputs skip geometry.
    /// Cancellation at native geometry completion prevents cache insertion;
    /// cancellation at the subsequent reuse event or a later stage may leave a
    /// valid completed geometry entry. It never returns a partial integral.
    pub fn generate(
        &mut self,
        input: &ParametricIntegrand,
        options: &GenerationOptions,
        progress: impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<GeneratedIntegral, GenerationError> {
        super::generate_inner(
            input,
            options,
            GeometrySource::Cached(&mut self.geometry),
            None,
            progress,
        )
    }

    /// Generate with caller-owned chart/cone workers and complete geometry reuse.
    ///
    /// Support admission and the full subsequent symbolic pipeline stay
    /// native and per-integral. On a cache miss the native cache dispatches both
    /// lazy work stages, validates opaque completions and merges canonically.
    /// Hits and empty integrands invoke no dispatcher. The caller owns its pool,
    /// joins and per-job progress channels; local job counters are not aggregate
    /// accepted counts and are not forwarded as ordinary generation events.
    ///
    /// The existing observer receives accepted native geometry completion,
    /// reuse status and all ordinary generation phases. The cancellation token
    /// is checked at these boundaries and inside native stage/merge admission;
    /// worker callbacks should read the same token. Cancellation remains
    /// cooperative, not preemption inside native arithmetic. Failed/cancelled
    /// geometry never enters the cache, while cancellation at a later reuse or
    /// symbolic phase can leave valid completed geometry, as in [`Self::generate`].
    pub fn generate_with_dispatch(
        &mut self,
        input: &ParametricIntegrand,
        options: &GenerationOptions,
        dispatch: &mut GeometryDispatch<'_>,
        cancelled: impl Fn() -> bool,
        mut progress: impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<GeneratedIntegral, GenerationError> {
        if cancelled() {
            return Err(GenerationError::Cancelled);
        }
        let result = super::generate_inner(
            input,
            options,
            GeometrySource::Dispatched {
                cache: &mut self.geometry,
                dispatch,
                cancelled: &cancelled,
            },
            None,
            |event| {
                if cancelled() {
                    ControlFlow::Break(())
                } else {
                    progress(event)
                }
            },
        )?;
        if cancelled() {
            Err(GenerationError::Cancelled)
        } else {
            Ok(result)
        }
    }
    /// Dispatch geometry, mapping, symmetry preparation and coefficient work
    /// on a caller-owned executor. Exact symmetry admission and ordered final
    /// assembly remain serial; native graph construction/canonization runs in jobs.
    /// Opaque completions are checked for call ownership and exact stage coverage.
    pub fn generate_with_all_dispatch(
        &mut self,
        input: &ParametricIntegrand,
        options: &GenerationOptions,
        geometry_dispatch: &mut GeometryDispatch<'_>,
        symbolic_dispatch: &mut super::SymbolicDispatch<'_>,
        cancelled: impl Fn() -> bool,
        mut progress: impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<GeneratedIntegral, GenerationError> {
        if cancelled() {
            return Err(GenerationError::Cancelled);
        }
        let result = super::generate_inner(
            input,
            options,
            GeometrySource::Dispatched {
                cache: &mut self.geometry,
                dispatch: geometry_dispatch,
                cancelled: &cancelled,
            },
            Some(symbolic_dispatch),
            |event| {
                if cancelled() {
                    ControlFlow::Break(())
                } else {
                    progress(event)
                }
            },
        )?;
        if cancelled() {
            Err(GenerationError::Cancelled)
        } else {
            Ok(result)
        }
    }
}

pub(super) fn emit(
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    event: impl Into<GenerationEvent>,
) -> Result<(), GenerationError> {
    if progress(&event.into()).is_break() {
        Err(GenerationError::Cancelled)
    } else {
        Ok(())
    }
}
