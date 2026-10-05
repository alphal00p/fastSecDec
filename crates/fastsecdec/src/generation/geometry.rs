use super::{GenerationError, GenerationEvent, GenerationProgress, context::emit};
use crate::status::GeometryReuseStatus;
use fastsecdec_sectors::{
    Decomposition, DecompositionOptions, GeometryCache, GeometryDispatch, ParametricDomain,
    PolynomialSupport, SectorMap, decompose,
};
use std::{borrow::Cow, ops::ControlFlow, sync::Arc};

pub(super) enum Geometry {
    Owned(Decomposition),
    Shared(Arc<Decomposition>),
}

// Only the geometry stage differs. Every route returns native complete maps to
// the same per-integral mapping, symmetry, subtraction and Laurent pipeline.
pub(super) enum GeometrySource<'a, 'dispatch> {
    Uncached,
    Cached(&'a mut GeometryCache),
    Dispatched {
        cache: &'a mut GeometryCache,
        dispatch: &'a mut GeometryDispatch<'dispatch>,
        cancelled: &'a dyn Fn() -> bool,
    },
}

impl Geometry {
    pub fn compute(
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        options: &DecompositionOptions,
        source: GeometrySource<'_, '_>,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<Self, GenerationError> {
        let observer = |status: &fastsecdec_sectors::DecompositionProgress| {
            if emit(progress, GenerationProgress::Decomposition(status.clone())).is_err() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        };
        let result = match source {
            GeometrySource::Uncached => {
                return Ok(Self::Owned(decompose(domain, supports, options, observer)?));
            }
            GeometrySource::Cached(cache) => {
                cache.decompose(domain, supports, options, observer)?
            }
            GeometrySource::Dispatched {
                cache,
                dispatch,
                cancelled,
            } => cache.decompose_with_dispatch(
                domain, supports, options, dispatch, cancelled, observer,
            )?,
        };
        emit(
            progress,
            GenerationEvent::GeometryReuse(GeometryReuseStatus {
                reused: result.reused,
                sectors: result.decomposition.sectors.len(),
            }),
        )?;
        Ok(Self::Shared(result.decomposition))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Owned(value) => value.sectors.len(),
            Self::Shared(value) => value.sectors.len(),
        }
    }

    // Keep the ordinary path's owned maps. Cache hits borrow native maps and
    // clone only those required by the generated result's owned metadata.
    pub fn maps(&mut self) -> Box<dyn Iterator<Item = Cow<'_, SectorMap>> + '_> {
        match self {
            Self::Owned(value) => Box::new(
                std::mem::take(&mut value.sectors)
                    .into_iter()
                    .map(Cow::Owned),
            ),
            Self::Shared(value) => Box::new(value.sectors.iter().map(Cow::Borrowed)),
        }
    }
}
