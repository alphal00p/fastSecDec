use super::algebra::Result;
use super::{Budget, Error, Ideal, Poly, Ring};
use std::{collections::BTreeSet, sync::Arc};
use symbolica::{
    domains::rational::{Q, Rational},
    poly::polynomial::PolynomialRing,
    tensors::matrix::Matrix,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Guard {
    pub factor: Poly,
    pub inverse_axis: usize,
}

/// Principal-open affine chart. The witness proves only this chart is nonempty.
/// No inequalities, global smoothness theorem, fibre coverage or orientation
/// are inferred from the witness. Active axes are smooth affine coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chart {
    ring: Arc<Ring>,
    axes: Vec<usize>,
    guards: Vec<Guard>,
    relations: Vec<Poly>,
    witness: Vec<Rational>,
}
impl Chart {
    pub fn new(
        ring: Arc<Ring>,
        axes: Vec<usize>,
        guards: Vec<Guard>,
        witness: Vec<Rational>,
        budget: &mut Budget,
    ) -> Result<Self> {
        let distinct: BTreeSet<_> = axes.iter().copied().collect();
        if axes.is_empty()
            || distinct.len() != axes.len()
            || axes
                .iter()
                .any(|i| *i >= ring.len() || ring.is_parameter(*i))
            || witness.len() != ring.len()
        {
            return Err(Error::Invalid("chart coordinates or witness length"));
        }
        budget.count(guards.len())?;
        let mut inverse_axes = BTreeSet::new();
        let mut relations = Vec::new();
        for g in &guards {
            ring.supports(&g.factor, &axes)?;
            budget.poly(&g.factor)?;
            if g.inverse_axis >= ring.len()
                || ring.is_parameter(g.inverse_axis)
                || distinct.contains(&g.inverse_axis)
                || !inverse_axes.insert(g.inverse_axis)
                || g.factor.replace_all(&witness).is_zero()
            {
                return Err(Error::Invalid(
                    "guard inverse axis or nonempty real witness",
                ));
            }
            let relation = &budget.mul(&ring.coordinate(g.inverse_axis)?, &g.factor)? - &ring.one();
            if !relation.replace_all(&witness).is_zero() {
                return Err(Error::Invalid("witness does not satisfy inverse guard"));
            }
            relations.push(relation);
        }
        Ok(Self {
            ring,
            axes,
            guards,
            relations,
            witness,
        })
    }
    pub fn ring(&self) -> &Arc<Ring> {
        &self.ring
    }
    pub fn axes(&self) -> &[usize] {
        &self.axes
    }
    pub fn guards(&self) -> &[Guard] {
        &self.guards
    }
    pub fn witness(&self) -> &[Rational] {
        &self.witness
    }
    pub(crate) fn supports(&self, p: &Poly) -> Result<()> {
        self.ring.supports(p, &self.axes)
    }
    pub fn contains(&self, ideal: &Ideal, p: &Poly, budget: &mut Budget) -> Result<bool> {
        if ideal.ring() != &self.ring {
            return Err(Error::Invalid("localized ideal chart"));
        }
        self.supports(p)?;
        for q in ideal.generators() {
            self.supports(q)?;
        }
        ideal.contains(p, &self.relations, budget)
    }
    pub fn unit(&self, p: &Poly, budget: &mut Budget) -> Result<bool> {
        self.supports(p)?;
        self.contains(
            &Ideal::new(self.ring.clone(), vec![p.clone()], budget)?,
            &self.ring.one(),
            budget,
        )
    }
}

/// Pullback of source active coordinates to target polynomials. Parameters
/// have no map entries and are held fixed. Disjoint axis names prevent
/// sequential native substitution from accidentally becoming recursive.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Map {
    source: Arc<Chart>,
    target: Arc<Chart>,
    images: Vec<Poly>,
}
impl Map {
    pub fn new(
        source: Arc<Chart>,
        target: Arc<Chart>,
        images: Vec<Poly>,
        budget: &mut Budget,
    ) -> Result<Self> {
        if source.ring != target.ring
            || source.axes.len() != target.axes.len()
            || images.len() != source.axes.len()
            || source.axes.iter().any(|i| target.axes.contains(i))
        {
            return Err(Error::Invalid(
                "map dimensions/ring or disjoint coordinate names",
            ));
        }
        for p in &images {
            target.supports(p)?;
            budget.poly(p)?;
        }
        let map = Self {
            source,
            target,
            images,
        };
        for g in &map.source.guards {
            if !map.target.unit(&map.pull(&g.factor, budget)?, budget)? {
                return Err(Error::Invalid("source guard not a target unit"));
            }
        }
        Ok(map)
    }
    pub fn source(&self) -> &Arc<Chart> {
        &self.source
    }
    pub fn target(&self) -> &Arc<Chart> {
        &self.target
    }
    pub fn images(&self) -> &[Poly] {
        &self.images
    }
    pub fn pull(&self, p: &Poly, budget: &mut Budget) -> Result<Poly> {
        self.source.supports(p)?;
        let mut out = p.clone();
        for (axis, image) in self.source.axes.iter().zip(&self.images) {
            out = budget.substitute(&out, *axis, image)?;
        }
        self.target.supports(&out)?;
        Ok(out)
    }
    pub fn jacobian(&self, budget: &mut Budget) -> Result<Poly> {
        let n = self.images.len();
        let count = n
            .checked_mul(n)
            .ok_or(Error::ResourceIncomplete("Jacobian matrix size"))?;
        budget.count(count)?;
        let max_terms = self.images.iter().map(Poly::nterms).max().unwrap_or(1);
        let mut bound = 1usize;
        for k in 1..=n {
            bound = bound
                .checked_mul(k)
                .and_then(|b| b.checked_mul(max_terms))
                .ok_or(Error::ResourceIncomplete("determinant term bound"))?;
            if bound > budget.limits.max_terms {
                return Err(Error::ResourceIncomplete("determinant term bound"));
            }
        }
        for axis in 0..self.target.ring.len() {
            let degree = self
                .images
                .iter()
                .try_fold(0usize, |sum, p| {
                    sum.checked_add(usize::from(p.degree(axis)))
                })
                .ok_or(Error::ResourceIncomplete("determinant degree bound"))?;
            if degree > usize::from(budget.limits.max_degree_per_axis) {
                return Err(Error::ResourceIncomplete("determinant degree bound"));
            }
        }
        budget.charge(count)?;
        let entries = self
            .images
            .iter()
            .flat_map(|p| self.target.axes.iter().map(move |axis| p.derivative(*axis)))
            .collect();
        let n = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("matrix dimension"))?;
        let det = Matrix::from_linear(entries, n, n, PolynomialRing::<_, u16>::new(Q))
            .map_err(|_| Error::Invalid("native Jacobian matrix"))?
            .det()
            .map_err(|_| Error::Invalid("native Jacobian determinant"))?;
        let det = self.target.ring.atom(&det.to_expression())?;
        budget.poly(&det)?;
        Ok(det)
    }
}

#[derive(Clone, Debug)]
pub struct Boundary {
    pub id: u64,
    pub birth_stage: usize,
    pub multiplicity: usize,
    pub source_equation: Poly,
    pub frame_axis: usize,
    pub frame_unit: Poly,
}
pub struct FrameCertificate {
    pub pullback: Map,
    pub inverse: Map,
    pub boundaries: Vec<Boundary>,
}
#[derive(Clone, Debug)]
pub struct VerifiedFrame {
    pub(crate) pullback: Map,
    pub(crate) inverse: Map,
    pub(crate) boundaries: Vec<Boundary>,
    pub(crate) jacobian: Poly,
}
impl FrameCertificate {
    pub fn verify(self, budget: &mut Budget) -> Result<VerifiedFrame> {
        if self.pullback.source != self.inverse.target
            || self.pullback.target != self.inverse.source
        {
            return Err(Error::Invalid("relative inverse frame chart pairing"));
        }
        for (map, opposite) in [
            (&self.pullback, &self.inverse),
            (&self.inverse, &self.pullback),
        ] {
            for (axis, image) in map.source.axes.iter().zip(&map.images) {
                if opposite.pull(image, budget)? != map.source.ring.coordinate(*axis)? {
                    return Err(Error::Invalid("frame compositions are not identity"));
                }
            }
        }
        let jacobian = self.pullback.jacobian(budget)?;
        if !self.pullback.target.unit(&jacobian, budget)? {
            return Err(Error::Invalid("frame Jacobian not a unit"));
        }
        let mut axes = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for divisor in &self.boundaries {
            if divisor.multiplicity == 0
                || !ids.insert(divisor.id)
                || !axes.insert(divisor.frame_axis)
                || !self.pullback.target.axes.contains(&divisor.frame_axis)
            {
                return Err(Error::Invalid(
                    "boundary IDs, multiplicity or distinct adapted axes",
                ));
            }
            let chart = &self.pullback.target;
            let coordinate = chart.ring.coordinate(divisor.frame_axis)?;
            if !chart.unit(&divisor.frame_unit, budget)?
                || self.pullback.pull(&divisor.source_equation, budget)?
                    != budget.mul(&divisor.frame_unit, &coordinate)?
            {
                return Err(Error::Invalid(
                    "boundary is not adapted coordinate times unit",
                ));
            }
        }
        Ok(VerifiedFrame {
            pullback: self.pullback,
            inverse: self.inverse,
            boundaries: self.boundaries,
            jacobian,
        })
    }
}
impl VerifiedFrame {
    pub fn map(&self) -> &Map {
        &self.pullback
    }
    pub fn inverse(&self) -> &Map {
        &self.inverse
    }
    pub fn boundaries(&self) -> &[Boundary] {
        &self.boundaries
    }
    pub fn jacobian(&self) -> &Poly {
        &self.jacobian
    }
}
