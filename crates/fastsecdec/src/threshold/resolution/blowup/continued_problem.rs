//! A fresh companion after a PROVED supported residual drop, retaining ancestry.
use super::super::*;
use super::{embedding::SupportEmbedding, helpers::Result};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct ContinuedSupportedCenter {
    drop: Arc<SupportedResidualDrop>,
    center: Arc<CompanionCenter>,
    embedding: Arc<SupportEmbedding>,
    child: Arc<EmbeddedChildCycle>,
    ancestors: Vec<Arc<EmbeddedPresentation>>,
    ideal: Ideal,
    normals: Vec<Poly>,
    clearings: Vec<UnitClearing>,
}
#[derive(Clone, Debug)]
pub enum ContinuedSupportedProduction {
    Center(Arc<ContinuedSupportedCenter>),
    NeedsPhysicalLocalization {
        drop: Arc<SupportedResidualDrop>,
        center: Arc<CompanionCenter>,
    },
}
impl ContinuedSupportedCenter {
    pub fn prepare(
        drop: Arc<SupportedResidualDrop>,
        center: Arc<CompanionCenter>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<ContinuedSupportedProduction> {
        let prior = drop.inventory().prior();
        let current = drop.inventory().open();
        prior.check_open(current)?;
        let Some(cycle) = center.coefficient().coefficient().cycle() else {
            return Err(Error::Invalid(
                "continued center missing residual-drop cycle",
            ));
        };
        if !matches!(cycle.origin(),CycleOrigin::AfterSupportedDrop(d) if Arc::ptr_eq(d,&drop))
            || !Arc::ptr_eq(center.coefficient().source().order(), drop.current())
        {
            return Err(Error::Invalid(
                "continued center actual drop/companion owner",
            ));
        }
        let restriction = center.coefficient().source().restriction();
        if !Arc::ptr_eq(restriction.open().source(), current.history()) {
            return Ok(ContinuedSupportedProduction::NeedsPhysicalLocalization { drop, center });
        }
        let base = SupportEmbedding::strict(
            current.source().support().clone(),
            current.source().open().clone(),
            b,
        )?;
        let embedding = match base.restrict_unit_open(restriction.clone(), b) {
            Ok(e) => e,
            Err(Error::ResourceIncomplete(
                "nested support requires actual ambient open"
                | "nested support requires ambient denominator open",
            )) => {
                return Ok(ContinuedSupportedProduction::NeedsPhysicalLocalization {
                    drop,
                    center,
                });
            }
            Err(e) => return Err(e),
        };
        let chart = prior.chart();
        if !super::helpers::unit(
            chart.geometry().frame().local(),
            current.source().open().relative_minor(),
            b,
        )? {
            return Ok(ContinuedSupportedProduction::NeedsPhysicalLocalization { drop, center });
        }
        let support_normals = current
            .source()
            .open()
            .chosen_equations()
            .iter()
            .map(|i| current.source().support().clearings()[*i].numerator.clone())
            .collect::<Vec<_>>();

        let (ideal, normals, clearings) = super::super::recursive::lift_embedded_geometry(
            chart.geometry().frame(),
            chart.parent().target(),
            center.frame(),
            embedding.extension(),
            support_normals.clone(),
            center.ideal(),
            center.normals(),
            b,
        )?;
        // Every retained ancestor remains an active constraint. Each original
        // C/J presentation will be transformed by the SAME physical chart.
        let ancestors = prior.ancestor_presentations(namespace, b)?;
        for ancestor in &ancestors {
            let _ = super::super::recursive::lift_embedded_geometry(
                chart.geometry().frame(),
                ancestor.companion().target(),
                center.frame(),
                embedding.extension(),
                support_normals.clone(),
                center.ideal(),
                center.normals(),
                b,
            )?;
            let source_embedding =
                SupportEmbedding::strict(ancestor.support().clone(), ancestor.open().clone(), b)?;
            let restricted = source_embedding.extension().ideal(&ideal, b)?;
            for source in [ancestor.coefficient(), ancestor.incidence_sum()] {
                super::lower_boundary::verify_marked_center(
                    source_embedding.frame(),
                    source,
                    &restricted,
                    b,
                )?;
            }
        }
        let child = EmbeddedChildCycle::new(center.clone(), b)?;
        Ok(ContinuedSupportedProduction::Center(Arc::new(Self {
            drop,
            center,
            embedding,
            child,
            ancestors,
            ideal,
            normals,
            clearings,
        })))
    }
    pub fn drop(&self) -> &Arc<SupportedResidualDrop> {
        &self.drop
    }
    pub fn center(&self) -> &Arc<CompanionCenter> {
        &self.center
    }
    pub fn embedding(&self) -> &Arc<SupportEmbedding> {
        &self.embedding
    }
    pub fn child_cycle(&self) -> &Arc<EmbeddedChildCycle> {
        &self.child
    }
    pub fn ancestors(&self) -> &[Arc<EmbeddedPresentation>] {
        &self.ancestors
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        self.drop.inventory().prior().chart().geometry().frame()
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        self.drop.inventory().prior().chart().parent().target()
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        self.drop.inventory().prior().chart().history()
    }
    pub fn ideal(&self) -> &Ideal {
        &self.ideal
    }
    pub fn normals(&self) -> &[Poly] {
        &self.normals
    }
    pub fn clearings(&self) -> &[UnitClearing] {
        &self.clearings
    }
    pub fn carry_ancestors(
        self: &Arc<Self>,
        chart: Arc<RelativeRecursiveChart>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Vec<Arc<EmbeddedTransition>>> {
        if !matches!(chart.geometry().center().origin(),RecursiveCenterOrigin::SupportedProblem(c) if Arc::ptr_eq(c,self))
        {
            return Err(Error::Invalid("continued ancestors actual physical center"));
        }
        let mut out = Vec::new();
        b.reserve_slots(self.ancestors.len())?;
        for (i, ancestor) in self.ancestors.iter().enumerate() {
            out.push(EmbeddedTransition::prepare(
                ancestor.clone(),
                chart.clone(),
                &format!("{namespace}_{i}"),
                b,
            )?);
        }
        Ok(out)
    }
}

#[derive(Clone, Debug)]
pub struct ContinuedSupportedChart {
    center: Arc<ContinuedSupportedCenter>,
    chart: Arc<RelativeRecursiveChart>,
    ancestors: Vec<Arc<EmbeddedTransition>>,
    child: Arc<OriginalTreePullback>,
    support: Arc<StrictContactSupport>,
    opens: Vec<Arc<SupportedProblemOpen>>,
}
impl ContinuedSupportedChart {
    pub fn prepare(
        center: Arc<ContinuedSupportedCenter>,
        chart: Arc<RelativeRecursiveChart>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        let ancestors =
            center.carry_ancestors(chart.clone(), &format!("{namespace}_ancestor"), b)?;
        // Issue every ancestor presentation now, checking full C/J and histories.
        for ancestor in &ancestors {
            let _ = ancestor.presentations(b)?;
        }
        let anchor = AnchoredOriginalTree::new(OriginalAnchorCenter::Continued(center.clone()), b)?;
        let child = anchor.pull_first(chart.clone(), &format!("{namespace}_child"), b)?;
        let _ = child.histories(b)?;
        let (support, opens) = super::supported_problem::pull_active(
            center.center(),
            center.embedding().clone(),
            chart.clone(),
            namespace,
            b,
        )?;
        Ok(Arc::new(Self {
            center,
            chart,
            ancestors,
            child,
            support,
            opens,
        }))
    }
    pub fn center(&self) -> &Arc<ContinuedSupportedCenter> {
        &self.center
    }
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn ancestors(&self) -> &[Arc<EmbeddedTransition>] {
        &self.ancestors
    }
    pub fn child(&self) -> &Arc<OriginalTreePullback> {
        &self.child
    }
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn opens(&self) -> &[Arc<SupportedProblemOpen>] {
        &self.opens
    }
}
