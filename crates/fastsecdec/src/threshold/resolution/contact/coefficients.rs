use super::super::{Budget, Error, EtaleFrame, Ideal, MarkedIdeal};
use super::{ContactJet, restrict};
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
        budget.reserve_slots(1)?;
        completed.push(ContactJet {
            order: j,
            ambient: layer.clone(),
            restricted: term.clone(),
        });
        result = Some(match result {
            None => term,
            Some(previous) => previous.sum(&term, budget)?,
        });
        if j + 1 == d {
            break;
        }
        let axes = match kind {
            Differentiation::Normal(i) => vec![i],
            Differentiation::FullInclusive => (0..graph.free_axes().len()).collect(),
        };
        let inclusive = matches!(kind, Differentiation::FullInclusive);
        let factor = axes
            .len()
            .checked_add(usize::from(inclusive))
            .ok_or(Error::ResourceIncomplete("coefficient derivative axes"))?;
        budget.reserve_slots(
            layer
                .generators()
                .len()
                .checked_mul(factor)
                .ok_or(Error::ResourceIncomplete("coefficient derivative count"))?,
        )?;
        let mut next = if inclusive {
            layer.generators().to_vec()
        } else {
            Vec::new()
        };
        for p in layer.generators() {
            for i in &axes {
                next.push(graph.derivative(*i, p, budget)?);
            }
        }
        layer = Ideal::new(ring.clone(), next, budget)?;
    }
    result.ok_or(Error::Invalid("empty coefficient construction"))
}
