use super::{Budget, Error, Ideal, Poly};
use super::{LocalizedAlgebra, localized::Result};
use std::sync::Arc;

/// Algebraic locus/open inventory supplied by the caller. Identification of
/// this locus with a marked-ideal cosupport is a separate certificate.
pub struct OpenCoverCertificate {
    pub algebra: Arc<LocalizedAlgebra>,
    pub support: Ideal,
    pub opens: Vec<Poly>,
}
#[derive(Clone, Debug)]
pub struct VerifiedOpenCover {
    algebra: Arc<LocalizedAlgebra>,
    support: Ideal,
    opens: Vec<Poly>,
    algebraic_locus_empty: bool,
}
impl OpenCoverCertificate {
    pub fn verify(self, budget: &mut Budget) -> Result<VerifiedOpenCover> {
        if self.support.ring() != self.algebra.ring() {
            return Err(Error::Invalid("cover ring mismatch"));
        }
        budget.reserve_slots(self.opens.len())?;
        for p in self.support.generators().iter().chain(&self.opens) {
            self.algebra.supports(p)?;
        }
        let locus = self.algebra.ideal().sum(&self.support, budget)?;
        let algebraic_locus_empty = locus.contains(
            &self.algebra.ring().one(),
            self.algebra.unit_relations(),
            budget,
        )?;
        let opens = Ideal::new(self.algebra.ring().clone(), self.opens.clone(), budget)?;
        let uncovered = locus.sum(&opens, budget)?;
        if !uncovered.contains(
            &self.algebra.ring().one(),
            self.algebra.unit_relations(),
            budget,
        )? {
            return Err(Error::Invalid(
                "proposed opens do not cover the supplied algebraic locus",
            ));
        }
        Ok(VerifiedOpenCover {
            algebra: self.algebra,
            support: self.support,
            opens: self.opens,
            algebraic_locus_empty,
        })
    }
}
impl VerifiedOpenCover {
    pub fn algebra(&self) -> &Arc<LocalizedAlgebra> {
        &self.algebra
    }
    pub fn support(&self) -> &Ideal {
        &self.support
    }
    pub fn opens(&self) -> &[Poly] {
        &self.opens
    }
    pub fn algebraic_locus_empty(&self) -> bool {
        self.algebraic_locus_empty
    }
}
