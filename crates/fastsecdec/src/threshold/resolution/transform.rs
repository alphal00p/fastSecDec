use super::algebra::Result;
use super::{Budget, Error, Ideal, Map, MarkedIdeal, Poly, VerifiedCenter};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct DivisorTransform {
    pub source_id: u64,
    pub birth_stage: usize,
    pub source_multiplicity: usize,
    pub exceptional_multiplicity: usize,
    pub strict_equation: Poly,
    pub pulled_unit: Poly,
}
pub struct ControlledTransformCertificate {
    pub center: VerifiedCenter,
    /// A map from the adapted chart to one standard coordinate blowup chart.
    pub blowup: Map,
    /// Position in the ordered active coordinates; must be a centre axis.
    pub pivot_position: usize,
    pub born_divisor_id: u64,
    pub born_stage: usize,
    pub target: MarkedIdeal,
    pub divisor_transforms: Vec<DivisorTransform>,
}
#[derive(Clone, Debug)]
pub struct VerifiedTransform {
    source: MarkedIdeal,
    adapted_source: MarkedIdeal,
    target: MarkedIdeal,
    center: VerifiedCenter,
    blowup: Map,
    jacobian: Poly,
    exceptional_factor: Poly,
    born_divisor_id: u64,
    born_stage: usize,
    generator_quotients: Vec<Poly>,
    divisor_transforms: Vec<DivisorTransform>,
}
impl ControlledTransformCertificate {
    pub fn verify(self, budget: &mut Budget) -> Result<VerifiedTransform> {
        let old = self.center.frame().map().target();
        let new = self.blowup.target();
        if self.blowup.source() != old
            || self.pivot_position >= old.axes().len()
            || self.target.ideal().ring() != old.ring()
            || self.target.mark() != self.center.adapted().mark()
            || !self
                .center
                .normal_axes()
                .contains(&old.axes()[self.pivot_position])
        {
            return Err(Error::Invalid("controlled transform chart, pivot or mark"));
        }
        let exceptional = new.ring().coordinate(new.axes()[self.pivot_position])?;
        for (position, axis) in old.axes().iter().enumerate() {
            let coordinate = new.ring().coordinate(new.axes()[position])?;
            let expected =
                if position != self.pivot_position && self.center.normal_axes().contains(axis) {
                    budget.mul(&exceptional, &coordinate)?
                } else {
                    coordinate
                };
            if self.blowup.images()[position] != expected {
                return Err(Error::Invalid("not the supplied standard blowup chart"));
            }
        }
        let jacobian = self.blowup.jacobian(budget)?;
        if jacobian != budget.power(&exceptional, self.center.normal_axes().len() - 1)? {
            return Err(Error::Invalid("relative blowup Jacobian"));
        }
        // The real witness lies on the new exceptional divisor. It does not
        // prove all real charts cover the source, nor fix |J| on a whole cell.
        if !new.witness()[new.axes()[self.pivot_position]].is_zero() {
            return Err(Error::Invalid("target witness not exceptional"));
        }
        let factor = budget.power(&exceptional, self.target.mark())?;
        let mut quotients = Vec::new();
        for f in self.center.adapted().ideal().generators() {
            let total = self.blowup.pull(f, budget)?;
            budget.charge(1)?;
            let (quotient, remainder) = total.quot_rem(&factor, false);
            if !remainder.is_zero() || budget.mul(&factor, &quotient)? != total {
                return Err(Error::Invalid("controlled exceptional division"));
            }
            budget.poly(&quotient)?;
            quotients.push(quotient);
        }
        let quotient_ideal = Ideal::new(new.ring().clone(), quotients.clone(), budget)?;
        for f in quotient_ideal.generators() {
            if !new.contains(self.target.ideal(), f, budget)? {
                return Err(Error::Invalid("target ideal missing quotient"));
            }
        }
        for f in self.target.ideal().generators() {
            if !new.contains(&quotient_ideal, f, budget)? {
                return Err(Error::Invalid("target ideal has extra generator"));
            }
        }
        let boundaries = self.center.frame().boundaries();
        if self.divisor_transforms.len() != boundaries.len()
            || boundaries
                .iter()
                .any(|b| b.id == self.born_divisor_id || b.birth_stage > self.born_stage)
        {
            return Err(Error::Invalid("divisor count, birth or ID"));
        }
        let mut seen = BTreeSet::new();
        for transform in &self.divisor_transforms {
            let divisor = boundaries
                .iter()
                .find(|b| b.id == transform.source_id)
                .ok_or(Error::Invalid("unknown inherited divisor"))?;
            if !seen.insert(divisor.id)
                || transform.birth_stage != divisor.birth_stage
                || transform.source_multiplicity != divisor.multiplicity
            {
                return Err(Error::Invalid("inherited divisor identity/multiplicity"));
            }
            let position = old
                .axes()
                .iter()
                .position(|i| *i == divisor.frame_axis)
                .ok_or(Error::Invalid("boundary axis"))?;
            let in_center = self.center.normal_axes().contains(&divisor.frame_axis);
            let expected_exceptional = usize::from(in_center);
            let expected_strict = if position == self.pivot_position {
                new.ring().one()
            } else {
                new.ring().coordinate(new.axes()[position])?
            };
            if transform.exceptional_multiplicity != expected_exceptional
                || transform.strict_equation != expected_strict
                || transform.pulled_unit != self.blowup.pull(&divisor.frame_unit, budget)?
                || !new.unit(&transform.pulled_unit, budget)?
            {
                return Err(Error::Invalid("inherited divisor factor/unit"));
            }
            let original = self
                .center
                .frame()
                .map()
                .pull(&divisor.source_equation, budget)?;
            let total = self.blowup.pull(&original, budget)?;
            let e = budget.power(&exceptional, expected_exceptional)?;
            let unit_exceptional = budget.mul(&transform.pulled_unit, &e)?;
            let recombined = budget.mul(&unit_exceptional, &transform.strict_equation)?;
            if total != recombined {
                return Err(Error::Invalid("divisor recombination"));
            }
        }
        Ok(VerifiedTransform {
            source: self.center.source().clone(),
            adapted_source: self.center.adapted().clone(),
            target: self.target,
            center: self.center,
            blowup: self.blowup,
            jacobian,
            exceptional_factor: factor,
            born_divisor_id: self.born_divisor_id,
            born_stage: self.born_stage,
            generator_quotients: quotients,
            divisor_transforms: self.divisor_transforms,
        })
    }
}
impl VerifiedTransform {
    pub fn source(&self) -> &MarkedIdeal {
        &self.source
    }
    pub fn adapted_source(&self) -> &MarkedIdeal {
        &self.adapted_source
    }
    pub fn target(&self) -> &MarkedIdeal {
        &self.target
    }
    pub fn center(&self) -> &VerifiedCenter {
        &self.center
    }
    pub fn map(&self) -> &Map {
        &self.blowup
    }
    pub fn jacobian(&self) -> &Poly {
        &self.jacobian
    }
    pub fn exceptional_factor(&self) -> &Poly {
        &self.exceptional_factor
    }
    pub fn born_divisor(&self) -> (u64, usize) {
        (self.born_divisor_id, self.born_stage)
    }
    pub fn generator_quotients(&self) -> &[Poly] {
        &self.generator_quotients
    }
    pub fn divisor_transforms(&self) -> &[DivisorTransform] {
        &self.divisor_transforms
    }
}
