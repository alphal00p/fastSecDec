//! Reuse the existing ContourMode strict DTO boundary for pinned Serde's
//! internally tagged unit-variant unknown-field issue (upstream PR 3109).
use super::*;
use serde::Deserialize;

impl<'de> Deserialize<'de> for ContributionKind {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Stochastic { coordinates: Vec<NativeSymbolId> },
            Exact {},
            CertifiedZero { certificate: Digest },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Stochastic { coordinates } => Self::Stochastic { coordinates },
            Wire::Exact {} => Self::Exact,
            Wire::CertifiedZero { certificate } => Self::CertifiedZero { certificate },
        })
    }
}
impl<'de> Deserialize<'de> for RecordKind {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Exact {},
            Stochastic { coordinates: Vec<NativeSymbolId> },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Exact {} => Self::Exact,
            Wire::Stochastic { coordinates } => Self::Stochastic { coordinates },
        })
    }
}
impl<'de> Deserialize<'de> for ResidentSelection {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            Complete {},
            Selected {
                stochastic_contributions: Vec<ContributionId>,
                exact_policy: crate::results::ExactContributionPolicy,
            },
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Complete {} => Self::Complete,
            Wire::Selected {
                stochastic_contributions,
                exact_policy,
            } => Self::Selected {
                stochastic_contributions,
                exact_policy,
            },
        })
    }
}
