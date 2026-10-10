//! First-center recursion and its first carried cycle: fixed parameters,
//! explicit contact opens/history owners. No full BM or real atlas authority.
mod coordinate;
mod lift;
mod state;
pub use coordinate::*;
pub use state::*;
#[cfg(test)]
mod tests;
use super::*;
use std::sync::Arc;
use symbolica::domains::rational::Rational;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct RecursiveLevel {
    pub(crate) frame: Arc<EtaleFrame>,
    pub(crate) source: Arc<MarkedIdeal>,
    pub(crate) contact: Arc<ContactQuotient>,
    pub(crate) normalization: Arc<VerifiedQuotientNormalization>,
    pub(crate) ratio: Rational,
}
#[derive(Clone, Debug)]
pub struct LiftReceipt {
    pub(crate) level: Arc<RecursiveLevel>,
    pub(crate) child: Arc<RecursiveCenter>,
    pub(crate) clearings: Vec<UnitClearing>,
}
#[derive(Clone, Debug)]
pub struct RecursiveCenter {
    pub(crate) frame: Arc<EtaleFrame>,
    pub(crate) source: Arc<MarkedIdeal>,
    pub(crate) ideal: Ideal,
    pub(crate) normals: Vec<Poly>,
    pub(crate) invariant: Vec<Rational>,
    pub(crate) lift: Option<LiftReceipt>,
}
impl RecursiveCenter {
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn ideal(&self) -> &Ideal {
        &self.ideal
    }
    pub fn normals(&self) -> &[Poly] {
        &self.normals
    }
    /// Finite ratios, each with old incidence zero. The terminal infinity
    /// is implicit in this proved center, not an element of this vector.
    pub fn invariant_pairs(&self) -> &[Rational] {
        &self.invariant
    }
    pub fn lift_receipt(&self) -> Option<&LiftReceipt> {
        self.lift.as_ref()
    }
}
impl RecursiveLevel {
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn contact(&self) -> &Arc<ContactQuotient> {
        &self.contact
    }
    pub fn normalization(&self) -> &Arc<VerifiedQuotientNormalization> {
        &self.normalization
    }
}
impl LiftReceipt {
    pub fn level(&self) -> &Arc<RecursiveLevel> {
        &self.level
    }
    pub fn child(&self) -> &Arc<RecursiveCenter> {
        &self.child
    }
    pub fn clearings(&self) -> &[UnitClearing] {
        &self.clearings
    }
}

mod companion;
pub use companion::{CompanionCenter, CompanionCenterOutcome, CompanionFirstCenter};
mod carry;
pub use carry::{
    CarriedFirstBlowup, CarriedFirstChart, ControlledGenerator, ResidualDrop, carry_first_blowup,
    prove_first_residual_drop,
};
mod cycle;
pub use cycle::CycleSnapshot;

#[cfg(test)]
mod continuation_tests;
