use super::super::{Budget, Error, Ideal, MarkedIdeal, Poly, QuotientNormalizer, Ring};
use super::RegularAlgebra;
use std::sync::Arc;
use symbolica::{atom::Symbol, poly::PolyVariable, symbol};
type Result<T> = std::result::Result<T, Error>;

/// Exact native lex problem. This contains no authority until produced by the
/// private constructor from the retained original quotient and ideal.
#[derive(Clone, Debug)]
pub struct EliminationEvidence {
    ring: Arc<Ring>,
    auxiliary_axes: usize,
    equations: Ideal,
    basis: Ideal,
}
impl EliminationEvidence {
    pub fn ring(&self) -> &Arc<Ring> {
        &self.ring
    }
    pub fn auxiliary_axes(&self) -> usize {
        self.auxiliary_axes
    }
    pub fn equations(&self) -> &Ideal {
        &self.equations
    }
    pub fn basis(&self) -> &Ideal {
        &self.basis
    }
}
#[derive(Clone, Debug)]
pub struct VerifiedAnnihilator {
    owner: Arc<RegularAlgebra>,
    source: Arc<Ideal>,
    annihilator: Arc<Ideal>,
    evidence: EliminationEvidence,
}
impl VerifiedAnnihilator {
    pub fn owner(&self) -> &Arc<RegularAlgebra> {
        &self.owner
    }
    pub fn source(&self) -> &Arc<Ideal> {
        &self.source
    }
    pub fn annihilator(&self) -> &Arc<Ideal> {
        &self.annihilator
    }
    pub fn evidence(&self) -> &EliminationEvidence {
        &self.evidence
    }
    /// Independent candidate admission recomputes the original elimination;
    /// product identities or a caller-supplied Groebner basis are insufficient.
    pub fn verify_candidate(
        &self,
        candidate: &Ideal,
        namespace: &str,
        budget: &mut Budget,
    ) -> Result<bool> {
        if candidate.ring() != self.owner.local().ring() {
            return Err(Error::Invalid("component candidate ring"));
        }
        budget.reserve_slots(candidate.generators().len())?;
        for f in candidate.generators() {
            self.owner.local().supports(f)?;
            budget.poly(f)?;
        }
        let fresh =
            produce_annihilator(self.owner.clone(), self.source.clone(), namespace, budget)?;
        let normalizer = QuotientNormalizer::prepare(self.owner.local().clone(), budget)?;
        let actual = MarkedIdeal::new((*fresh.annihilator).clone(), 1, budget)?;
        let candidate = MarkedIdeal::new(candidate.clone(), 1, budget)?;
        normalizer.verify_candidate(&actual, &candidate, budget)
    }
}
/// Native variable-map construction only: all original coordinates and parameter
/// roles survive. New axes precede them in Lex for the elimination theorem.
pub(super) fn auxiliary_ring(
    source: &Arc<Ring>,
    count: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<Arc<Ring>> {
    let size = source
        .len()
        .checked_add(count)
        .ok_or(Error::ResourceIncomplete("component variable count"))?;
    budget.reserve_slots(size)?;
    let old = source.one();
    let mut symbols = (0..count)
        .map(|i| symbol!(format!("{namespace}::aux_{i}")))
        .collect::<Vec<Symbol>>();
    for variable in old.variables().iter() {
        match variable {
            PolyVariable::Symbol(s) => symbols.push(*s),
            _ => return Err(Error::Invalid("component source variable kind")),
        }
    }
    let parameters = (0..source.len())
        .filter(|i| source.is_parameter(*i))
        .map(|i| i + count)
        .collect();
    Ok(Arc::new(Ring::new(symbols, parameters)?))
}
pub(super) fn remap(p: &Poly, target: &Arc<Ring>, budget: &mut Budget) -> Result<Poly> {
    budget.poly(p)?;
    budget.charge(1)?;
    let out = p
        .rearrange_with_growth(target.one().variables())
        .map_err(|_| Error::Invalid("component native variable remapping"))?;
    target.check(&out)?;
    budget.poly(&out)?;
    Ok(out)
}
pub(super) fn native_basis(
    ring: Arc<Ring>,
    auxiliary_axes: usize,
    equations: Vec<Poly>,
    budget: &mut Budget,
) -> Result<EliminationEvidence> {
    let equations = Ideal::new(ring.clone(), equations, budget)?;
    let native = super::super::native_basis::checked(equations.generators(), budget)?;
    let basis = Ideal::new(ring.clone(), native, budget)?;
    Ok(EliminationEvidence {
        ring,
        auxiliary_axes,
        equations,
        basis,
    })
}
pub(super) fn produce_annihilator(
    owner: Arc<RegularAlgebra>,
    source: Arc<Ideal>,
    namespace: &str,
    budget: &mut Budget,
) -> Result<Arc<VerifiedAnnihilator>> {
    let local = owner.local();
    if source.ring() != local.ring() {
        return Err(Error::Invalid("component source ring"));
    }
    for f in source.generators() {
        local.supports(f)?;
        budget.poly(f)?;
    }
    let count = source.generators().len();
    let extended = auxiliary_ring(local.ring(), count, namespace, budget)?;
    let mut relation = extended.one();
    for (i, f) in source.generators().iter().enumerate() {
        let mapped = remap(f, &extended, budget)?;
        relation = relation - budget.mul(&extended.coordinate(i)?, &mapped)?;
        budget.poly(&relation)?;
    }
    let count_relations = local
        .ideal()
        .generators()
        .len()
        .checked_add(local.unit_relations().len())
        .and_then(|n| n.checked_add(1))
        .ok_or(Error::ResourceIncomplete("component relation count"))?;
    budget.reserve_slots(count_relations)?;
    let mut equations = local
        .ideal()
        .generators()
        .iter()
        .chain(local.unit_relations())
        .map(|f| remap(f, &extended, budget))
        .collect::<Result<Vec<_>>>()?;
    equations.push(relation);
    let evidence = native_basis(extended, count, equations, budget)?;
    let normalizer = QuotientNormalizer::prepare(local.clone(), budget)?;
    budget.reserve_slots(evidence.basis.generators().len())?;
    let mut eliminated = Vec::new();
    for f in evidence.basis.generators() {
        if (0..count).any(|i| f.contains(i)) {
            continue;
        }
        let original = remap(f, local.ring(), budget)?;
        budget.charge(1)?;
        let reduced = original.reduce(normalizer.relation_basis().generators());
        local.supports(&reduced)?;
        budget.poly(&reduced)?;
        eliminated.push(reduced);
    }
    let annihilator = Arc::new(Ideal::new(local.ring().clone(), eliminated, budget)?);
    let products = annihilator
        .generators()
        .len()
        .checked_mul(source.generators().len())
        .ok_or(Error::ResourceIncomplete("annihilator identity count"))?;
    budget.count(products)?;
    for a in annihilator.generators() {
        for f in source.generators() {
            let product = budget.mul(a, f)?;
            if !local.zero(&product, budget)? {
                return Err(Error::Invalid(
                    "native annihilator product in original quotient",
                ));
            }
        }
    }
    Ok(Arc::new(VerifiedAnnihilator {
        owner,
        source,
        annihilator,
        evidence,
    }))
}
