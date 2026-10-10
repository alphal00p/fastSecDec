use super::super::{Budget, Error, Ideal, LocalizedHistory, MarkedIdeal, Poly, ResolutionHistory};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

/// Original generator association is retained even when localization makes
/// generators equal or zero. Temporary ideals alone may canonicalize them.
#[derive(Clone, Debug)]
pub struct FactorData {
    history: Arc<ResolutionHistory>,
    source: Arc<MarkedIdeal>,
    originals: Vec<Poly>,
    quotients: Vec<Poly>,
    powers: Vec<usize>,
}
impl FactorData {
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn originals(&self) -> &[Poly] {
        &self.originals
    }
    pub fn quotients(&self) -> &[Poly] {
        &self.quotients
    }
    pub fn powers(&self) -> &[usize] {
        &self.powers
    }
    pub(crate) fn checked_clone(&self, b: &mut Budget) -> Result<Self> {
        b.reserve_slots(
            self.originals
                .len()
                .checked_add(self.quotients.len())
                .and_then(|n| n.checked_add(self.powers.len()))
                .ok_or(Error::ResourceIncomplete("factor snapshot allocation"))?,
        )?;
        Ok(self.clone())
    }
    pub(crate) fn initial(
        history: Arc<ResolutionHistory>,
        source: Arc<MarkedIdeal>,
        b: &mut Budget,
    ) -> Result<Self> {
        if history.ledger().frame().local().ring() != source.ideal().ring() {
            return Err(Error::Invalid("factor frontier source ring"));
        }
        b.reserve_slots(
            source
                .ideal()
                .generators()
                .len()
                .checked_mul(2)
                .ok_or(Error::ResourceIncomplete("factor source count"))?,
        )?;
        b.reserve_slots(history.ledger().divisors().len())?;
        let out = Self {
            powers: vec![0; history.ledger().divisors().len()],
            history,
            originals: source.ideal().generators().to_vec(),
            quotients: source.ideal().generators().to_vec(),
            source,
        };
        out.verify(b)?;
        Ok(out)
    }
    pub(crate) fn residual(&self, b: &mut Budget) -> Result<Arc<Ideal>> {
        b.reserve_slots(self.quotients.len())?;
        Ok(Arc::new(Ideal::new(
            self.history.ledger().frame().local().ring().clone(),
            self.quotients.clone(),
            b,
        )?))
    }
    pub(crate) fn monomial(&self, b: &mut Budget) -> Result<Poly> {
        let mut m = self.history.ledger().frame().local().ring().one();
        for (d, p) in self.history.ledger().divisors().iter().zip(&self.powers) {
            let f = b.power(&d.equation, *p)?;
            m = b.mul(&m, &f)?;
        }
        Ok(m)
    }
    pub(crate) fn verify(&self, b: &mut Budget) -> Result<()> {
        if self.originals.len() != self.source.ideal().generators().len()
            || self.quotients.len() != self.originals.len()
            || self.powers.len() != self.history.ledger().divisors().len()
        {
            return Err(Error::Invalid("factor association shape"));
        }
        let local = self.history.ledger().frame().local();
        let m = self.monomial(b)?;
        for (f, q) in self.originals.iter().zip(&self.quotients) {
            local.supports(f)?;
            local.supports(q)?;
            let product = b.mul(&m, q)?;
            if !local.zero(&(f - &product), b)? {
                return Err(Error::Invalid("component factor recombination"));
            }
        }
        Ok(())
    }
    pub(crate) fn restricted(&self, open: &LocalizedHistory, b: &mut Budget) -> Result<Self> {
        if !Arc::ptr_eq(open.open().source(), &self.history) {
            return Err(Error::Invalid("factor restriction owner"));
        }
        b.reserve_slots(
            self.originals
                .len()
                .checked_mul(2)
                .ok_or(Error::ResourceIncomplete("factor pull count"))?,
        )?;
        let originals = self
            .originals
            .iter()
            .map(|f| open.open().extension().pull(f, b))
            .collect::<Result<Vec<_>>>()?;
        let quotients = self
            .quotients
            .iter()
            .map(|f| open.open().extension().pull(f, b))
            .collect::<Result<Vec<_>>>()?;
        b.reserve_slots(self.powers.len())?;
        let out = Self {
            history: open.history().clone(),
            source: self.source.clone(),
            originals,
            quotients,
            powers: self.powers.clone(),
        };
        out.verify(b)?;
        Ok(out)
    }
    pub(crate) fn divided(
        &self,
        index: usize,
        quotients: Vec<Poly>,
        b: &mut Budget,
    ) -> Result<Self> {
        if quotients.len() != self.quotients.len() {
            return Err(Error::Invalid("divided source association"));
        }
        let mut out = self.checked_clone(b)?;
        let power = out
            .powers
            .get_mut(index)
            .ok_or(Error::Invalid("factor divisor index"))?;
        *power = power
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("factor power overflow"))?;
        out.quotients = quotients;
        out.verify(b)?;
        Ok(out)
    }
    pub(crate) fn absorb_unit(&self, index: usize, b: &mut Budget) -> Result<Self> {
        let d = self
            .history
            .ledger()
            .divisors()
            .get(index)
            .ok_or(Error::Invalid("unit factor index"))?;
        let local = self.history.ledger().frame().local();
        let h = Ideal::new(local.ring().clone(), vec![d.equation.clone()], b)?;
        if !local
            .ideal()
            .sum(&h, b)?
            .contains(&local.ring().one(), local.unit_relations(), b)?
        {
            return Err(Error::Invalid("unproved unit factor"));
        }
        let power = self.powers[index];
        if power == 0 {
            return self.checked_clone(b);
        }
        let factor = b.power(&d.equation, power)?;
        let quotients = self
            .quotients
            .iter()
            .map(|q| b.mul(q, &factor))
            .collect::<Result<Vec<_>>>()?;
        let mut out = self.checked_clone(b)?;
        out.quotients = quotients;
        out.powers[index] = 0;
        out.verify(b)?;
        Ok(out)
    }
}
