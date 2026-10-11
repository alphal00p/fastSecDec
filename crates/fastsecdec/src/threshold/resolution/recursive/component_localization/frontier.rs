use super::*;
#[derive(Clone, Debug)]
pub enum ComponentLocalizationState {
    Pending,
    EmptyPrincipal {
        receipt: CoverLocalizationProduction,
    },
    /// The ambient open remains a physical contribution.
    EmptyContact {
        parent: Arc<MarkedIdeal>,
        parent_normalized: Ideal,
        receipt: PhysicalSupportRestriction,
    },
    /// Empty child cosupport does not mean a zero physical density.
    UnitChild(Arc<LocalizedComponentSources>),
    Center(Arc<LocalizedComponentCenter>),
    UnresolvedChild(Arc<LocalizedComponentSources>),
}
#[derive(Clone, Debug)]
pub struct ComponentLocalizationFrontier {
    owner: Arc<ComponentLocalization>,
    states: Vec<ComponentLocalizationState>,
    namespace: String,
}
#[derive(Clone, Debug)]
pub enum ComponentLocalizationAdvance {
    Progress,
    Accepted,
    Incomplete { reason: &'static str },
}
impl ComponentLocalizationFrontier {
    pub fn new(
        owner: Arc<ComponentLocalization>,
        namespace: String,
        b: &mut Budget,
    ) -> Result<Self> {
        b.reserve_slots(owner.cover().opens().len())?;
        Ok(Self {
            states: vec![ComponentLocalizationState::Pending; owner.cover().opens().len()],
            owner,
            namespace,
        })
    }
    pub fn owner(&self) -> &Arc<ComponentLocalization> {
        &self.owner
    }
    pub fn states(&self) -> &[ComponentLocalizationState] {
        &self.states
    }
    /// This means only complete localization inventory, never completed resolution.
    pub fn complete_inventory(&self) -> bool {
        self.states.iter().all(|s| {
            !matches!(
                s,
                ComponentLocalizationState::Pending
                    | ComponentLocalizationState::UnresolvedChild(_)
            )
        })
    }
    /// Caller-stepped in-memory progress. Native incomplete work is retried; no durable checkpoint.
    pub fn advance(
        &mut self,
        index: usize,
        b: &mut Budget,
    ) -> Result<ComponentLocalizationAdvance> {
        let state = self
            .states
            .get(index)
            .ok_or(Error::Invalid("component localization index"))?;
        match state {
            ComponentLocalizationState::Pending => {}
            ComponentLocalizationState::UnresolvedChild(_) => {
                return Ok(ComponentLocalizationAdvance::Incomplete {
                    reason: "localized component source remains nonzero and nonunit",
                });
            }
            _ => return Ok(ComponentLocalizationAdvance::Accepted),
        }
        match self.one(index, b) {
            Ok(state) => {
                let unresolved = matches!(state, ComponentLocalizationState::UnresolvedChild(_));
                self.states[index] = state;
                Ok(if unresolved {
                    ComponentLocalizationAdvance::Incomplete {
                        reason: "localized component source remains nonzero and nonunit",
                    }
                } else {
                    ComponentLocalizationAdvance::Progress
                })
            }
            Err(Error::ResourceIncomplete(reason)) => {
                Ok(ComponentLocalizationAdvance::Incomplete { reason })
            }
            Err(e) => Err(e),
        }
    }
    fn one(&self, index: usize, b: &mut Budget) -> Result<ComponentLocalizationState> {
        let origin = self.owner.origin();
        let localization = match self.owner.cover().localize(
            origin.history().clone(),
            index,
            &format!("{}_open{index}", self.namespace),
            b,
        )? {
            CoverLocalizationProduction::Complete(open) => Arc::new(*open),
            receipt @ CoverLocalizationProduction::Empty { .. } => {
                return Ok(ComponentLocalizationState::EmptyPrincipal { receipt });
            }
            CoverLocalizationProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
        };
        if !localization.history().same_root(origin.history())
            || localization.history().stage() != origin.history().stage()
        {
            return Err(Error::Invalid(
                "component localization changed auxiliary history",
            ));
        }
        let level = self.owner.level();
        let parent = Arc::new(MarkedIdeal::new(
            localization
                .open()
                .extension()
                .ideal(level.source().ideal(), b)?,
            level.source().mark(),
            b,
        )?);
        let parent_normalized = localization
            .open()
            .extension()
            .ideal(level.normalization().normalized().ideal(), b)?;
        sources::equal(
            localization.history().ledger().frame().local(),
            parent.ideal(),
            &parent_normalized,
            b,
        )?;
        let support = match self.owner.cover().restrict_support(
            localization.clone(),
            &format!("{}_contact{index}", self.namespace),
            b,
        )? {
            PhysicalSupportRestriction::Support(s) => s,
            receipt @ PhysicalSupportRestriction::EmptySupport { .. } => {
                return Ok(ComponentLocalizationState::EmptyContact {
                    parent,
                    parent_normalized,
                    receipt,
                });
            }
        };
        let sources = LocalizedComponentSources::prepare(
            self.owner.clone(),
            localization,
            support,
            parent,
            parent_normalized,
            b,
        )?;
        let local = sources.support().frame().local();
        let mut zero = true;
        for f in sources.child().ideal().generators() {
            zero &= local.zero(f, b)?;
        }
        if zero {
            return Ok(ComponentLocalizationState::Center(
                LocalizedComponentCenter::prepare(sources, b)?,
            ));
        }
        if local.ideal().sum(sources.child().ideal(), b)?.contains(
            &local.ring().one(),
            local.unit_relations(),
            b,
        )? {
            Ok(ComponentLocalizationState::UnitChild(sources))
        } else {
            Ok(ComponentLocalizationState::UnresolvedChild(sources))
        }
    }
}
