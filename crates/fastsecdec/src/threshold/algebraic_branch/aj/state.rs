use super::super::quasiordinary::QuasiOrdinaryPolynomial;
use super::valuation::{LocalGerm, Valuations};
use super::*;
use std::collections::VecDeque;
use symbolica::symbol;

pub struct GermDiscriminant {
    source: Arc<MonicPolynomial>,
    native: Poly,
    evidence: Valuations,
}
impl GermDiscriminant {
    pub fn source(&self) -> &Arc<MonicPolynomial> {
        &self.source
    }
    pub fn native(&self) -> &Poly {
        &self.native
    }
    pub fn powers(&self) -> &[usize] {
        &self.evidence.powers[0]
    }
    pub fn unit(&self) -> &Poly {
        &self.evidence.residuals[0]
    }
    pub fn point(&self) -> &Arc<GermPoint> {
        &self.evidence.germ.point
    }
    pub fn quotient_receipts(&self) -> &[VerifiedCartierQuotient] {
        &self.evidence.quotients
    }
}
pub struct DegreeStep {
    source_degree: usize,
    discriminant: Arc<GermDiscriminant>,
    shift: Poly,
    beta: Vec<Rational>,
    change: Arc<PowerChange>,
    clusters: Arc<FiberClusters>,
    factor: Arc<Factorization>,
    child_degree: usize,
    valuation: Arc<Valuations>,
}
impl DegreeStep {
    pub fn source_degree(&self) -> usize {
        self.source_degree
    }
    pub fn child_degree(&self) -> usize {
        self.child_degree
    }
    pub fn discriminant(&self) -> &Arc<GermDiscriminant> {
        &self.discriminant
    }
    pub fn shift(&self) -> &Poly {
        &self.shift
    }
    pub fn beta(&self) -> &[Rational] {
        &self.beta
    }
    pub fn change(&self) -> &Arc<PowerChange> {
        &self.change
    }
    pub fn clusters(&self) -> &Arc<FiberClusters> {
        &self.clusters
    }
    pub fn factor(&self) -> &Arc<Factorization> {
        &self.factor
    }
    pub fn valuation_quotients(&self) -> &[VerifiedCartierQuotient] {
        &self.valuation.quotients
    }
}
/// A real algebraic root germ after a retained sequence of computed signed
/// ramification and exact factor maps. No unique original CAD branch is claimed.
pub struct RootGerm {
    original: Arc<QuasiOrdinaryPolynomial>,
    point: Arc<GermPoint>,
    numerator: Poly,
    denominator: Poly,
    steps: Vec<Arc<DegreeStep>>,
    terminal_coefficients: Vec<Poly>,
    original_coefficients: Vec<Poly>,
    original_equation_residual: Poly,
}
impl RootGerm {
    pub fn original(&self) -> &Arc<QuasiOrdinaryPolynomial> {
        &self.original
    }
    pub fn point(&self) -> &Arc<GermPoint> {
        &self.point
    }
    pub fn numerator(&self) -> &Poly {
        &self.numerator
    }
    pub fn denominator(&self) -> &Poly {
        &self.denominator
    }
    pub fn steps(&self) -> &[Arc<DegreeStep>] {
        &self.steps
    }
    pub fn terminal_coefficients(&self) -> &[Poly] {
        &self.terminal_coefficients
    }
    pub fn original_coefficients(&self) -> &[Poly] {
        &self.original_coefficients
    }
    pub fn original_equation_residual(&self) -> &Poly {
        &self.original_equation_residual
    }
}
#[derive(Clone)]
struct Node {
    germ: LocalGerm,
    coefficients: Vec<Poly>,
    original_coefficients: Vec<Poly>,
    offset: Poly,
    scale: Poly,
    original_scale: Poly,
    steps: Vec<Arc<DegreeStep>>,
}
pub struct NonRealFiber {
    pub change: Arc<PowerChange>,
    pub clusters: Arc<FiberClusters>,
    pub prior: Vec<Arc<DegreeStep>>,
}
pub enum Advance {
    Progress,
    Complete,
    ResourceIncomplete { reason: &'static str },
    CoordinatePreparationRequired,
    MonomialGeneratorUnresolved,
    DiscriminantUnitUnresolved,
}
/// Caller-stepped degree induction. A failed step leaves the accepted frontier
/// intact; native inner work is not a durable checkpoint and is not uncharged.
pub struct AjRecursion {
    original: Arc<QuasiOrdinaryPolynomial>,
    pending: VecDeque<Node>,
    roots: Vec<Arc<RootGerm>>,
    nonreal: Vec<Arc<NonRealFiber>>,
    namespace: String,
    accepted_steps: usize,
}
impl AjRecursion {
    pub fn new(
        original: Arc<QuasiOrdinaryPolynomial>,
        point: Arc<GermPoint>,
        namespace: &str,
    ) -> Result<Self> {
        let germ = LocalGerm::new(point, original.ledger().clone())?;
        let ring = germ.point.frame().local().ring();
        let node = Node {
            germ: germ.clone(),
            coefficients: original.prepared().monic().coefficients().to_vec(),
            original_coefficients: original
                .prepared()
                .cleared()
                .source()
                .coefficients()
                .to_vec(),
            offset: ring.one().zero(),
            scale: ring.one(),
            original_scale: original.prepared().root_scale().clone(),
            steps: vec![],
        };
        Ok(Self {
            original,
            pending: VecDeque::from([node]),
            roots: vec![],
            nonreal: vec![],
            namespace: namespace.into(),
            accepted_steps: 0,
        })
    }
    pub fn original(&self) -> &Arc<QuasiOrdinaryPolynomial> {
        &self.original
    }
    pub fn roots(&self) -> &[Arc<RootGerm>] {
        &self.roots
    }
    pub fn nonreal_fibers(&self) -> &[Arc<NonRealFiber>] {
        &self.nonreal
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn accepted_steps(&self) -> usize {
        self.accepted_steps
    }
    pub fn advance(&mut self, b: &mut Budget) -> Result<Advance> {
        let Some(node) = self.pending.front().cloned() else {
            return Ok(Advance::Complete);
        };
        let next_count = self
            .accepted_steps
            .checked_add(1)
            .ok_or(Error::ResourceIncomplete("AJ step count"))?;
        let result = step(
            node,
            self.original.clone(),
            &format!("{}::n{}", self.namespace, self.accepted_steps),
            b,
        );
        let result = match result {
            Err(Error::ResourceIncomplete(reason)) => {
                return Ok(Advance::ResourceIncomplete { reason });
            }
            other => other?,
        };
        match result {
            StepResult::Children { children, nonreal } => {
                self.pending.pop_front();
                self.pending.extend(children);
                self.nonreal.extend(nonreal);
            }
            StepResult::Root(r) => {
                self.pending.pop_front();
                self.roots.push(Arc::from(r));
            }
            StepResult::Coordinates => return Ok(Advance::CoordinatePreparationRequired),
            StepResult::Monomial => return Ok(Advance::MonomialGeneratorUnresolved),
            StepResult::Discriminant => return Ok(Advance::DiscriminantUnitUnresolved),
        }
        self.accepted_steps = next_count;
        Ok(if self.pending.is_empty() {
            Advance::Complete
        } else {
            Advance::Progress
        })
    }
}
enum StepResult {
    Root(Box<RootGerm>),
    Children {
        children: Vec<Node>,
        nonreal: Vec<Arc<NonRealFiber>>,
    },
    Coordinates,
    Monomial,
    Discriminant,
}
fn pull_node(node: &mut Node, open: &Arc<LocalizedHistory>, b: &mut Budget) -> Result<()> {
    for p in node
        .coefficients
        .iter_mut()
        .chain(&mut node.original_coefficients)
    {
        *p = open.open().extension().pull(p, b)?;
    }
    node.offset = open.open().extension().pull(&node.offset, b)?;
    node.scale = open.open().extension().pull(&node.scale, b)?;
    node.original_scale = open.open().extension().pull(&node.original_scale, b)?;
    Ok(())
}
fn snc_after_factor(
    branch: &Arc<Factorization>,
    ledger: &Arc<VerifiedRelativeSnc>,
    b: &mut Budget,
) -> Result<Arc<VerifiedRelativeSnc>> {
    let divisors = ledger
        .divisors()
        .iter()
        .map(|h| {
            Ok(InitialDivisor {
                id: h.id,
                equation: branch.extension().pull(&h.equation, b)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    match verify_initial_relative_snc(branch.frame().clone(), divisors, b)? {
        SncProduction::Verified(l) => Ok(l),
        SncProduction::Incomplete { reason, .. } => Err(Error::ResourceIncomplete(reason)),
        SncProduction::Unresolved { .. } => {
            Err(Error::Invalid("factor extension lost reduced SNC"))
        }
    }
}
fn step(
    mut node: Node,
    original: Arc<QuasiOrdinaryPolynomial>,
    namespace: &str,
    b: &mut Budget,
) -> Result<StepResult> {
    let d = node.coefficients.len() - 1;
    if d == 1 {
        let numerator = &node.offset - &b.mul(&node.scale, &node.coefficients[0])?;
        if node.germ.point.is_zero(&node.original_scale)? {
            return Err(Error::Invalid(
                "terminal original monicization scale vanishes",
            ));
        }
        let mut residual = node
            .original_coefficients
            .last()
            .ok_or(Error::Invalid("original AJ polynomial absent"))?
            .clone();
        let mut denominator_power = node.original_scale.one();
        for c in node.original_coefficients.iter().rev().skip(1) {
            denominator_power = b.mul(&denominator_power, &node.original_scale)?;
            residual = b.mul(&residual, &numerator)? + b.mul(c, &denominator_power)?;
        }
        if !node.germ.point.frame().local().zero(&residual, b)? {
            return Err(Error::Invalid(
                "terminal root fails original exact polynomial",
            ));
        }
        return Ok(StepResult::Root(Box::new(RootGerm {
            original,
            point: node.germ.point,
            numerator,
            denominator: node.original_scale,
            steps: node.steps,
            terminal_coefficients: node.coefficients,
            original_coefficients: node.original_coefficients,
            original_equation_residual: residual,
        })));
    }
    let variable = original.prepared().monic().variable();
    let monic = MonicPolynomial::from_regular_coefficients(
        node.germ.point.frame().clone(),
        variable,
        node.coefficients.clone(),
        b,
    )?;
    let disc = super::super::quasiordinary::discriminant(&monic, b)?;
    if node.germ.point.frame().local().zero(&disc, b)? {
        return Err(Error::Invalid("AJ child polynomial is nonreduced"));
    }
    let old_opens = node.germ.localizations.len();
    let dv = valuation::extract(
        node.germ.clone(),
        vec![disc.clone()],
        &format!("{namespace}::disc"),
        b,
    )?;
    if dv.germ.point.is_zero(&dv.residuals[0])? {
        return Ok(StepResult::Discriminant);
    }
    for open in &dv.germ.localizations[old_opens..] {
        pull_node(&mut node, open, b)?;
    }
    node.germ = dv.germ.clone();
    let discriminant = Arc::new(GermDiscriminant {
        source: monic,
        native: disc,
        evidence: dv,
    });
    let shift = node.coefficients[d - 1].clone().mul_coeff(
        Rational::from(-1)
            / Rational::from(i64::try_from(d).map_err(|_| Error::ResourceIncomplete("AJ degree"))?),
    );
    let p = UnivariatePolynomial::from_coefficients(
        &PolynomialRing::new(Q),
        node.coefficients.clone(),
        Arc::new(PolyVariable::from(variable)),
    );
    b.charge(2)?;
    let depressed = p.shift_var(&shift);
    if !depressed.coefficients()[d - 1].is_zero() || depressed.shift_var(&(-shift.clone())) != p {
        return Err(Error::Invalid("native Tschirnhaus identity"));
    }
    node.offset = &node.offset + &b.mul(&node.scale, &shift)?;
    node.coefficients = depressed.coefficients().to_vec();
    let old_opens = node.germ.localizations.len();
    let valuation = Arc::new(valuation::extract(
        node.germ.clone(),
        node.coefficients.clone(),
        &format!("{namespace}::coeff"),
        b,
    )?);
    let Some(beta) = valuation::monomial_generator(&valuation, d)? else {
        return Ok(StepResult::Monomial);
    };
    for open in &valuation.germ.localizations[old_opens..] {
        pull_node(&mut node, open, b)?;
    }
    node.germ = valuation.germ.clone();
    node.coefficients = valuation.original.clone();
    let q = ramification::denominators(&beta, b)?;
    let signs = ramification::sign_vectors(&q, b)?;
    let mut children = Vec::new();
    let mut nonreal = Vec::new();
    for (s, sign) in signs.into_iter().enumerate() {
        let Some(change) = ramification::change(
            &node.germ,
            &node.coefficients,
            &beta,
            sign,
            &format!("{namespace}::ram{s}"),
            b,
        )?
        else {
            return Ok(StepResult::Coordinates);
        };
        let change = Arc::new(change);
        let reduced = MonicPolynomial::from_regular_coefficients(
            change.point.frame().clone(),
            variable,
            change.coefficients.clone(),
            b,
        )?;
        let clusters = Arc::new(discover_clusters(reduced, change.point.clone(), b)?);
        if clusters.nonreal_multiplicity() > 0 {
            nonreal.push(Arc::new(NonRealFiber {
                change: change.clone(),
                clusters: clusters.clone(),
                prior: node.steps.clone(),
            }));
        }
        for (k, c) in clusters.real().iter().enumerate() {
            let seed = c
                .seed()
                .ok_or(Error::Invalid("AJ scaled fiber did not separate"))?
                .clone();
            let count = if c.multiplicity() == 1 {
                2
            } else {
                d + usize::from(!seed.context().is_trivial()) + 1
            };
            b.reserve_slots(count)?;
            let fresh = (0..count)
                .map(|i| symbol!(format!("{namespace}::s{s}_c{k}_v{i}")))
                .collect::<Vec<_>>();
            let (factor, point) = if c.multiplicity() == 1 {
                let root = SimpleRootBranch::prepare(seed, &fresh, b)?;
                let point = root.point().clone();
                (Arc::new(Factorization::Simple(root)), point)
            } else {
                let factor = FactorBranch::prepare(seed, &fresh, b)?;
                let point = GermPoint::after_factor(&factor, b)?;
                (Arc::new(Factorization::Coprime(factor)), point)
            };
            let ledger = snc_after_factor(&factor, &change.ledger, b)?;
            let germ = LocalGerm::new(point, ledger)?;
            let pull = |p: &Poly, b: &mut Budget| -> Result<Poly> {
                let q = if let Some(e) = &change.extension {
                    e.pull(p, b)?
                } else {
                    p.clone()
                };
                factor.extension().pull(&q, b)
            };
            let offset = pull(&node.offset, b)?;
            let scale = pull(&node.scale, b)?;
            let rs = factor.extension().pull(&change.root_scale, b)?;
            let scale = b.mul(&scale, &rs)?;
            let original_scale = pull(&node.original_scale, b)?;
            let coefficients = factor.factors().0.to_vec();
            let child_degree = coefficients.len() - 1;
            if child_degree >= d || child_degree != c.multiplicity() {
                return Err(Error::Invalid("AJ factor degree did not strictly decrease"));
            }
            let mut steps = node.steps.clone();
            steps.push(Arc::new(DegreeStep {
                source_degree: d,
                discriminant: discriminant.clone(),
                shift: shift.clone(),
                beta: beta.clone(),
                change: change.clone(),
                clusters: clusters.clone(),
                factor: factor.clone(),
                child_degree,
                valuation: valuation.clone(),
            }));
            let original_coefficients = node
                .original_coefficients
                .iter()
                .map(|p| pull(p, b))
                .collect::<Result<Vec<_>>>()?;
            children.push(Node {
                germ,
                coefficients,
                original_coefficients,
                offset,
                scale,
                original_scale,
                steps,
            });
        }
    }
    Ok(StepResult::Children { children, nonreal })
}
