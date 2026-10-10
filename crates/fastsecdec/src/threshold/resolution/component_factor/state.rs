use super::super::{
    ComponentProduction, LocalizationProduction, LocalizedHistory, VerifiedComponentSplit,
};
use super::data::FactorData;
use std::sync::Arc;
pub type FactorPath = Vec<usize>;
#[derive(Clone, Debug)]
pub struct ComponentFactorLimits {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_power: usize,
}
impl Default for ComponentFactorLimits {
    fn default() -> Self {
        Self {
            max_nodes: 4096,
            max_depth: 256,
            max_power: 256,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum SplitPhase {
    Ambient,
    Boundary(usize),
}
#[derive(Clone, Debug)]
pub enum FactorPendingEvidence {
    Component(ComponentProduction),
    Localization(LocalizationProduction),
    Operation(&'static str),
}
#[derive(Clone, Debug)]
pub enum FactorChild {
    Nonempty {
        path: FactorPath,
        restriction: Arc<LocalizedHistory>,
    },
    Empty {
        side: usize,
        evidence: LocalizationProduction,
    },
}
#[derive(Clone, Debug)]
pub(crate) enum State {
    Ambient,
    Boundary(usize),
    Divide(usize),
    Split {
        split: Arc<VerifiedComponentSplit>,
        phase: SplitPhase,
        children: Vec<FactorChild>,
    },
    Expanded {
        split: Arc<VerifiedComponentSplit>,
        children: Vec<FactorChild>,
    },
    Finalize,
    Complete(Arc<FactorLeaf>),
    ZeroIdeal(Arc<VerifiedComponentSplit>),
}
impl State {
    pub(crate) fn pending(&self) -> bool {
        !matches!(
            self,
            Self::Expanded { .. } | Self::Complete(_) | Self::ZeroIdeal(_)
        )
    }
}
#[derive(Clone, Debug)]
pub enum BoundaryMaximality {
    Unit {
        divisor: usize,
    },
    Residual {
        divisor: usize,
        split: Arc<VerifiedComponentSplit>,
    },
}
/// All boundary components have finite extracted powers on this algebraic open.
/// A residual need not be a unit; it is not identically zero along a component.
#[derive(Clone, Debug)]
pub struct FactorLeaf {
    pub(crate) data: FactorData,
    pub(crate) maximality: Vec<BoundaryMaximality>,
}
impl FactorLeaf {
    pub fn data(&self) -> &FactorData {
        &self.data
    }
    pub fn maximality(&self) -> &[BoundaryMaximality] {
        &self.maximality
    }
}
#[derive(Clone, Debug)]
pub struct FactorNode {
    pub(crate) data: FactorData,
    pub(crate) parent: Option<Arc<LocalizedHistory>>,
    pub(crate) state: State,
    pub(crate) evidence: Option<FactorPendingEvidence>,
}
impl FactorNode {
    pub fn data(&self) -> &FactorData {
        &self.data
    }
    pub fn parent(&self) -> Option<&Arc<LocalizedHistory>> {
        self.parent.as_ref()
    }
    pub fn evidence(&self) -> Option<&FactorPendingEvidence> {
        self.evidence.as_ref()
    }
    pub fn leaf(&self) -> Option<&Arc<FactorLeaf>> {
        if let State::Complete(v) = &self.state {
            Some(v)
        } else {
            None
        }
    }
    pub fn zero_ideal(&self) -> Option<&Arc<VerifiedComponentSplit>> {
        if let State::ZeroIdeal(v) = &self.state {
            Some(v)
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
pub enum FactorAdvance {
    Progress { spawned: Option<FactorPath> },
    ResourceIncomplete(&'static str),
}
