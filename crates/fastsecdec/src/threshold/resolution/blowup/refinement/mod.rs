//! Common refinements of actual nested strict-support minor covers.
//! Native variable maps and all localized equations are retained. This is
//! algebraic cover evidence, not disjoint real-integration ownership.
pub(super) mod frame;
use super::super::*;
use super::support::{StrictContactSupport, StrictSupportOpen};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
pub use frame::{RefinedSupportOpen, SupportFrameMap};
#[derive(Clone, Debug)]
pub struct EmptySupportRefinement {
    parent: usize,
    child: usize,
    equations: Ideal,
    unit_relations: Vec<Poly>,
}
impl EmptySupportRefinement {
    pub fn indices(&self) -> (usize, usize) {
        (self.parent, self.child)
    }
    pub fn equations(&self) -> &Ideal {
        &self.equations
    }
    pub fn unit_relations(&self) -> &[Poly] {
        &self.unit_relations
    }
}
#[derive(Clone, Debug)]
pub struct RefinedSupportCover {
    parent: Arc<StrictContactSupport>,
    child: Arc<StrictContactSupport>,
    proof: VerifiedOpenCover,
    opens: Vec<Arc<RefinedSupportOpen>>,
    empty: Vec<EmptySupportRefinement>,
}
impl RefinedSupportCover {
    pub fn parent(&self) -> &Arc<StrictContactSupport> {
        &self.parent
    }
    pub fn child(&self) -> &Arc<StrictContactSupport> {
        &self.child
    }
    pub fn proof(&self) -> &VerifiedOpenCover {
        &self.proof
    }
    pub fn opens(&self) -> &[Arc<RefinedSupportOpen>] {
        &self.opens
    }
    pub fn empty(&self) -> &[EmptySupportRefinement] {
        &self.empty
    }
    pub(crate) fn prepare(
        parent: Arc<StrictContactSupport>,
        child: Arc<StrictContactSupport>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !Arc::ptr_eq(parent.geometry(), child.geometry())
            || !child.embedding().descends_from(parent.embedding())
        {
            return Err(Error::Invalid(
                "common support actual physical/embedding ancestry",
            ));
        }
        let local = child.geometry().frame().local();
        let target = local.ideal().sum(child.saturation().result(), b)?;
        for f in parent.saturation().result().generators() {
            if !target.contains(f, local.unit_relations(), b)? {
                return Err(Error::Invalid("nested strict supports lost containment"));
            }
        }
        let count = parent
            .opens()
            .len()
            .checked_mul(child.opens().len())
            .ok_or(Error::ResourceIncomplete("common support product count"))?;
        b.reserve_slots(count)?;
        let mut factors = Vec::new();
        for p in parent.opens() {
            for c in child.opens() {
                factors.push(b.mul(p.relative_minor(), c.relative_minor())?);
            }
        }
        let proof = OpenCoverCertificate {
            algebra: local.clone(),
            support: (**child.saturation().result()).clone(),
            opens: factors,
        }
        .verify(b)?;
        let mut opens = Vec::new();
        let mut empty = Vec::new();
        b.reserve_slots(count)?;
        for (pi, p) in parent.opens().iter().enumerate() {
            for (ci, c) in child.opens().iter().enumerate() {
                match frame::prepare(
                    p.clone(),
                    c.clone(),
                    local.ring(),
                    &format!("{namespace}_p{pi}_c{ci}"),
                    b,
                )? {
                    frame::FrameProduction::Complete(o) => opens.push(o),
                    frame::FrameProduction::Empty {
                        equations,
                        unit_relations,
                    } => empty.push(EmptySupportRefinement {
                        parent: pi,
                        child: ci,
                        equations,
                        unit_relations,
                    }),
                }
            }
        }
        if opens.len().checked_add(empty.len()) != Some(count) {
            return Err(Error::Invalid("common support pair inventory"));
        }
        Ok(Arc::new(Self {
            parent,
            child,
            proof,
            opens,
            empty,
        }))
    }
}
