//! Shared native elimination records and variable maps.
use super::{Budget, Error, Ideal, Poly, Ring};
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
/// Native variable-map construction only: all original coordinates and parameter
/// roles survive. New axes precede them in Lex for the elimination theorem.
pub(crate) fn auxiliary_ring(
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
pub(crate) fn remap(p: &Poly, target: &Arc<Ring>, budget: &mut Budget) -> Result<Poly> {
    budget.poly(p)?;
    budget.charge(1)?;
    let out = p
        .rearrange_with_growth(target.one().variables())
        .map_err(|_| Error::Invalid("component native variable remapping"))?;
    target.check(&out)?;
    budget.poly(&out)?;
    Ok(out)
}
pub(crate) fn native_basis(
    ring: Arc<Ring>,
    auxiliary_axes: usize,
    equations: Vec<Poly>,
    budget: &mut Budget,
) -> Result<EliminationEvidence> {
    let equations = Ideal::new(ring.clone(), equations, budget)?;
    let native = super::native_basis::checked(equations.generators(), budget)?;
    let basis = Ideal::new(ring.clone(), native, budget)?;
    Ok(EliminationEvidence {
        ring,
        auxiliary_axes,
        equations,
        basis,
    })
}
