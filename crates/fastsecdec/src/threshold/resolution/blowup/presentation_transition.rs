//! A single actual ambient morphism induces a checked lower-history step.
use super::super::*;
use super::{
    coefficient::*, embedding::SupportEmbedding, general::RecursiveCenterOrigin,
    general_transform::controlled_marked, presentation::*, support::*,
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
    pub(super) boundary: Arc<super::lower_boundary::LowerBoundaryTransform>,
}
#[derive(Clone, Debug)]
pub struct EmbeddedTransition {
    pub(crate) source: Arc<EmbeddedPresentation>,
    pub(crate) chart: Arc<RelativeRecursiveChart>,
    pub(crate) support: Arc<StrictContactSupport>,
    pub(crate) companion: Arc<RelativeMarkedTransform>,
    pub(crate) lower_center: Ideal,
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
                (RecursiveCenterOrigin::EmbeddedProblem(c), _)
                    if Arc::ptr_eq(c.presentation(), &source) =>
                {
                    (c.lower_ideal().clone(), c.lower_normals().to_vec())
                }
                (RecursiveCenterOrigin::SupportedProblem(c), _)
                    if c.ancestors().iter().any(|p| Arc::ptr_eq(p, &source)) =>
                {
                    let embedding = SupportEmbedding::strict(
                        source.support().clone(),
                        source.open().clone(),
                        b,
                    )?;
                    let lower = embedding.extension().ideal(c.ideal(), b)?;
                    for marked in [source.coefficient(), source.incidence_sum()] {
                        super::lower_boundary::verify_marked_center(
                            embedding.frame(),
                            marked,
                            &lower,
                            b,
                        )?;
                    }
                    let normals = c
                        .normals()
                        .iter()
                        .map(|f| embedding.extension().pull(f, b))
                        .collect::<Result<Vec<_>>>()?;
                    (lower, normals)
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
            let boundary = super::lower_boundary::LowerBoundaryTransform::prepare(
                source.history().clone(),
                chart.clone(),
                support.clone(),
                center.clone(),
                lower_normals.clone(),
                &format!("{namespace}_boundary{i}"),
                b,
            )?;
            let divisors = boundary.divisors.clone();
            let ledger = boundary.ledger.clone();
            let born = boundary.born;
            opens.push(Arc::new(EmbeddedTransitionOpen {
                coefficient: c,
                incidence_sum: j,
                center,
                divisors,
                ledger,
                born,
                boundary,
            }));
        }
        Ok(Arc::new(Self {
            source,
            chart,
            support,
            companion,
            lower_center,
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
