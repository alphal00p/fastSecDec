use super::*;

/// Private germ-local localization chain. The initial history is only adapter
/// scaffolding for the existing native localization checker. It is never
/// returned as a restartable BM history or a complete integration-domain proof.
#[derive(Clone)]
pub(super) struct LocalGerm {
    pub point: Arc<GermPoint>,
    history: Arc<ResolutionHistory>,
    pub localizations: Vec<Arc<LocalizedHistory>>,
}
impl LocalGerm {
    pub fn new(point: Arc<GermPoint>, ledger: Arc<VerifiedRelativeSnc>) -> Result<Self> {
        if !Arc::ptr_eq(point.frame(), ledger.frame()) {
            return Err(Error::Invalid("germ SNC/frame owner"));
        }
        Ok(Self {
            point,
            history: ResolutionHistory::initial(ledger)?,
            localizations: vec![],
        })
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        self.history.ledger()
    }
    fn select(
        &mut self,
        split: Arc<VerifiedComponentSplit>,
        polynomials: &mut [Poly],
        namespace: &str,
        b: &mut Budget,
    ) -> Result<()> {
        let value = self.point.evaluate(split.idempotent())?;
        if value != self.point.context().field().one() {
            return Err(Error::Invalid(
                "selected germ does not lie on divisible component",
            ));
        }
        if self.point.is_zero(&split.opens()[0].clearing().numerator)? {
            return Err(Error::Invalid("selected germ component open vanishes"));
        }
        let localized = match localize_component_open(self.history.clone(), split, 0, namespace, b)?
        {
            LocalizationProduction::Complete(v) => Arc::new(*v),
            LocalizationProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            LocalizationProduction::Empty { .. } => {
                return Err(Error::Invalid("selected germ open empty"));
            }
        };
        for p in polynomials {
            *p = localized.open().extension().pull(p, b)?;
        }
        self.point = self.point.extend(
            localized.open().ledger().frame().clone(),
            self.point.values().to_vec(),
            b,
        )?;
        self.history = localized.history().clone();
        self.localizations.push(localized);
        Ok(())
    }
}
pub(super) struct Valuations {
    pub germ: LocalGerm,
    /// Original coefficients pulled through every selected principal open.
    pub original: Vec<Poly>,
    pub residuals: Vec<Poly>,
    pub powers: Vec<Vec<usize>>,
    pub quotients: Vec<VerifiedCartierQuotient>,
    pub zero: Vec<bool>,
}
/// Extract only on exact selected boundary components. Polynomial nonmembership
/// along each divisor is not a claim that the final residual is a point-unit.
pub(super) fn extract(
    mut germ: LocalGerm,
    coefficients: Vec<Poly>,
    namespace: &str,
    b: &mut Budget,
) -> Result<Valuations> {
    let count = coefficients.len();
    let divisors = germ.ledger().divisors().len();
    b.reserve_slots(
        count
            .checked_mul(divisors)
            .ok_or(Error::ResourceIncomplete("germ valuation slots"))?,
    )?;
    b.reserve_slots(
        count
            .checked_mul(2)
            .ok_or(Error::ResourceIncomplete("germ coefficient slots"))?,
    )?;
    let mut all = coefficients.clone();
    all.extend(coefficients);
    let mut powers = vec![vec![0usize; divisors]; count];
    let mut zero = vec![false; count];
    let mut quotients = Vec::new();
    for i in 0..count {
        if germ.ledger().frame().local().zero(&all[count + i], b)? {
            zero[i] = true;
            continue;
        }
        for (j, power) in powers[i].iter_mut().enumerate() {
            if !germ.point.is_zero(&germ.ledger().divisors()[j].equation)? {
                continue;
            }
            loop {
                let q = divide_cartier(
                    germ.ledger().clone(),
                    j,
                    all[count + i].clone(),
                    &format!("{namespace}::c{i}_h{j}_q{}", *power),
                    b,
                )?;
                match q {
                    CartierDivision::Quotient(q) => {
                        if *power >= b.limits.max_mark {
                            return Err(Error::ResourceIncomplete("germ Cartier extraction cap"));
                        }
                        all[count + i] = q.quotient().clone();
                        *power += 1;
                        quotients.push(q);
                    }
                    CartierDivision::UnitDivisor { .. } => {
                        return Err(Error::Invalid("active germ divisor is unit"));
                    }
                    CartierDivision::NotDivisible { .. } => {
                        let BoundaryAlgebra::Regular(owner) =
                            RegularAlgebra::boundary(germ.ledger().clone(), j, b)?
                        else {
                            return Err(Error::Invalid("active germ boundary missing"));
                        };
                        let input = Arc::new(Ideal::new(
                            owner.local().ring().clone(),
                            vec![all[count + i].clone()],
                            b,
                        )?);
                        let split = match produce_component_split(
                            owner,
                            input,
                            &format!("{namespace}::c{i}_h{j}_split{}", *power),
                            b,
                        )? {
                            ComponentProduction::Complete(s) => Arc::new(*s),
                            ComponentProduction::Incomplete { reason, .. } => {
                                return Err(Error::ResourceIncomplete(reason));
                            }
                        };
                        let e = germ.point.evaluate(split.idempotent())?;
                        if germ.point.context().field().is_zero(&e) {
                            break;
                        }
                        if e != germ.point.context().field().one() {
                            return Err(Error::Invalid(
                                "component idempotent not zero/one at germ",
                            ));
                        }
                        germ.select(
                            split,
                            &mut all,
                            &format!("{namespace}::c{i}_h{j}_open{}", *power),
                            b,
                        )?;
                        // Next iteration must divide in the checked selected
                        // localization. No guessed rational inverse is added.
                        if !matches!(
                            divide_cartier(
                                germ.ledger().clone(),
                                j,
                                all[count + i].clone(),
                                &format!("{namespace}::post_open"),
                                b
                            )?,
                            CartierDivision::Quotient(_)
                        ) {
                            return Err(Error::Invalid("selected component failed exact division"));
                        }
                    }
                }
            }
        }
    }
    let residuals = all.split_off(count);
    // Recheck complete simultaneous recombination in the final localization.
    for i in 0..count {
        let mut p = residuals[i].clone();
        for (j, k) in powers[i].iter().enumerate() {
            let h = b.power(&germ.ledger().divisors()[j].equation, *k)?;
            p = b.mul(&p, &h)?;
        }
        if !germ.ledger().frame().local().zero(&(&all[i] - &p), b)? {
            return Err(Error::Invalid("germ valuation recombination"));
        }
    }
    Ok(Valuations {
        germ,
        original: all,
        residuals,
        powers,
        quotients,
        zero,
    })
}

/// The coefficient index is descending weight: for ascending slot k, i=d-k.
/// A selected candidate must be a monomial TIMES A POINT-UNIT.
pub(super) fn monomial_generator(v: &Valuations, d: usize) -> Result<Option<Vec<Rational>>> {
    for k in 0..d {
        if v.zero[k] || v.germ.point.is_zero(&v.residuals[k])? {
            continue;
        }
        let weight = d - k;
        let weight = exact_usize(weight)?;
        let beta = v.powers[k]
            .iter()
            .map(|e| Ok(exact_usize(*e)? / &weight))
            .collect::<Result<Vec<_>>>()?;
        let mut good = true;
        for j in 0..d {
            if v.zero[j] {
                continue;
            }
            for (axis, x) in beta.iter().enumerate() {
                if exact_usize(v.powers[j][axis])? < x * &exact_usize(d - j)? {
                    good = false;
                    break;
                }
            }
        }
        if good {
            return Ok(Some(beta));
        }
    }
    Ok(None)
}
