use super::algebra::Result;
use super::{Budget, Error, Ideal, Poly, VerifiedFrame};
use std::collections::BTreeSet;
use symbolica::domains::{integer::gcd_unsigned, rational::Rational};

#[derive(Clone, Debug)]
pub struct MarkedIdeal {
    ideal: Ideal,
    mark: usize,
}
impl MarkedIdeal {
    pub fn new(ideal: Ideal, mark: usize, budget: &Budget) -> Result<Self> {
        if mark == 0 {
            return Err(Error::Invalid("zero marked-ideal weight"));
        }
        if mark > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("marked-ideal weight"));
        }
        Ok(Self { ideal, mark })
    }
    pub fn ideal(&self) -> &Ideal {
        &self.ideal
    }
    pub fn mark(&self) -> usize {
        self.mark
    }
    /// Weighted sum, retaining every mixed generator in each ideal power.
    pub fn sum(&self, other: &Self, budget: &mut Budget) -> Result<Self> {
        let a =
            u64::try_from(self.mark).map_err(|_| Error::ResourceIncomplete("mark conversion"))?;
        let b =
            u64::try_from(other.mark).map_err(|_| Error::ResourceIncomplete("mark conversion"))?;
        let gcd = usize::try_from(gcd_unsigned(a, b))
            .map_err(|_| Error::ResourceIncomplete("mark gcd conversion"))?;
        let common = (self.mark / gcd)
            .checked_mul(other.mark)
            .ok_or(Error::ResourceIncomplete("mark LCM overflow"))?;
        if common > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("mark LCM budget"));
        }
        let left = self.ideal.power(common / self.mark, budget)?;
        let right = other.ideal.power(common / other.mark, budget)?;
        Self::new(left.sum(&right, budget)?, common, budget)
    }
}

pub struct CenterCertificate {
    pub frame: VerifiedFrame,
    pub source: MarkedIdeal,
    /// Native axis indices in the adapted frame, not source-coordinate indices.
    pub normal_axes: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct VerifiedCenter {
    pub(crate) frame: VerifiedFrame,
    pub(crate) source: MarkedIdeal,
    pub(crate) adapted: MarkedIdeal,
    pub(crate) normal_axes: Vec<usize>,
}

/// A coefficient ideal lives on H and on the restricted principal opens.
/// Retain those guards explicitly; the ambient chart's h(x,z)!=0 cannot be
/// silently substituted for h(x,0)!=0 after restricting to H={z=0}.
#[derive(Clone, Debug)]
pub struct CoefficientRestriction {
    pub marked_ideal: MarkedIdeal,
    pub contact_axis: usize,
    pub remaining_axes: Vec<usize>,
    pub restricted_guard_factors: Vec<Poly>,
}

#[derive(Clone, Debug)]
pub struct CompanionArithmetic {
    pub monomial_factor: Poly,
    pub residual: Ideal,
    pub verified_local_residual_order: usize,
    /// None denotes a verified unit residual and the separate monomial stage.
    pub companion: Option<MarkedIdeal>,
    pub boundary_powers: Vec<(u64, usize)>,
}
impl CenterCertificate {
    pub fn verify(self, budget: &mut Budget) -> Result<VerifiedCenter> {
        let map = self.frame.map();
        let chart = map.target();
        if self.source.ideal.ring() != chart.ring()
            || self.normal_axes.is_empty()
            || self.normal_axes.iter().collect::<BTreeSet<_>>().len() != self.normal_axes.len()
            || self.normal_axes.iter().any(|a| !chart.axes().contains(a))
        {
            return Err(Error::Invalid("adapted centre axes or source ring"));
        }
        for axis in &self.normal_axes {
            if !chart.witness()[*axis].is_zero() {
                return Err(Error::Invalid("real witness not on centre"));
            }
        }
        budget.reserve_slots(self.source.ideal.generators().len())?;
        let generators = self
            .source
            .ideal
            .generators()
            .iter()
            .map(|p| map.pull(p, budget))
            .collect::<Result<Vec<_>>>()?;
        let adapted = MarkedIdeal::new(
            Ideal::new(chart.ring().clone(), generators, budget)?,
            self.source.mark,
            budget,
        )?;
        let center = Ideal::new(
            chart.ring().clone(),
            self.normal_axes
                .iter()
                .map(|i| chart.ring().coordinate(*i))
                .collect::<Result<_>>()?,
            budget,
        )?;
        let mut layer = adapted.ideal.clone();
        // Characteristic-zero derivative criterion in the verified relative
        // frame: all normal multi-partials of total order < d vanish on C.
        for order in 0..adapted.mark {
            for f in layer.generators() {
                if !chart.contains(&center, f, budget)? {
                    return Err(Error::Invalid("source order below mark along centre"));
                }
            }
            if order + 1 < adapted.mark {
                layer = derivative_ideal(&layer, &self.normal_axes, &[], budget)?;
            }
        }
        Ok(VerifiedCenter {
            frame: self.frame,
            source: self.source,
            adapted,
            normal_axes: self.normal_axes,
        })
    }
}

/// Only adapted boundary axes may request logarithmic rather than ordinary
/// derivatives. This is native arithmetic, not an inferred BM stage choice.
fn derivative_ideal(
    ideal: &Ideal,
    axes: &[usize],
    log_axes: &[usize],
    budget: &mut Budget,
) -> Result<Ideal> {
    let count = ideal
        .generators()
        .len()
        .checked_mul(
            axes.len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("derivative axes"))?,
        )
        .ok_or(Error::ResourceIncomplete("derivative generator count"))?;
    budget.reserve_slots(count)?;
    let mut out = ideal.generators().to_vec();
    for f in ideal.generators() {
        for axis in axes {
            if ideal.ring().is_parameter(*axis) {
                return Err(Error::Invalid("parameters are inert"));
            }
            budget.charge(1)?;
            let mut derivative = f.derivative(*axis);
            if log_axes.contains(axis) {
                derivative = budget.mul(&ideal.ring().coordinate(*axis)?, &derivative)?;
            }
            out.push(derivative);
        }
    }
    Ideal::new(ideal.ring().clone(), out, budget)
}

impl VerifiedCenter {
    pub fn source(&self) -> &MarkedIdeal {
        &self.source
    }
    pub fn adapted(&self) -> &MarkedIdeal {
        &self.adapted
    }
    pub fn frame(&self) -> &VerifiedFrame {
        &self.frame
    }
    pub fn normal_axes(&self) -> &[usize] {
        &self.normal_axes
    }

    /// Check a supplied coordinate maximal-contact witness on this local patch:
    /// order <= d by D^d I = 1, H in D_E^(d-1)I, and H transverse to E. This
    /// supplies the hypotheses of the local max-order coefficient operation;
    /// it does not identify a global invariant maximum or a next BM centre.
    pub fn coefficient_ideal(
        &self,
        contact_axis: usize,
        budget: &mut Budget,
    ) -> Result<CoefficientRestriction> {
        let chart = self.frame.map().target();
        let log_axes: Vec<_> = self
            .frame
            .boundaries()
            .iter()
            .map(|d| d.frame_axis)
            .collect();
        if !chart.axes().contains(&contact_axis) || log_axes.contains(&contact_axis) {
            return Err(Error::Invalid(
                "contact axis is not transverse to the boundary",
            ));
        }
        let mut ordinary = self.adapted.ideal.clone();
        for _ in 0..self.adapted.mark {
            ordinary = derivative_ideal(&ordinary, chart.axes(), &[], budget)?;
        }
        if !chart.contains(&ordinary, &chart.ring().one(), budget)? {
            return Err(Error::Invalid("no verified local upper order bound"));
        }
        let mut logarithmic = self.adapted.ideal.clone();
        let mut terms = Vec::new();
        for j in 0..self.adapted.mark {
            budget.reserve_slots(logarithmic.generators().len())?;
            let restricted = logarithmic
                .generators()
                .iter()
                .map(|p| p.replace(contact_axis, &Rational::from(0)))
                .collect();
            terms.push(MarkedIdeal::new(
                Ideal::new(chart.ring().clone(), restricted, budget)?,
                self.adapted.mark - j,
                budget,
            )?);
            if j + 1 < self.adapted.mark {
                logarithmic = derivative_ideal(&logarithmic, chart.axes(), &log_axes, budget)?;
            }
        }
        if !chart.contains(
            &logarithmic,
            &chart.ring().coordinate(contact_axis)?,
            budget,
        )? {
            return Err(Error::Invalid(
                "contact coordinate not in logarithmic derivative ideal",
            ));
        }
        // Restriction is meaningful only where H meets this declared chart.
        if !chart.witness()[contact_axis].is_zero() {
            return Err(Error::Invalid("witness not on contact hypersurface"));
        }
        let mut iter = terms.into_iter();
        let mut result = iter.next().ok_or(Error::Invalid("empty coefficient sum"))?;
        for term in iter {
            result = result.sum(&term, budget)?;
        }
        let restricted_guard_factors = chart
            .guards()
            .iter()
            .map(|g| g.factor.replace(contact_axis, &Rational::from(0)))
            .collect::<Vec<_>>();
        if restricted_guard_factors
            .iter()
            .any(|g| g.replace_all(chart.witness()).is_zero())
        {
            return Err(Error::Invalid("empty supplied contact-patch witness"));
        }
        Ok(CoefficientRestriction {
            marked_ideal: result,
            contact_axis,
            remaining_axes: chart
                .axes()
                .iter()
                .filter(|i| **i != contact_axis)
                .copied()
                .collect(),
            restricted_guard_factors,
        })
    }

    /// Supplied monomial extraction and local residual-order certificate.
    /// Boundary multiplicity history is kept separately; these powers describe
    /// the marked ideal, not a mutation of the boundary divisor multiplicities.
    pub fn companion(
        &self,
        powers: Vec<(u64, usize)>,
        order: usize,
        budget: &mut Budget,
    ) -> Result<CompanionArithmetic> {
        if order > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("residual order"));
        }
        let chart = self.frame.map().target();
        let mut seen = BTreeSet::new();
        let mut monomial = chart.ring().one();
        for (id, power) in &powers {
            if !seen.insert(*id) || *power == 0 {
                return Err(Error::Invalid("repeated/zero monomial factor"));
            }
            let boundary = self
                .frame
                .boundaries()
                .iter()
                .find(|b| b.id == *id)
                .ok_or(Error::Invalid("unknown monomial boundary"))?;
            let factor = budget.power(&chart.ring().coordinate(boundary.frame_axis)?, *power)?;
            monomial = budget.mul(&monomial, &factor)?;
        }
        budget.reserve_slots(self.adapted.ideal.generators().len())?;
        let mut residuals = Vec::new();
        for f in self.adapted.ideal.generators() {
            budget.charge(1)?;
            let (q, r) = f.quot_rem(&monomial, false);
            if !r.is_zero() || budget.mul(&monomial, &q)? != *f {
                return Err(Error::Invalid("monomial extraction/recombination"));
            }
            residuals.push(q);
        }
        let residual = Ideal::new(chart.ring().clone(), residuals, budget)?;
        // No hidden exceptional factor may remain. Ordinary exact division is
        // sufficient here because these are the supplied polynomial generators.
        for boundary in self.frame.boundaries() {
            let coordinate = chart.ring().coordinate(boundary.frame_axis)?;
            if residual
                .generators()
                .iter()
                .all(|f| f.quot_rem(&coordinate, false).1.is_zero())
            {
                return Err(Error::Invalid("residual still has common boundary factor"));
            }
        }
        let center = Ideal::new(
            chart.ring().clone(),
            self.normal_axes
                .iter()
                .map(|i| chart.ring().coordinate(*i))
                .collect::<Result<_>>()?,
            budget,
        )?;
        let mut normal = residual.clone();
        for j in 0..order {
            for f in normal.generators() {
                if !chart.contains(&center, f, budget)? {
                    return Err(Error::Invalid("residual below supplied order on centre"));
                }
            }
            if j + 1 < order {
                normal = derivative_ideal(&normal, &self.normal_axes, &[], budget)?;
            }
        }
        let mut full = residual.clone();
        for _ in 0..order {
            full = derivative_ideal(&full, chart.axes(), &[], budget)?;
        }
        if !chart.contains(&full, &chart.ring().one(), budget)? {
            return Err(Error::Invalid(
                "residual lacks local upper-order certificate",
            ));
        }
        let companion = if order == 0 {
            None
        } else {
            let n = MarkedIdeal::new(residual.clone(), order, budget)?;
            if order < self.adapted.mark {
                let m = MarkedIdeal::new(
                    Ideal::new(chart.ring().clone(), vec![monomial.clone()], budget)?,
                    self.adapted.mark - order,
                    budget,
                )?;
                Some(n.sum(&m, budget)?)
            } else {
                Some(n)
            }
        };
        Ok(CompanionArithmetic {
            monomial_factor: monomial,
            residual,
            verified_local_residual_order: order,
            companion,
            boundary_powers: powers,
        })
    }
}

/// Supplied bookkeeping only. This deliberately has no conversion to a
/// certified maximum locus, permissible BM next step, or termination claim.
#[derive(Clone, Debug)]
pub struct RecordedInvariant {
    pub residual_orders: Vec<Rational>,
    pub exceptional_counts: Vec<usize>,
    pub boundary_births: Vec<(u64, usize)>,
}
