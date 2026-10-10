use super::super::{Budget, Error, Ideal, Poly, RingExtension, VerifiedRelativeSnc};
use std::sync::Arc;
use symbolica::symbol;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotientMethod {
    Ambient,
    NativeElimination,
}

#[derive(Clone, Debug)]
pub struct VerifiedCartierQuotient {
    owner: Arc<VerifiedRelativeSnc>,
    divisor: usize,
    dividend: Poly,
    quotient: Poly,
    method: QuotientMethod,
}
impl VerifiedCartierQuotient {
    pub fn owner(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.owner
    }
    pub fn divisor_index(&self) -> usize {
        self.divisor
    }
    pub fn dividend(&self) -> &Poly {
        &self.dividend
    }
    pub fn quotient(&self) -> &Poly {
        &self.quotient
    }
    pub fn method(&self) -> QuotientMethod {
        self.method
    }
}

#[derive(Clone, Debug)]
pub enum CartierDivision {
    Quotient(VerifiedCartierQuotient),
    /// Whole-equation nonmembership in the original localization. This is not
    /// componentwise maximality when a smooth boundary is disconnected.
    NotDivisible {
        owner: Arc<VerifiedRelativeSnc>,
        divisor: usize,
        dividend: Poly,
    },
    /// The equation is absent on this localization; do not extract arbitrary
    /// powers of a unit or treat its empty support as an active divisor.
    UnitDivisor {
        owner: Arc<VerifiedRelativeSnc>,
        divisor: usize,
    },
}

/// Native principal quotient discovery with independent recombination in the
/// original checked smooth chart. The temporary saturation inverse never
/// becomes an output guard. The owner binds actual Cartier regularity.
pub fn divide_cartier(
    owner: Arc<VerifiedRelativeSnc>,
    divisor: usize,
    dividend: Poly,
    namespace: &str,
    budget: &mut Budget,
) -> Result<CartierDivision> {
    let h = owner
        .divisors()
        .get(divisor)
        .ok_or(Error::Invalid("Cartier divisor index"))?
        .equation
        .clone();
    let local = owner.frame().local();
    local.supports(&dividend)?;
    budget.poly(&dividend)?;
    budget.reserve_slots(1)?;
    let principal = Ideal::new(local.ring().clone(), vec![h.clone()], budget)?;
    let total = local.ideal().sum(&principal, budget)?;
    if total.contains(&local.ring().one(), local.unit_relations(), budget)? {
        return Ok(CartierDivision::UnitDivisor { owner, divisor });
    }
    if !total.contains(&dividend, local.unit_relations(), budget)? {
        return Ok(CartierDivision::NotDivisible {
            owner,
            divisor,
            dividend,
        });
    }
    budget.charge(1)?;
    let (direct, remainder) = dividend.quot_rem(&h, false);
    budget.poly(&direct)?;
    budget.poly(&remainder)?;
    let (quotient, method) = if local.zero(&remainder, budget)? {
        (direct, QuotientMethod::Ambient)
    } else {
        let extension = RingExtension::new(
            local.ring().clone(),
            [
                symbol!(format!("{namespace}::saturation")),
                symbol!(format!("{namespace}::quotient")),
            ],
            budget,
        )?;
        let original = local.ring().one().variables().clone();
        let extended = extension.target().one();
        let mut variables = vec![
            extended.variables()[original.len()].clone(),
            extended.variables()[original.len() + 1].clone(),
        ];
        variables.extend(original.iter().cloned());
        budget.reserve_slots(variables.len())?;
        let lift = |f: &Poly| {
            f.rearrange_with_growth(&variables)
                .map_err(|_| Error::Invalid("native quotient extension"))
        };
        let h_ext = lift(&h)?;
        let f_ext = lift(&dividend)?;
        let t = h_ext
            .variable(&variables[1])
            .map_err(|_| Error::Invalid("native quotient axis"))?;
        let v = h_ext
            .variable(&variables[0])
            .map_err(|_| Error::Invalid("native saturation axis"))?;
        let count = local
            .ideal()
            .generators()
            .len()
            .checked_add(local.unit_relations().len())
            .and_then(|n| n.checked_add(2))
            .ok_or(Error::ResourceIncomplete("quotient relation count"))?;
        budget.reserve_slots(count)?;
        let mut equations = local
            .ideal()
            .generators()
            .iter()
            .chain(local.unit_relations())
            .map(lift)
            .collect::<Result<Vec<_>>>()?;
        equations.push(budget.mul(&h_ext, &t)? - f_ext);
        equations.push(budget.mul(&h_ext, &v)? - h_ext.one());
        equations.retain(|p| !p.is_zero());
        for equation in &equations {
            budget.poly(equation)?;
        }
        let basis = super::super::native_basis::checked(&equations, budget)?;
        budget.charge(1)?;
        let q = t.reduce(&basis);
        budget.poly(&q)?;
        if q.contains(0) || q.contains(1) {
            return Err(Error::Invalid("native principal quotient not eliminated"));
        }
        let q = q
            .rearrange_with_growth(&original)
            .map_err(|_| Error::Invalid("native quotient original variable map"))?;
        (q, QuotientMethod::NativeElimination)
    };
    local.supports(&quotient)?;
    if !local.zero(&(&dividend - &budget.mul(&h, &quotient)?), budget)? {
        return Err(Error::Invalid(
            "Cartier quotient original-localization recombination",
        ));
    }
    Ok(CartierDivision::Quotient(VerifiedCartierQuotient {
        owner,
        divisor,
        dividend,
        quotient,
        method,
    }))
}
