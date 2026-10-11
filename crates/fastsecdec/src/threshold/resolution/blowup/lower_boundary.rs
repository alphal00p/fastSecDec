//! Shared checked boundary transport on an issued embedded support.
//! Constructors are private to resolution orchestration; no arbitrary history
//! or source supplied by a library caller acquires transition authority here.
use super::super::*;
use super::{
    coefficient::{SupportedMarkedTransform, transform_supported},
    helpers::{Result, unit},
    support::StrictContactSupport,
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(super) struct LowerBoundaryTransform {
    pub prior: Arc<ResolutionHistory>,
    pub chart: Arc<RelativeRecursiveChart>,
    pub support: Arc<StrictContactSupport>,
    pub center: Arc<SupportedMarkedTransform>,
    pub normals: Vec<Poly>,
    pub divisors: Vec<DivisorReceipt>,
    pub ledger: Arc<VerifiedRelativeSnc>,
    pub born: Option<(BoundaryId, u64)>,
}
impl LowerBoundaryTransform {
    pub fn prepare(
        prior: Arc<ResolutionHistory>,
        chart: Arc<RelativeRecursiveChart>,
        support: Arc<StrictContactSupport>,
        center: Arc<SupportedMarkedTransform>,
        normals: Vec<Poly>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !Arc::ptr_eq(prior.ledger().frame(), support.embedding().frame())
            || !Arc::ptr_eq(support.geometry(), chart.geometry())
            || !Arc::ptr_eq(center.support(), &support)
            || !support
                .opens()
                .iter()
                .any(|o| Arc::ptr_eq(o, center.open()))
            || center.source().mark() != 1
            || center.exceptional_power() != usize::from(chart.geometry().exceptional().is_some())
        {
            return Err(Error::Invalid(
                "lower boundary actual source/geometry association",
            ));
        }
        let old = prior.ledger().frame().local();
        let lower = center.source().ideal();
        let pulled = support
            .embedding()
            .extension()
            .ideal(chart.geometry().center().ideal(), b)?;
        let represented = Ideal::new(old.ring().clone(), normals.clone(), b)?;
        for other in [&pulled, &represented] {
            for (a, z) in [(other, lower), (lower, other)] {
                let target = old.ideal().sum(z, b)?;
                for f in a.generators() {
                    if !target.contains(f, old.unit_relations(), b)? {
                        return Err(Error::Invalid(
                            "lower boundary full physical center restriction",
                        ));
                    }
                }
            }
        }
        let local = center.open().frame().local();
        if !local.ideal().sum(center.target().ideal(), b)?.contains(
            &local.ring().one(),
            local.unit_relations(),
            b,
        )? {
            return Err(Error::Invalid(
                "lower boundary center not exceptional times unit",
            ));
        }
        let born = chart
            .geometry()
            .exceptional()
            .map(|_| prior.next_transition())
            .transpose()?;
        let mut divisors = Vec::new();
        let mut active = Vec::new();
        b.reserve_slots(prior.ledger().divisors().len())?;
        for (di, div) in prior.ledger().divisors().iter().enumerate() {
            let contained =
                old.ideal()
                    .sum(lower, b)?
                    .contains(&div.equation, old.unit_relations(), b)?;
            let power = usize::from(born.is_some() && contained);
            let one = Arc::new(MarkedIdeal::new(
                Ideal::new(old.ring().clone(), vec![div.equation.clone()], b)?,
                1,
                b,
            )?);
            let transformed = transform_supported(
                support.clone(),
                center.open().clone(),
                one,
                power,
                &format!("{namespace}_old{di}"),
                b,
            )?;
            if transformed.generators().len() != 1 {
                return Err(Error::Invalid("lower strict divisor generator association"));
            }
            let g = &transformed.generators()[0];
            let absent = unit(local, &g.quotient, b)?;
            if !absent {
                active.push(InitialDivisor {
                    id: div.id,
                    equation: g.quotient.clone(),
                });
            }
            divisors.push(DivisorReceipt {
                id: div.id,
                total: g.total.clone(),
                exceptional_power: power,
                strict_equation: g.quotient.clone(),
                unit: local.ring().one(),
                absent,
            });
        }
        if let (Some(e), Some((id, _))) = (chart.geometry().exceptional(), born) {
            let equation = center.open().extension().pull(e, b)?;
            if !unit(local, &equation, b)? {
                active.push(InitialDivisor { id, equation });
            }
        }
        let ledger = match verify_initial_relative_snc(center.open().frame().clone(), active, b)? {
            SncProduction::Verified(l) => l,
            SncProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            SncProduction::Unresolved { .. } => {
                return Err(Error::ResourceIncomplete(
                    "lower boundary needs SNC refinement",
                ));
            }
        };
        Ok(Arc::new(Self {
            prior,
            chart,
            support,
            center,
            normals,
            divisors,
            ledger,
            born,
        }))
    }
}

pub(super) fn verify_marked_center(
    frame: &Arc<EtaleFrame>,
    source: &Arc<MarkedIdeal>,
    center: &Ideal,
    b: &mut Budget,
) -> Result<()> {
    let local = frame.local();
    if source.ideal().ring() != local.ring() || center.ring() != local.ring() {
        return Err(Error::Invalid("marked center source frame"));
    }
    let locus = local.ideal().sum(center, b)?;
    let mut layer = source.ideal().clone();
    for order in 0..source.mark() {
        for f in layer.generators() {
            if !locus.contains(f, local.unit_relations(), b)? {
                return Err(Error::Invalid("center outside retained marked source"));
            }
        }
        if order + 1 < source.mark() {
            layer = super::super::differential::relative_differential_ideal(frame, &layer, b)?;
        }
    }
    Ok(())
}
