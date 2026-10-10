use super::super::{
    AdaptedCover, AdaptedProduction, BlowupProduction, Budget, Error, MonomialBlowup,
    MonomialProduction, MonomialWitness, ProducedMonomialCenter, ResolutionHistory,
    produce_adapted_cover, produce_monomial_blowup, produce_monomial_center,
};
use super::{CarriedMonomialWitness, carry_monomial_witness, exponent_rank};
use std::{collections::BTreeMap, sync::Arc};
type Result<T> = std::result::Result<T, Error>;
pub type NodePath = Vec<usize>;
#[derive(Clone, Debug)]
pub struct IterationLimits {
    pub max_nodes: usize,
    pub max_depth: usize,
}
impl Default for IterationLimits {
    fn default() -> Self {
        Self {
            max_nodes: 4096,
            max_depth: 256,
        }
    }
}
#[derive(Clone, Debug)]
pub enum PendingEvidence {
    Center(MonomialProduction),
    Cover(AdaptedProduction),
    Blowup(BlowupProduction),
    Operation(&'static str),
}
#[derive(Clone, Debug)]
enum NodeState {
    Center,
    Cover(Arc<ProducedMonomialCenter>),
    Blowup {
        center: Arc<ProducedMonomialCenter>,
        cover: Arc<AdaptedCover>,
    },
    Children {
        step: Arc<MonomialBlowup>,
        next: usize,
    },
    Expanded(Arc<MonomialBlowup>),
    Terminal(MonomialProduction),
}
impl NodeState {
    fn pending(&self) -> bool {
        !matches!(self, Self::Expanded(_) | Self::Terminal(_))
    }
}
#[derive(Clone, Debug)]
pub struct ResolutionNode {
    history: Arc<ResolutionHistory>,
    witness: Arc<MonomialWitness>,
    rank: usize,
    parent: Option<Arc<CarriedMonomialWitness>>,
    state: NodeState,
    evidence: Option<PendingEvidence>,
}
impl ResolutionNode {
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn witness(&self) -> &Arc<MonomialWitness> {
        &self.witness
    }
    pub fn rank(&self) -> usize {
        self.rank
    }
    pub fn parent(&self) -> Option<&Arc<CarriedMonomialWitness>> {
        self.parent.as_ref()
    }
    pub fn evidence(&self) -> Option<&PendingEvidence> {
        self.evidence.as_ref()
    }
    pub fn terminal(&self) -> bool {
        matches!(&self.state, NodeState::Terminal(_))
    }
    pub fn terminal_proof(&self) -> Option<&MonomialProduction> {
        if let NodeState::Terminal(v) = &self.state {
            Some(v)
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
pub struct MonomialFrontier {
    nodes: BTreeMap<NodePath, ResolutionNode>,
    initial_rank: usize,
    namespace: String,
}
#[derive(Clone, Debug)]
pub enum IterationAdvance {
    Progress { spawned: Option<NodePath> },
    ResourceIncomplete(&'static str),
}
#[derive(Debug)]
pub struct CompletedMonomialResolution {
    frontier: MonomialFrontier,
}
impl CompletedMonomialResolution {
    pub fn nodes(&self) -> &BTreeMap<NodePath, ResolutionNode> {
        &self.frontier.nodes
    }
    pub fn initial_rank(&self) -> usize {
        self.frontier.initial_rank
    }
}
pub enum IterationCompletion {
    Complete(CompletedMonomialResolution),
    Incomplete(MonomialFrontier),
}
impl MonomialFrontier {
    pub fn new(
        history: Arc<ResolutionHistory>,
        witness: MonomialWitness,
        namespace: String,
    ) -> Result<Self> {
        if witness.source.ideal().ring() != history.ledger().frame().local().ring()
            || witness.powers.len() != history.ledger().divisors().len()
        {
            return Err(Error::Invalid("iteration source shape/owner"));
        }
        let rank = exponent_rank(&witness.powers)?;
        let node = ResolutionNode {
            history,
            witness: Arc::new(witness),
            rank,
            parent: None,
            state: NodeState::Center,
            evidence: None,
        };
        Ok(Self {
            nodes: BTreeMap::from([(vec![], node)]),
            initial_rank: rank,
            namespace,
        })
    }
    pub fn nodes(&self) -> &BTreeMap<NodePath, ResolutionNode> {
        &self.nodes
    }
    pub fn pending(&self) -> impl Iterator<Item = &NodePath> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.state.pending())
            .map(|(p, _)| p)
    }
    pub fn advance(
        &mut self,
        path: &[usize],
        limits: &IterationLimits,
        b: &mut Budget,
    ) -> Result<IterationAdvance> {
        let node = self
            .nodes
            .get(path)
            .ok_or(Error::Invalid("unknown iteration node"))?;
        if !node.state.pending() {
            return Err(Error::Invalid("iteration node already accepted"));
        }
        let state = node.state.clone();
        let history = node.history.clone();
        let witness = node.witness.clone();
        let namespace = format!(
            "{}_node{}",
            self.namespace,
            path.iter().map(|i| format!("_{i}")).collect::<String>()
        );
        let result = self.prepare_advance(path, state, history, witness, &namespace, limits, b);
        match result {
            Ok((state, child, evidence)) => {
                if let Some(evidence) = evidence {
                    let reason = match &evidence {
                        PendingEvidence::Center(MonomialProduction::Incomplete {
                            reason, ..
                        })
                        | PendingEvidence::Cover(AdaptedProduction::Incomplete {
                            reason, ..
                        })
                        | PendingEvidence::Blowup(BlowupProduction::Incomplete {
                            reason, ..
                        })
                        | PendingEvidence::Operation(reason) => *reason,
                        _ => return Err(Error::Invalid("wrong pending proof kind")),
                    };
                    self.nodes
                        .get_mut(path)
                        .ok_or(Error::Invalid("lost iteration node"))?
                        .evidence = Some(evidence);
                    return Ok(IterationAdvance::ResourceIncomplete(reason));
                }
                let node = self
                    .nodes
                    .get_mut(path)
                    .ok_or(Error::Invalid("lost iteration node"))?;
                node.state = state;
                node.evidence = None;
                let spawned = child.as_ref().map(|(p, _)| p.clone());
                if let Some((path, node)) = child
                    && self.nodes.insert(path, node).is_some()
                {
                    return Err(Error::Invalid("duplicate iteration child"));
                }
                Ok(IterationAdvance::Progress { spawned })
            }
            Err(Error::ResourceIncomplete(reason)) => {
                self.nodes
                    .get_mut(path)
                    .ok_or(Error::Invalid("lost iteration node"))?
                    .evidence = Some(PendingEvidence::Operation(reason));
                Ok(IterationAdvance::ResourceIncomplete(reason))
            }
            Err(e) => Err(e),
        }
    }
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn prepare_advance(
        &self,
        path: &[usize],
        state: NodeState,
        history: Arc<ResolutionHistory>,
        witness: Arc<MonomialWitness>,
        namespace: &str,
        limits: &IterationLimits,
        b: &mut Budget,
    ) -> Result<(
        NodeState,
        Option<(NodePath, ResolutionNode)>,
        Option<PendingEvidence>,
    )> {
        match state {
            NodeState::Center => {
                match produce_monomial_center(history.ledger().clone(), (*witness).clone(), b)? {
                    MonomialProduction::Center(center) => {
                        Ok((NodeState::Cover(Arc::new(*center)), None, None))
                    }
                    complete @ MonomialProduction::EmptyCosupport { .. } => {
                        Ok((NodeState::Terminal(complete), None, None))
                    }
                    incomplete @ MonomialProduction::Incomplete { .. } => Ok((
                        NodeState::Center,
                        None,
                        Some(PendingEvidence::Center(incomplete)),
                    )),
                }
            }
            NodeState::Cover(center) => match produce_adapted_cover(
                center.ledger().clone(),
                &format!("{namespace}_adapt"),
                b,
            )? {
                AdaptedProduction::Complete(cover) => {
                    Ok((NodeState::Blowup { center, cover }, None, None))
                }
                incomplete @ AdaptedProduction::Incomplete { .. } => Ok((
                    NodeState::Cover(center),
                    None,
                    Some(PendingEvidence::Cover(incomplete)),
                )),
            },
            NodeState::Blowup { center, cover } => match produce_monomial_blowup(
                center.clone(),
                history,
                cover.clone(),
                &format!("{namespace}_blowup"),
                b,
            )? {
                BlowupProduction::Complete(step) => Ok((
                    NodeState::Children {
                        step: Arc::from(step),
                        next: 0,
                    },
                    None,
                    None,
                )),
                incomplete @ BlowupProduction::Incomplete { .. } => Ok((
                    NodeState::Blowup { center, cover },
                    None,
                    Some(PendingEvidence::Blowup(incomplete)),
                )),
            },
            NodeState::Children { step, next } => {
                if next >= step.charts().len() {
                    return Err(Error::Invalid("iteration child frontier"));
                }
                if self.nodes.len() >= limits.max_nodes {
                    return Err(Error::ResourceIncomplete("monomial node limit"));
                }
                let depth = path
                    .len()
                    .checked_add(1)
                    .ok_or(Error::ResourceIncomplete("monomial depth overflow"))?;
                if depth > limits.max_depth || depth > self.initial_rank {
                    return Err(Error::ResourceIncomplete("monomial depth limit"));
                }
                let carried = Arc::new(carry_monomial_witness(step.clone(), next, b)?);
                if carried.ranks().0 != self.nodes[path].rank {
                    return Err(Error::Invalid("monomial parent rank"));
                }
                let mut child_path = path.to_vec();
                child_path.push(next);
                if self.nodes.contains_key(&child_path) {
                    return Err(Error::Invalid("duplicate monomial ancestry"));
                }
                let child = ResolutionNode {
                    history: carried.chart().history().clone(),
                    witness: Arc::new(carried.witness().clone()),
                    rank: carried.ranks().1,
                    parent: Some(carried),
                    state: NodeState::Center,
                    evidence: None,
                };
                let next = next
                    .checked_add(1)
                    .ok_or(Error::ResourceIncomplete("child frontier overflow"))?;
                let state = if next == step.charts().len() {
                    NodeState::Expanded(step)
                } else {
                    NodeState::Children { step, next }
                };
                Ok((state, Some((child_path, child)), None))
            }
            _ => Err(Error::Invalid("accepted node resubmitted")),
        }
    }
    pub fn try_complete(self) -> Result<IterationCompletion> {
        if self.pending().next().is_some() {
            return Ok(IterationCompletion::Incomplete(self));
        }
        for (path, node) in &self.nodes {
            if let NodeState::Expanded(step) = &node.state {
                for (i, chart) in step.charts().iter().enumerate() {
                    let mut child = path.clone();
                    child.push(i);
                    let child = self
                        .nodes
                        .get(&child)
                        .ok_or(Error::Invalid("missing accepted descendant"))?;
                    let parent = child
                        .parent
                        .as_ref()
                        .ok_or(Error::Invalid("missing child certificate"))?;
                    if !Arc::ptr_eq(parent.source(), step)
                        || !Arc::ptr_eq(parent.chart(), chart)
                        || parent.chart_index() != i
                        || child.rank >= node.rank
                    {
                        return Err(Error::Invalid("descendant proof mismatch"));
                    }
                }
            } else if !matches!(
                &node.state,
                NodeState::Terminal(MonomialProduction::EmptyCosupport { .. })
            ) {
                return Err(Error::Invalid("unaccepted leaf"));
            }
        }
        Ok(IterationCompletion::Complete(CompletedMonomialResolution {
            frontier: self,
        }))
    }
}
