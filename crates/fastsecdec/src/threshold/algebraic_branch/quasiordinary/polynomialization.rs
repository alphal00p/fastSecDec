use super::{InputPolynomial, Result};
use crate::threshold::{
    algebraic_branch::MonicPolynomial,
    resolution::{Budget, Error, Ideal, Poly, UnitClearing, clear_units},
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct ClearedPolynomial {
    pub(super) source: Arc<InputPolynomial>,
    pub(super) clearings: Vec<UnitClearing>,
    pub(super) denominator: Poly,
    pub(super) coefficients: Vec<Poly>,
    pub(super) denominator_unit: Ideal,
}
impl ClearedPolynomial {
    pub fn source(&self) -> &Arc<InputPolynomial> {
        &self.source
    }
    pub fn clearings(&self) -> &[UnitClearing] {
        &self.clearings
    }
    pub fn common_denominator(&self) -> &Poly {
        &self.denominator
    }
    pub fn coefficients(&self) -> &[Poly] {
        &self.coefficients
    }
    pub fn leading(&self) -> &Poly {
        self.coefficients.last().expect("private positive degree")
    }
    pub fn denominator_unit_test(&self) -> &Ideal {
        &self.denominator_unit
    }
}
#[derive(Clone, Debug)]
pub struct Monicization {
    pub(super) cleared: Arc<ClearedPolynomial>,
    pub(super) monic: Arc<MonicPolynomial>,
    pub(super) leading_unit: Ideal,
}
impl Monicization {
    pub fn cleared(&self) -> &Arc<ClearedPolynomial> {
        &self.cleared
    }
    pub fn monic(&self) -> &Arc<MonicPolynomial> {
        &self.monic
    }
    /// T=b*z. The inverse z=T/b is valid on the original proved unit domain.
    pub fn root_scale(&self) -> &Poly {
        self.cleared.leading()
    }
    pub fn leading_unit_test(&self) -> &Ideal {
        &self.leading_unit
    }
}
pub(super) fn unit_test(
    source: &InputPolynomial,
    p: &Poly,
    budget: &mut Budget,
) -> Result<(Ideal, bool)> {
    let local = source.frame.local();
    let test = local.ideal().sum(
        &Ideal::new(local.ring().clone(), vec![p.clone()], budget)?,
        budget,
    )?;
    let yes = test.contains(&local.ring().one(), local.unit_relations(), budget)?;
    Ok((test, yes))
}
// Bookkeeping before existing native rational conversion, not a second CAS.
// Bound a common guard denominator and every resulting numerator monomial.
fn preflight(source: &InputPolynomial, p: &Poly, budget: &mut Budget) -> Result<()> {
    let local = source.frame.local();
    budget.reserve_slots(p.nvars())?;
    let mut terms = p.nterms().max(1);
    let mut degrees = (0..p.nvars())
        .map(|i| usize::from(p.degree(i)))
        .collect::<Vec<_>>();
    for g in local.guards() {
        let exponent = usize::from(p.degree(g.inverse_axis));
        let exponent32 =
            u32::try_from(exponent).map_err(|_| Error::ResourceIncomplete("guard exponent"))?;
        terms = terms
            .checked_mul(
                g.factor
                    .nterms()
                    .max(1)
                    .checked_pow(exponent32)
                    .ok_or(Error::ResourceIncomplete("guard clearing term bound"))?,
            )
            .ok_or(Error::ResourceIncomplete("guard clearing term product"))?;
        if terms > budget.limits.max_terms {
            return Err(Error::ResourceIncomplete("guard clearing terms"));
        }
        for (i, d) in degrees.iter_mut().enumerate() {
            *d = d
                .checked_add(
                    exponent
                        .checked_mul(usize::from(g.factor.degree(i)))
                        .ok_or(Error::ResourceIncomplete("guard clearing degree product"))?,
                )
                .ok_or(Error::ResourceIncomplete("guard clearing degree sum"))?;
            if *d > usize::from(budget.limits.max_degree_per_axis).min(usize::from(u16::MAX) / 2) {
                return Err(Error::ResourceIncomplete("guard clearing degree"));
            }
        }
    }
    Ok(())
}
pub(super) fn clear(
    source: &Arc<InputPolynomial>,
    progress: &mut super::Progress,
    budget: &mut Budget,
) -> Result<Arc<ClearedPolynomial>> {
    let local = source.frame.local();
    budget.reserve_slots(source.coefficients.len())?;
    for p in &source.coefficients {
        preflight(source, p, budget)?;
        progress.clearings.push(clear_units(local, p, budget)?);
    }
    let mut denominator = local.ring().one();
    for c in &progress.clearings {
        denominator = budget.mul(&denominator, &c.denominator)?;
    }
    let (denominator_unit, valid) = unit_test(source, &denominator, budget)?;
    if !valid {
        return Err(Error::Invalid("common clearing denominator not a unit"));
    }
    budget.reserve_slots(source.coefficients.len())?;
    let mut coefficients = Vec::with_capacity(source.coefficients.len());
    for c in &progress.clearings {
        budget.charge(1)?;
        let other = denominator
            .try_div(&c.denominator)
            .ok_or(Error::Invalid("native exact common denominator division"))?;
        budget.poly(&other)?;
        let v = budget.mul(&c.numerator, &other)?;
        if !local.zero(&(&budget.mul(&c.original, &denominator)? - &v), budget)? {
            return Err(Error::Invalid("cleared coefficient identity"));
        }
        coefficients.push(v);
    }
    Ok(Arc::new(ClearedPolynomial {
        source: source.clone(),
        clearings: std::mem::take(&mut progress.clearings),
        denominator,
        coefficients,
        denominator_unit,
    }))
}
pub(super) fn monic(
    cleared: Arc<ClearedPolynomial>,
    leading_unit: Ideal,
    budget: &mut Budget,
) -> Result<Arc<Monicization>> {
    let source = &cleared.source;
    let n = source.degree();
    let b = cleared.leading();
    budget.reserve_slots(n + 1)?;
    let mut coefficients = Vec::with_capacity(n + 1);
    for (i, p) in cleared.coefficients[..n].iter().enumerate() {
        let power = budget.power(b, n - 1 - i)?;
        coefficients.push(budget.mul(p, &power)?);
    }
    coefficients.push(source.frame.local().ring().one());
    let scale = budget.power(b, n - 1)?;
    for (i, q) in coefficients.iter().enumerate() {
        let power = budget.power(b, i)?;
        let left = budget.mul(q, &power)?;
        let rhs = budget.mul(&cleared.coefficients[i], &scale)?;
        if !source.frame.local().zero(&(&left - &rhs), budget)? {
            return Err(Error::Invalid("monicization coefficient identity"));
        }
    }
    let monic = MonicPolynomial::new(source.frame.clone(), source.variable, coefficients, budget)?;
    Ok(Arc::new(Monicization {
        cleared,
        monic,
        leading_unit,
    }))
}
