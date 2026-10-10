use super::super::{Budget, Error, Ideal, MarkedIdeal, Poly};
use super::VerifiedRelativeSnc;
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
/// Quotients are a supplied factorization witness, checked independently in the
/// exact localized ring; no general divisor-valuation algorithm is inferred.
#[derive(Clone, Debug)]
pub struct MonomialWitness {
    pub source: Arc<MarkedIdeal>,
    pub powers: Vec<usize>,
    pub quotients: Vec<Poly>,
}
#[derive(Clone, Debug)]
pub struct MonomialCandidate {
    pub indices: Vec<usize>,
    pub exponent_sum: usize,
    pub empty: bool,
}
#[derive(Clone, Debug, Default)]
pub struct MonomialProgress {
    pub recombinations_checked: usize,
    pub residual_unit: bool,
    pub candidates: Vec<MonomialCandidate>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct ProducedMonomialCenter {
    ledger: Arc<VerifiedRelativeSnc>,
    witness: MonomialWitness,
    monomial: Poly,
    residual: Ideal,
    indices: Vec<usize>,
    center: Ideal,
    exceptional_power: usize,
    progress: MonomialProgress,
}
impl ProducedMonomialCenter {
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    pub fn witness(&self) -> &MonomialWitness {
        &self.witness
    }
    pub fn monomial(&self) -> &Poly {
        &self.monomial
    }
    pub fn residual(&self) -> &Ideal {
        &self.residual
    }
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
    pub fn center(&self) -> &Ideal {
        &self.center
    }
    pub fn exceptional_power(&self) -> usize {
        self.exceptional_power
    }
    pub fn progress(&self) -> &MonomialProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum MonomialProduction {
    Center(Box<ProducedMonomialCenter>),
    EmptyCosupport {
        ledger: Arc<VerifiedRelativeSnc>,
        witness: MonomialWitness,
        progress: MonomialProgress,
    },
    Incomplete {
        ledger: Arc<VerifiedRelativeSnc>,
        witness: MonomialWitness,
        reason: &'static str,
        progress: MonomialProgress,
    },
}
/// Produce the incidence-maximum monomial center of BM 2008 §5 Step II.A
/// (pp. 628–629) in one verified initial SNC chart, not the alternate Γ ordering.
/// Does not publish blowup charts, new birth history or a real atlas.
pub fn produce_monomial_center(
    ledger: Arc<VerifiedRelativeSnc>,
    witness: MonomialWitness,
    budget: &mut Budget,
) -> Result<MonomialProduction> {
    if witness.powers.len() != ledger.divisors().len()
        || witness.quotients.len() != witness.source.ideal().generators().len()
        || witness.source.ideal().ring() != ledger.frame().local().ring()
    {
        return Err(Error::Invalid("monomial witness shape/ring"));
    }
    let mut progress = MonomialProgress::default();
    let outcome = produce(&ledger, &witness, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match outcome {
        Ok(Some(mut center)) => {
            center.progress = progress;
            Ok(MonomialProduction::Center(Box::new(center)))
        }
        Ok(None) => Ok(MonomialProduction::EmptyCosupport {
            ledger,
            witness,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(MonomialProduction::Incomplete {
            ledger,
            witness,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn produce(
    ledger: &Arc<VerifiedRelativeSnc>,
    witness: &MonomialWitness,
    budget: &mut Budget,
    progress: &mut MonomialProgress,
) -> Result<Option<ProducedMonomialCenter>> {
    let local = ledger.frame().local();
    let mut monomial = local.ring().one();
    for (divisor, power) in ledger.divisors().iter().zip(&witness.powers) {
        let factor = budget.power(&divisor.equation, *power)?;
        monomial = budget.mul(&monomial, &factor)?;
    }
    for (f, q) in witness
        .source
        .ideal()
        .generators()
        .iter()
        .zip(&witness.quotients)
    {
        local.supports(q)?;
        if !local.zero(&(f - &budget.mul(&monomial, q)?), budget)? {
            return Err(Error::Invalid("monomial recombination"));
        }
        progress.recombinations_checked += 1;
    }
    let residual = Ideal::new(local.ring().clone(), witness.quotients.clone(), budget)?;
    let full = local.ideal().sum(&residual, budget)?;
    if !full.contains(&local.ring().one(), local.unit_relations(), budget)? {
        return Err(Error::Invalid("residual ideal is not unit"));
    }
    progress.residual_unit = true;
    let mut selected: Option<Vec<usize>> = None;
    let mut selected_sum = 0usize;
    // The SNC certificate has already bounded and exhausted all nonempty
    // index subsets. Empty geometric intersections remain explicit receipts.
    for stratum in &ledger.progress().intersections {
        let sum = stratum
            .indices
            .iter()
            .try_fold(0usize, |s, i| s.checked_add(witness.powers[*i]))
            .ok_or(Error::ResourceIncomplete("monomial exponent sum"))?;
        let mark = witness.source.mark();
        if sum < mark
            || stratum
                .indices
                .iter()
                .any(|i| sum - witness.powers[*i] >= mark)
        {
            continue;
        }
        budget.reserve_slots(1)?;
        progress.candidates.push(MonomialCandidate {
            indices: stratum.indices.clone(),
            exponent_sum: sum,
            empty: stratum.empty,
        });
        if stratum.empty {
            continue;
        }
        // Published lexicographic order on incidence vectors in stable divisor
        // order: first differing included divisor makes that subset larger.
        let better = selected.as_ref().is_none_or(|previous| {
            (0..ledger.divisors().len())
                .map(|i| stratum.indices.contains(&i))
                .cmp((0..ledger.divisors().len()).map(|i| previous.contains(&i)))
                .is_gt()
        });
        if better {
            selected = Some(stratum.indices.clone());
            selected_sum = sum;
        }
    }
    let Some(indices) = selected else {
        return Ok(None);
    };
    let center = ledger
        .intersection(&indices)
        .ok_or(Error::Invalid("missing selected SNC stratum"))?
        .locus
        .clone();
    Ok(Some(ProducedMonomialCenter {
        ledger: ledger.clone(),
        witness: witness.clone(),
        monomial,
        residual,
        indices,
        center,
        exceptional_power: selected_sum - witness.source.mark(),
        progress: MonomialProgress::default(),
    }))
}
