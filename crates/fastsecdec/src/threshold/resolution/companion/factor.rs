use super::super::{Budget, Error, Ideal, MarkedIdeal, Poly, VerifiedRelativeSnc};
use super::division::{CartierDivision, QuotientMethod, divide_cartier};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Default)]
pub struct FactorProgress {
    pub current_divisor: usize,
    pub powers: Vec<usize>,
    /// These keep their association with the original, already canonical
    /// source generators; normalization occurs only after recombination.
    pub quotients: Vec<Poly>,
    pub divisions: usize,
    pub native_eliminations: usize,
    pub recombinations: usize,
    pub operations: usize,
    pub ideal_slots: usize,
}

#[derive(Clone, Debug)]
pub struct WholeCartierFactorization {
    ledger: Arc<VerifiedRelativeSnc>,
    source: Arc<MarkedIdeal>,
    monomial: Poly,
    residual: Arc<Ideal>,
    /// None is a proved unit equation. Otherwise this is an actual retained
    /// residual generator not divisible by the whole boundary equation.
    indivisible_generators: Vec<Option<usize>>,
    progress: FactorProgress,
}
impl WholeCartierFactorization {
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn monomial(&self) -> &Poly {
        &self.monomial
    }
    pub fn residual(&self) -> &Arc<Ideal> {
        &self.residual
    }
    pub fn indivisible_generators(&self) -> &[Option<usize>] {
        &self.indivisible_generators
    }
    pub fn progress(&self) -> &FactorProgress {
        &self.progress
    }
}

#[derive(Clone, Debug)]
pub enum FactorProduction {
    WholeEquations(Box<WholeCartierFactorization>),
    ZeroIdeal {
        ledger: Arc<VerifiedRelativeSnc>,
        source: Arc<MarkedIdeal>,
        progress: FactorProgress,
    },
    Incomplete {
        ledger: Arc<VerifiedRelativeSnc>,
        source: Arc<MarkedIdeal>,
        reason: &'static str,
        progress: FactorProgress,
    },
}

/// Automatically extracts whole Cartier equations in the native quotient.
/// The result deliberately makes NO componentwise BM maximality claim: a
/// disconnected equation can have unequal local powers. A component refinement
/// certificate is a separate gate, never inferred from this constructor.
pub fn factor_whole_cartier_equations(
    ledger: Arc<VerifiedRelativeSnc>,
    source: Arc<MarkedIdeal>,
    max_power: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<FactorProduction> {
    if source.ideal().ring() != ledger.frame().local().ring() {
        return Err(Error::Invalid("Cartier factorization ring"));
    }
    let mut progress = FactorProgress::default();
    let result = run(
        &ledger,
        &source,
        max_power,
        namespace,
        budget,
        &mut progress,
    );
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(None) => Ok(FactorProduction::ZeroIdeal {
            ledger,
            source,
            progress,
        }),
        Ok(Some((monomial, residual, indivisible_generators))) => Ok(
            FactorProduction::WholeEquations(Box::new(WholeCartierFactorization {
                ledger,
                source,
                monomial,
                residual,
                indivisible_generators,
                progress,
            })),
        ),
        Err(Error::ResourceIncomplete(reason)) => Ok(FactorProduction::Incomplete {
            ledger,
            source,
            reason,
            progress,
        }),
        Err(error) => Err(error),
    }
}
type Factored = (Poly, Arc<Ideal>, Vec<Option<usize>>);
fn run(
    ledger: &Arc<VerifiedRelativeSnc>,
    source: &Arc<MarkedIdeal>,
    max_power: usize,
    namespace: &str,
    budget: &mut Budget,
    progress: &mut FactorProgress,
) -> Result<Option<Factored>> {
    let local = ledger.frame().local();
    budget.reserve_slots(source.ideal().generators().len())?;
    progress.quotients = source.ideal().generators().to_vec();
    budget.reserve_slots(ledger.divisors().len())?;
    progress.powers = vec![0; ledger.divisors().len()];
    let mut nonzero = false;
    for f in source.ideal().generators() {
        local.supports(f)?;
        if !local.zero(f, budget)? {
            nonzero = true;
        }
    }
    if !nonzero {
        return Ok(None);
    }
    for index in 0..ledger.divisors().len() {
        progress.current_divisor = index;
        loop {
            budget.reserve_slots(progress.quotients.len())?;
            let mut divided = Vec::with_capacity(progress.quotients.len());
            for (generator, f) in progress.quotients.iter().enumerate() {
                let name = format!(
                    "{namespace}::d{index}_p{}_g{generator}",
                    progress.powers[index]
                );
                match divide_cartier(ledger.clone(), index, f.clone(), &name, budget)? {
                    CartierDivision::Quotient(q) => {
                        progress.divisions = progress
                            .divisions
                            .checked_add(1)
                            .ok_or(Error::ResourceIncomplete("quotient counter"))?;
                        if q.method() == QuotientMethod::NativeElimination {
                            progress.native_eliminations = progress
                                .native_eliminations
                                .checked_add(1)
                                .ok_or(Error::ResourceIncomplete("elimination counter"))?;
                        }
                        divided.push(q.quotient().clone());
                    }
                    CartierDivision::NotDivisible { .. } | CartierDivision::UnitDivisor { .. } => {
                        break;
                    }
                }
            }
            if divided.len() != progress.quotients.len() {
                break;
            }
            if progress.powers[index] >= max_power {
                return Err(Error::ResourceIncomplete("Cartier extraction power cap"));
            }
            progress.powers[index] = progress.powers[index]
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("Cartier exponent overflow"))?;
            progress.quotients = divided;
        }
    }
    progress.current_divisor = ledger.divisors().len();
    let mut monomial = local.ring().one();
    for (d, power) in ledger.divisors().iter().zip(&progress.powers) {
        let factor = budget.power(&d.equation, *power)?;
        monomial = budget.mul(&monomial, &factor)?;
    }
    for (f, q) in source.ideal().generators().iter().zip(&progress.quotients) {
        if !local.zero(&(f - &budget.mul(&monomial, q)?), budget)? {
            return Err(Error::Invalid("whole Cartier recombination"));
        }
        progress.recombinations = progress
            .recombinations
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("recombination counter"))?;
    }
    let residual = Arc::new(Ideal::new(
        local.ring().clone(),
        progress.quotients.clone(),
        budget,
    )?);
    budget.reserve_slots(ledger.divisors().len())?;
    let mut witnesses = Vec::new();
    for index in 0..ledger.divisors().len() {
        let mut witness = None;
        let mut unit = false;
        for (generator, f) in residual.generators().iter().enumerate() {
            match divide_cartier(
                ledger.clone(),
                index,
                f.clone(),
                &format!("{namespace}::max_d{index}_g{generator}"),
                budget,
            )? {
                CartierDivision::NotDivisible { .. } => {
                    witness = Some(generator);
                    break;
                }
                CartierDivision::UnitDivisor { .. } => {
                    unit = true;
                    break;
                }
                CartierDivision::Quotient(_) => {}
            }
        }
        if witness.is_none() && !unit {
            return Err(Error::Invalid(
                "residual remains divisible by whole equation",
            ));
        }
        witnesses.push(witness);
    }
    Ok(Some((monomial, residual, witnesses)))
}
