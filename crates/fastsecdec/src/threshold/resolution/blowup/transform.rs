use super::super::*;
use super::{helpers::*, standard::PolynomialPullback, *};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct DivisorReceipt {
    pub id: BoundaryId,
    pub total: Poly,
    pub exceptional_power: usize,
    pub strict_equation: Poly,
    pub unit: Poly,
    pub absent: bool,
}
#[derive(Clone, Debug)]
pub struct MonomialBlowupChart {
    open: Arc<AdaptedOpen>,
    pivot: Option<usize>,
    frame: Arc<EtaleFrame>,
    target: Arc<MarkedIdeal>,
    history: Arc<ResolutionHistory>,
    pullback: PolynomialPullback,
    jacobian: Poly,
    original_coordinate_jacobian: Poly,
    exceptional: Option<Poly>,
    exceptional_is_empty: bool,
    divisors: Vec<DivisorReceipt>,
    recombinations: usize,
}
impl MonomialBlowupChart {
    pub fn open(&self) -> &Arc<AdaptedOpen> {
        &self.open
    }
    pub fn pivot(&self) -> Option<usize> {
        self.pivot
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn target(&self) -> &Arc<MarkedIdeal> {
        &self.target
    }
    pub fn history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn jacobian(&self) -> &Poly {
        &self.jacobian
    }
    /// Relative Jacobian in the source's original free coordinate frame.
    /// This does not include any earlier projective/integration chart map.
    pub fn original_coordinate_jacobian(&self) -> &Poly {
        &self.original_coordinate_jacobian
    }
    pub fn exceptional(&self) -> Option<&Poly> {
        self.exceptional.as_ref()
    }
    pub fn exceptional_is_empty(&self) -> bool {
        self.exceptional_is_empty
    }
    pub fn divisors(&self) -> &[DivisorReceipt] {
        &self.divisors
    }
    pub fn recombinations(&self) -> usize {
        self.recombinations
    }
    /// Pull a polynomial already expressed in this chart's adapted graph ring.
    pub fn pull_adapted(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        self.pullback.pull(p, b)
    }
    /// Compose source-to-adapted embedding with the checked blowup pullback.
    pub fn pull_from_source(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        self.pullback.pull(&self.open.extension.pull(p, b)?, b)
    }
}
#[derive(Clone, Debug, Default)]
pub struct BlowupProgress {
    pub completed: Vec<Arc<MonomialBlowupChart>>,
    pub pending_boundary: Option<SncProduction>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct MonomialBlowup {
    source: Arc<ProducedMonomialCenter>,
    history: Arc<ResolutionHistory>,
    cover: Arc<AdaptedCover>,
    born: BoundaryId,
    stage: u64,
    progress: BlowupProgress,
}
impl MonomialBlowup {
    pub fn source(&self) -> &Arc<ProducedMonomialCenter> {
        &self.source
    }
    pub fn prior_history(&self) -> &Arc<ResolutionHistory> {
        &self.history
    }
    pub fn cover(&self) -> &Arc<AdaptedCover> {
        &self.cover
    }
    pub fn born(&self) -> (BoundaryId, u64) {
        (self.born, self.stage)
    }
    pub fn charts(&self) -> &[Arc<MonomialBlowupChart>] {
        &self.progress.completed
    }
    pub fn progress(&self) -> &BlowupProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum BlowupProduction {
    Complete(Box<MonomialBlowup>),
    Incomplete {
        source: Arc<ProducedMonomialCenter>,
        history: Arc<ResolutionHistory>,
        cover: Arc<AdaptedCover>,
        reason: &'static str,
        progress: BlowupProgress,
    },
}
pub fn produce_monomial_blowup(
    source: Arc<ProducedMonomialCenter>,
    history: Arc<ResolutionHistory>,
    cover: Arc<AdaptedCover>,
    namespace: &str,
    b: &mut Budget,
) -> Result<BlowupProduction> {
    if !Arc::ptr_eq(source.ledger(), history.ledger())
        || !Arc::ptr_eq(source.ledger(), cover.source())
    {
        return Err(Error::Invalid("blowup source/history/cover owner"));
    }
    let (born, stage) = history.next_transition()?;
    let mut progress = BlowupProgress::default();
    let result = build(&source, &history, &cover, namespace, b, &mut progress);
    progress.operations = b.operations();
    progress.ideal_slots = b.ideal_slots();
    match result {
        Ok(()) => Ok(BlowupProduction::Complete(Box::new(MonomialBlowup {
            source,
            history,
            cover,
            born,
            stage,
            progress,
        }))),
        Err(Error::ResourceIncomplete(reason)) => Ok(BlowupProduction::Incomplete {
            source,
            history,
            cover,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
fn build(
    source: &Arc<ProducedMonomialCenter>,
    history: &Arc<ResolutionHistory>,
    cover: &Arc<AdaptedCover>,
    namespace: &str,
    b: &mut Budget,
    progress: &mut BlowupProgress,
) -> Result<()> {
    for (oi, open) in cover.opens().iter().enumerate() {
        if source.indices().iter().any(|i| !open.incidence.contains(i)) {
            b.reserve_slots(1)?;
            let chart = one(
                source,
                history,
                open,
                None,
                &format!("{namespace}_identity{oi}"),
                b,
                &mut progress.pending_boundary,
            )?;
            progress.completed.push(Arc::new(chart));
        } else {
            for pivot in source.indices() {
                b.reserve_slots(1)?;
                let chart = one(
                    source,
                    history,
                    open,
                    Some(*pivot),
                    &format!("{namespace}_chart{oi}_{pivot}"),
                    b,
                    &mut progress.pending_boundary,
                )?;
                progress.completed.push(Arc::new(chart));
            }
        }
    }
    Ok(())
}
fn one(
    source: &ProducedMonomialCenter,
    history: &ResolutionHistory,
    open: &Arc<AdaptedOpen>,
    pivot: Option<usize>,
    namespace: &str,
    b: &mut Budget,
    pending_boundary: &mut Option<SncProduction>,
) -> Result<MonomialBlowupChart> {
    let center = source.indices();
    let axis_for = |index: usize| -> Result<usize> {
        open.incidence
            .iter()
            .position(|i| *i == index)
            .map(|j| open.boundary_axes[j])
            .ok_or(Error::Invalid("center missing graph coordinate"))
    };
    // An identity complement need not contain all center boundary axes. Its
    // available coordinate list suffices because no axis is changed.
    let center_axes = if pivot.is_some() {
        center
            .iter()
            .map(|i| axis_for(*i))
            .collect::<Result<Vec<_>>>()?
    } else {
        open.boundary_axes.clone()
    };
    let geometry = super::standard::standard_chart(
        open.geometry.clone(),
        &center_axes,
        pivot.map(axis_for).transpose()?,
        namespace,
        b,
    )?;
    let frame = geometry.frame.clone();
    let ring = frame.local().ring().clone();
    let pullback = geometry.pullback.clone();
    let swaps = geometry.swaps.clone();
    let exceptional = geometry.exceptional.clone();
    let jacobian = geometry.jacobian.clone();
    let original_coordinate_jacobian = geometry.original_coordinate_jacobian.clone();
    let mut receipts = Vec::new();
    let mut active = Vec::new();
    let mut strict = Vec::new();
    for (i, divisor) in source.ledger().divisors().iter().enumerate() {
        let h = open.extension.pull(&divisor.equation, b)?;
        let total = pullback.pull(&h, b)?;
        let in_open = open.incidence.contains(&i);
        let in_center = pivot.is_some() && center.contains(&i);
        let (equation, unit_factor) = if !in_open {
            (ring.one(), total.clone())
        } else if Some(i) == pivot {
            (ring.one(), ring.one())
        } else {
            let axis = axis_for(i)?;
            let new = swaps
                .iter()
                .find(|(old, _)| *old == axis)
                .map_or(axis, |(_, new)| *new);
            (ring.coordinate(new)?, ring.one())
        };
        let exceptional_power = usize::from(in_center);
        let mut recombined = b.mul(&equation, &unit_factor)?;
        if in_center {
            recombined = b.mul(
                &recombined,
                exceptional
                    .as_ref()
                    .ok_or(Error::Invalid("exceptional divisor"))?,
            )?;
        }
        if !frame.local().zero(&(&total - &recombined), b)?
            || !unit(frame.local(), &unit_factor, b)?
        {
            return Err(Error::Invalid("strict divisor recombination/unit"));
        }
        let absent = unit(frame.local(), &equation, b)?;
        if !absent {
            active.push(InitialDivisor {
                id: divisor.id,
                equation: equation.clone(),
            });
        }
        strict.push((equation.clone(), unit_factor.clone()));
        receipts.push(DivisorReceipt {
            id: divisor.id,
            total,
            exceptional_power,
            strict_equation: equation,
            unit: unit_factor,
            absent,
        });
    }
    let (born, stage) = history.next_transition()?;
    let exceptional_is_empty = match &exceptional {
        Some(e) => unit(frame.local(), e, b)?,
        None => true,
    };
    if let Some(e) = &exceptional
        && !exceptional_is_empty
    {
        active.push(InitialDivisor {
            id: born,
            equation: e.clone(),
        });
    }
    let ledger = match verify_initial_relative_snc(frame.clone(), active, b)? {
        SncProduction::Verified(v) => v,
        incomplete @ SncProduction::Incomplete { reason, .. } => {
            *pending_boundary = Some(incomplete);
            return Err(Error::ResourceIncomplete(reason));
        }
        SncProduction::Unresolved { .. } => {
            return Err(Error::Invalid("transformed boundary not SNC"));
        }
    };
    let step = HistoryChartStep {
        center: center
            .iter()
            .map(|i| source.ledger().divisors()[*i].id)
            .collect(),
        incidence: open
            .incidence
            .iter()
            .map(|i| source.ledger().divisors()[*i].id)
            .collect(),
        columns: open.columns.clone(),
        pivot: pivot.map(|i| source.ledger().divisors()[i].id),
    };
    let target_history = history.advanced(ledger, born, stage, step)?;
    let mut monomial = ring.one();
    for (i, (equation, u)) in strict.iter().enumerate() {
        let factor = b.mul(equation, u)?;
        let power = b.power(&factor, source.witness().powers[i])?;
        monomial = b.mul(&monomial, &power)?;
    }
    if let Some(e) = &exceptional {
        let power = b.power(e, source.exceptional_power())?;
        monomial = b.mul(&monomial, &power)?;
    }
    let factor = match &exceptional {
        Some(e) => b.power(e, source.witness().source.mark())?,
        None => ring.one(),
    };
    let mut quotients = Vec::new();
    let mut residual = Vec::new();
    for (f, q) in source
        .witness()
        .source
        .ideal()
        .generators()
        .iter()
        .zip(&source.witness().quotients)
    {
        let q = pullback.pull(&open.extension.pull(q, b)?, b)?;
        let quotient = b.mul(&monomial, &q)?;
        let total = pullback.pull(&open.extension.pull(f, b)?, b)?;
        if !frame
            .local()
            .zero(&(&total - &b.mul(&factor, &quotient)?), b)?
        {
            return Err(Error::Invalid("controlled generator recombination"));
        }
        quotients.push(quotient);
        residual.push(q);
    }
    let residual = Ideal::new(ring.clone(), residual, b)?;
    if !frame.local().ideal().sum(&residual, b)?.contains(
        &ring.one(),
        frame.local().unit_relations(),
        b,
    )? {
        return Err(Error::Invalid("pulled residual ideal not unit"));
    }
    let target = Arc::new(MarkedIdeal::new(
        Ideal::new(ring, quotients, b)?,
        source.witness().source.mark(),
        b,
    )?);
    Ok(MonomialBlowupChart {
        open: open.clone(),
        pivot,
        frame,
        target,
        history: target_history,
        pullback,
        jacobian,
        original_coordinate_jacobian,
        exceptional,
        exceptional_is_empty,
        divisors: receipts,
        recombinations: source.witness().quotients.len(),
    })
}
