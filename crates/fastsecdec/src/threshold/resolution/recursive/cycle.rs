use super::*;

/// A cycle's fixed old exceptional set, separate from geometric birth history.
/// The only continuation constructor requires an actual proved residual drop.
#[derive(Clone, Debug)]
pub struct CycleSnapshot {
    origin: Arc<ResidualDrop>,
    history: Arc<ResolutionHistory>,
    old: Vec<BoundaryId>,
}
impl CycleSnapshot {
    pub fn after_first_drop(origin: Arc<ResidualDrop>, b: &mut Budget) -> Result<Arc<Self>> {
        if origin.current().algebraic_maximum_on_cosupport() == 0 {
            return Err(Error::Invalid(
                "zero residual maximum uses monomial terminal",
            ));
        }
        let history = origin.current().factor().data().history().clone();
        b.reserve_slots(history.ledger().divisors().len())?;
        let old = history.ledger().divisors().iter().map(|d| d.id).collect();
        Ok(Arc::new(Self {
            origin,
            history,
            old,
        }))
    }
    pub fn origin(&self) -> &Arc<ResidualDrop> {
        &self.origin
    }
    pub fn birth_history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn old_ids(&self) -> &[BoundaryId] {
        &self.old
    }
    pub(crate) fn check_history(&self, history: &ResolutionHistory) -> Result<()> {
        if !history.same_root(&self.history)
            || !history.chart_path().starts_with(self.history.chart_path())
            || self
                .old
                .iter()
                .any(|id| history.births().get(id) != self.history.births().get(id))
        {
            return Err(Error::Invalid("cycle snapshot geometric ancestry"));
        }
        Ok(())
    }
    pub(crate) fn check_open(&self, open: &CompanionContactOpen) -> Result<()> {
        if !Arc::ptr_eq(open.order(), self.origin.current()) {
            return Err(Error::Invalid("cycle snapshot companion source owner"));
        }
        self.check_history(open.restriction().history())
    }
}
