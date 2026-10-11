use super::Result;
use crate::threshold::resolution::{Budget, Error, EtaleFrame, Poly};
use std::sync::Arc;
use symbolica::atom::Symbol;

/// Actual native coefficients in ascending order, including the leading one.
/// The polynomial variable is distinct from every base axis and parameter.
#[derive(Clone, Debug)]
pub struct MonicPolynomial {
    frame: Arc<EtaleFrame>,
    variable: Symbol,
    coefficients: Vec<Poly>,
}
impl MonicPolynomial {
    pub fn new(
        frame: Arc<EtaleFrame>,
        variable: Symbol,
        coefficients: Vec<Poly>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        Self::construct(frame, variable, coefficients, false, budget)
    }
    pub(super) fn from_regular_coefficients(
        frame: Arc<EtaleFrame>,
        variable: Symbol,
        coefficients: Vec<Poly>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        Self::construct(frame, variable, coefficients, true, budget)
    }
    fn construct(
        frame: Arc<EtaleFrame>,
        variable: Symbol,
        coefficients: Vec<Poly>,
        regular: bool,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        budget.reserve_slots(coefficients.len())?;
        if coefficients.len() < 3 {
            return Err(Error::Invalid("factor polynomial degree"));
        }
        if coefficients.len() - 1 > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("factor polynomial degree cap"));
        }
        let ring = frame.local().ring();
        if ring
            .one()
            .variables()
            .iter()
            .any(|v| v.to_atom() == symbolica::atom::Atom::var(variable))
        {
            return Err(Error::Invalid("factor variable aliases base role"));
        }
        for p in &coefficients {
            budget.poly(p)?;
            // Initial owner admits polynomial coefficients; fractions represented
            // by guard slots require a separate certified unit-clearing stage.
            if regular {
                frame.local().supports(p)?;
            } else {
                ring.supports(p, frame.local().axes())?;
            }
        }
        if coefficients.last() != Some(&ring.one()) {
            return Err(Error::Invalid("factor polynomial is not monic"));
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
