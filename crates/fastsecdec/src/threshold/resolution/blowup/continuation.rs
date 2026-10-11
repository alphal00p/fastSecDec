//! Caller-stepped local continuation through the actual carried coefficient
//! problem. Distinct local centers and further nonmonomial cycles stay pending.
use super::super::*;
use super::{
    coefficient::*, general::*, general_transform::*, helpers::Result, induced::*, terminal::*,
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone, Debug)]
pub struct ContinuationLimits {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub factor: ComponentFactorLimits,
}
impl Default for ContinuationLimits {
    fn default() -> Self {
        Self {
            max_nodes: 4096,
            max_depth: 64,
            factor: ComponentFactorLimits::default(),
        }
    }
}
#[derive(Clone, Debug)]
struct LowerTask {
    induced: Arc<InducedChildChart>,
    frontier: ComponentFactorFrontier,
    done: Option<Arc<CompletedComponentFactors>>,
}
#[derive(Clone, Debug)]
enum Stage {
    Factor(ComponentFactorFrontier),
    Classify(Arc<CompletedComponentFactors>),
    Carry {
        factors: Arc<CompletedComponentFactors>,
    },
    Lower {
        factors: Arc<CompletedComponentFactors>,
        carry: Arc<CarriedCompanionChart>,
        tasks: Vec<LowerTask>,
    },
    Expand {
        factors: Arc<CompletedComponentFactors>,
        carry: Arc<CarriedCompanionChart>,
        tasks: Vec<LowerTask>,
        centers: Vec<Arc<CarriedMonomialCenter>>,
    },
    Expanded {
        factors: Arc<CompletedComponentFactors>,
        carry: Arc<CarriedCompanionChart>,
        tasks: Vec<LowerTask>,
        centers: Vec<Arc<CarriedMonomialCenter>>,
        blowup: Arc<RelativeRecursiveBlowup>,
        children: Vec<Vec<usize>>,
    },
    Terminal {
        factors: Arc<CompletedComponentFactors>,
        certificates: Vec<Arc<LocalPrincipalization>>,
    },
}
#[derive(Clone, Debug)]
pub struct ContinuationNode {
    chart: Arc<RelativeRecursiveChart>,
    origin: Option<Arc<EmbeddedChildCycle>>,
    stage: Stage,
    pending: Option<&'static str>,
}
impl ContinuationNode {
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn pending_reason(&self) -> Option<&'static str> {
        self.pending
    }
    pub fn terminal(&self) -> bool {
        matches!(self.stage, Stage::Terminal { .. })
    }
    pub fn certificates(&self) -> &[Arc<LocalPrincipalization>] {
        match &self.stage {
            Stage::Terminal { certificates, .. } => certificates,
            _ => &[],
        }
    }
    fn actionable(&self) -> bool {
        !matches!(self.stage, Stage::Terminal { .. } | Stage::Expanded { .. })
    }
}
#[derive(Clone, Debug)]
pub struct LocalCompanionContinuation {
    origin: Arc<EmbeddedChildCycle>,
    initial: Arc<RelativeRecursiveBlowup>,
    nodes: BTreeMap<Vec<usize>, ContinuationNode>,
    namespace: String,
}
#[derive(Clone, Debug)]
pub enum ContinuationAdvance {
    Progress,
    Incomplete(&'static str),
}
#[derive(Debug)]
pub struct CompletedLocalContinuation {
    frontier: LocalCompanionContinuation,
}
impl CompletedLocalContinuation {
    pub fn origin(&self) -> &Arc<EmbeddedChildCycle> {
        &self.frontier.origin
    }
    pub fn initial(&self) -> &Arc<RelativeRecursiveBlowup> {
        &self.frontier.initial
    }
    pub fn nodes(&self) -> &BTreeMap<Vec<usize>, ContinuationNode> {
        &self.frontier.nodes
    }
}
pub enum ContinuationCompletion {
    Complete(CompletedLocalContinuation),
    Incomplete(LocalCompanionContinuation),
}
impl LocalCompanionContinuation {
    pub fn new(center: Arc<CompanionCenter>, namespace: String, b: &mut Budget) -> Result<Self> {
        let origin = EmbeddedChildCycle::new(center.clone(), b)?;
        let checked = CheckedRecursiveCenter::new(RecursiveCenterOrigin::Companion(center), b)?;
        let cover = adapt_recursive_center(checked, &format!("{namespace}_adapt"), b)?;
        let initial = blowup_recursive_center(cover, &format!("{namespace}_blowup"), b)?;
        b.reserve_slots(initial.charts().len())?;
        let mut nodes = BTreeMap::new();
        for (i, chart) in initial.charts().iter().enumerate() {
            nodes.insert(
                vec![i],
                node(
                    chart.clone(),
                    Some(origin.clone()),
                    &format!("{namespace}_{i}"),
                    b,
                )?,
            );
        }
        Ok(Self {
            origin,
            initial,
            nodes,
            namespace,
        })
    }
    pub fn nodes(&self) -> &BTreeMap<Vec<usize>, ContinuationNode> {
        &self.nodes
    }
    pub fn pending(&self) -> impl Iterator<Item = &Vec<usize>> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.actionable())
            .map(|(p, _)| p)
    }
    pub fn advance(
        &mut self,
        path: &[usize],
        limits: &ContinuationLimits,
        b: &mut Budget,
    ) -> Result<ContinuationAdvance> {
        let current = self
            .nodes
            .get(path)
            .ok_or(Error::Invalid("unknown continuation path"))?;
        if !current.actionable() {
            return Err(Error::Invalid("accepted continuation path"));
        }
        if path.len() > limits.max_depth || self.nodes.len() > limits.max_nodes {
            return Ok(ContinuationAdvance::Incomplete(
                "local continuation node/depth cap",
            ));
        }
        let name = format!(
            "{}_{}",
            self.namespace,
            path.iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("_")
        );
        let mut proposed = current.clone();
        proposed.pending = None;
        let result = step(&mut proposed, path, &name, limits, self.nodes.len(), b);
        match result {
            Ok(children) => {
                for (p, _) in &children {
                    if self.nodes.contains_key(p) {
                        return Err(Error::Invalid("duplicate continuation child"));
                    }
                }
                let reason = proposed.pending;
                self.nodes.insert(path.to_vec(), proposed);
                self.nodes.extend(children);
                Ok(reason.map_or(
                    ContinuationAdvance::Progress,
                    ContinuationAdvance::Incomplete,
                ))
            }
            Err(Error::ResourceIncomplete(reason)) => {
                self.nodes
                    .get_mut(path)
                    .ok_or(Error::Invalid("lost continuation node"))?
                    .pending = Some(reason);
                Ok(ContinuationAdvance::Incomplete(reason))
            }
            Err(e) => Err(e),
        }
    }
    pub fn try_complete(self) -> Result<ContinuationCompletion> {
        if self.nodes.values().any(|n| n.actionable()) {
            return Ok(ContinuationCompletion::Incomplete(self));
        }
        for (i, chart) in self.initial.charts().iter().enumerate() {
            if !self
                .nodes
                .get(&vec![i])
                .is_some_and(|n| Arc::ptr_eq(&n.chart, chart))
            {
                return Err(Error::Invalid("local continuation initial coverage"));
            }
        }
        for node in self.nodes.values() {
            match &node.stage {
                Stage::Terminal {
                    factors,
                    certificates,
                } => {
                    if factors.nodes().values().any(|n| n.zero_ideal().is_some())
                        || factors.nodes().values().filter_map(|n| n.leaf()).count()
                            != certificates.len()
                    {
                        return Err(Error::Invalid("terminal factor coverage"));
                    }
                    for leaf in factors.nodes().values().filter_map(|n| n.leaf()) {
                        if !certificates.iter().any(|c| Arc::ptr_eq(c.factor(), leaf)) {
                            return Err(Error::Invalid("terminal certificate owner coverage"));
                        }
                    }
                }
                Stage::Expanded {
                    factors,
                    carry,
                    tasks,
                    centers,
                    blowup,
                    children,
                } => {
                    if centers.is_empty()
                        || tasks.len() != carry.opens().len()
                        || factors.nodes().is_empty()
                        || children.len() != blowup.charts().len()
                    {
                        return Err(Error::Invalid("expanded continuation evidence"));
                    }
                    for (p, chart) in children.iter().zip(blowup.charts()) {
                        if !self
                            .nodes
                            .get(p)
                            .is_some_and(|n| Arc::ptr_eq(&n.chart, chart))
                        {
                            return Err(Error::Invalid("expanded continuation chart coverage"));
                        }
                    }
                }
                _ => return Err(Error::Invalid("unaccepted continuation state")),
            }
        }
        Ok(ContinuationCompletion::Complete(
            CompletedLocalContinuation { frontier: self },
        ))
    }
}
fn node(
    chart: Arc<RelativeRecursiveChart>,
    origin: Option<Arc<EmbeddedChildCycle>>,
    name: &str,
    b: &mut Budget,
) -> Result<ContinuationNode> {
    let factors = ComponentFactorFrontier::new(
        chart.history().clone(),
        chart.parent().target().clone(),
        format!("{name}_factor"),
        b,
    )?;
    Ok(ContinuationNode {
        chart,
        origin,
        stage: Stage::Factor(factors),
        pending: None,
    })
}
fn finish_factor(
    frontier: &mut ComponentFactorFrontier,
    limits: &ComponentFactorLimits,
    b: &mut Budget,
) -> Result<Option<Arc<CompletedComponentFactors>>> {
    let path = frontier.pending().next().cloned();
    if let Some(path) = path {
        match frontier.advance(&path, limits, b)? {
            FactorAdvance::Progress { .. } => return Ok(None),
            FactorAdvance::ResourceIncomplete(reason) => {
                return Err(Error::ResourceIncomplete(reason));
            }
        }
    }
    match frontier.clone().try_complete()? {
        ComponentFactorCompletion::Complete(done) => Ok(Some(Arc::new(done))),
        ComponentFactorCompletion::Incomplete(_) => {
            Err(Error::Invalid("factor pending exhaustion"))
        }
    }
}
fn step(
    node: &mut ContinuationNode,
    path: &[usize],
    name: &str,
    limits: &ContinuationLimits,
    count: usize,
    b: &mut Budget,
) -> Result<Vec<(Vec<usize>, ContinuationNode)>> {
    match &mut node.stage {
        Stage::Factor(frontier) => {
            if let Some(done) = finish_factor(frontier, &limits.factor, b)? {
                node.stage = Stage::Classify(done);
            }
        }
        Stage::Classify(factors) => {
            if factors.nodes().values().any(|n| n.zero_ideal().is_some()) {
                node.pending = Some("zero ideal component needs separate continuation");
                return Ok(vec![]);
            }
            let mut certificates = Vec::new();
            let mut further = false;
            for (i, leaf) in factors
                .nodes()
                .values()
                .filter_map(|n| n.leaf())
                .enumerate()
            {
                match certify_local_principalization(
                    leaf.clone(),
                    &format!("{name}_principal{i}"),
                    b,
                )? {
                    PrincipalizationProduction::Principal(c) => certificates.push(c),
                    PrincipalizationProduction::FurtherResolution { .. } => further = true,
                }
            }
            if !further {
                node.stage = Stage::Terminal {
                    factors: factors.clone(),
                    certificates,
                };
            } else if node.origin.is_some() {
                node.stage = Stage::Carry {
                    factors: factors.clone(),
                };
            } else {
                node.pending = Some("further carried coefficient or companion cycle required");
            }
        }
        Stage::Carry { factors } => {
            let origin = node
                .origin
                .as_ref()
                .ok_or(Error::Invalid("lost original child cycle"))?;
            let carry = carry_companion_chart(node.chart.clone(), &format!("{name}_carry"), b)?;
            let mut tasks = Vec::new();
            b.reserve_slots(carry.opens().len())?;
            for (i, open) in carry.opens().iter().enumerate() {
                let induced = induce_child_chart(
                    origin.clone(),
                    carry.clone(),
                    i,
                    &format!("{name}_induced{i}"),
                    b,
                )?;
                let frontier = ComponentFactorFrontier::new(
                    induced.history().clone(),
                    open.incidence_sum().target().clone(),
                    format!("{name}_lower_factor{i}"),
                    b,
                )?;
                tasks.push(LowerTask {
                    induced,
                    frontier,
                    done: None,
                });
            }
            node.stage = Stage::Lower {
                factors: factors.clone(),
                carry,
                tasks,
            };
        }
        Stage::Lower {
            factors,
            carry,
            tasks,
        } => {
            if let Some(task) = tasks.iter_mut().find(|t| t.done.is_none()) {
                task.done = finish_factor(&mut task.frontier, &limits.factor, b)?;
                return Ok(vec![]);
            }
            let mut centers = Vec::new();
            for task in tasks.iter() {
                let done = task
                    .done
                    .as_ref()
                    .ok_or(Error::Invalid("lower coverage unfinished"))?;
                if done.nodes().values().any(|n| n.zero_ideal().is_some()) {
                    node.pending = Some("zero lower ideal needs full-support continuation");
                    return Ok(vec![]);
                }
                for leaf in done.nodes().values().filter_map(|n| n.leaf()) {
                    match produce_induced_monomial_center(task.induced.clone(), leaf.clone(), b)? {
                        InducedCenterProduction::Center(c) => centers.push(c),
                        InducedCenterProduction::Empty { .. } => {}
                        InducedCenterProduction::NeedsLowerRecursion { .. } => {
                            node.pending = Some("nonmonomial lower coefficient recursion required");
                            return Ok(vec![]);
                        }
                        InducedCenterProduction::NeedsLocalization { .. } => {
                            node.pending =
                                Some("induced center needs compatible ambient localization");
                            return Ok(vec![]);
                        }
                    }
                }
            }
            if centers.is_empty() {
                node.pending = Some("resolved lower J needs incidence/drop continuation");
                return Ok(vec![]);
            }
            // Every retained support/minor open and complete factor leaf was
            // examined. This restricted path admits only a single globally
            // defined center; distinct local centers require actual gluing.
            let first = &centers[0];
            let local = first.frame().local();
            for center in &centers {
                if !Arc::ptr_eq(first.frame(), center.frame()) {
                    return Err(Error::Invalid("carried candidate ambient mismatch"));
                }
                for (a, z) in [
                    (first.ideal(), center.ideal()),
                    (center.ideal(), first.ideal()),
                ] {
                    let total = local.ideal().sum(z, b)?;
                    for f in a.generators() {
                        if !total.contains(f, local.unit_relations(), b)? {
                            node.pending = Some("distinct lower centers require gluing");
                            return Ok(vec![]);
                        }
                    }
                }
            }
            node.stage = Stage::Expand {
                factors: factors.clone(),
                carry: carry.clone(),
                tasks: tasks.clone(),
                centers,
            };
        }
        Stage::Expand {
            factors,
            carry,
            tasks,
            centers,
        } => {
            if path.len() >= limits.max_depth {
                return Err(Error::ResourceIncomplete("local continuation depth cap"));
            }
            let selected = centers
                .first()
                .ok_or(Error::Invalid("missing carried center"))?
                .clone();
            let checked =
                CheckedRecursiveCenter::new(RecursiveCenterOrigin::CarriedMonomial(selected), b)?;
            let cover = adapt_recursive_center(checked, &format!("{name}_next_adapt"), b)?;
            let blowup = blowup_recursive_center(cover, &format!("{name}_next_blowup"), b)?;
            if count
                .checked_add(blowup.charts().len())
                .is_none_or(|n| n > limits.max_nodes)
            {
                return Err(Error::ResourceIncomplete("local continuation node cap"));
            }
            b.reserve_slots(blowup.charts().len())?;
            let mut children = Vec::new();
            let mut created = Vec::new();
            for (i, chart) in blowup.charts().iter().enumerate() {
                let mut child = path.to_vec();
                child.push(i);
                children.push(child.clone());
                created.push((
                    child,
                    node_new(chart.clone(), &format!("{name}_next{i}"), b)?,
                ));
            }
            node.stage = Stage::Expanded {
                factors: factors.clone(),
                carry: carry.clone(),
                tasks: tasks.clone(),
                centers: centers.clone(),
                blowup,
                children,
            };
            return Ok(created);
        }
        Stage::Terminal { .. } | Stage::Expanded { .. } => {
            return Err(Error::Invalid("accepted local continuation step"));
        }
    }
    Ok(vec![])
}
fn node_new(
    chart: Arc<RelativeRecursiveChart>,
    name: &str,
    b: &mut Budget,
) -> Result<ContinuationNode> {
    node(chart, None, name, b)
}
