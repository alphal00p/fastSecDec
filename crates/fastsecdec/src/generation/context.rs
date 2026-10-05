use super::{GeneratedIntegral, GenerationError, GenerationOptions, GenerationProgress};
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
    /// Every call reassesses its own domain before consulting the cache and
    /// checks mapped residuals afterward. A `GeometryReuse` event reports native
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
        super::generate_inner(input, options, Some(&mut self.geometry), progress)
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
