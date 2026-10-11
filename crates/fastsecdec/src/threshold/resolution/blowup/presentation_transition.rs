//! A single actual ambient morphism induces a checked lower-history step.
use super::super::*;
use super::{
    coefficient::*, embedding::SupportEmbedding, general::RecursiveCenterOrigin,
    general_transform::controlled_marked, helpers::unit, presentation::*, support::*,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct EmbeddedTransitionOpen {
    pub(crate) coefficient: Arc<SupportedMarkedTransform>,
    pub(crate) incidence_sum: Arc<SupportedMarkedTransform>,
    pub(crate) center: Arc<SupportedMarkedTransform>,
    pub(crate) divisors: Vec<DivisorReceipt>,
    pub(crate) ledger: Arc<VerifiedRelativeSnc>,
    pub(crate) born: Option<(BoundaryId, u64)>,
}
#[derive(Clone, Debug)]
pub struct EmbeddedTransition {
    pub(crate) source: Arc<EmbeddedPresentation>,
    pub(crate) chart: Arc<RelativeRecursiveChart>,
    pub(crate) support: Arc<StrictContactSupport>,
    pub(crate) companion: Arc<RelativeMarkedTransform>,
    pub(crate) lower_center: Ideal,
    pub(crate) lower_normals: Vec<Poly>,
    pub(crate) opens: Vec<Arc<EmbeddedTransitionOpen>>,
}
impl EmbeddedTransition {
    pub fn source(&self) -> &Arc<EmbeddedPresentation> {
        &self.source
    }
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn opens(&self) -> &[Arc<EmbeddedTransitionOpen>] {
        &self.opens
    }
    pub fn prepare(
        source: Arc<EmbeddedPresentation>,
        chart: Arc<RelativeRecursiveChart>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        let (lower_center, lower_normals) =
            match (chart.geometry().center().origin(), &source.origin) {
                (RecursiveCenterOrigin::CarriedMonomial(c), PresentationOrigin::Initial(i))
                    if Arc::ptr_eq(c.induced(), i) =>
                {
                    let lower = c.child();
                    let normals = lower
                        .indices()
                        .iter()
                        .map(|j| lower.ledger().divisors()[*j].equation.clone())
                        .collect();
                    (lower.center().clone(), normals)
                }
                (RecursiveCenterOrigin::EmbeddedMonomial(c), _)
                    if Arc::ptr_eq(c.presentation(), &source) =>
                {
                    let lower = c.child();
                    let normals = lower
                        .indices()
                        .iter()
                        .map(|j| lower.ledger().divisors()[*j].equation.clone())
                        .collect();
                    (lower.center().clone(), normals)
                }
                _ => {
                    return Err(Error::Invalid(
                        "repeated embedded transition actual center owner",
                    ));
                }
            };
        if !Arc::ptr_eq(chart.parent().source(), source.chart().parent().target())
            || !Arc::ptr_eq(
                chart.geometry().center().history(),
                source.chart().history(),
            )
        {
            return Err(Error::Invalid("repeated embedded transition parent owner"));
        }
        let embedding =
            SupportEmbedding::strict(source.support().clone(), source.open().clone(), b)?;
        let support = transport_embedding(
            chart.geometry().clone(),
            embedding,
            &format!("{namespace}_support"),
            b,
        )?;
        let companion = controlled_marked(
            chart.geometry().clone(),
            source.companion().target().clone(),
            &format!("{namespace}_G"),
            b,
        )?;
        let center_source = Arc::new(MarkedIdeal::new(lower_center.clone(), 1, b)?);
        let mut opens = Vec::new();
        b.reserve_slots(support.opens().len())?;
        for (i, open) in support.opens().iter().enumerate() {
            let c = controlled_supported(
                support.clone(),
                open.clone(),
                source.coefficient().clone(),
                &format!("{namespace}_C{i}"),
                b,
            )?;
            let j = controlled_supported(
                support.clone(),
                open.clone(),
                source.incidence_sum().clone(),
                &format!("{namespace}_J{i}"),
                b,
            )?;
            let center = controlled_supported(
                support.clone(),
                open.clone(),
                center_source.clone(),
                &format!("{namespace}_center{i}"),
                b,
            )?;
            let local = open.frame().local();
            if !local.ideal().sum(center.target().ideal(), b)?.contains(
                &local.ring().one(),
                local.unit_relations(),
                b,
            )? {
                return Err(Error::Invalid(
                    "complete lower center is not exceptional times unit",
                ));
            }
            let prior = source.history();
            let born = chart
                .geometry()
                .exceptional()
                .map(|_| prior.next_transition())
                .transpose()?;
            let mut divisors = Vec::new();
            let mut active = Vec::new();
            b.reserve_slots(prior.ledger().divisors().len())?;
            for (di, div) in prior.ledger().divisors().iter().enumerate() {
                let old = prior.ledger().frame().local();
                let contained = old.ideal().sum(&lower_center, b)?.contains(
                    &div.equation,
                    old.unit_relations(),
                    b,
                )?;
                let power = usize::from(born.is_some() && contained);
                let one = Arc::new(MarkedIdeal::new(
                    Ideal::new(old.ring().clone(), vec![div.equation.clone()], b)?,
                    1,
                    b,
                )?);
                let pulled = transform_supported(
                    support.clone(),
                    open.clone(),
                    one,
                    power,
                    &format!("{namespace}_old{i}_{di}"),
                    b,
                )?;
                if pulled.generators().len() != 1 {
                    return Err(Error::Invalid("strict divisor generator association"));
                }
                let g = &pulled.generators()[0];
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
                let e = open.extension().pull(e, b)?;
                if !unit(local, &e, b)? {
                    active.push(InitialDivisor { id, equation: e });
                }
            }
            let ledger = match verify_initial_relative_snc(open.frame().clone(), active, b)? {
                SncProduction::Verified(l) => l,
                SncProduction::Incomplete { reason, .. } => {
                    return Err(Error::ResourceIncomplete(reason));
                }
                SncProduction::Unresolved { .. } => {
                    return Err(Error::ResourceIncomplete(
                        "repeated lower boundary needs SNC refinement",
                    ));
                }
            };
            opens.push(Arc::new(EmbeddedTransitionOpen {
                coefficient: c,
                incidence_sum: j,
                center,
                divisors,
                ledger,
                born,
            }));
        }
        Ok(Arc::new(Self {
            source,
            chart,
            support,
            companion,
            lower_center,
            lower_normals,
            opens,
        }))
    }
    pub fn presentations(
        self: &Arc<Self>,
        b: &mut Budget,
    ) -> Result<Vec<Arc<EmbeddedPresentation>>> {
        let mut out = Vec::new();
        b.reserve_slots(self.opens.len())?;
        for (index, open) in self.opens.iter().enumerate() {
            let history = self
                .source
                .history()
                .advanced_embedded_presentation(self, open, b)?;
            out.push(Arc::new(EmbeddedPresentation {
                origin: PresentationOrigin::Transition {
                    owner: self.clone(),
                    open: index,
                },
                chart: self.chart.clone(),
                companion: self.companion.clone(),
                support: self.support.clone(),
                open: open.coefficient.open().clone(),
                coefficient: open.coefficient.target().clone(),
                incidence_sum: open.incidence_sum.target().clone(),
                history,
                old: self.source.old_ids().to_vec(),
                incidence: self.source.old_incidence(),
            }));
        }
        Ok(out)
    }
}
