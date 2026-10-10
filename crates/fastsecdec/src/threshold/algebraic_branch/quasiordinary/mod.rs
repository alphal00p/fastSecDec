//! Checked discriminant-SNC prerequisite on the original localization.
//!
//! No ramification, branch continuation, real atlas or endpoint authority.
mod polynomialization;
mod prepare;
use crate::threshold::resolution::{Budget, Error, EtaleFrame, Poly};
pub use polynomialization::{ClearedPolynomial, Monicization};
pub use prepare::{Preparation, Progress, QuasiOrdinaryPolynomial, prepare};
use std::sync::Arc;
use symbolica::atom::{Atom, Symbol};
type Result<T> = std::result::Result<T, Error>;
/// Full native polynomial, ascending coefficients. Existing guard inverses
/// are allowed; every base coordinate/physical parameter keeps its role.
#[derive(Clone, Debug)]
pub struct InputPolynomial {
    frame: Arc<EtaleFrame>,
    variable: Symbol,
    coefficients: Vec<Poly>,
}
impl InputPolynomial {
    pub fn new(
        frame: Arc<EtaleFrame>,
        variable: Symbol,
        coefficients: Vec<Poly>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if coefficients.len() < 3 {
            return Err(Error::Invalid("quasiordinary source degree below two"));
        }
        if coefficients.len() - 1 > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("quasiordinary source degree"));
        }
        budget.reserve_slots(coefficients.len())?;
        if frame
            .local()
            .ring()
            .one()
            .variables()
            .iter()
            .any(|v| v.to_atom() == Atom::var(variable))
        {
            return Err(Error::Invalid("polynomial variable aliases base role"));
        }
        for c in &coefficients {
            frame.local().supports(c)?;
            budget.poly(c)?;
        }
        Ok(Arc::new(Self {
            frame,
            variable,
            coefficients,
        }))
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn variable(&self) -> Symbol {
        self.variable
    }
    pub fn coefficients(&self) -> &[Poly] {
        &self.coefficients
    }
    pub fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }
}
#[cfg(test)]
mod tests;
