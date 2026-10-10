//! Checked exact local factor germs. No compact real branch or endpoint certificate.
mod constants;
mod factor;
mod polynomial;
pub mod quasiordinary;
use crate::threshold::resolution::Error;
pub use constants::FactorSeed;
pub use factor::{FactorBranch, FactorJet};
pub use polynomial::MonicPolynomial;
pub type Result<T> = std::result::Result<T, Error>;
#[cfg(test)]
mod tests;
