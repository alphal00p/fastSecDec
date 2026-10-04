use super::{
    IneligibilityReason, Pull, ReferenceComparison, ReferenceUncertainty, UnavailablePull,
};
use std::fmt;

impl fmt::Display for IneligibilityReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::IncompleteProduction => "production allocation is incomplete",
            Self::UnverifiedReference => "reference has not been independently checked",
            Self::NormalizationUnconfirmed => "normalization agreement is unconfirmed",
            Self::NormalizationMismatch => "normalizations differ",
            Self::KinematicsUnconfirmed => "kinematic agreement is unconfirmed",
            Self::KinematicsMismatch => "kinematics differ",
            Self::IndependenceUnconfirmed => "statistical independence is unconfirmed",
            Self::EstimatesCorrelated => "estimates are correlated",
            Self::MissingEstimate => "some coefficients have no estimate",
            Self::MissingReference => "some coefficients have no reference",
            Self::UnknownReferenceUncertainty => "some reference uncertainties are unknown",
        })
    }
}

impl fmt::Display for UnavailablePull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingEstimate => "estimate missing",
            Self::MissingReference => "reference missing",
            Self::UnknownReferenceUncertainty => "reference uncertainty unknown",
            Self::IndependenceUnconfirmed => "statistical independence unconfirmed",
            Self::EstimatesCorrelated => "estimates are correlated",
        })
    }
}

impl fmt::Display for ReferenceComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Reference: {} ({})",
            self.provenance.source, self.provenance.convention
        )?;
        if self.eligibility.eligible {
            writeln!(
                f,
                "Eligible for statistical comparison under the recorded assumptions"
            )?;
        } else {
            writeln!(f, "Comparison limitations:")?;
            for reason in &self.eligibility.reasons {
                writeln!(f, "  - {reason}")?;
            }
        }
        writeln!(
            f,
            "  {:<13} {:>13} {:>10} {:>13} {:>11} {:>11} {:>18}",
            "Coefficient",
            "Estimate",
            "Std. error",
            "Reference",
            "Ref. error",
            "Difference",
            "Diagnostic pull"
        )?;
        for row in &self.rows {
            let component = match row.key.component {
                crate::status::CoefficientComponent::Real => "Re",
                crate::status::CoefficientComponent::Imag => "Im",
            };
            let coefficient = format!("{component} eps^{}", row.key.order);
            let actual = row.estimate.map_or_else(
                || "missing".into(),
                |actual| format!("{:.6e}", actual.value),
            );
            let error = row.estimate.map_or_else(
                || "—".into(),
                |actual| format!("{:.3e}", actual.standard_error),
            );
            let reference = row.reference.as_ref().map_or_else(
                || "missing".into(),
                |reference| format!("{:.6e}", reference.value),
            );
            let reference_error = row.reference.as_ref().map_or_else(
                || "—".into(),
                |reference| match reference.uncertainty {
                    ReferenceUncertainty::Exact => "exact".into(),
                    ReferenceUncertainty::StandardError(error) => format!("{error:.3e}"),
                    ReferenceUncertainty::Unknown => "unknown".into(),
                },
            );
            let difference = row
                .difference
                .map_or_else(|| "—".into(), |difference| format!("{difference:.3e}"));
            let pull = match row.pull {
                Pull::Value(value) => format!("{value:.3}"),
                Pull::ZeroCombinedError { equal: true } => "equal; zero SE".into(),
                Pull::ZeroCombinedError { equal: false } => "unequal; zero SE".into(),
                Pull::Unavailable(_) => "unavailable".into(),
            };
            writeln!(
                f,
                "  {coefficient:<13} {actual:>13} {error:>10} {reference:>13} {reference_error:>11} {difference:>11} {pull:>18}"
            )?;
            if let Pull::Unavailable(reason) = row.pull {
                writeln!(f, "    Pull unavailable: {reason}.")?;
            }
        }
        Ok(())
    }
}
