use super::*;
impl ResolutionHistory {
    pub(crate) fn advanced_embedded_presentation(
        &self,
        owner: &super::super::presentation_transition::EmbeddedTransition,
        open: &super::super::presentation_transition::EmbeddedTransitionOpen,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, owner.source().history().as_ref())
            || !self.same_root(owner.source().original_cycle().initial_history())
            || !owner.opens().iter().any(|o| std::ptr::eq(o.as_ref(), open))
            || !Arc::ptr_eq(open.coefficient.source(), owner.source().coefficient())
            || !Arc::ptr_eq(open.incidence_sum.source(), owner.source().incidence_sum())
            || !Arc::ptr_eq(open.center.support(), owner.support())
            || !Arc::ptr_eq(open.coefficient.open(), open.center.open())
            || !Arc::ptr_eq(open.incidence_sum.open(), open.center.open())
            || open.center.source().ideal() != &owner.lower_center
            || open.center.source().mark() != 1
            || open.center.exceptional_power()
                != usize::from(owner.chart().geometry().exceptional().is_some())
            || open.born.is_some() != owner.chart().geometry().exceptional().is_some()
            || open.divisors.len() != self.ledger.divisors().len()
            || !Arc::ptr_eq(open.ledger.frame(), open.center.open().frame())
        {
            return Err(Error::Invalid(
                "repeated embedded history actual source/transition owner",
            ));
        }
        if !Arc::ptr_eq(&open.boundary.prior, owner.source().history())
            || !Arc::ptr_eq(&open.boundary.chart, owner.chart())
            || !Arc::ptr_eq(&open.boundary.support, owner.support())
            || !Arc::ptr_eq(&open.boundary.center, &open.center)
            || !Arc::ptr_eq(&open.boundary.ledger, &open.ledger)
            || open.boundary.born != open.born
        {
            return Err(Error::Invalid("embedded shared boundary owner"));
        }
        self.advanced_lower_boundary(&open.boundary, b)
    }
}
