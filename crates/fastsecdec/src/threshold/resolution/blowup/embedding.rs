//! An actual embedded support presentation, retained through ambient maps.
//! Constructors consume existing checked owners, not supplied defining ideals.
use super::super::*;
use super::support::{StrictContactSupport, StrictSupportOpen};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
enum EmbeddingOrigin {
    Contact(Arc<ContactQuotient>),
    Strict {
        parent: Arc<StrictContactSupport>,
        open: Arc<StrictSupportOpen>,
    },
}
#[derive(Clone, Debug)]
pub struct SupportEmbedding {
    origin: EmbeddingOrigin,
    ambient: Arc<EtaleFrame>,
    frame: Arc<EtaleFrame>,
    extension: RingExtension,
    equations: Arc<Ideal>,
    codimension: usize,
}
impl SupportEmbedding {
    pub(crate) fn contact(source: Arc<ContactQuotient>, b: &mut Budget) -> Result<Arc<Self>> {
        let ambient = source.source().frame().clone();
        let p = &source.clearing().numerator;
        let ring = ambient.local().ring();
        if (ring.len()..p.nvars()).any(|i| p.degree(i) > 0) {
            return Err(Error::Invalid("contact embedding has a foreign graph slot"));
        }
        let p = super::super::elimination::remap(p, ring, b)?;
        let equations = Arc::new(Ideal::new(ring.clone(), vec![p], b)?);
        let frame = source.contact().clone();
        let codimension = ambient
            .free_axes()
            .len()
            .checked_sub(frame.free_axes().len())
            .ok_or(Error::Invalid("contact embedding dimension"))?;
        if codimension != 1 {
            return Err(Error::Invalid("contact embedding codimension"));
        }
        Ok(Arc::new(Self {
            extension: source.extension().clone(),
            origin: EmbeddingOrigin::Contact(source),
            ambient,
            frame,
            equations,
            codimension,
        }))
    }
    pub(crate) fn strict(
        parent: Arc<StrictContactSupport>,
        open: Arc<StrictSupportOpen>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !parent.opens().iter().any(|o| Arc::ptr_eq(o, &open)) {
            return Err(Error::Invalid("carried support selected open owner"));
        }
        let ambient = parent.geometry().frame().clone();
        let frame = open.frame().clone();
        let codimension = ambient
            .free_axes()
            .len()
            .checked_sub(frame.free_axes().len())
            .ok_or(Error::Invalid("carried embedding dimension"))?;
        if codimension != parent.embedding().codimension()
            || open.extension().source() != ambient.local().ring()
            || open.extension().target() != frame.local().ring()
            || parent.saturation().empty()
        {
            return Err(Error::Invalid("carried strict embedding association"));
        }
        // Retain ALL saturated equations, including redundant ones; the actual
        // Etale owner proves their equality to its selected relative equations.
        for f in parent.saturation().result().generators() {
            if !frame.local().zero(&open.extension().pull(f, b)?, b)? {
                return Err(Error::Invalid("carried support equation restriction"));
            }
        }
        Ok(Arc::new(Self {
            equations: parent.saturation().result().clone(),
            extension: open.extension().clone(),
            origin: EmbeddingOrigin::Strict { parent, open },
            ambient,
            frame,
            codimension,
        }))
    }
    pub fn original_contact(&self) -> &Arc<ContactQuotient> {
        match &self.origin {
            EmbeddingOrigin::Contact(q) => q,
            EmbeddingOrigin::Strict { parent, .. } => parent.source(),
        }
    }
    pub fn ambient(&self) -> &Arc<EtaleFrame> {
        &self.ambient
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn equations(&self) -> &Arc<Ideal> {
        &self.equations
    }
    pub fn codimension(&self) -> usize {
        self.codimension
    }
    pub fn previous(&self) -> Option<(&Arc<StrictContactSupport>, &Arc<StrictSupportOpen>)> {
        match &self.origin {
            EmbeddingOrigin::Strict { parent, open } => Some((parent, open)),
            EmbeddingOrigin::Contact(_) => None,
        }
    }
}
