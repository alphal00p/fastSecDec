//! Actual construction events at which a new general marked problem begins.
//! Existing geometric history is retained; a later transformed source cannot
//! retroactively manufacture an initial cycle.
use super::*;
#[derive(Clone, Debug)]
pub enum ProblemConstruction {
    OriginalCoefficient(Arc<EmbeddedChildCycle>),
    IncidenceRedefinition(Arc<IncidenceDrop>),
}
#[derive(Clone, Debug)]
pub struct InitialProblem {
    construction: ProblemConstruction,
    source: Arc<MarkedIdeal>,
    history: Arc<ResolutionHistory>,
}
impl EmbeddedChildCycle {
    /// Start recursion only from this actual original coefficient construction.
    pub fn initial_problem(self: &Arc<Self>) -> Result<Arc<InitialProblem>> {
        InitialProblem::original(self.clone())
    }
}
impl IncidenceDrop {
    /// Start the redefined marked problem while retaining its existing history.
    pub fn initial_problem(self: &Arc<Self>) -> Result<Arc<InitialProblem>> {
        InitialProblem::incidence(self.clone())
    }
}
impl InitialProblem {
    /// Create caller-stepped local recursion; completion inventories outcomes,
    /// and does not certify physical resolution or glued global centers.
    pub fn recursion(
        self: &Arc<Self>,
        namespace: String,
        b: &mut Budget,
    ) -> Result<InitialProblemRecursion> {
        InitialProblemRecursion::new(self.clone(), namespace, b)
    }
    pub(crate) fn original(owner: Arc<EmbeddedChildCycle>) -> Result<Arc<Self>> {
        let source = owner.center().child().source().clone();
        let history = owner.initial_history().clone();
        if !Arc::ptr_eq(history.ledger().frame(), owner.center().child().frame())
            || !history.ledger().divisors().is_empty()
            || history.stage() != 0
            || source.ideal()
                != owner
                    .center()
                    .coefficient()
                    .coefficient()
                    .coefficient()
                    .ideal()
            || source.mark()
                != owner
                    .center()
                    .coefficient()
                    .coefficient()
                    .coefficient()
                    .mark()
        {
            return Err(Error::Invalid(
                "initial lower marked problem original constructor",
            ));
        }
        Ok(Arc::new(Self {
            construction: ProblemConstruction::OriginalCoefficient(owner),
            source,
            history,
        }))
    }
    pub(crate) fn incidence(owner: Arc<IncidenceDrop>) -> Result<Arc<Self>> {
        let source = owner.target().clone();
        let history = owner.source().history().clone();
        if source.ideal().ring() != history.ledger().frame().local().ring()
            || owner.maximum() >= owner.source().old_incidence()
        {
            return Err(Error::Invalid(
                "initial lower marked problem incidence constructor",
            ));
        }
        Ok(Arc::new(Self {
            construction: ProblemConstruction::IncidenceRedefinition(owner),
            source,
            history,
        }))
    }
    pub fn construction(&self) -> &ProblemConstruction {
        &self.construction
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub(crate) fn check_order(&self, order: &ComponentResidualOrder) -> Result<()> {
        let data = order.factor().data();
        let h = data.history();
        if !Arc::ptr_eq(data.source(), &self.source)
            || !h.same_root(&self.history)
            || h.stage() != self.history.stage()
            || h.births() != self.history.births()
            || !h.chart_path().starts_with(self.history.chart_path())
        {
            return Err(Error::Invalid(
                "initial cycle source was transformed or substituted",
            ));
        }
        for step in &h.chart_path()[self.history.chart_path().len()..] {
            if !matches!(step, HistoryStep::PrincipalOpen { .. }) {
                return Err(Error::Invalid(
                    "initial cycle has a late physical transition",
                ));
            }
        }
        Ok(())
    }
}
