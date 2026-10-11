//! Native F4 with explicit Buchberger closure and original-generator inclusion.
//! Reverse inclusion relies on F4's constructive combinations. This helper does
//! not validate an arbitrary caller-supplied basis or expose alternative CAS.
use super::{Budget, Error, Poly};
use symbolica::{
    domains::rational::RationalField,
    poly::{MonomialOrder, groebner::GroebnerBasis, polynomial::MultivariatePolynomial},
};
type OrderedPoly<O> = MultivariatePolynomial<RationalField, u16, O>;
type Result<T> = std::result::Result<T, Error>;

pub(super) fn checked(equations: &[Poly], budget: &mut Budget) -> Result<Vec<Poly>> {
    checked_ordered(equations, budget)
}
pub(super) fn checked_ordered<O: MonomialOrder>(
    equations: &[OrderedPoly<O>],
    budget: &mut Budget,
) -> Result<Vec<OrderedPoly<O>>> {
    budget.reserve_slots(equations.len())?;
    for f in equations {
        budget.poly(f)?;
        if equations
            .first()
            .is_some_and(|p| p.variables() != f.variables())
        {
            return Err(Error::Invalid("native basis input variable maps"));
        }
    }
    // The adopted native F4 does not accept literal zero generators. The zero
    // ideal is represented by an empty input, which its public API supports.
    let input = equations
        .iter()
        .filter(|f| !f.is_zero())
        .cloned()
        .collect::<Vec<_>>();
    budget.charge(1)?;
    let result = GroebnerBasis::new(&input, false).system;
    budget.reserve_slots(result.len())?;
    for f in &result {
        budget.poly(f)?;
        if f.is_zero()
            || input
                .first()
                .is_some_and(|p| p.variables() != f.variables())
        {
            return Err(Error::Invalid("native basis output shape"));
        }
    }
    verify(&input, &result, budget)?;
    Ok(result)
}
fn verify<O: MonomialOrder>(
    input: &[OrderedPoly<O>],
    basis: &[OrderedPoly<O>],
    budget: &mut Budget,
) -> Result<()> {
    let n = basis.len();
    let pairs = n
        .checked_mul(n.saturating_sub(1))
        .map(|v| v / 2)
        .ok_or(Error::ResourceIncomplete("native basis S-pair count"))?;
    budget.charge(pairs)?;
    // Bounds only the explicit S-polynomial construction. Native F4/reduction
    // internals remain subject to caller-owned hard process/RSS limits.
    for (i, p) in basis.iter().enumerate() {
        for q in &basis[i + 1..] {
            if p.nterms()
                .checked_add(q.nterms())
                .is_none_or(|v| v > budget.limits.max_terms)
            {
                return Err(Error::ResourceIncomplete("native S-polynomial terms"));
            }
            for axis in 0..p.nvars() {
                let lcm = p.max_exp()[axis].max(q.max_exp()[axis]);
                for f in [p, q] {
                    if f.degree(axis)
                        .checked_add(lcm - f.max_exp()[axis])
                        .is_none_or(|v| v > budget.limits.max_degree_per_axis)
                    {
                        return Err(Error::ResourceIncomplete("native S-polynomial degree"));
                    }
                }
            }
        }
    }
    if !GroebnerBasis::is_groebner_basis(basis) {
        return Err(Error::Invalid("native basis fails Buchberger closure"));
    }
    for f in input {
        budget.charge(1)?;
        let remainder = f.reduce(basis);
        budget.poly(&remainder)?;
        if !remainder.is_zero() {
            return Err(Error::Invalid("native basis loses an input equation"));
        }
    }
    Ok(())
}
#[cfg(test)]
#[path = "native_basis/tests.rs"]
mod tests;
