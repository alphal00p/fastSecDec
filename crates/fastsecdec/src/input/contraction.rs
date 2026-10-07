//! Serializable selection of Idenso's structural contraction policy.

use idenso::tensor::{AlgebraContraction, AlgebraSettings};
use serde::{Deserialize, Serialize};

/// Graph-numerator contraction before Gaussian parameterization.
///
/// Gamma, color and epsilon identities retain the normal HEP settings. This
/// selects only the native structural policy; it does not expand the entire
/// scalar numerator separately or relax polynomial admission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeratorContraction {
    /// Preserve independent sum alternatives where the identities permit it.
    #[default]
    Minimal,
    /// Fully contract, then normalize to canonical index-free scalar products.
    Dots,
    /// Fully contract; FastSecDec still normalizes scalar-product notation.
    Full,
    /// Perform only structural prerequisites of the enabled algebra identities.
    None,
}

impl NumeratorContraction {
    pub fn algebra_settings(self) -> AlgebraSettings {
        AlgebraSettings {
            contract: match self {
                Self::Minimal => AlgebraContraction::Minimal,
                Self::Dots => AlgebraContraction::Dots,
                Self::Full => AlgebraContraction::Fully,
                Self::None => AlgebraContraction::None,
            },
            ..super::default_algebra_settings()
        }
    }
}
