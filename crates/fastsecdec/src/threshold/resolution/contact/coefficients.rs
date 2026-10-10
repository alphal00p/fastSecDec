use super::super::differential::relative_differential_ideal;
use super::super::{
    Budget, Error, EtaleFrame, Ideal, MarkedIdeal, NormalizationOutcome, QuotientNormalizer,
    VerifiedQuotientNormalization,
};
use super::{ContactJet, restrict};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;
pub(crate) enum Differentiation {
    Normal(usize),
    FullInclusive,
}
/// All derivative lifts, including inverse-guard derivatives, are owned by the
/// already verified relative frame. Marks and mixed powers use native ideals.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build(
    graph: &EtaleFrame,
    contact: &EtaleFrame,
    initial: &Ideal,
    z: usize,
    d: usize,
    kind: Differentiation,
    completed: &mut Vec<ContactJet>,
    budget: &mut Budget,
) -> Result<MarkedIdeal> {
    let ring = graph.local().ring();
    let normalizer = QuotientNormalizer::prepare(contact.local().clone(), budget)?;
    let mut layer = initial.clone();
    let mut result: Option<MarkedIdeal> = None;
    for j in 0..d {
        budget.reserve_slots(layer.generators().len())?;
        let restricted = Ideal::new(
            ring.clone(),
            layer
                .generators()
                .iter()
                .map(|p| restrict(p, z, budget))
                .collect::<Result<Vec<_>>>()?,
            budget,
        )?;
        for p in restricted.generators() {
            contact.local().supports(p)?;
        }
        let term = MarkedIdeal::new(restricted, d - j, budget)?;
        let restricted_normalization = normalize(&normalizer, term.clone(), budget)?;
        let normalized_term = restricted_normalization.normalized().as_ref().clone();
        let combined = match result {
            None => normalized_term,
            Some(previous) => previous.sum(&normalized_term, budget)?,
        };
        let accumulated_normalization = normalize(&normalizer, combined, budget)?;
        result = Some(accumulated_normalization.normalized().as_ref().clone());
        budget.reserve_slots(1)?;
        completed.push(ContactJet {
            order: j,
            ambient: layer.clone(),
            restricted: term,
            restricted_normalization,
            accumulated_normalization,
        });
        if j + 1 == d {
            break;
        }
        layer = match kind {
            Differentiation::FullInclusive => relative_differential_ideal(graph, &layer, budget)?,
            Differentiation::Normal(index) => {
                budget.reserve_slots(layer.generators().len())?;
                let mut next = Vec::new();
                for p in layer.generators() {
                    next.push(graph.derivative(index, p, budget)?);
                }
                Ideal::new(ring.clone(), next, budget)?
            }
        };
    }
    result.ok_or(Error::Invalid("empty coefficient construction"))
}

fn normalize(
    normalizer: &Arc<QuotientNormalizer>,
    source: MarkedIdeal,
    budget: &mut Budget,
) -> Result<Arc<VerifiedQuotientNormalization>> {
    match normalizer.normalize(Arc::new(source), budget)? {
        NormalizationOutcome::Complete(proof) => Ok(Arc::new(*proof)),
        NormalizationOutcome::Incomplete { reason, .. } => Err(Error::ResourceIncomplete(reason)),
    }
}
