//! Constructive selected real germ operations; no global real atlas authority.
mod clusters;
mod point;
mod ramification;
mod state;
#[cfg(test)]
mod tests;
mod valuation;
use super::{FactorBranch, FactorSeed, MonicPolynomial};
use crate::threshold::resolution::*;
pub use clusters::{FiberClusters, RealCluster, discover_clusters};
pub use point::GermPoint;
pub use ramification::PowerChange;
pub use state::*;
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::{
        Field, RealEmbedding, Ring as _, RingOps as _,
        algebraic::{AlgebraicContext, AlgebraicExtension, AlgebraicNumber, Root},
        rational::{Q, Rational},
    },
    poly::{PolyVariable, polynomial::PolynomialRing, univariate::UnivariatePolynomial},
};
type Result<T> = std::result::Result<T, Error>;

fn exact_usize(value: usize) -> Result<Rational> {
    Ok(Rational::from(i64::try_from(value).map_err(|_| {
        Error::ResourceIncomplete("AJ exact integer conversion")
    })?))
}

mod simple;
pub use simple::{Factorization, SimpleRootBranch};
