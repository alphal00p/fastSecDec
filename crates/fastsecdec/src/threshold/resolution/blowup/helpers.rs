pub(super) use super::super::matrix::{combinations, determinant};
use super::super::*;
use std::sync::Arc;
use symbolica::symbol;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct Extension {
    source: Arc<Ring>,
    target: Arc<Ring>,
    steps: Vec<RingExtension>,
}
impl Extension {
    pub fn new(
        source: Arc<Ring>,
        count: usize,
        namespace: &str,
        budget: &mut Budget,
    ) -> Result<Self> {
        let slots = count
            .checked_add(count % 2)
            .ok_or(Error::ResourceIncomplete("ring slots"))?;
        budget.reserve_slots(
            source
                .len()
                .checked_add(slots)
                .ok_or(Error::ResourceIncomplete("ring slots"))?,
        )?;
        let mut target = source.clone();
        let mut steps = Vec::new();
        for index in (0..slots).step_by(2) {
            let step = RingExtension::new(
                target,
                [
                    symbol!(format!("{namespace}::v{index}")),
                    symbol!(format!("{namespace}::v{}", index + 1)),
                ],
                budget,
            )?;
            target = step.target().clone();
            steps.push(step);
        }
        Ok(Self {
            source,
            target,
            steps,
        })
    }
    pub fn source(&self) -> &Arc<Ring> {
        &self.source
    }
    pub fn target(&self) -> &Arc<Ring> {
        &self.target
    }
    pub fn pull(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        self.source.check(p)?;
        let mut p = p.clone();
        for step in &self.steps {
            p = step.pull(&p, b)?;
        }
        Ok(p)
    }
    pub fn ideal(&self, i: &Ideal, b: &mut Budget) -> Result<Ideal> {
        if i.ring() != &self.source {
            return Err(Error::Invalid("embedding ring"));
        }
        b.reserve_slots(i.generators().len())?;
        Ideal::new(
            self.target.clone(),
            i.generators()
                .iter()
                .map(|p| self.pull(p, b))
                .collect::<Result<_>>()?,
            b,
        )
    }
    pub fn guards(&self, g: &[Guard], b: &mut Budget) -> Result<Vec<Guard>> {
        b.reserve_slots(g.len())?;
        g.iter()
            .map(|g| {
                Ok(Guard {
                    factor: self.pull(&g.factor, b)?,
                    inverse_axis: g.inverse_axis,
                })
            })
            .collect()
    }
}
pub fn indices(i: &Ideal, selected: &[Poly]) -> Result<Vec<usize>> {
    selected
        .iter()
        .map(|p| {
            i.generators()
                .iter()
                .position(|q| q == p)
                .ok_or(Error::Invalid("selected equation disappeared"))
        })
        .collect()
}
pub fn empty(i: &Ideal, guards: &[Guard], b: &mut Budget) -> Result<bool> {
    let mut relations = Vec::new();
    for g in guards {
        relations.push(&b.mul(&i.ring().coordinate(g.inverse_axis)?, &g.factor)? - &i.ring().one());
    }
    i.contains(&i.ring().one(), &relations, b)
}
pub fn unit(local: &LocalizedAlgebra, p: &Poly, b: &mut Budget) -> Result<bool> {
    local
        .ideal()
        .sum(&Ideal::new(local.ring().clone(), vec![p.clone()], b)?, b)?
        .contains(&local.ring().one(), local.unit_relations(), b)
}
