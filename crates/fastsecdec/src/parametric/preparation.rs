//! Conservative orchestration of native exact family preparation.

mod entry;

use std::borrow::Cow;

use feynkit_graph::{IntegralFamily, IntegralFamilyError};
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore};
use symbolica::utils::resources::{self, ResourceBudget, ResourceLimits};

use crate::Result;

/// Optional exact preprocessing; the original native graph/family stays intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyPreparationPolicy {
    /// Preserve every source propagator and its Schwinger parameter.
    Original,
    /// Admit only a single coefficient-one positive-power native projection.
    /// The optional diagnostic counts native exponent states. None imposes no
    /// count ceiling; RAM/time budgets remain independent of this policy.
    SingleUnitTerm {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_states: Option<usize>,
    },
}

impl Default for FamilyPreparationPolicy {
    fn default() -> Self {
        Self::SingleUnitTerm { max_states: None }
    }
}

/// Why an optional projection was not applied. These are not invalid integrals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyPreparationFallback {
    OriginalRequested,
    Independent,
    PowerOutsideNativeRange,
    StateLimit,
    MultipleTerms,
    NonUnitCoefficient,
    NegativePowers,
    NoReduction,
    EmptySupport,
    ChangedMomentumBasis,
    IdentityNotCanonical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyPreparationStatus {
    Original(FamilyPreparationFallback),
    Projected,
}

/// Exact correspondence to the original ordered native denominators.
///
/// Original stable graph edge IDs remain owned by the caller's `GraphIntegral`;
/// indices here refer to that same order. A projection represents the complete
/// integral, not a selected-sector numerical scope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyPreparationReport {
    pub policy: FamilyPreparationPolicy,
    pub status: FamilyPreparationStatus,
    pub original_powers: Vec<u32>,
    pub active_original_indices: Vec<usize>,
    pub active_powers: Vec<u32>,
}

/// Borrow the original native family, or own the sector produced by FeynKit.
#[derive(Clone, Debug)]
pub struct PreparedFamily<'a> {
    family: Cow<'a, IntegralFamily>,
    report: FamilyPreparationReport,
}

impl PreparedFamily<'_> {
    pub fn family(&self) -> &IntegralFamily {
        &self.family
    }

    pub fn powers(&self) -> &[u32] {
        &self.report.active_powers
    }

    pub fn report(&self) -> &FamilyPreparationReport {
        &self.report
    }
}

/// Reuse native partial fractions only when one exact unit-coefficient term
/// removes denominators without changing the momentum basis or numerator.
///
/// No shifts, scaleless removal, numerator changes or additional measure factors
/// occur here. All other admitted decompositions preserve the original route;
/// native invalid-family errors propagate. Validate/contract a graph's numerator
/// before calling `from_family` and apply its overall weight exactly once.
pub fn prepare_family<'a>(
    family: &'a IntegralFamily,
    powers: &[u32],
    policy: FamilyPreparationPolicy,
) -> Result<PreparedFamily<'a>> {
    prepare_family_with_resources(family, powers, policy, ResourceLimits::default())
}

/// Prepare a native family with cooperative process-RSS and elapsed-time limits.
///
/// The scope covers admission, exact decomposition and the final report. An
/// enclosing budget remains active; a native resource failure never selects an
/// original-family fallback. Checks between opaque algebra operations are
/// cooperative, so hard interruption requires an external supervisor.
///
/// ```no_run
/// use fastsecdec::parametric::{FamilyPreparationPolicy, prepare_family_with_resources};
/// use fastsecdec::IntegralFamily;
/// use symbolica::utils::resources::ResourceLimits;
///
/// # fn prepare(family: &IntegralFamily, powers: &[u32]) -> fastsecdec::Result<()> {
/// let prepared = prepare_family_with_resources(
///     family, powers, FamilyPreparationPolicy::default(), ResourceLimits::default(),
/// )?;
/// assert_eq!(prepared.powers().len(), prepared.family().denominators().len());
/// # Ok(())
/// # }
/// ```
pub fn prepare_family_with_resources<'a>(
    family: &'a IntegralFamily,
    powers: &[u32],
    policy: FamilyPreparationPolicy,
    limits: ResourceLimits,
) -> Result<PreparedFamily<'a>> {
    ResourceBudget::new(limits).run(|| {
        resources::checkpoint()?;
        let prepared = prepare_family_scoped(family, powers, policy)?;
        resources::checkpoint()?;
        Ok(prepared)
    })
}

fn prepare_family_scoped<'a>(
    family: &'a IntegralFamily,
    powers: &[u32],
    policy: FamilyPreparationPolicy,
) -> Result<PreparedFamily<'a>> {
    if powers.len() != family.denominators().len() || powers.contains(&0) {
        return Err(super::ParametricError::Invalid(
            "family preparation requires one strictly positive power per denominator".into(),
        )
        .into());
    }
    if family
        .denominators()
        .iter()
        .any(|denominator| denominator.is_zero())
    {
        return Err(IntegralFamilyError::InvalidBasis(
            "a positive-power denominator is identically zero".into(),
        )
        .into());
    }
    let original = |reason| PreparedFamily {
        family: Cow::Borrowed(family),
        report: FamilyPreparationReport {
            policy,
            status: FamilyPreparationStatus::Original(reason),
            original_powers: powers.to_vec(),
            active_original_indices: (0..powers.len()).collect(),
            active_powers: powers.to_vec(),
        },
    };
    let FamilyPreparationPolicy::SingleUnitTerm { max_states } = policy else {
        return Ok(original(FamilyPreparationFallback::OriginalRequested));
    };
    if max_states == Some(0) {
        return Err(super::ParametricError::Invalid(
            "native family preparation state bound must be positive".into(),
        )
        .into());
    }
    if family.is_independent() {
        return Ok(original(FamilyPreparationFallback::Independent));
    }
    let Ok(signed) = powers
        .iter()
        .copied()
        .map(i32::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
    else {
        return Ok(original(FamilyPreparationFallback::PowerOutsideNativeRange));
    };
    let terms = match family.partial_fraction(&signed, max_states, ResourceLimits::default()) {
        Ok(terms) => terms,
        Err(IntegralFamilyError::PartialFractionLimit(_)) => {
            return Ok(original(FamilyPreparationFallback::StateLimit));
        }
        Err(IntegralFamilyError::PowerOverflow) => {
            return Ok(original(FamilyPreparationFallback::PowerOutsideNativeRange));
        }
        Err(IntegralFamilyError::Resource(error)) => return Err(error.into()),
        Err(error) => return Err(error.into()),
    };
    let [(coefficient, projected_powers)] = terms.as_slice() else {
        return Ok(original(FamilyPreparationFallback::MultipleTerms));
    };
    if !coefficient.is_one() {
        return Ok(original(FamilyPreparationFallback::NonUnitCoefficient));
    }
    if projected_powers.iter().any(|p| *p < 0) {
        return Ok(original(FamilyPreparationFallback::NegativePowers));
    }
    let active_original_indices = projected_powers
        .iter()
        .enumerate()
        .filter_map(|(index, power)| (*power > 0).then_some(index))
        .collect::<Vec<_>>();
    if active_original_indices.is_empty() {
        return Ok(original(FamilyPreparationFallback::EmptySupport));
    }
    if active_original_indices.len() >= powers.len() {
        return Ok(original(FamilyPreparationFallback::NoReduction));
    }
    let projected = family.sector(projected_powers)?;
    if projected.loop_momenta() != family.loop_momenta()
        || projected.external_momenta() != family.external_momenta()
    {
        return Ok(original(FamilyPreparationFallback::ChangedMomentumBasis));
    }
    // A conservative exact admission check. Native Atom canonicalization owns
    // products/powers; do not expand or construct an alternate simplifier.
    let inverse_product = |powers: &[i32]| {
        family
            .denominators()
            .iter()
            .zip(powers)
            .map(|(denominator, power)| denominator.pow(-i64::from(*power)))
            .product::<Atom>()
    };
    if inverse_product(&signed) != inverse_product(projected_powers) {
        return Ok(original(FamilyPreparationFallback::IdentityNotCanonical));
    }
    let active_powers = active_original_indices
        .iter()
        .map(|index| projected_powers[*index] as u32)
        .collect();
    Ok(PreparedFamily {
        family: Cow::Owned(projected),
        report: FamilyPreparationReport {
            policy,
            status: FamilyPreparationStatus::Projected,
            original_powers: powers.to_vec(),
            active_original_indices,
            active_powers,
        },
    })
}
