//! Exact strict-support saturation in the ORIGINAL target localization.
//! The temporary exceptional inverse is eliminated, never retained as a guard.
use super::super::{
    elimination::{EliminationEvidence, auxiliary_ring, native_basis, remap},
    *,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct SaturatedSupport {
    local: Arc<LocalizedAlgebra>,
    source: Arc<Ideal>,
    factor: Poly,
    result: Arc<Ideal>,
    evidence: EliminationEvidence,
    empty: bool,
}
impl SaturatedSupport {
    pub fn local(&self) -> &Arc<LocalizedAlgebra> {
        &self.local
    }
    pub fn source(&self) -> &Arc<Ideal> {
        &self.source
    }
    pub fn factor(&self) -> &Poly {
        &self.factor
    }
    pub fn result(&self) -> &Arc<Ideal> {
        &self.result
    }
    pub fn evidence(&self) -> &EliminationEvidence {
        &self.evidence
    }
    pub fn empty(&self) -> bool {
        self.empty
    }
    /// Recompute from the retained original equations. Comparing against an
    /// unchecked supplied Groebner basis does not confer transform authority.
    pub fn verify_candidate(
        &self,
        candidate: &Ideal,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<bool> {
        if candidate.ring() != self.local.ring() {
            return Err(Error::Invalid("saturation candidate ring"));
        }
        let fresh = saturate_support(
            self.local.clone(),
            self.source.clone(),
            self.factor.clone(),
            namespace,
            b,
        )?;
        let left = self.local.ideal().sum(candidate, b)?;
        let right = self.local.ideal().sum(&fresh.result, b)?;
        for f in left.generators() {
            if !right.contains(f, self.local.unit_relations(), b)? {
                return Ok(false);
            }
        }
        for f in right.generators() {
            if !left.contains(f, self.local.unit_relations(), b)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
/// Algebraic saturation only. Geometric strict-transform authority requires a
/// separate actual support and checked ambient blowup owner.
pub(crate) fn saturate_support(
    local: Arc<LocalizedAlgebra>,
    source: Arc<Ideal>,
    factor: Poly,
    namespace: &str,
    b: &mut Budget,
) -> Result<Arc<SaturatedSupport>> {
    if source.ring() != local.ring() {
        return Err(Error::Invalid("saturation source ring"));
    }
    local.supports(&factor)?;
    b.poly(&factor)?;
    for f in source.generators() {
        local.supports(f)?;
        b.poly(f)?;
    }
    let ring = auxiliary_ring(local.ring(), 1, namespace, b)?;
    let count = local
        .ideal()
        .generators()
        .len()
        .checked_add(local.unit_relations().len())
        .and_then(|n| n.checked_add(source.generators().len()))
        .and_then(|n| n.checked_add(1))
        .ok_or(Error::ResourceIncomplete("saturation equation count"))?;
    b.reserve_slots(count)?;
    let mut equations = local
        .ideal()
        .generators()
        .iter()
        .chain(local.unit_relations())
        .chain(source.generators())
        .map(|f| remap(f, &ring, b))
        .collect::<Result<Vec<_>>>()?;
    let e = remap(&factor, &ring, b)?;
    equations.push(b.mul(&ring.coordinate(0)?, &e)? - ring.one());
    let evidence = native_basis(ring, 1, equations, b)?;
    let normalizer = QuotientNormalizer::prepare(local.clone(), b)?;
    b.reserve_slots(evidence.basis().generators().len())?;
    let mut generators = Vec::new();
    for f in evidence.basis().generators() {
        if f.degree(0) > 0 {
            continue;
        }
        let mapped = remap(f, local.ring(), b)?;
        b.charge(1)?;
        let reduced = mapped.reduce(normalizer.relation_basis().generators());
        local.supports(&reduced)?;
        b.poly(&reduced)?;
        generators.push(reduced);
    }
    let result = Arc::new(Ideal::new(local.ring().clone(), generators, b)?);
    let combined = local.ideal().sum(&result, b)?;
    for f in source.generators() {
        if !combined.contains(f, local.unit_relations(), b)? {
            return Err(Error::Invalid("saturation lost original support equation"));
        }
    }
    let empty = combined.contains(&local.ring().one(), local.unit_relations(), b)?;
    Ok(Arc::new(SaturatedSupport {
        local,
        source,
        factor,
        result,
        evidence,
        empty,
    }))
}
