use super::{GenerationError, GenerationEvent, GenerationProgress, context::emit};
use crate::status::GeometryReuseStatus;
use fastsecdec_sectors::{
    Decomposition, DecompositionOptions, GeometryCache, ParametricDomain, PolynomialSupport,
    SectorMap, decompose,
};
use std::{borrow::Cow, ops::ControlFlow, sync::Arc};

pub(super) enum Geometry {
    Owned(Decomposition),
    Shared(Arc<Decomposition>),
}

impl Geometry {
    pub fn compute(
        domain: ParametricDomain,
        supports: &[PolynomialSupport],
        options: &DecompositionOptions,
        cache: Option<&mut GeometryCache>,
        progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
    ) -> Result<Self, GenerationError> {
        let observer = |status: &fastsecdec_sectors::DecompositionProgress| {
            if emit(progress, GenerationProgress::Decomposition(status.clone())).is_err() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        };
        if let Some(cache) = cache {
            let result = cache.decompose(domain, supports, options, observer)?;
            emit(
                progress,
                GenerationEvent::GeometryReuse(GeometryReuseStatus {
                    reused: result.reused,
                    sectors: result.decomposition.sectors.len(),
                }),
            )?;
            Ok(Self::Shared(result.decomposition))
        } else {
            Ok(Self::Owned(decompose(domain, supports, options, observer)?))
        }
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
