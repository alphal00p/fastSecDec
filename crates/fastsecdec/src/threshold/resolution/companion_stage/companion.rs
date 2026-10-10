use super::super::{
    Budget, Error, Ideal, MarkedIdeal, MonomialProduction, MonomialWitness, OrderProduction,
    ProducedContactCover, produce_monomial_center, produce_ordinary_contact_cover,
};
use super::super::{
    ComponentResidualOrder, CoverLocalizationProduction, LocalizedCoverHistory,
    localize_verified_cover_open,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Default)]
pub struct CompanionOpenProgress {
    pub localization: Option<CoverLocalizationProduction>,
    pub restriction: Option<Arc<LocalizedCoverHistory>>,
    pub contact: Option<OrderProduction>,
    pub monomial: Option<MonomialProduction>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct CompanionContactOpen {
    order: Arc<ComponentResidualOrder>,
    restriction: Arc<LocalizedCoverHistory>,
    companion: Arc<MarkedIdeal>,
    contact: Arc<ProducedContactCover>,
}
impl CompanionContactOpen {
    pub fn order(&self) -> &Arc<ComponentResidualOrder> {
        &self.order
    }
    pub fn restriction(&self) -> &Arc<LocalizedCoverHistory> {
        &self.restriction
    }
    pub fn companion(&self) -> &Arc<MarkedIdeal> {
        &self.companion
    }
    pub fn contact(&self) -> &Arc<ProducedContactCover> {
        &self.contact
    }
}
#[derive(Clone, Debug)]
pub enum CompanionOpenProduction {
    Contact(Box<CompanionContactOpen>),
    Monomial {
        order: Arc<ComponentResidualOrder>,
        restriction: Arc<LocalizedCoverHistory>,
        witness: MonomialWitness,
        checked: MonomialProduction,
    },
    EmptyOpen {
        order: Arc<ComponentResidualOrder>,
        evidence: CoverLocalizationProduction,
    },
    BelowMaximum {
        order: Arc<ComponentResidualOrder>,
        restriction: Arc<LocalizedCoverHistory>,
        companion: Arc<MarkedIdeal>,
        evidence: OrderProduction,
    },
    Incomplete {
        order: Arc<ComponentResidualOrder>,
        index: usize,
        reason: &'static str,
        progress: Box<CompanionOpenProgress>,
    },
}
/// Opens cover only the source marked cosupport. BelowMaximum and EmptyOpen
/// outcomes are geometric invariant-stage receipts, never integral zeroes.
pub fn produce_companion_open(
    order: Arc<ComponentResidualOrder>,
    index: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<CompanionOpenProduction> {
    let mut progress = CompanionOpenProgress::default();
    let result = run(&order, index, namespace, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(v) => Ok(v),
        Err(Error::ResourceIncomplete(reason)) => Ok(CompanionOpenProduction::Incomplete {
            order,
            index,
            reason,
            progress: Box::new(progress),
        }),
        Err(e) => Err(e),
    }
}
fn run(
    order: &Arc<ComponentResidualOrder>,
    index: usize,
    namespace: &str,
    b: &mut Budget,
    progress: &mut CompanionOpenProgress,
) -> Result<CompanionOpenProduction> {
    b.reserve_slots(order.upper_order_cover().opens().len())?;
    b.reserve_slots(order.upper_order_cover().support().generators().len())?;
    let cover = Arc::new(order.upper_order_cover().clone());
    let restriction = match localize_verified_cover_open(
        order.factor().data().history().clone(),
        cover,
        index,
        namespace,
        b,
    )? {
        empty @ CoverLocalizationProduction::Empty { .. } => {
            return Ok(CompanionOpenProduction::EmptyOpen {
                order: order.clone(),
                evidence: empty,
            });
        }
        incomplete @ CoverLocalizationProduction::Incomplete { .. } => {
            let reason = if let CoverLocalizationProduction::Incomplete { reason, .. } = &incomplete
            {
                *reason
            } else {
                unreachable!()
            };
            progress.localization = Some(incomplete);
            return Err(Error::ResourceIncomplete(reason));
        }
        CoverLocalizationProduction::Complete(v) => Arc::new(*v),
    };
    progress.restriction = Some(restriction.clone());
    let local = restriction.history().ledger().frame().local();
    let extension = restriction.open().extension();
    let Some(companion) = order.companion_arithmetic() else {
        // Pull the individually associated generators/quotients before Ideal
        // canonicalization. Unit divisors are absorbed in the same local ring.
        let data = order.factor().data();
        b.reserve_slots(
            data.originals()
                .len()
                .checked_mul(2)
                .ok_or(Error::ResourceIncomplete("monomial source association"))?,
        )?;
        let original = data
            .originals()
            .iter()
            .map(|f| extension.pull(f, b))
            .collect::<Result<Vec<_>>>()?;
        let mut quotients = data
            .quotients()
            .iter()
            .map(|q| extension.pull(q, b))
            .collect::<Result<Vec<_>>>()?;
        b.reserve_slots(data.powers().len())?;
        let mut powers = data.powers().to_vec();
        for (i, d) in restriction.history().ledger().divisors().iter().enumerate() {
            let singleton = restriction
                .history()
                .ledger()
                .intersection(&[i])
                .ok_or(Error::Invalid("monomial singleton receipt"))?;
            if singleton.empty && powers[i] > 0 {
                let factor = b.power(&d.equation, powers[i])?;
                for q in &mut quotients {
                    *q = b.mul(q, &factor)?;
                }
                powers[i] = 0;
            }
        }
        let source = Arc::new(MarkedIdeal::new(
            Ideal::new(local.ring().clone(), original.clone(), b)?,
            data.source().mark(),
            b,
        )?);
        b.reserve_slots(source.ideal().generators().len())?;
        let associated = source
            .ideal()
            .generators()
            .iter()
            .map(|f| {
                original
                    .iter()
                    .position(|g| g == f)
                    .map(|i| quotients[i].clone())
                    .ok_or(Error::Invalid("monomial canonical quotient association"))
            })
            .collect::<Result<Vec<_>>>()?;
        let witness = MonomialWitness {
            source,
            powers,
            quotients: associated,
        };
        let checked =
            produce_monomial_center(restriction.history().ledger().clone(), witness.clone(), b)?;
        if let MonomialProduction::Incomplete { reason, .. } = &checked {
            let reason = *reason;
            progress.monomial = Some(checked);
            return Err(Error::ResourceIncomplete(reason));
        }
        return Ok(CompanionOpenProduction::Monomial {
            order: order.clone(),
            restriction,
            witness,
            checked,
        });
    };
    let companion = Arc::new(MarkedIdeal::new(
        extension.ideal(companion.ideal(), b)?,
        companion.mark(),
        b,
    )?);
    let result = produce_ordinary_contact_cover(
        restriction.history().ledger().frame().clone(),
        Arc::new(companion.ideal().clone()),
        b,
    )?;
    match result {
        OrderProduction::ContactCover(contact) => {
            if contact.algebraic_maximum_order() > companion.mark() {
                return Err(Error::Invalid("companion upper order bound contradicted"));
            }
            if contact.algebraic_maximum_order() < companion.mark() {
                return Ok(CompanionOpenProduction::BelowMaximum {
                    order: order.clone(),
                    restriction,
                    companion,
                    evidence: OrderProduction::ContactCover(contact),
                });
            }
            Ok(CompanionOpenProduction::Contact(Box::new(
                CompanionContactOpen {
                    order: order.clone(),
                    restriction,
                    companion,
                    contact: Arc::new(contact),
                },
            )))
        }
        evidence @ OrderProduction::UnitIdeal { .. } => Ok(CompanionOpenProduction::BelowMaximum {
            order: order.clone(),
            restriction,
            companion,
            evidence,
        }),
        evidence @ OrderProduction::Incomplete { .. } => {
            let reason = if let OrderProduction::Incomplete { reason, .. } = &evidence {
                *reason
            } else {
                unreachable!()
            };
            progress.contact = Some(evidence);
            Err(Error::ResourceIncomplete(reason))
        }
        evidence @ OrderProduction::TerminalParameterLocus { .. } => {
            progress.contact = Some(evidence);
            Err(Error::ResourceIncomplete(
                "companion contact needs parameter-stratum handling",
            ))
        }
        OrderProduction::ZeroIdeal { .. } => Err(Error::Invalid(
            "positive companion became zero under proved finite order bound",
        )),
    }
}
