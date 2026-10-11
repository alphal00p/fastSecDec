use super::super::matrix::combinations;
use super::super::{
    BoundaryId, Budget, ContactQuotient, Error, Ideal, MarkedIdeal, ResolutionHistory,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct OldIncidence {
    pub ids: Vec<BoundaryId>,
    pub empty: bool,
}
#[derive(Clone, Debug, Default)]
pub struct OldBoundaryProgress {
    pub incidence: Vec<OldIncidence>,
    pub boundary_factors_completed: usize,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct OldBoundaryCoefficient {
    contact: Arc<ContactQuotient>,
    history: Arc<ResolutionHistory>,
    cycle: Option<Arc<super::super::recursive::CycleSnapshot>>,
    active_old: Vec<BoundaryId>,
    maximum_count: usize,
    boundary: Option<MarkedIdeal>,
    coefficient: MarkedIdeal,
    progress: OldBoundaryProgress,
}
impl OldBoundaryCoefficient {
    pub fn contact(&self) -> &Arc<ContactQuotient> {
        &self.contact
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn cycle(&self) -> Option<&Arc<super::super::recursive::CycleSnapshot>> {
        self.cycle.as_ref()
    }
    pub fn active_old(&self) -> &[BoundaryId] {
        &self.active_old
    }
    pub fn algebraic_maximum_old_count(&self) -> usize {
        self.maximum_count
    }
    pub fn boundary_ideal(&self) -> Option<&MarkedIdeal> {
        self.boundary.as_ref()
    }
    pub fn coefficient(&self) -> &MarkedIdeal {
        &self.coefficient
    }
    pub fn progress(&self) -> &OldBoundaryProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum OldBoundaryProduction {
    Coefficient(Box<OldBoundaryCoefficient>),
    Incomplete {
        contact: Arc<ContactQuotient>,
        history: Arc<ResolutionHistory>,
        cycle: Option<Arc<super::super::recursive::CycleSnapshot>>,
        reason: &'static str,
        progress: OldBoundaryProgress,
    },
}

/// Local arithmetic of BM 2008 Eq. 5.2 from an actual ordinary maximal-contact
/// owner and fixed checked old-history snapshot. It is not a lifted centre,
/// overlap gluing proof, recursive resolver, or authority to reset history.
pub fn produce_old_boundary_coefficient(
    contact: Arc<ContactQuotient>,
    history: Arc<ResolutionHistory>,
    budget: &mut Budget,
) -> Result<OldBoundaryProduction> {
    produce(contact, history, None, budget)
}
pub(crate) fn produce_old_boundary_coefficient_for_cycle(
    contact: Arc<ContactQuotient>,
    history: Arc<ResolutionHistory>,
    cycle: Arc<super::super::recursive::CycleSnapshot>,
    budget: &mut Budget,
) -> Result<OldBoundaryProduction> {
    cycle.check_history(&history)?;
    produce(contact, history, Some(cycle), budget)
}
fn produce(
    contact: Arc<ContactQuotient>,
    history: Arc<ResolutionHistory>,
    cycle: Option<Arc<super::super::recursive::CycleSnapshot>>,
    budget: &mut Budget,
) -> Result<OldBoundaryProduction> {
    if !Arc::ptr_eq(contact.source().frame(), history.ledger().frame()) {
        return Err(Error::Invalid("old-boundary/contact source owner mismatch"));
    }
    let mut progress = OldBoundaryProgress::default();
    let ids = cycle
        .as_ref()
        .map_or_else(|| history.old_snapshot(), |c| c.old_ids());
    let result = run(&contact, &history, ids, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok((active_old, maximum_count, boundary, coefficient)) => Ok(
            OldBoundaryProduction::Coefficient(Box::new(OldBoundaryCoefficient {
                contact,
                history,
                cycle,
                active_old,
                maximum_count,
                boundary,
                coefficient,
                progress,
            })),
        ),
        Err(Error::ResourceIncomplete(reason)) => Ok(OldBoundaryProduction::Incomplete {
            contact,
            history,
            cycle,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
pub(crate) type Built = (Vec<BoundaryId>, usize, Option<MarkedIdeal>, MarkedIdeal);
fn run(
    contact: &ContactQuotient,
    history: &ResolutionHistory,
    old_ids: &[BoundaryId],
    budget: &mut Budget,
    progress: &mut OldBoundaryProgress,
) -> Result<Built> {
    let source = contact.source();
    let d = source.algebraic_maximum_order();
    let cosupport = &source
        .progress()
        .stages
        .get(
            d.checked_sub(1)
                .ok_or(Error::Invalid("zero contact order"))?,
        )
        .ok_or(Error::Invalid("missing checked contact cosupport"))?
        .ideal;
    combine_old_boundary(
        BoundarySumInput {
            frame: source.frame(),
            cosupport,
            history,
            old_ids,
            target: contact.contact(),
            extension: contact.extension(),
            coefficient: contact.differential_coefficient(),
        },
        budget,
        progress,
    )
}
pub(crate) struct BoundarySumInput<'a> {
    pub frame: &'a super::super::EtaleFrame,
    pub cosupport: &'a Ideal,
    pub history: &'a ResolutionHistory,
    pub old_ids: &'a [BoundaryId],
    pub target: &'a super::super::EtaleFrame,
    pub extension: &'a super::super::RingExtension,
    pub coefficient: &'a MarkedIdeal,
}
pub(crate) fn combine_old_boundary(
    input: BoundarySumInput<'_>,
    budget: &mut Budget,
    progress: &mut OldBoundaryProgress,
) -> Result<Built> {
    let BoundarySumInput {
        frame,
        cosupport,
        history,
        old_ids,
        target,
        extension,
        coefficient,
    } = input;
    let ledger = history.ledger();
    let local = frame.local();
    if !std::ptr::eq(frame, ledger.frame().as_ref())
        || cosupport.ring() != local.ring()
        || extension.source() != local.ring()
        || extension.target() != target.local().ring()
        || coefficient.ideal().ring() != target.local().ring()
    {
        return Err(Error::Invalid(
            "old boundary sum actual frame/embedding owner",
        ));
    }
    budget.reserve_slots(old_ids.len())?;
    let old = old_ids
        .iter()
        .filter_map(|id| ledger.divisors().iter().find(|h| h.id == *id))
        .collect::<Vec<_>>();
    let mut maximum = None;
    for count in (0..=old.len()).rev() {
        let mut proper = false;
        for subset in combinations(old.len(), count, budget)? {
            budget.reserve_slots(subset.len())?;
            let equations = Ideal::new(
                local.ring().clone(),
                subset.iter().map(|i| old[*i].equation.clone()).collect(),
                budget,
            )?;
            let total = local
                .ideal()
                .sum(cosupport, budget)?
                .sum(&equations, budget)?;
            let empty = total.contains(&local.ring().one(), local.unit_relations(), budget)?;
            budget.reserve_slots(1)?;
            progress.incidence.push(OldIncidence {
                ids: subset.iter().map(|i| old[*i].id).collect(),
                empty,
            });
            proper |= !empty;
        }
        if proper {
            maximum = Some(count);
            break;
        }
    }
    let maximum = maximum.ok_or(Error::Invalid("checked contact cosupport became empty"))?;
    let active_old = old.iter().map(|d| d.id).collect();
    if maximum == 0 {
        return Ok((active_old, maximum, None, coefficient.clone()));
    }
    let target = target.local();
    let mark = coefficient.mark();
    let mut product = Ideal::new(target.ring().clone(), vec![target.ring().one()], budget)?;
    for subset in combinations(old.len(), maximum, budget)? {
        budget.reserve_slots(subset.len())?;
        let mut terms = Vec::new();
        for index in subset {
            let equation = extension.pull(&old[index].equation, budget)?;
            target.supports(&equation)?;
            terms.push(budget.power(&equation, mark)?);
        }
        let sum = Ideal::new(target.ring().clone(), terms, budget)?;
        product = product.product(&sum, budget)?;
        progress.boundary_factors_completed = progress
            .boundary_factors_completed
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("old boundary factor count"))?;
    }
    let boundary = MarkedIdeal::new(product, mark, budget)?;
    let result = coefficient.sum(&boundary, budget)?;
    Ok((active_old, maximum, Some(boundary), result))
}
