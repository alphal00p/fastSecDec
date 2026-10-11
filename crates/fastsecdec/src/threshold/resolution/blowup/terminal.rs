//! Local principalization evidence. This is stronger than empty marked
//! cosupport, and remains distinct from a real integration atlas certificate.
use super::super::*;
use super::helpers::{Result, unit};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct LocalPrincipalization {
    factor: Arc<FactorLeaf>,
    full_snc: Arc<VerifiedRelativeSnc>,
    residual_generator: Poly,
    residual_quotients: Vec<Poly>,
    clearing: Option<UnitClearing>,
}
impl LocalPrincipalization {
    pub fn factor(&self) -> &Arc<FactorLeaf> {
        &self.factor
    }
    /// Extra strict factor here is a density-divisor certificate, not a newly
    /// issued exceptional birth in ResolutionHistory.
    pub fn full_snc(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.full_snc
    }
    pub fn residual_generator(&self) -> &Poly {
        &self.residual_generator
    }
    pub fn residual_quotients(&self) -> &[Poly] {
        &self.residual_quotients
    }
    pub fn clearing(&self) -> Option<&UnitClearing> {
        self.clearing.as_ref()
    }
}
#[derive(Clone, Debug)]
pub enum PrincipalizationProduction {
    Principal(Arc<LocalPrincipalization>),
    FurtherResolution { factor: Arc<FactorLeaf> },
}
/// Attempt only a fully checked unit or principal-SNC residual. Candidate
/// search may be incomplete in a general quotient: failure requests further
/// resolution; it is never an assertion that no principal generator exists.
pub fn certify_local_principalization(
    factor: Arc<FactorLeaf>,
    namespace: &str,
    b: &mut Budget,
) -> Result<PrincipalizationProduction> {
    let data = factor.data();
    let history = data.history();
    let frame = history.ledger().frame();
    let local = frame.local();
    data.verify(b)?;
    let residual = Ideal::new(local.ring().clone(), data.quotients().to_vec(), b)?;
    if local
        .ideal()
        .sum(&residual, b)?
        .contains(&local.ring().one(), local.unit_relations(), b)?
    {
        return Ok(PrincipalizationProduction::Principal(Arc::new(
            LocalPrincipalization {
                factor: factor.clone(),
                full_snc: history.ledger().clone(),
                residual_generator: local.ring().one(),
                residual_quotients: data.quotients().to_vec(),
                clearing: None,
            },
        )));
    }
    for (i, candidate) in data.quotients().iter().enumerate() {
        if local.zero(candidate, b)? {
            continue;
        }
        let clearing = clear_units(local, candidate, b)?;
        let h = &clearing.numerator;
        if unit(local, h, b)? {
            return Err(Error::Invalid("residual unit proof inconsistency"));
        }
        let (id, _) = history.next_transition()?;
        b.reserve_slots(
            history
                .ledger()
                .divisors()
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("terminal divisor count"))?,
        )?;
        let mut divisors = history.ledger().divisors().to_vec();
        divisors.push(InitialDivisor {
            id,
            equation: h.clone(),
        });
        let snc = match verify_initial_relative_snc(frame.clone(), divisors, b)? {
            SncProduction::Verified(v) => v,
            SncProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            SncProduction::Unresolved { .. } => continue,
        };
        let principal = local
            .ideal()
            .sum(&Ideal::new(local.ring().clone(), vec![h.clone()], b)?, b)?;
        let mut quotients = Vec::new();
        b.reserve_slots(data.quotients().len())?;
        for (j, f) in data.quotients().iter().enumerate() {
            if !principal.contains(f, local.unit_relations(), b)? {
                break;
            }
            quotients.push(
                super::super::companion::divide_regular_in_localization(
                    local,
                    h,
                    f,
                    &format!("{namespace}_candidate{i}_generator{j}"),
                    b,
                )?
                .0,
            );
        }
        if quotients.len() != data.quotients().len() {
            continue;
        }
        let q = Ideal::new(local.ring().clone(), quotients.clone(), b)?;
        if !local
            .ideal()
            .sum(&q, b)?
            .contains(&local.ring().one(), local.unit_relations(), b)?
        {
            continue;
        }
        return Ok(PrincipalizationProduction::Principal(Arc::new(
            LocalPrincipalization {
                factor: factor.clone(),
                full_snc: snc,
                residual_generator: h.clone(),
                residual_quotients: quotients,
                clearing: Some(clearing),
            },
        )));
    }
    Ok(PrincipalizationProduction::FurtherResolution { factor })
}
