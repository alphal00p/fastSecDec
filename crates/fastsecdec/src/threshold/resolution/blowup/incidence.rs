//! Child completion and the distinct old-incidence reset event (BM §5 I.B).
//! The saved coefficient, history root and fixed old snapshot survive unchanged.
use super::super::*;
use super::{presentation::*, presentation_transition::*};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct MarkedCosupport {
    frame: Arc<EtaleFrame>,
    source: Arc<MarkedIdeal>,
    layers: Vec<Ideal>,
    locus: Arc<Ideal>,
    empty: bool,
}
impl MarkedCosupport {
    pub(crate) fn prove(
        frame: Arc<EtaleFrame>,
        source: Arc<MarkedIdeal>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if source.ideal().ring() != frame.local().ring() {
            return Err(Error::Invalid("marked cosupport source frame"));
        }
        b.reserve_slots(source.mark())?;
        let mut layers = vec![source.ideal().clone()];
        for _ in 1..source.mark() {
            let next = super::super::differential::relative_differential_ideal(
                &frame,
                layers.last().unwrap(),
                b,
            )?;
            layers.push(next);
        }
        let local = frame.local();
        let locus = Arc::new(local.ideal().sum(layers.last().unwrap(), b)?);
        let empty = locus.contains(&local.ring().one(), local.unit_relations(), b)?;
        Ok(Arc::new(Self {
            frame,
            source,
            layers,
            locus,
            empty,
        }))
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn layers(&self) -> &[Ideal] {
        &self.layers
    }
    pub fn locus(&self) -> &Arc<Ideal> {
        &self.locus
    }
    pub fn empty(&self) -> bool {
        self.empty
    }
}
#[derive(Clone, Debug)]
pub struct CompletedEmbeddedChild {
    transition: Arc<EmbeddedTransition>,
    proofs: Vec<Arc<MarkedCosupport>>,
}
impl CompletedEmbeddedChild {
    pub fn prove(transition: Arc<EmbeddedTransition>, b: &mut Budget) -> Result<Option<Arc<Self>>> {
        let mut proofs = Vec::new();
        b.reserve_slots(transition.opens().len())?;
        for open in transition.opens() {
            let proof = MarkedCosupport::prove(
                open.incidence_sum.open().frame().clone(),
                open.incidence_sum.target().clone(),
                b,
            )?;
            if !proof.empty() {
                return Ok(None);
            }
            proofs.push(proof);
        }
        Ok(Some(Arc::new(Self { transition, proofs })))
    }
    pub fn transition(&self) -> &Arc<EmbeddedTransition> {
        &self.transition
    }
    pub fn proofs(&self) -> &[Arc<MarkedCosupport>] {
        &self.proofs
    }
}
#[derive(Clone, Debug)]
pub struct IncidenceDrop {
    source: Arc<EmbeddedPresentation>,
    completion: Arc<CompletedEmbeddedChild>,
    companion_cosupport: Arc<MarkedCosupport>,
    boundary: Option<MarkedIdeal>,
    target: Arc<MarkedIdeal>,
    maximum: usize,
    progress: OldBoundaryProgress,
}
#[derive(Clone, Debug)]
pub enum IncidenceContinuation {
    Dropped(Arc<IncidenceDrop>),
    CompanionResolved {
        source: Arc<EmbeddedPresentation>,
        completion: Arc<CompletedEmbeddedChild>,
        proof: Arc<MarkedCosupport>,
    },
}
impl IncidenceDrop {
    pub fn source(&self) -> &Arc<EmbeddedPresentation> {
        &self.source
    }
    pub fn completion(&self) -> &Arc<CompletedEmbeddedChild> {
        &self.completion
    }
    pub fn companion_cosupport(&self) -> &Arc<MarkedCosupport> {
        &self.companion_cosupport
    }
    pub fn boundary(&self) -> Option<&MarkedIdeal> {
        self.boundary.as_ref()
    }
    pub fn target(&self) -> &Arc<MarkedIdeal> {
        &self.target
    }
    pub fn maximum(&self) -> usize {
        self.maximum
    }
    pub fn progress(&self) -> &OldBoundaryProgress {
        &self.progress
    }
    pub fn prepare(
        source: Arc<EmbeddedPresentation>,
        completion: Arc<CompletedEmbeddedChild>,
        b: &mut Budget,
    ) -> Result<IncidenceContinuation> {
        let Some((transition, index)) = source.previous_transition() else {
            return Err(Error::Invalid(
                "incidence drop requires actual carried transition",
            ));
        };
        if !Arc::ptr_eq(transition, completion.transition())
            || completion.proofs.len() != transition.opens().len()
            || !completion.proofs.get(index).is_some_and(|p| {
                Arc::ptr_eq(p.source(), source.incidence_sum())
                    && Arc::ptr_eq(p.frame(), source.open().frame())
                    && p.empty()
            })
        {
            return Err(Error::Invalid(
                "incidence completion current source/coverage owner",
            ));
        }
        let proof = MarkedCosupport::prove(
            source.chart().geometry().frame().clone(),
            source.companion().target().clone(),
            b,
        )?;
        if proof.empty() {
            return Ok(IncidenceContinuation::CompanionResolved {
                source,
                completion,
                proof,
            });
        }
        let mut progress = OldBoundaryProgress::default();
        let (_, maximum, boundary, target) = super::super::companion::combine_old_boundary(
            super::super::companion::BoundarySumInput {
                frame: source.chart().geometry().frame(),
                cosupport: proof.locus(),
                history: source.chart().history(),
                old_ids: source.old_ids(),
                target: source.open().frame(),
                extension: source.open().extension(),
                coefficient: source.coefficient(),
            },
            b,
            &mut progress,
        )?;
        if maximum >= source.old_incidence() {
            return Err(Error::Invalid(
                "resolved lower child did not prove a strict old-incidence drop",
            ));
        }
        progress.operations = b.operations();
        progress.ideal_slots = b.ideal_slots();
        Ok(IncidenceContinuation::Dropped(Arc::new(Self {
            source,
            completion,
            companion_cosupport: proof,
            boundary,
            target: Arc::new(target),
            maximum,
            progress,
        })))
    }
    pub fn presentation(self: &Arc<Self>) -> Arc<EmbeddedPresentation> {
        let source = &self.source;
        Arc::new(EmbeddedPresentation {
            origin: PresentationOrigin::Incidence(self.clone()),
            chart: source.chart.clone(),
            companion: source.companion.clone(),
            support: source.support.clone(),
            open: source.open.clone(),
            coefficient: source.coefficient.clone(),
            incidence_sum: self.target.clone(),
            history: source.history.clone(),
            old: source.old.clone(),
            incidence: self.maximum,
        })
    }
}
