//! Checked component principal opens. This is localization with unit coordinate
//! Jacobian; inverse presentation slots are not integration coordinates.
use super::{
    Budget, Error, EtaleCertificate, EtaleFrame, Guard, InitialDivisor, LocalizedAlgebra, Poly,
    RegularOrigin, ResolutionHistory, RingExtension, SncProduction, SncProgress,
    VerifiedComponentSplit, VerifiedRelativeSnc, verify_initial_relative_snc,
};
use std::sync::Arc;
use symbolica::symbol;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Default)]
pub struct LocalizationProgress {
    pub stage: &'static str,
    pub frame: Option<Arc<EtaleFrame>>,
    pub snc: Option<SncProgress>,
    pub old_coordinate_derivatives_checked: usize,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct VerifiedPrincipalOpen {
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    factor: Poly,
    extension: RingExtension,
    ledger: Arc<VerifiedRelativeSnc>,
}
impl VerifiedPrincipalOpen {
    pub fn source(&self) -> &Arc<ResolutionHistory> {
        &self.source
    }
    pub fn split(&self) -> &Arc<VerifiedComponentSplit> {
        &self.split
    }
    pub fn side(&self) -> usize {
        self.side
    }
    pub fn factor(&self) -> &Poly {
        &self.factor
    }
    pub fn extension(&self) -> &RingExtension {
        &self.extension
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    /// Identity on the retained relative free coordinates. This is only this
    /// localization's Jacobian; previous projective/blowup maps remain separate.
    pub fn coordinate_jacobian(&self) -> Poly {
        self.ledger.frame().local().ring().one()
    }
}
#[derive(Clone, Debug)]
pub struct LocalizedHistory {
    open: Arc<VerifiedPrincipalOpen>,
    history: Arc<ResolutionHistory>,
    progress: LocalizationProgress,
}
impl LocalizedHistory {
    pub fn open(&self) -> &Arc<VerifiedPrincipalOpen> {
        &self.open
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn progress(&self) -> &LocalizationProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum LocalizationProduction {
    Complete(Box<LocalizedHistory>),
    Empty {
        source: Arc<ResolutionHistory>,
        split: Arc<VerifiedComponentSplit>,
        side: usize,
        factor: Poly,
        progress: LocalizationProgress,
    },
    Incomplete {
        source: Arc<ResolutionHistory>,
        split: Arc<VerifiedComponentSplit>,
        side: usize,
        reason: &'static str,
        progress: LocalizationProgress,
    },
}
pub fn localize_component_open(
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    namespace: &str,
    budget: &mut Budget,
) -> Result<LocalizationProduction> {
    if side >= 2 {
        return Err(Error::Invalid("component open side"));
    }
    let source_frame = source.ledger().frame();
    match split.owner().origin() {
        RegularOrigin::Ambient(frame) if !Arc::ptr_eq(frame, source_frame) => {
            return Err(Error::Invalid("ambient split frame owner"));
        }
        RegularOrigin::Boundary { ledger, .. } if !Arc::ptr_eq(ledger, source.ledger()) => {
            return Err(Error::Invalid("boundary split ledger owner"));
        }
        _ => {}
    }
    if !Arc::ptr_eq(split.owner().ambient_local(), source_frame.local()) {
        return Err(Error::Invalid("component open original localization"));
    }
    let mut progress = LocalizationProgress::default();
    let result = build(
        source.clone(),
        split.clone(),
        side,
        namespace,
        budget,
        &mut progress,
    );
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(Some(open)) => {
            let open = Arc::new(open);
            match source.restricted(&open, budget) {
                Ok(history) => {
                    progress.stage = "complete";
                    progress.operations = budget.operations();
                    progress.ideal_slots = budget.ideal_slots();
                    Ok(LocalizationProduction::Complete(Box::new(
                        LocalizedHistory {
                            open,
                            history,
                            progress,
                        },
                    )))
                }
                Err(Error::ResourceIncomplete(reason)) => {
                    progress.operations = budget.operations();
                    progress.ideal_slots = budget.ideal_slots();
                    Ok(LocalizationProduction::Incomplete {
                        source,
                        split,
                        side,
                        reason,
                        progress,
                    })
                }
                Err(e) => Err(e),
            }
        }
        Ok(None) => Ok(LocalizationProduction::Empty {
            factor: split.opens()[side].clearing().numerator.clone(),
            source,
            split,
            side,
            progress,
        }),
        Err(Error::ResourceIncomplete(reason)) => Ok(LocalizationProduction::Incomplete {
            source,
            split,
            side,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn build(
    source: Arc<ResolutionHistory>,
    split: Arc<VerifiedComponentSplit>,
    side: usize,
    namespace: &str,
    budget: &mut Budget,
    progress: &mut LocalizationProgress,
) -> Result<Option<VerifiedPrincipalOpen>> {
    let old = source.ledger().frame();
    let factor = split.opens()[side].clearing().numerator.clone();
    progress.stage = "original principal open emptiness";
    if old.local().zero(&factor, budget)? {
        return Ok(None);
    }
    progress.stage = "principal open ring extension";
    let extension = RingExtension::new(
        old.local().ring().clone(),
        [
            symbol!(format!("{namespace}::open_inverse")),
            symbol!(format!("{namespace}::minor_inverse")),
        ],
        budget,
    )?;
    let start = extension.source().len();
    let relations = extension.ideal(old.local().ideal(), budget)?;
    let mut guards = extension.guards(old.local().guards(), budget)?;
    budget.reserve_slots(1)?;
    guards.push(Guard {
        factor: extension.pull(&factor, budget)?,
        inverse_axis: start,
    });
    budget.reserve_slots(old.selected_equations().len())?;
    let selected = old
        .selected_equations()
        .iter()
        .map(|index| extension.pull(&old.source().ideal().generators()[*index], budget))
        .collect::<Result<Vec<_>>>()?;
    let indices = selected
        .iter()
        .map(|p| {
            relations
                .generators()
                .iter()
                .position(|q| q == p)
                .ok_or(Error::Invalid("localized selected equation disappeared"))
        })
        .collect::<Result<Vec<_>>>()?;
    budget.reserve_slots(old.local().axes().len())?;
    let local = LocalizedAlgebra::new(relations, old.local().axes().to_vec(), guards, budget)?;
    progress.stage = "native localized relative frame";
    budget.reserve_slots(old.dependent_axes().len())?;
    budget.reserve_slots(old.free_axes().len())?;
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: indices,
            dependent_axes: old.dependent_axes().to_vec(),
            free_axes: old.free_axes().to_vec(),
            determinant_inverse_axis: if selected.is_empty() {
                None
            } else {
                Some(start + 1)
            },
        }
        .verify(budget)?,
    );
    progress.frame = Some(frame.clone());
    let determinant = extension.pull(old.determinant(), budget)?;
    if !frame
        .local()
        .zero(&(frame.determinant() - &determinant), budget)?
    {
        return Err(Error::Invalid("principal open determinant changed"));
    }
    progress.stage = "identity on old coordinates and derivatives";
    let checks = old
        .free_axes()
        .len()
        .checked_mul(old.local().ring().len())
        .ok_or(Error::ResourceIncomplete(
            "localized derivative identity count",
        ))?;
    budget.count(checks)?;
    for j in 0..old.free_axes().len() {
        for i in 0..old.local().ring().len() {
            if !old.local().ring().is_parameter(i)
                && !old.local().axes().contains(&i)
                && !old.local().guards().iter().any(|g| g.inverse_axis == i)
            {
                continue;
            }
            let coordinate = old.local().ring().coordinate(i)?;
            let old_derivative = old.derivative(j, &coordinate, budget)?;
            let expected = extension.pull(&old_derivative, budget)?;
            let pulled = extension.pull(&coordinate, budget)?;
            let actual = frame.derivative(j, &pulled, budget)?;
            if !frame.local().zero(&(&actual - &expected), budget)? {
                return Err(Error::Invalid("principal open changed relative derivative"));
            }
            progress.old_coordinate_derivatives_checked = progress
                .old_coordinate_derivatives_checked
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("localization derivative counter"))?;
        }
    }
    progress.stage = "pulled original boundary SNC";
    budget.reserve_slots(source.ledger().divisors().len())?;
    let divisors = source
        .ledger()
        .divisors()
        .iter()
        .map(|d| {
            Ok(InitialDivisor {
                id: d.id,
                equation: extension.pull(&d.equation, budget)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let ledger = match verify_initial_relative_snc(frame, divisors, budget)? {
        SncProduction::Verified(ledger) => ledger,
        SncProduction::Incomplete {
            reason,
            progress: inner,
            ..
        } => {
            progress.snc = Some(inner);
            return Err(Error::ResourceIncomplete(reason));
        }
        SncProduction::Unresolved { .. } => {
            return Err(Error::Invalid("localization lost checked SNC geometry"));
        }
    };
    progress.snc = Some(ledger.progress().clone());
    progress.stage = "checked history restriction";
    Ok(Some(VerifiedPrincipalOpen {
        source,
        split,
        side,
        factor,
        extension,
        ledger,
    }))
}

#[cfg(test)]
mod tests;
