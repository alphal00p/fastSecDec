use super::super::{Budget, Error, Guard, Ideal, LocalizedAlgebra, Poly, Ring};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{integer::Z, rational::Q},
    poly::PolyVariable,
};
type Result<T> = std::result::Result<T, Error>;

/// Identity on all existing coordinates, including inert parameter axes.
#[derive(Clone, Debug)]
pub struct RingExtension {
    source: Arc<Ring>,
    target: Arc<Ring>,
}
impl RingExtension {
    pub fn new(source: Arc<Ring>, fresh: [Symbol; 2], budget: &mut Budget) -> Result<Self> {
        budget.reserve_slots(
            source
                .len()
                .checked_add(2)
                .ok_or(Error::ResourceIncomplete("extended ring size"))?,
        )?;
        let mut symbols = source
            .one()
            .variables()
            .iter()
            .map(|v| match v {
                PolyVariable::Symbol(s) => Ok(*s),
                _ => Err(Error::Invalid("ring coordinate is not a symbol")),
            })
            .collect::<Result<Vec<_>>>()?;
        symbols.extend(fresh);
        let params = (0..source.len())
            .filter(|i| source.is_parameter(*i))
            .collect();
        let target = Arc::new(Ring::new(symbols, params)?);
        Ok(Self { source, target })
    }
    pub fn source(&self) -> &Arc<Ring> {
        &self.source
    }
    pub fn target(&self) -> &Arc<Ring> {
        &self.target
    }
    pub fn pull(&self, p: &Poly, budget: &mut Budget) -> Result<Poly> {
        self.source.check(p)?;
        budget.poly(p)?;
        budget.charge(1)?;
        let mut out = p.clone();
        self.target.one().unify_variables(&mut out);
        self.target.check(&out)?;
        budget.poly(&out)?;
        Ok(out)
    }
    pub fn ideal(&self, ideal: &Ideal, budget: &mut Budget) -> Result<Ideal> {
        if ideal.ring() != &self.source {
            return Err(Error::Invalid("embedding source ring"));
        }
        budget.reserve_slots(ideal.generators().len())?;
        Ideal::new(
            self.target.clone(),
            ideal
                .generators()
                .iter()
                .map(|p| self.pull(p, budget))
                .collect::<Result<_>>()?,
            budget,
        )
    }
    pub fn guards(&self, guards: &[Guard], budget: &mut Budget) -> Result<Vec<Guard>> {
        budget.reserve_slots(guards.len())?;
        guards
            .iter()
            .map(|g| {
                Ok(Guard {
                    factor: self.pull(&g.factor, budget)?,
                    inverse_axis: g.inverse_axis,
                })
            })
            .collect()
    }
}

/// Native rational normalization plus exact unit and recombination checks.
#[derive(Clone, Debug)]
pub struct UnitClearing {
    pub original: Poly,
    pub numerator: Poly,
    pub denominator: Poly,
}
pub fn clear_units(
    local: &LocalizedAlgebra,
    p: &Poly,
    budget: &mut Budget,
) -> Result<UnitClearing> {
    local.supports(p)?;
    budget.poly(p)?;
    // All guard factors are independent of inverse slots. Substitution order
    // consequently cannot hide nested inverse dependencies.
    let mut expression = p.to_expression();
    for guard in local.guards() {
        budget.charge(1)?;
        expression = expression
            .replace(local.ring().coordinate(guard.inverse_axis)?.to_expression())
            .with(Atom::one() / guard.factor.to_expression());
    }
    budget.charge(1)?;
    let fraction = expression
        .try_to_rational_polynomial::<_, _, u16>(
            &Q,
            &Z,
            Some(local.ring().one().variables().clone()),
        )
        .map_err(|_| Error::Invalid("native rational unit clearing"))?;
    let numerator = local.ring().atom(&fraction.numerator.to_expression())?;
    let denominator = local.ring().atom(&fraction.denominator.to_expression())?;
    budget.poly(&numerator)?;
    budget.poly(&denominator)?;
    local.ring().supports(&numerator, local.axes())?;
    local.ring().supports(&denominator, local.axes())?;
    let recombined = &budget.mul(p, &denominator)? - &numerator;
    if !local.zero(&recombined, budget)? {
        return Err(Error::Invalid("unit clearing recombination"));
    }
    let unit = local.ideal().sum(
        &Ideal::new(local.ring().clone(), vec![denominator.clone()], budget)?,
        budget,
    )?;
    if !unit.contains(&local.ring().one(), local.unit_relations(), budget)? {
        return Err(Error::Invalid(
            "clearing denominator is not a certified unit",
        ));
    }
    Ok(UnitClearing {
        original: p.clone(),
        numerator,
        denominator,
    })
}
