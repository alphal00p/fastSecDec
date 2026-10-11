//! Original lower hierarchy anchored in the chosen physical ancestor BEFORE a pivot.
use super::*;
#[derive(Clone, Debug)]
pub enum OriginalAnchorCenter {
    Original(Arc<OriginalProblemCenter>),
    Incidence(Arc<EmbeddedProblemCenter>),
    Continued(Arc<ContinuedSupportedCenter>),
}
#[derive(Clone, Debug)]
pub struct AnchoredOriginalTree {
    center: OriginalAnchorCenter,
    original: Arc<OriginalRecursionTree>,
    embeddings: Vec<Arc<SupportEmbedding>>,
}
impl AnchoredOriginalTree {
    pub fn new(center: OriginalAnchorCenter, b: &mut Budget) -> Result<Arc<Self>> {
        let (original, mut embedding, physical_ideal) = match &center {
            OriginalAnchorCenter::Original(c) => (
                c.child_cycle().original_tree().clone(),
                c.embedding().clone(),
                c.ideal(),
            ),
            OriginalAnchorCenter::Continued(c) => (
                c.child_cycle().original_tree().clone(),
                c.embedding().clone(),
                c.ideal(),
            ),
            OriginalAnchorCenter::Incidence(c) => {
                let p = c.presentation();
                let base = SupportEmbedding::strict(p.support().clone(), p.open().clone(), b)?;
                if !Arc::ptr_eq(base.frame(), p.history().ledger().frame()) {
                    return Err(Error::Invalid("anchored hierarchy actual lower history"));
                }
                let embedding = base.restrict_unit_open(
                    c.prepared()
                        .center()
                        .coefficient()
                        .source()
                        .restriction()
                        .clone(),
                    b,
                )?;
                (
                    c.child_cycle().original_tree().clone(),
                    embedding,
                    c.ideal(),
                )
            }
        };
        if !Arc::ptr_eq(embedding.frame(), original.center().frame()) {
            return Err(Error::Invalid("anchored hierarchy companion frame"));
        }
        let mut contact = Some(original.center().coefficient().contact().clone());
        let mut embeddings = Vec::new();
        b.reserve_slots(original.levels().len())?;
        for level in original.levels() {
            let q = contact
                .take()
                .ok_or(Error::Invalid("anchored hierarchy missing contact level"))?;
            embedding = SupportEmbedding::nested(embedding, q, b)?;
            if !Arc::ptr_eq(embedding.frame(), level.history().ledger().frame())
                || !Arc::ptr_eq(embedding.frame(), level.center().frame())
            {
                return Err(Error::Invalid("anchored hierarchy original lower frame"));
            }
            let restricted = embedding.extension().ideal(physical_ideal, b)?;
            let local = embedding.frame().local();
            for (a, z) in [
                (&restricted, level.center().ideal()),
                (level.center().ideal(), &restricted),
            ] {
                let target = local.ideal().sum(z, b)?;
                for f in a.generators() {
                    if !target.contains(f, local.unit_relations(), b)? {
                        return Err(Error::Invalid(
                            "anchored hierarchy complete physical center restriction",
                        ));
                    }
                }
            }
            contact = level
                .center()
                .lift_receipt()
                .map(|r| r.level().contact().clone());
            embeddings.push(embedding.clone());
        }
        if contact.is_some() {
            return Err(Error::Invalid("anchored hierarchy omitted terminal level"));
        }
        Ok(Arc::new(Self {
            center,
            original,
            embeddings,
        }))
    }
    pub fn original(&self) -> &Arc<OriginalRecursionTree> {
        &self.original
    }
    pub fn center(&self) -> &OriginalAnchorCenter {
        &self.center
    }
    pub fn embeddings(&self) -> &[Arc<SupportEmbedding>] {
        &self.embeddings
    }
    pub fn pull_first(
        self: &Arc<Self>,
        chart: Arc<RelativeRecursiveChart>,
        namespace: &str,
        b: &mut Budget,
    ) -> Result<Arc<OriginalTreePullback>> {
        OriginalTreePullback::prepare_anchored(self.clone(), chart, namespace, b)
    }
    pub(super) fn check_chart(&self, chart: &RelativeRecursiveChart) -> Result<()> {
        match (&self.center, chart.geometry().center().origin()) {
            (OriginalAnchorCenter::Original(a), RecursiveCenterOrigin::OriginalProblem(c))
                if Arc::ptr_eq(a, c) =>
            {
                Ok(())
            }
            (OriginalAnchorCenter::Continued(a), RecursiveCenterOrigin::SupportedProblem(c))
                if Arc::ptr_eq(a, c) =>
            {
                Ok(())
            }
            (OriginalAnchorCenter::Incidence(a), RecursiveCenterOrigin::EmbeddedProblem(c))
                if Arc::ptr_eq(a, c) =>
            {
                Ok(())
            }
            _ => Err(Error::Invalid(
                "anchored hierarchy actual physical center owner",
            )),
        }
    }
}
