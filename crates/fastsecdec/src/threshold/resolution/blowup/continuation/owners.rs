use super::*;
#[derive(Clone, Debug)]
pub(super) enum LowerSource {
    Initial(Arc<InducedChildChart>),
    Repeated(Arc<EmbeddedPresentation>),
}
impl LowerSource {
    pub(super) fn history(&self) -> &Arc<ResolutionHistory> {
        match self {
            Self::Initial(x) => x.history(),
            Self::Repeated(x) => x.history(),
        }
    }
    pub(super) fn ideal(&self) -> &Arc<MarkedIdeal> {
        match self {
            Self::Initial(x) => x.support().incidence_sum().target(),
            Self::Repeated(x) => x.incidence_sum(),
        }
    }
}
#[derive(Clone, Debug)]
pub(super) enum CarryOwner {
    Initial(Arc<CarriedCompanionChart>),
    Repeated {
        transition: Arc<EmbeddedTransition>,
        completion: Option<Arc<CompletedEmbeddedChild>>,
    },
}
impl CarryOwner {
    pub(super) fn opens(&self) -> usize {
        match self {
            Self::Initial(c) => c.opens().len(),
            Self::Repeated { transition, .. } => transition.opens().len(),
        }
    }
    pub(super) fn check_tasks(&self, tasks: &[LowerTask]) -> Result<()> {
        if tasks.len() != self.opens() {
            return Err(Error::Invalid("continuation support inventory"));
        }
        for (index, task) in tasks.iter().enumerate() {
            match (self, &task.source) {
                (Self::Initial(owner), LowerSource::Initial(source))
                    if Arc::ptr_eq(source.ambient(), owner)
                        && Arc::ptr_eq(source.support(), &owner.opens()[index]) => {}
                (
                    Self::Repeated {
                        transition,
                        completion,
                    },
                    LowerSource::Repeated(source),
                ) => {
                    let raw = &transition.opens()[index];
                    if !Arc::ptr_eq(source.coefficient(), raw.coefficient.target())
                        || !Arc::ptr_eq(source.chart(), transition.chart())
                        || !Arc::ptr_eq(source.history().ledger(), &raw.ledger)
                    {
                        return Err(Error::Invalid(
                            "repeated continuation original coefficient/history owner",
                        ));
                    }
                    let prior = match &source.origin {
                        PresentationOrigin::Transition { owner, open }
                            if Arc::ptr_eq(owner, transition) && *open == index =>
                        {
                            if completion.is_some()
                                || !Arc::ptr_eq(source.incidence_sum(), raw.incidence_sum.target())
                            {
                                return Err(Error::Invalid(
                                    "repeated continuation current J owner",
                                ));
                            }
                            source
                        }
                        PresentationOrigin::Incidence(drop) => {
                            if !completion
                                .as_ref()
                                .is_some_and(|c| Arc::ptr_eq(c, drop.completion()))
                                || !Arc::ptr_eq(drop.target(), source.incidence_sum())
                                || !Arc::ptr_eq(drop.source().history(), source.history())
                            {
                                return Err(Error::Invalid(
                                    "continuation incidence construction owner",
                                ));
                            }
                            drop.source()
                        }
                        _ => {
                            return Err(Error::Invalid(
                                "continuation presentation source definition",
                            ));
                        }
                    };
                    if !prior
                        .previous_transition()
                        .is_some_and(|(o, i)| Arc::ptr_eq(o, transition) && i == index)
                    {
                        return Err(Error::Invalid("continuation carried presentation index"));
                    }
                }
                _ => return Err(Error::Invalid("continuation lower owner kind")),
            }
            let done = task
                .done
                .as_ref()
                .ok_or(Error::Invalid("unfinished lower factor inventory"))?;
            let root = done
                .nodes()
                .get(&Vec::new())
                .ok_or(Error::Invalid("missing lower factor root"))?;
            if !Arc::ptr_eq(root.data().source(), task.source.ideal()) {
                return Err(Error::Invalid("lower factor original source owner"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub(super) enum SupportedCenter {
    Initial(Arc<CarriedMonomialCenter>),
    Repeated(Arc<EmbeddedMonomialCenter>),
}
impl SupportedCenter {
    pub(super) fn frame(&self) -> &Arc<EtaleFrame> {
        match self {
            Self::Initial(c) => c.frame(),
            Self::Repeated(c) => c.frame(),
        }
    }
    pub(super) fn ideal(&self) -> &Ideal {
        match self {
            Self::Initial(c) => c.ideal(),
            Self::Repeated(c) => c.ideal(),
        }
    }
    pub(super) fn checked(&self, b: &mut Budget) -> Result<Arc<CheckedRecursiveCenter>> {
        CheckedRecursiveCenter::new(
            match self {
                Self::Initial(c) => RecursiveCenterOrigin::CarriedMonomial(c.clone()),
                Self::Repeated(c) => RecursiveCenterOrigin::EmbeddedMonomial(c.clone()),
            },
            b,
        )
    }
    pub(super) fn belongs(&self, task: &LowerTask) -> bool {
        let (associated, factor) = match (self, &task.source) {
            (Self::Initial(c), LowerSource::Initial(s)) => {
                (Arc::ptr_eq(c.induced(), s), c.factor())
            }
            (Self::Repeated(c), LowerSource::Repeated(s)) => {
                (Arc::ptr_eq(c.presentation(), s), c.factor())
            }
            _ => return false,
        };
        associated
            && task.done.as_ref().is_some_and(|d| {
                d.nodes()
                    .values()
                    .filter_map(|n| n.leaf())
                    .any(|f| Arc::ptr_eq(f, factor))
            })
    }
}
#[derive(Clone, Debug)]
pub(super) struct LowerTask {
    pub(super) source: LowerSource,
    pub(super) frontier: ComponentFactorFrontier,
    pub(super) done: Option<Arc<CompletedComponentFactors>>,
}
