use super::{Budget, Error, Guard, Ideal, Poly, Ring};
use std::{collections::BTreeSet, sync::Arc};
use symbolica::domains::rational::Rational;
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Supplied polynomial quotient on a principal open. Not a smoothness or real
/// nonemptiness certificate. Inverse variables never occur in source/guards.
#[derive(Clone, Debug)]
pub struct LocalizedAlgebra {
    ideal: Ideal,
    axes: Vec<usize>,
    guards: Vec<Guard>,
    unit_relations: Vec<Poly>,
}
impl LocalizedAlgebra {
    pub fn new(
        ideal: Ideal,
        axes: Vec<usize>,
        guards: Vec<Guard>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        let ring = ideal.ring();
        let unique: BTreeSet<_> = axes.iter().copied().collect();
        if unique.len() != axes.len()
            || axes
                .iter()
                .any(|i| *i >= ring.len() || ring.is_parameter(*i))
        {
            return Err(Error::Invalid("local algebra coordinate roles"));
        }
        budget.count(axes.len())?;
        budget.reserve_slots(guards.len())?;
        for f in ideal.generators() {
            ring.supports(f, &axes)?;
        }
        let mut inverse_axes = BTreeSet::new();
        let mut unit_relations = Vec::new();
        for guard in &guards {
            ring.supports(&guard.factor, &axes)?;
            budget.poly(&guard.factor)?;
            if guard.inverse_axis >= ring.len()
                || ring.is_parameter(guard.inverse_axis)
                || unique.contains(&guard.inverse_axis)
                || !inverse_axes.insert(guard.inverse_axis)
            {
                return Err(Error::Invalid("local algebra inverse roles"));
            }
            unit_relations.push(
                &budget.mul(&ring.coordinate(guard.inverse_axis)?, &guard.factor)? - &ring.one(),
            );
        }
        if ideal.contains(&ring.one(), &unit_relations, budget)? {
            return Err(Error::Invalid("empty algebraic principal open"));
        }
        Ok(Arc::new(Self {
            ideal,
            axes,
            guards,
            unit_relations,
        }))
    }
    pub fn ring(&self) -> &Arc<Ring> {
        self.ideal.ring()
    }
    pub fn ideal(&self) -> &Ideal {
        &self.ideal
    }
    pub fn axes(&self) -> &[usize] {
        &self.axes
    }
    pub fn guards(&self) -> &[Guard] {
        &self.guards
    }
    pub fn unit_relations(&self) -> &[Poly] {
        &self.unit_relations
    }
    pub(crate) fn supports(&self, p: &Poly) -> Result<()> {
        let mut allowed = self.axes.clone();
        allowed.extend(self.guards.iter().map(|g| g.inverse_axis));
        self.ring().supports(p, &allowed)
    }
    pub fn zero(&self, p: &Poly, budget: &mut Budget) -> Result<bool> {
        self.supports(p)?;
        self.ideal.contains(p, &self.unit_relations, budget)
    }
    /// One exact real-point check, never an all-fiber or real-cover assertion.
    pub fn check_real_point(&self, point: &[Rational]) -> Result<bool> {
        if point.len() != self.ring().len() {
            return Err(Error::Invalid("real-point shape"));
        }
        Ok(self
            .ideal
            .generators()
            .iter()
            .chain(&self.unit_relations)
            .all(|f| f.replace_all(point).is_zero()))
    }
}
