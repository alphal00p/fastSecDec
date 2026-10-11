//! Source-bound supported J/G through one actual physical morphism.
use super::super::*;
use super::{
    coefficient::controlled_supported,
    embedding::SupportEmbedding,
    helpers::Result,
    lower_boundary::LowerBoundaryTransform,
    support::{StrictContactSupport, transport_embedding},
};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub enum SupportedProblemAncestor {
    Original(Arc<CarriedCompanionChart>),
    Incidence(Arc<EmbeddedTransition>),
}
#[derive(Clone, Debug)]
pub struct SupportedProblemOpen {
    source: Arc<SupportedMarkedTransform>,
    companion: Arc<SupportedMarkedTransform>,
    boundary: Arc<LowerBoundaryTransform>,
    history: Arc<ResolutionHistory>,
    source_cosupport: Arc<MarkedCosupport>,
    companion_cosupport: Arc<MarkedCosupport>,
}
impl SupportedProblemOpen {
    pub fn source(&self) -> &Arc<SupportedMarkedTransform> {
        &self.source
    }
    pub fn companion(&self) -> &Arc<SupportedMarkedTransform> {
        &self.companion
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn source_cosupport(&self) -> &Arc<MarkedCosupport> {
        &self.source_cosupport
    }
    pub fn companion_cosupport(&self) -> &Arc<MarkedCosupport> {
        &self.companion_cosupport
    }
}
#[derive(Clone, Debug)]
pub struct SupportedProblemChart {
    anchor: Arc<AnchoredOriginalTree>,
    prepared: Arc<PreparedProblemCenter>,
    chart: Arc<RelativeRecursiveChart>,
    support: Arc<StrictContactSupport>,
    ancestor: SupportedProblemAncestor,
    tree: Arc<OriginalTreePullback>,
    opens: Vec<Arc<SupportedProblemOpen>>,
}
impl SupportedProblemChart {
    pub fn prepare(
        anchor: Arc<AnchoredOriginalTree>,
        chart: Arc<RelativeRecursiveChart>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        let tree = anchor.pull_first(chart.clone(), &format!("{namespace}_tree"), b)?;
        // All original child roots/births are checked before a current J open is issued.
        let _ = tree.histories(b)?;
        let (prepared, embedding, ancestor) = match anchor.center() {
            OriginalAnchorCenter::Original(c) => (
                c.prepared(),
                c.embedding().clone(),
                SupportedProblemAncestor::Original(carry_companion_chart(
                    chart.clone(),
                    &format!("{namespace}_ancestor"),
                    b,
                )?),
            ),
            OriginalAnchorCenter::Continued(_) => {
                return Err(Error::Invalid(
                    "initial supported carry cannot reset a continued cycle",
                ));
            }
            OriginalAnchorCenter::Incidence(c) => {
                let p = c.presentation();
                let embedding = SupportEmbedding::strict(p.support().clone(), p.open().clone(), b)?
                    .restrict_unit_open(
                        c.prepared()
                            .center()
                            .coefficient()
                            .source()
                            .restriction()
                            .clone(),
                        b,
                    )?;
                let ancestor = EmbeddedTransition::prepare(
                    p.clone(),
                    chart.clone(),
                    &format!("{namespace}_ancestor"),
                    b,
                )?;
                (
                    c.prepared(),
                    embedding,
                    SupportedProblemAncestor::Incidence(ancestor),
                )
            }
        };
        let center = prepared.center();
        let restricted = center.coefficient().source();
        let prior = restricted.restriction().history();
        prepared.cycle().check_history(prior)?;
        if !Arc::ptr_eq(embedding.frame(), prior.ledger().frame())
            || !Arc::ptr_eq(embedding.frame(), center.frame())
        {
            return Err(Error::Invalid("supported problem actual source frame"));
        }
        let (support, opens) = pull_active(center, embedding, chart.clone(), namespace, b)?;
        let prepared = prepared.clone();
        Ok(Arc::new(Self {
            anchor,
            prepared,
            chart,
            support,
            ancestor,
            tree,
            opens,
        }))
    }
    pub fn ancestor_presentations(
        &self,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Vec<Arc<EmbeddedPresentation>>> {
        match (&self.ancestor, self.anchor.center()) {
            (SupportedProblemAncestor::Original(owner), OriginalAnchorCenter::Original(c)) => {
                let mut out = Vec::new();
                b.reserve_slots(owner.opens().len())?;
                for i in 0..owner.opens().len() {
                    let induced = induce_child_chart(
                        c.origin().clone(),
                        owner.clone(),
                        i,
                        &format!("{namespace}_{i}"),
                        b,
                    )?;
                    out.push(EmbeddedPresentation::from_induced(induced, b)?);
                }
                Ok(out)
            }
            (SupportedProblemAncestor::Incidence(owner), OriginalAnchorCenter::Incidence(_)) => {
                owner.presentations(b)
            }
            _ => Err(Error::Invalid("supported ancestor construction identity")),
        }
    }
    pub fn anchor(&self) -> &Arc<AnchoredOriginalTree> {
        &self.anchor
    }
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn ancestor(&self) -> &SupportedProblemAncestor {
        &self.ancestor
    }
    pub fn original_tree(&self) -> &Arc<OriginalTreePullback> {
        &self.tree
    }
    pub fn opens(&self) -> &[Arc<SupportedProblemOpen>] {
        &self.opens
    }
    pub fn prepared(&self) -> &Arc<PreparedProblemCenter> {
        &self.prepared
    }
    pub(crate) fn check_open(&self, open: &Arc<SupportedProblemOpen>) -> Result<()> {
        if !self.opens.iter().any(|o| Arc::ptr_eq(o, open))
            || !Arc::ptr_eq(open.source.support(), &self.support)
            || !Arc::ptr_eq(open.companion.support(), &self.support)
            || !Arc::ptr_eq(open.source.open(), open.companion.open())
            || !Arc::ptr_eq(
                open.source.source(),
                self.prepared().center().parent_source(),
            )
            || !Arc::ptr_eq(
                open.companion.source(),
                self.prepared().center().coefficient().source().companion(),
            )
            || !Arc::ptr_eq(&open.boundary.chart, &self.chart)
            || !Arc::ptr_eq(&open.boundary.support, &self.support)
            || !open.history.same_root(self.prepared().problem().history())
        {
            return Err(Error::Invalid("supported problem chart/open/role owner"));
        }
        Ok(())
    }
}

pub(super) fn pull_active(
    center: &Arc<CompanionCenter>,
    embedding: Arc<SupportEmbedding>,
    chart: Arc<RelativeRecursiveChart>,
    namespace: &str,
    b: &mut Budget,
) -> Result<(Arc<StrictContactSupport>, Vec<Arc<SupportedProblemOpen>>)> {
    let restricted = center.coefficient().source();
    let prior = restricted.restriction().history();
    if !Arc::ptr_eq(embedding.frame(), prior.ledger().frame())
        || !Arc::ptr_eq(embedding.frame(), center.frame())
    {
        return Err(Error::Invalid("active supported role source frame"));
    }
    let support = transport_embedding(
        chart.geometry().clone(),
        embedding,
        &format!("{namespace}_support"),
        b,
    )?;
    let center_source = Arc::new(MarkedIdeal::new(center.ideal().clone(), 1, b)?);
    let mut opens = Vec::new();
    b.reserve_slots(support.opens().len())?;
    for (i, open) in support.opens().iter().enumerate() {
        let source = controlled_supported(
            support.clone(),
            open.clone(),
            center.parent_source().clone(),
            &format!("{namespace}_J{i}"),
            b,
        )?;
        let companion = controlled_supported(
            support.clone(),
            open.clone(),
            restricted.companion().clone(),
            &format!("{namespace}_G{i}"),
            b,
        )?;
        let pulled_center = controlled_supported(
            support.clone(),
            open.clone(),
            center_source.clone(),
            &format!("{namespace}_center{i}"),
            b,
        )?;
        let boundary = LowerBoundaryTransform::prepare(
            prior.clone(),
            chart.clone(),
            support.clone(),
            pulled_center,
            center.normals().to_vec(),
            &format!("{namespace}_boundary{i}"),
            b,
        )?;
        let history = prior.advanced_lower_boundary(&boundary, b)?;
        let source_cosupport =
            MarkedCosupport::prove(open.frame().clone(), source.target().clone(), b)?;
        let companion_cosupport =
            MarkedCosupport::prove(open.frame().clone(), companion.target().clone(), b)?;
        opens.push(Arc::new(SupportedProblemOpen {
            source,
            companion,
            boundary,
            history,
            source_cosupport,
            companion_cosupport,
        }));
    }
    Ok((support, opens))
}
