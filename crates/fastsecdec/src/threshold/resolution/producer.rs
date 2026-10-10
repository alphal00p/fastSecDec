use super::differential::relative_differential_ideal;
use super::localized::Result;
use super::{Budget, Error, Ideal, Poly};
use super::{EtaleFrame, OpenCoverCertificate, VerifiedOpenCover};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DerivativeStage {
    pub order: usize,
    pub ideal: Arc<Ideal>,
    /// None means resource stop before the native unit-ideal check completed.
    pub is_unit: Option<bool>,
}
#[derive(Clone, Debug, Default)]
pub struct ProductionProgress {
    pub stages: Vec<DerivativeStage>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct ContactCandidate {
    pub equation: Poly,
    pub free_derivative_index: usize,
    pub differential: Poly,
}
#[derive(Clone, Debug)]
pub struct ProducedContactCover {
    frame: Arc<EtaleFrame>,
    source: Arc<Ideal>,
    order: usize,
    candidates: Vec<ContactCandidate>,
    cover: VerifiedOpenCover,
    progress: ProductionProgress,
}
#[derive(Clone, Debug)]
pub enum OrderProduction {
    UnitIdeal {
        frame: Arc<EtaleFrame>,
        source: Arc<Ideal>,
        progress: ProductionProgress,
    },
    /// Zero in the localized quotient; no finite maximum-order claim follows.
    ZeroIdeal {
        frame: Arc<EtaleFrame>,
        source: Arc<Ideal>,
        progress: ProductionProgress,
    },
    /// With no relative free axes, a proper nonzero ideal needs base-stratum
    /// handling. It is not a recursively resolvable integration direction.
    TerminalParameterLocus {
        frame: Arc<EtaleFrame>,
        source: Arc<Ideal>,
        progress: ProductionProgress,
    },
    ContactCover(ProducedContactCover),
    Incomplete {
        frame: Arc<EtaleFrame>,
        source: Arc<Ideal>,
        reason: &'static str,
        progress: ProductionProgress,
    },
}
impl ProducedContactCover {
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn source(&self) -> &Arc<Ideal> {
        &self.source
    }
    pub fn algebraic_maximum_order(&self) -> usize {
        self.order
    }
    pub fn candidates(&self) -> &[ContactCandidate] {
        &self.candidates
    }
    pub fn cover(&self) -> &VerifiedOpenCover {
        &self.cover
    }
    pub fn progress(&self) -> &ProductionProgress {
        &self.progress
    }
}

/// Construct an ordinary relative maximum-order/contact-open stage. No boundary
/// ledger or logarithmic companion stage is inferred from this input. All
/// derivative ideals are inclusive: D^(j+1)I contains D^j I and its relative
/// first derivatives. This is an algebraic maximum, not a real-fiber maximum.
pub fn produce_ordinary_contact_cover(
    frame: Arc<EtaleFrame>,
    source: Arc<Ideal>,
    budget: &mut Budget,
) -> Result<OrderProduction> {
    if source.ring() != frame.local().ring() {
        return Err(Error::Invalid("producer ideal/frame ring mismatch"));
    }
    for f in source.generators() {
        frame.local().supports(f)?;
    }
    let mut progress = ProductionProgress::default();
    let produced = produce(&frame, &source, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match produced {
        Ok(Produced::Unit) => Ok(OrderProduction::UnitIdeal {
            frame,
            source,
            progress,
        }),
        Ok(Produced::Zero) => Ok(OrderProduction::ZeroIdeal {
            frame,
            source,
            progress,
        }),
        Ok(Produced::Terminal) => Ok(OrderProduction::TerminalParameterLocus {
            frame,
            source,
            progress,
        }),
        Ok(Produced::Contact(order, candidates, cover)) => {
            Ok(OrderProduction::ContactCover(ProducedContactCover {
                frame,
                source,
                order,
                candidates,
                cover,
                progress,
            }))
        }
        Err(Error::ResourceIncomplete(reason)) => Ok(OrderProduction::Incomplete {
            frame,
            source,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
enum Produced {
    Unit,
    Zero,
    Terminal,
    Contact(usize, Vec<ContactCandidate>, VerifiedOpenCover),
}
fn produce(
    frame: &Arc<EtaleFrame>,
    source: &Arc<Ideal>,
    budget: &mut Budget,
    progress: &mut ProductionProgress,
) -> Result<Produced> {
    budget.reserve_slots(1)?;
    for f in source.generators() {
        budget.poly(f)?;
    }
    progress.stages.push(DerivativeStage {
        order: 0,
        ideal: source.clone(),
        is_unit: None,
    });
    for order in 0..=budget.limits.max_mark {
        let current = progress.stages.last().expect("initial stage").ideal.clone();
        let combined = frame.local().ideal().sum(&current, budget)?;
        let unit = combined.contains(
            &frame.local().ring().one(),
            frame.local().unit_relations(),
            budget,
        )?;
        progress.stages.last_mut().expect("current stage").is_unit = Some(unit);
        if unit {
            if order == 0 {
                return Ok(Produced::Unit);
            }
            let previous = &progress.stages[order - 1];
            if previous.is_unit != Some(false) {
                return Err(Error::Invalid("missing proper previous derivative ideal"));
            }
            let count = previous
                .ideal
                .generators()
                .len()
                .checked_mul(frame.free_axes().len())
                .ok_or(Error::ResourceIncomplete("contact candidate count"))?;
            budget.reserve_slots(count)?;
            let mut candidates = Vec::new();
            for h in previous.ideal.generators() {
                for index in 0..frame.free_axes().len() {
                    let differential = frame.derivative(index, h, budget)?;
                    if frame.local().zero(&differential, budget)? {
                        continue;
                    }
                    candidates.push(ContactCandidate {
                        equation: h.clone(),
                        free_derivative_index: index,
                        differential,
                    });
                }
            }
            budget.reserve_slots(candidates.len())?;
            let cover = OpenCoverCertificate {
                algebra: frame.local().clone(),
                support: (*previous.ideal).clone(),
                opens: candidates.iter().map(|c| c.differential.clone()).collect(),
            }
            .verify(budget)?;
            if cover.algebraic_locus_empty() {
                return Err(Error::Invalid(
                    "proper derivative ideal contradicted by empty cosupport",
                ));
            }
            return Ok(Produced::Contact(order, candidates, cover));
        }
        if order == 0 {
            let mut zero = true;
            for f in current.generators() {
                if !frame.local().zero(f, budget)? {
                    zero = false;
                    break;
                }
            }
            if zero {
                return Ok(Produced::Zero);
            }
            if frame.free_axes().is_empty() {
                return Ok(Produced::Terminal);
            }
        }
        if order == budget.limits.max_mark {
            break;
        }
        let next = Arc::new(relative_differential_ideal(frame, &current, budget)?);
        budget.reserve_slots(1)?;
        progress.stages.push(DerivativeStage {
            order: order + 1,
            ideal: next,
            is_unit: None,
        });
    }
    Err(Error::ResourceIncomplete(
        "maximum derivative order reached with unresolved algebraic locus",
    ))
}
