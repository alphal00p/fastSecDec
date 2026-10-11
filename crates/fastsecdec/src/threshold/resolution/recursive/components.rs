//! A verified disconnected terminal contact requires a physical component cover.
//! This receipt is explicitly incomplete: it cannot issue a global zero center.
use super::*;
#[derive(Clone, Debug)]
pub struct RecursiveComponentCover {
    origin: Arc<AuxiliaryRecursionOrigin>,
    frame: Arc<EtaleFrame>,
    source: Arc<MarkedIdeal>,
    normalization: Arc<VerifiedQuotientNormalization>,
    levels: Vec<Arc<RecursiveLevel>>,
    split: Arc<VerifiedComponentSplit>,
}
impl RecursiveComponentCover {
    pub fn origin(&self) -> &Arc<AuxiliaryRecursionOrigin> {
        &self.origin
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn normalization(&self) -> &Arc<VerifiedQuotientNormalization> {
        &self.normalization
    }
    pub fn levels(&self) -> &[Arc<RecursiveLevel>] {
        &self.levels
    }
    pub fn split(&self) -> &Arc<VerifiedComponentSplit> {
        &self.split
    }
    pub(super) fn prepare(
        origin: Arc<AuxiliaryRecursionOrigin>,
        frame: Arc<EtaleFrame>,
        source: Arc<MarkedIdeal>,
        normalization: Arc<VerifiedQuotientNormalization>,
        levels: &[Arc<RecursiveLevel>],
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !frame.free_axes().is_empty()
            || (0..frame.local().ring().len()).any(|i| frame.local().ring().is_parameter(i))
            || !Arc::ptr_eq(normalization.source(), &source)
            || !Arc::ptr_eq(normalization.normalizer().local(), frame.local())
        {
            return Err(Error::Invalid(
                "terminal component actual fixed-fiber source",
            ));
        }
        let regular = RegularAlgebra::ambient(frame.clone());
        let split =
            match produce_component_split(regular, Arc::new(source.ideal().clone()), namespace, b)?
            {
                ComponentProduction::Complete(s) => Arc::new(*s),
                ComponentProduction::Incomplete { reason, .. } => {
                    return Err(Error::ResourceIncomplete(reason));
                }
            };
        if split.pattern() != ComponentPattern::Mixed {
            return Err(Error::Invalid(
                "proper zero-dimensional ideal requires mixed components",
            ));
        }
        b.reserve_slots(levels.len())?;
        Ok(Arc::new(Self {
            origin,
            frame,
            source,
            normalization,
            levels: levels.to_vec(),
            split,
        }))
    }
}
pub(super) const COMPONENT_REASON: &str =
    "verified terminal components need common physical localization";

#[cfg(test)]
impl RecursiveComponentCover {
    pub(super) fn test_replaced_origin(&self, origin: Arc<AuxiliaryRecursionOrigin>) -> Arc<Self> {
        let mut copy = self.clone();
        copy.origin = origin;
        Arc::new(copy)
    }
}
