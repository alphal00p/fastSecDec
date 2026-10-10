use super::super::{
    Budget, Error, EtaleCertificate, Guard, InitialDivisor, LocalizedAlgebra, Poly,
    ResolutionHistory, RingExtension, SncProduction, VerifiedRelativeSnc,
    verify_initial_relative_snc,
};
use super::LocalizationProgress;
use std::sync::Arc;
use symbolica::symbol;
type Result<T> = std::result::Result<T, Error>;
/// Only this common constructor issues the native geometry authority.
#[derive(Clone, Debug)]
pub(crate) struct CheckedOpenGeometry {
    pub(super) source: Arc<ResolutionHistory>,
    pub(super) factor: Poly,
    pub(super) extension: RingExtension,
    pub(super) ledger: Arc<VerifiedRelativeSnc>,
}
pub(super) fn build(
    source: Arc<ResolutionHistory>,
    factor: Poly,
    namespace: &str,
    budget: &mut Budget,
    progress: &mut LocalizationProgress,
) -> Result<Option<CheckedOpenGeometry>> {
    let old = source.ledger().frame();
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
    Ok(Some(CheckedOpenGeometry {
        source,
        factor,
        extension,
        ledger,
    }))
}
