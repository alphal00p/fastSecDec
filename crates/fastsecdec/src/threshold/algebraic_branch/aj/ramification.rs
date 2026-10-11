use super::valuation::LocalGerm;
use super::*;
use symbolica::symbol;

pub struct PowerChange {
    pub(super) source: Arc<GermPoint>,
    pub(super) source_ledger: Arc<VerifiedRelativeSnc>,
    pub(super) exponents: Vec<usize>,
    pub(super) signs: Vec<i8>,
    pub(super) extension: Option<RingExtension>,
    pub(super) point: Arc<GermPoint>,
    pub(super) ledger: Arc<VerifiedRelativeSnc>,
    pub(super) root_scale: Poly,
    pub(super) coefficients: Vec<Poly>,
    pub(super) quotients: Vec<VerifiedCartierQuotient>,
}
impl PowerChange {
    pub fn source(&self) -> &Arc<GermPoint> {
        &self.source
    }
    pub fn source_ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.source_ledger
    }
    pub fn exponents(&self) -> &[usize] {
        &self.exponents
    }
    /// Target coordinates are signed germ coordinates. Odd q uses +1; even q
    /// retains both source signs. This does not supply atlas multiplicities.
    pub fn signs(&self) -> &[i8] {
        &self.signs
    }
    pub fn extension(&self) -> Option<&RingExtension> {
        self.extension.as_ref()
    }
    pub fn point(&self) -> &Arc<GermPoint> {
        &self.point
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    pub fn root_scale(&self) -> &Poly {
        &self.root_scale
    }
    pub fn coefficients(&self) -> &[Poly] {
        &self.coefficients
    }
    pub fn quotients(&self) -> &[VerifiedCartierQuotient] {
        &self.quotients
    }
}
fn integer(q: &Rational) -> Result<usize> {
    if !q.is_integer() {
        return Err(Error::Invalid(
            "ramification failed to integralize valuation",
        ));
    }
    q.numerator_ref()
        .to_i64()
        .and_then(|i| usize::try_from(i).ok())
        .ok_or(Error::ResourceIncomplete("ramified exponent size"))
}
pub(super) fn denominators(beta: &[Rational], b: &Budget) -> Result<Vec<usize>> {
    beta.iter()
        .map(|q| {
            let n = q
                .denominator_ref()
                .to_i64()
                .and_then(|i| usize::try_from(i).ok())
                .ok_or(Error::ResourceIncomplete("ramification denominator"))?;
            if n > b.limits.max_mark || n > usize::from(b.limits.max_degree_per_axis) {
                return Err(Error::ResourceIncomplete("ramification degree cap"));
            }
            Ok(n)
        })
        .collect()
}
pub(super) fn sign_vectors(q: &[usize], b: &mut Budget) -> Result<Vec<Vec<i8>>> {
    let even = q.iter().filter(|n| **n % 2 == 0).count();
    let count = 1usize
        .checked_shl(
            u32::try_from(even)
                .map_err(|_| Error::ResourceIncomplete("ramification sign count"))?,
        )
        .ok_or(Error::ResourceIncomplete("ramification sign count"))?;
    b.reserve_slots(
        count
            .checked_mul(q.len())
            .ok_or(Error::ResourceIncomplete("ramification sign slots"))?,
    )?;
    Ok((0..count)
        .map(|bits| {
            let mut j = 0;
            q.iter()
                .map(|n| {
                    if n % 2 == 0 {
                        let s = if bits & (1 << j) == 0 { 1 } else { -1 };
                        j += 1;
                        s
                    } else {
                        1
                    }
                })
                .collect()
        })
        .collect())
}
/// None retains a valid source requiring an adapted SNC coordinate chart. No
/// ambient coordinate is silently substituted for a nonlinear Cartier factor.
pub(super) fn change(
    germ: &LocalGerm,
    coefficients: &[Poly],
    beta: &[Rational],
    signs: Vec<i8>,
    namespace: &str,
    b: &mut Budget,
) -> Result<Option<PowerChange>> {
    let q = denominators(beta, b)?;
    if beta.len() != germ.ledger().divisors().len()
        || q.len() != signs.len()
        || signs
            .iter()
            .zip(&q)
            .any(|(s, n)| ![-1, 1].contains(s) || (*s == -1 && n % 2 != 0))
    {
        return Err(Error::Invalid("signed ramification roles"));
    }
    let source = germ.point.clone();
    let source_ledger = germ.ledger().clone();
    let old = source.frame();
    let mut changed = Vec::new();
    for (j, n) in q.iter().enumerate() {
        if *n == 1 {
            continue;
        }
        if !source.is_zero(&source_ledger.divisors()[j].equation)? {
            return Err(Error::Invalid(
                "ramified divisor absent from selected fiber",
            ));
        }
        let mut selected = None;
        for axis in old.free_axes() {
            if old.local().zero(
                &(&source_ledger.divisors()[j].equation - &old.local().ring().coordinate(*axis)?),
                b,
            )? {
                selected = Some(*axis);
                break;
            }
        }
        let Some(axis) = selected else {
            return Ok(None);
        };
        if changed.iter().any(|(_, a)| *a == axis) {
            return Err(Error::Invalid("ramification repeated coordinate"));
        }
        changed.push((j, axis));
    }
    let (extension, point, ledger, pulled) = if changed.is_empty() {
        (
            None,
            source.clone(),
            source_ledger.clone(),
            coefficients.to_vec(),
        )
    } else {
        let mut fresh = (0..changed.len())
            .map(|i| symbol!(format!("{namespace}::t{i}")))
            .collect::<Vec<_>>();
        fresh.push(symbol!(format!("{namespace}::minor_inverse")));
        let extension = RingExtension::with_symbols(old.local().ring().clone(), &fresh, b)?;
        let start = old.local().ring().len();
        let ring = extension.target();
        let mut equations = extension
            .ideal(old.local().ideal(), b)?
            .generators()
            .to_vec();
        let mut selected = Vec::new();
        for i in old.selected_equations() {
            let p = extension.pull(&old.source().ideal().generators()[*i], b)?;
            selected.push(
                equations
                    .iter()
                    .position(|e| *e == p)
                    .ok_or(Error::Invalid("ramification inherited equation"))?,
            );
        }
        let mut dependent = old.dependent_axes().to_vec();
        let mut free = old.free_axes().to_vec();
        let mut axes = old.local().axes().to_vec();
        for (k, (j, axis)) in changed.iter().enumerate() {
            let t = ring.coordinate(start + k)?;
            let power = b
                .power(&t, q[*j])?
                .mul_coeff(Rational::from(i64::from(signs[*j])));
            selected.push(equations.len());
            equations.push(&ring.coordinate(*axis)? - &power);
            dependent.push(*axis);
            axes.push(start + k);
            let slot = free
                .iter()
                .position(|i| i == axis)
                .ok_or(Error::Invalid("ramification free axis"))?;
            free[slot] = start + k;
        }
        let local = LocalizedAlgebra::new(
            Ideal::new(ring.clone(), equations, b)?,
            axes,
            extension.guards(old.local().guards(), b)?,
            b,
        )?;
        let frame = Arc::new(
            EtaleCertificate {
                source: local,
                equations: selected,
                dependent_axes: dependent,
                free_axes: free,
                determinant_inverse_axis: Some(start + changed.len()),
            }
            .verify(b)?,
        );
        let point = source.extend(frame.clone(), source.values().to_vec(), b)?;
        let divisors = source_ledger
            .divisors()
            .iter()
            .enumerate()
            .map(|(j, h)| {
                Ok(InitialDivisor {
                    id: h.id,
                    equation: if let Some(k) = changed.iter().position(|(a, _)| *a == j) {
                        ring.coordinate(start + k)?
                    } else {
                        extension.pull(&h.equation, b)?
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let ledger = match verify_initial_relative_snc(frame, divisors, b)? {
            SncProduction::Verified(l) => l,
            SncProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            SncProduction::Unresolved { .. } => {
                return Err(Error::Invalid("ramified reduced divisor is not SNC"));
            }
        };
        let pulled = coefficients
            .iter()
            .map(|p| extension.pull(p, b))
            .collect::<Result<Vec<_>>>()?;
        (Some(extension), point, ledger, pulled)
    };
    let powers = beta
        .iter()
        .zip(&q)
        .map(|(v, n)| integer(&(v * &exact_usize(*n)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut root_scale = ledger.frame().local().ring().one();
    for (j, n) in powers.iter().enumerate() {
        let p = b.power(&ledger.divisors()[j].equation, *n)?;
        root_scale = b.mul(&root_scale, &p)?;
    }
    let d = pulled.len() - 1;
    let mut reduced = pulled.clone();
    let mut quotients = Vec::new();
    for (i, c) in reduced.iter_mut().enumerate().take(d) {
        if ledger.frame().local().zero(c, b)? {
            continue;
        }
        for (j, n) in powers.iter().enumerate() {
            let count = n
                .checked_mul(d - i)
                .ok_or(Error::ResourceIncomplete("ramified coefficient exponent"))?;
            if count > b.limits.max_mark {
                return Err(Error::ResourceIncomplete(
                    "ramified coefficient extraction cap",
                ));
            }
            for k in 0..count {
                let CartierDivision::Quotient(receipt) = divide_cartier(
                    ledger.clone(),
                    j,
                    c.clone(),
                    &format!("{namespace}::c{i}_h{j}_{k}"),
                    b,
                )?
                else {
                    return Err(Error::Invalid("ramified coefficient not divisible"));
                };
                *c = receipt.quotient().clone();
                quotients.push(receipt);
            }
        }
        let power = b.power(&root_scale, d - i)?;
        let recombination = b.mul(c, &power)?;
        if !ledger
            .frame()
            .local()
            .zero(&(&pulled[i] - &recombination), b)?
        {
            return Err(Error::Invalid("ramified coefficient identity"));
        }
    }
    Ok(Some(PowerChange {
        source,
        source_ledger,
        exponents: q,
        signs,
        extension,
        point,
        ledger,
        root_scale,
        coefficients: reduced,
        quotients,
    }))
}
