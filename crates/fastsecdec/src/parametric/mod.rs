//! Parametric integral data backed by Symbolica atoms and native HEPKit U/F.

mod integrand;
mod numerator;
mod scalar;

pub use fastsecdec_sectors::ParametricDomain;
pub use integrand::{
    FactorRole, ParametricError, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    polynomial_support,
};
pub use scalar::ScalarParametricIntegral;
