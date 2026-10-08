//! Graph/family parameterization with the native preparation report.

use super::super::{
    ParametricIntegrand,
    numerator::{parameterize_validated_family, validate_family_input},
};
use super::{
    FamilyPreparationPolicy, FamilyPreparationReport, FamilyPreparationStatus, prepare_family,
};
use crate::{
    Result,
    input::{GraphIntegral, default_algebra_settings},
};
use feynkit_graph::IntegralFamily;
use symbolica::atom::{Atom, Symbol};

impl ParametricIntegrand {
    /// Parameterize a graph after optional native single-term family preparation.
    ///
    /// Supply one fresh parameter per original propagator. The complete original
    /// label/numerator/dimension contract is validated before preparation, and
    /// projected inputs retain the active subset of those same labels. Graph
    /// weights, the native preparation coefficient and measure multiplier are
    /// applied once. [`Self::from_graph`]
    /// remains the explicit original-family route.
    pub fn from_graph_prepared(
        integral: &GraphIntegral,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
        policy: FamilyPreparationPolicy,
    ) -> Result<(Self, FamilyPreparationReport)> {
        let numerator =
            integral.scalar_numerator(&default_algebra_settings())? * integral.measure_multiplier();
        Self::from_family_prepared(
            integral.family(),
            integral.powers(),
            numerator,
            parameters,
            regulator,
            dimension,
            policy,
        )
    }

    /// The prepared counterpart of [`Self::from_family`], retaining its complete
    /// weighted-numerator convention and validating all original labels first.
    ///
    /// Native original U/F construction performs that admission and is reused
    /// for a no-op. A successful projection constructs active U/F once more.
    /// This deliberately avoids duplicating FeynKit's private label validator.
    pub fn from_family_prepared(
        family: &IntegralFamily,
        powers: &[u32],
        weighted_numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
        policy: FamilyPreparationPolicy,
    ) -> Result<(Self, FamilyPreparationReport)> {
        let original_uf = validate_family_input(
            family,
            powers,
            &weighted_numerator,
            &parameters,
            regulator,
            &dimension,
        )?;
        let prepared = prepare_family(family, powers, policy)?;
        let report = prepared.report().clone();
        let active_parameters = report
            .active_original_indices
            .iter()
            .map(|index| parameters[*index])
            .collect::<Vec<_>>();
        let uf = if report.status == FamilyPreparationStatus::Projected {
            prepared.family().symanzik(
                &active_parameters
                    .iter()
                    .map(|p| Atom::var(*p))
                    .collect::<Vec<_>>(),
            )?
        } else {
            original_uf
        };
        let integrand = parameterize_validated_family(
            prepared.family(),
            prepared.powers(),
            weighted_numerator * prepared.coefficient(),
            active_parameters,
            regulator,
            dimension,
            uf,
        )?;
        Ok((integrand, report))
    }
}
