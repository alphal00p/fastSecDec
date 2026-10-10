use super::super::{
    Budget, Error, Ideal, MonomialBlowup, MonomialBlowupChart, MonomialWitness, Poly,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct CarriedMonomialWitness {
    source: Arc<MonomialBlowup>,
    chart_index: usize,
    witness: MonomialWitness,
    unit_factor: Poly,
    source_rank: usize,
    target_rank: usize,
    associations: Vec<usize>,
}
impl CarriedMonomialWitness {
    pub fn source(&self) -> &Arc<MonomialBlowup> {
        &self.source
    }
    pub fn chart_index(&self) -> usize {
        self.chart_index
    }
    pub fn chart(&self) -> &Arc<MonomialBlowupChart> {
        &self.source.charts()[self.chart_index]
    }
    pub fn witness(&self) -> &MonomialWitness {
        &self.witness
    }
    pub fn unit_factor(&self) -> &Poly {
        &self.unit_factor
    }
    pub fn ranks(&self) -> (usize, usize) {
        (self.source_rank, self.target_rank)
    }
    pub fn associations(&self) -> &[usize] {
        &self.associations
    }
}
pub fn exponent_rank(powers: &[usize]) -> Result<usize> {
    powers.iter().try_fold(0usize, |sum, p| {
        sum.checked_add(*p)
            .ok_or(Error::ResourceIncomplete("monomial rank overflow"))
    })
}
pub fn carry_monomial_witness(
    source: Arc<MonomialBlowup>,
    chart_index: usize,
    b: &mut Budget,
) -> Result<CarriedMonomialWitness> {
    let chart = source
        .charts()
        .get(chart_index)
        .ok_or(Error::Invalid("carried chart index"))?;
    let local = chart.frame().local();
    let ring = local.ring();
    let old = source.source();
    let previous = old.witness();
    let source_rank = exponent_rank(&previous.powers)?;
    let mut powers = Vec::new();
    b.reserve_slots(chart.history().ledger().divisors().len())?;
    for divisor in chart.history().ledger().divisors() {
        let power = match old
            .ledger()
            .divisors()
            .iter()
            .position(|d| d.id == divisor.id)
        {
            Some(i) => previous.powers[i],
            None if divisor.id == source.born().0 && !chart.exceptional_is_empty() => {
                old.exceptional_power()
            }
            _ => return Err(Error::Invalid("carried unissued divisor")),
        };
        powers.push(power);
    }
    let target_rank = exponent_rank(&powers)?;
    if target_rank >= source_rank {
        return Err(Error::Invalid("monomial rank did not strictly decrease"));
    }
    let mut unit_factor = ring.one();
    for (i, divisor) in old.ledger().divisors().iter().enumerate() {
        let receipt = chart
            .divisors()
            .iter()
            .find(|r| r.id == divisor.id)
            .ok_or(Error::Invalid("carried missing divisor receipt"))?;
        let mut u = receipt.unit.clone();
        if receipt.absent {
            u = b.mul(&u, &receipt.strict_equation)?;
        }
        let u = b.power(&u, previous.powers[i])?;
        unit_factor = b.mul(&unit_factor, &u)?;
    }
    if chart.exceptional_is_empty()
        && let Some(e) = chart.exceptional()
    {
        let e = b.power(e, old.exceptional_power())?;
        unit_factor = b.mul(&unit_factor, &e)?;
    }
    if !local
        .ideal()
        .sum(&Ideal::new(ring.clone(), vec![unit_factor.clone()], b)?, b)?
        .contains(&ring.one(), local.unit_relations(), b)?
    {
        return Err(Error::Invalid("carried residual multiplier not unit"));
    }
    let mut monomial = ring.one();
    for (d, p) in chart.history().ledger().divisors().iter().zip(&powers) {
        let factor = b.power(&d.equation, *p)?;
        monomial = b.mul(&monomial, &factor)?;
    }
    b.reserve_slots(previous.quotients.len())?;
    let mut candidates = Vec::new();
    for q in &previous.quotients {
        let pulled = chart.pull_from_source(q, b)?;
        candidates.push(b.mul(&unit_factor, &pulled)?);
    }
    // Ideal::new may remove zero/repeated literal generators. Associate each
    // actual retained target generator by a native exact recombination, not by
    // old vector position. The final residual-unit proof checks the kept list.
    let mut quotients = Vec::new();
    let mut associations = Vec::new();
    b.reserve_slots(chart.target().ideal().generators().len())?;
    for f in chart.target().ideal().generators() {
        let mut found = None;
        for (i, q) in candidates.iter().enumerate() {
            let product = b.mul(&monomial, q)?;
            if local.zero(&(f - &product), b)? {
                found = Some((i, q.clone()));
                break;
            }
        }
        let (i, q) = found.ok_or(Error::Invalid("carried target generator has no quotient"))?;
        associations.push(i);
        quotients.push(q);
    }
    let residual = Ideal::new(ring.clone(), quotients.clone(), b)?;
    if !local
        .ideal()
        .sum(&residual, b)?
        .contains(&ring.one(), local.unit_relations(), b)?
    {
        return Err(Error::Invalid("carried residual ideal is not unit"));
    }
    let witness = MonomialWitness {
        source: chart.target().clone(),
        powers,
        quotients,
    };
    Ok(CarriedMonomialWitness {
        source,
        chart_index,
        witness,
        unit_factor,
        source_rank,
        target_rank,
        associations,
    })
}
