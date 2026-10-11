//! Caller-stepped first recursion of an ACTUALLY constructed marked problem.
//! Inventory completion is not physical resolution or center gluing.
use super::*;
use symbolica::symbol;

#[derive(Clone, Debug)]
pub struct PreparedProblemCenter {
    problem: Arc<InitialProblem>,
    cycle: Arc<CycleSnapshot>,
    center: Arc<CompanionCenter>,
}
impl PreparedProblemCenter {
    /// Lift an actually prepared center through its retained physical support.
    /// Missing physical localizations remain explicit refusal outcomes.
    pub fn ascend(self: &Arc<Self>, b: &mut Budget) -> Result<EmbeddedProblemProduction> {
        super::super::blowup::produce_embedded_problem_center(self.clone(), b)
    }
    pub fn problem(&self) -> &Arc<InitialProblem> {
        &self.problem
    }
    pub fn cycle(&self) -> &Arc<CycleSnapshot> {
        &self.cycle
    }
    pub fn center(&self) -> &Arc<CompanionCenter> {
        &self.center
    }
}
#[derive(Clone, Debug)]
pub enum ProblemRecursionRecord {
    Residual(ComponentResidualProduction),
    Monomial(Arc<ComponentResidualOrder>),
    Open(CompanionOpenProduction),
    NeedsContactCover(Arc<CompanionContactOpen>),
    Coefficient(CompanionCoefficientProduction),
    Child(CompanionCenterOutcome),
    Center(Arc<PreparedProblemCenter>),
}
#[derive(Clone, Debug)]
enum ProblemStage {
    Factor(ComponentFactorFrontier),
    Leaf(usize),
    Open {
        leaf: usize,
        index: usize,
        order: Arc<ComponentResidualOrder>,
        cycle: Arc<CycleSnapshot>,
    },
    Child {
        leaf: usize,
        index: usize,
        order: Arc<ComponentResidualOrder>,
        cycle: Arc<CycleSnapshot>,
        driver: CompanionFirstCenter,
    },
    Complete,
    UnresolvedZeroIdealComponent,
}
#[derive(Clone, Debug)]
pub struct InitialProblemRecursion {
    problem: Arc<InitialProblem>,
    factors: Option<Arc<CompletedComponentFactors>>,
    leaves: Vec<Arc<FactorLeaf>>,
    stage: ProblemStage,
    records: Vec<ProblemRecursionRecord>,
    namespace: String,
    accepted_steps: usize,
}
impl InitialProblemRecursion {
    pub(crate) fn new(
        problem: Arc<InitialProblem>,
        namespace: String,
        b: &mut Budget,
    ) -> Result<Self> {
        let factor = ComponentFactorFrontier::new(
            problem.history().clone(),
            problem.source().clone(),
            format!("{namespace}_factor"),
            b,
        )?;
        Ok(Self {
            problem,
            factors: None,
            leaves: vec![],
            stage: ProblemStage::Factor(factor),
            records: vec![],
            namespace,
            accepted_steps: 0,
        })
    }
    pub fn problem(&self) -> &Arc<InitialProblem> {
        &self.problem
    }
    pub fn records(&self) -> &[ProblemRecursionRecord] {
        &self.records
    }
    pub fn factors(&self) -> Option<&Arc<CompletedComponentFactors>> {
        self.factors.as_ref()
    }
    pub fn complete_inventory(&self) -> bool {
        matches!(self.stage, ProblemStage::Complete)
    }
    /// At least one unresolved source component; other factor leaves remain
    /// retained too. This is neither a complete inventory nor a zero density.
    pub fn unresolved_zero_ideal_component(&self) -> bool {
        matches!(self.stage, ProblemStage::UnresolvedZeroIdealComponent)
    }
    pub fn accepted_steps(&self) -> usize {
        self.accepted_steps
    }
    /// Atomic caller-owned progress. Missing geometry remains in the inventory;
    /// resource-incomplete inner work is recomputed on retry, not checkpointed.
    pub fn advance(
        &mut self,
        limits: &ComponentFactorLimits,
        b: &mut Budget,
    ) -> Result<RecursiveAdvance> {
        if self.unresolved_zero_ideal_component() {
            return Ok(RecursiveAdvance::Incomplete {
                reason: "unresolved zero-ideal component with retained factor inventory",
            });
        }
        if self.complete_inventory() {
            return Ok(RecursiveAdvance::Complete);
        }
        let mut proposed = self.clone();
        match proposed.step(limits, b) {
            Ok(()) => {
                proposed.accepted_steps = proposed
                    .accepted_steps
                    .checked_add(1)
                    .ok_or(Error::ResourceIncomplete("initial problem step count"))?;
                *self = proposed;
                Ok(if self.unresolved_zero_ideal_component() {
                    RecursiveAdvance::Incomplete {
                        reason: "unresolved zero-ideal component with retained factor inventory",
                    }
                } else if self.complete_inventory() {
                    RecursiveAdvance::Complete
                } else {
                    RecursiveAdvance::Progress
                })
            }
            Err(Error::ResourceIncomplete(reason)) => Ok(RecursiveAdvance::Incomplete { reason }),
            Err(e) => Err(e),
        }
    }
    fn record(&mut self, record: ProblemRecursionRecord, b: &mut Budget) -> Result<()> {
        b.reserve_slots(1)?;
        self.records.push(record);
        Ok(())
    }
    fn step(&mut self, limits: &ComponentFactorLimits, b: &mut Budget) -> Result<()> {
        match self.stage.clone() {
            ProblemStage::Factor(mut factor) => {
                let next = factor.pending().next().cloned();
                if let Some(path) = next {
                    if let FactorAdvance::ResourceIncomplete(reason) =
                        factor.advance(&path, limits, b)?
                    {
                        return Err(Error::ResourceIncomplete(reason));
                    }
                    self.stage = ProblemStage::Factor(factor);
                } else {
                    let ComponentFactorCompletion::Complete(done) = factor.try_complete()? else {
                        return Err(Error::Invalid("initial problem factor inventory"));
                    };
                    let done = Arc::new(done);
                    if done.nodes().values().any(|n| n.zero_ideal().is_some()) {
                        self.stage = ProblemStage::UnresolvedZeroIdealComponent;
                    } else {
                        b.reserve_slots(done.nodes().len())?;
                        self.leaves = done
                            .nodes()
                            .values()
                            .filter_map(|n| n.leaf().cloned())
                            .collect();
                        self.stage = ProblemStage::Leaf(0);
                    }
                    self.factors = Some(done);
                }
            }
            ProblemStage::Leaf(leaf) => {
                let Some(source) = self.leaves.get(leaf).cloned() else {
                    self.stage = ProblemStage::Complete;
                    return Ok(());
                };
                match produce_component_residual_order(source, b)? {
                    ComponentResidualProduction::Order(order) => {
                        let order = Arc::new(*order);
                        self.problem.check_order(&order)?;
                        if order.algebraic_maximum_on_cosupport() == 0 {
                            self.record(ProblemRecursionRecord::Monomial(order), b)?;
                            self.stage =
                                ProblemStage::Leaf(leaf.checked_add(1).ok_or(
                                    Error::ResourceIncomplete("initial problem leaf count"),
                                )?);
                        } else {
                            let cycle =
                                CycleSnapshot::initial(self.problem.clone(), order.clone(), b)?;
                            self.stage = ProblemStage::Open {
                                leaf,
                                index: 0,
                                order,
                                cycle,
                            };
                        }
                    }
                    ComponentResidualProduction::Incomplete { reason, .. } => {
                        return Err(Error::ResourceIncomplete(reason));
                    }
                    evidence => {
                        self.record(ProblemRecursionRecord::Residual(evidence), b)?;
                        self.stage = ProblemStage::Leaf(
                            leaf.checked_add(1)
                                .ok_or(Error::ResourceIncomplete("initial problem leaf count"))?,
                        );
                    }
                }
            }
            ProblemStage::Open {
                leaf,
                index,
                order,
                cycle,
            } => {
                if index == order.upper_order_cover().opens().len() {
                    self.stage = ProblemStage::Leaf(
                        leaf.checked_add(1)
                            .ok_or(Error::ResourceIncomplete("initial problem leaf count"))?,
                    );
                    return Ok(());
                }
                let name = format!("{}_l{leaf}_o{index}", self.namespace);
                let next_index = index
                    .checked_add(1)
                    .ok_or(Error::ResourceIncomplete("initial problem open count"))?;
                self.stage = ProblemStage::Open {
                    leaf,
                    index: next_index,
                    order: order.clone(),
                    cycle: cycle.clone(),
                };
                let open = match produce_companion_open(order.clone(), index, &name, b)? {
                    CompanionOpenProduction::Contact(open) => Arc::new(*open),
                    CompanionOpenProduction::Incomplete { reason, .. } => {
                        return Err(Error::ResourceIncomplete(reason));
                    }
                    evidence => {
                        self.record(ProblemRecursionRecord::Open(evidence), b)?;
                        return Ok(());
                    }
                };
                let local = open.contact().frame().local();
                let mut selected = None;
                for (i, c) in open.contact().candidates().iter().enumerate() {
                    let ideal = local.ideal().sum(
                        &Ideal::new(local.ring().clone(), vec![c.differential.clone()], b)?,
                        b,
                    )?;
                    if ideal.contains(&local.ring().one(), local.unit_relations(), b)? {
                        selected = Some(i);
                        break;
                    }
                }
                let Some(selected) = selected else {
                    self.record(ProblemRecursionRecord::NeedsContactCover(open), b)?;
                    return Ok(());
                };
                match construct_companion_coefficient_for_cycle(
                    open,
                    selected,
                    [
                        symbol!(format!("{name}_contact")),
                        symbol!(format!("{name}_inverse")),
                    ],
                    cycle.clone(),
                    b,
                )? {
                    CompanionCoefficientProduction::Coefficient(q) => {
                        let driver = CompanionFirstCenter::new(
                            Arc::new(*q),
                            &format!("{name}_child"),
                            false,
                        )?;
                        self.stage = ProblemStage::Child {
                            leaf,
                            index: next_index,
                            order,
                            cycle,
                            driver,
                        };
                    }
                    evidence => self.record(ProblemRecursionRecord::Coefficient(evidence), b)?,
                }
            }
            ProblemStage::Child {
                leaf,
                index,
                order,
                cycle,
                mut driver,
            } => match driver.advance(b)? {
                RecursiveAdvance::Progress => {
                    self.stage = ProblemStage::Child {
                        leaf,
                        index,
                        order,
                        cycle,
                        driver,
                    }
                }
                RecursiveAdvance::Incomplete { reason } => {
                    return Err(Error::ResourceIncomplete(reason));
                }
                RecursiveAdvance::Complete => {
                    match driver.outcome() {
                        CompanionCenterOutcome::Center(center) => self.record(
                            ProblemRecursionRecord::Center(Arc::new(PreparedProblemCenter {
                                problem: self.problem.clone(),
                                cycle: cycle.clone(),
                                center: center.clone(),
                            })),
                            b,
                        )?,
                        evidence => {
                            self.record(ProblemRecursionRecord::Child(evidence.clone()), b)?
                        }
                    }
                    self.stage = ProblemStage::Open {
                        leaf,
                        index,
                        order,
                        cycle,
                    };
                }
            },
            ProblemStage::Complete | ProblemStage::UnresolvedZeroIdealComponent => {
                return Err(Error::Invalid("accepted initial problem inventory"));
            }
        }
        Ok(())
    }
}
