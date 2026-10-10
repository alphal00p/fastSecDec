use super::super::{Budget, Error, LocalizationProduction, MarkedIdeal, ResolutionHistory};
use super::{
    data::FactorData,
    state::*,
    steps::{self, StepContext},
};
use std::{collections::BTreeMap, sync::Arc};
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct ComponentFactorFrontier {
    nodes: BTreeMap<FactorPath, FactorNode>,
    namespace: String,
}
#[derive(Debug)]
pub struct CompletedComponentFactors {
    frontier: ComponentFactorFrontier,
}
impl CompletedComponentFactors {
    pub fn nodes(&self) -> &BTreeMap<FactorPath, FactorNode> {
        &self.frontier.nodes
    }
}
pub enum ComponentFactorCompletion {
    Complete(CompletedComponentFactors),
    Incomplete(ComponentFactorFrontier),
}
impl ComponentFactorFrontier {
    pub fn new(
        history: Arc<ResolutionHistory>,
        source: Arc<MarkedIdeal>,
        namespace: String,
        b: &mut Budget,
    ) -> Result<Self> {
        let data = FactorData::initial(history, source, b)?;
        Ok(Self {
            nodes: BTreeMap::from([(
                vec![],
                FactorNode {
                    data,
                    parent: None,
                    state: State::Ambient,
                    evidence: None,
                },
            )]),
            namespace,
        })
    }
    pub fn nodes(&self) -> &BTreeMap<FactorPath, FactorNode> {
        &self.nodes
    }
    pub fn pending(&self) -> impl Iterator<Item = &FactorPath> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.state.pending())
            .map(|(p, _)| p)
    }
    pub fn advance(
        &mut self,
        path: &[usize],
        limits: &ComponentFactorLimits,
        b: &mut Budget,
    ) -> Result<FactorAdvance> {
        let node = self
            .nodes
            .get(path)
            .ok_or(Error::Invalid("unknown component factor node"))?;
        if !node.state.pending() {
            return Err(Error::Invalid("accepted component factor node"));
        }
        let namespace = format!(
            "{}_node{}",
            self.namespace,
            path.iter().map(|i| format!("_{i}")).collect::<String>()
        );
        let c = StepContext {
            path,
            namespace: &namespace,
            limits,
            node_count: self.nodes.len(),
        };
        let out = node
            .data
            .checked_clone(b)
            .and_then(|data| steps::prepare(data, node.state.clone(), &c, b));
        match out {
            Ok(step) => {
                if let Some(evidence) = step.evidence {
                    let reason = match &evidence {
                        FactorPendingEvidence::Component(
                            super::super::ComponentProduction::Incomplete { reason, .. },
                        )
                        | FactorPendingEvidence::Localization(
                            LocalizationProduction::Incomplete { reason, .. },
                        )
                        | FactorPendingEvidence::Operation(reason) => *reason,
                        _ => return Err(Error::Invalid("unexpected pending component evidence")),
                    };
                    self.nodes
                        .get_mut(path)
                        .ok_or(Error::Invalid("lost factor node"))?
                        .evidence = Some(evidence);
                    return Ok(FactorAdvance::ResourceIncomplete(reason));
                }
                if let Some((child_path, _)) = &step.child
                    && self.nodes.contains_key(child_path)
                {
                    return Err(Error::Invalid("duplicate factor ancestry"));
                }
                let node = self
                    .nodes
                    .get_mut(path)
                    .ok_or(Error::Invalid("lost factor node"))?;
                node.data = step.data;
                node.state = step.state;
                node.evidence = None;
                let spawned = step.child.as_ref().map(|(p, _)| p.clone());
                if let Some((p, n)) = step.child {
                    self.nodes.insert(p, n);
                }
                Ok(FactorAdvance::Progress { spawned })
            }
            Err(Error::ResourceIncomplete(reason)) => {
                self.nodes
                    .get_mut(path)
                    .ok_or(Error::Invalid("lost factor node"))?
                    .evidence = Some(FactorPendingEvidence::Operation(reason));
                Ok(FactorAdvance::ResourceIncomplete(reason))
            }
            Err(e) => Err(e),
        }
    }
    pub fn try_complete(self) -> Result<ComponentFactorCompletion> {
        if self.pending().next().is_some() {
            return Ok(ComponentFactorCompletion::Incomplete(self));
        }
        for (path, node) in &self.nodes {
            if let State::Expanded { split, children } = &node.state {
                if children.len() != 2 {
                    return Err(Error::Invalid("incomplete factor split coverage"));
                }
                for (side, child) in children.iter().enumerate() {
                    match child {
                        FactorChild::Empty {
                            side: seen,
                            evidence:
                                LocalizationProduction::Empty {
                                    source,
                                    split: owner,
                                    side: actual,
                                    ..
                                },
                        } => {
                            if *seen != side
                                || *actual != side
                                || !Arc::ptr_eq(source, node.data.history())
                                || !Arc::ptr_eq(owner, split)
                            {
                                return Err(Error::Invalid("empty child coverage owner"));
                            }
                        }
                        FactorChild::Nonempty {
                            path: child_path,
                            restriction,
                        } => {
                            let mut expected = path.clone();
                            expected.push(side);
                            if child_path != &expected
                                || restriction.open().side() != side
                                || !Arc::ptr_eq(restriction.open().split(), split)
                                || !Arc::ptr_eq(restriction.open().source(), node.data.history())
                            {
                                return Err(Error::Invalid("factor child coverage provenance"));
                            }
                            let child = self
                                .nodes
                                .get(child_path)
                                .ok_or(Error::Invalid("missing factor child"))?;
                            if !child
                                .parent
                                .as_ref()
                                .is_some_and(|v| Arc::ptr_eq(v, restriction))
                                || !Arc::ptr_eq(child.data.history(), restriction.history())
                            {
                                return Err(Error::Invalid("factor child history provenance"));
                            }
                        }
                        _ => return Err(Error::Invalid("unaccepted empty factor child")),
                    }
                }
            } else if !matches!(node.state, State::Complete(_) | State::ZeroIdeal(_)) {
                return Err(Error::Invalid("unaccepted component factor leaf"));
            }
        }
        Ok(ComponentFactorCompletion::Complete(
            CompletedComponentFactors { frontier: self },
        ))
    }
}
