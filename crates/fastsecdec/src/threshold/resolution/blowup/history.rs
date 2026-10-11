use super::super::*;
use super::helpers::Result;
use std::{collections::BTreeMap, sync::Arc};
/// Checked local chart lineage, not a global BM-center gluing certificate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryChartStep {
    pub center: Vec<BoundaryId>,
    pub incidence: Vec<BoundaryId>,
    pub columns: Vec<usize>,
    pub pivot: Option<BoundaryId>,
}
/// Exact local ancestry includes opens as well as blowups; neither identifies
/// globally glued centers or divisors across unrelated local branches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryStep {
    Blowup(HistoryChartStep),
    CoordinateBlowup {
        center: HistoryCenter,
        pivot_axis: usize,
    },
    RelativeBlowup {
        center: HistoryCenter,
        source_open: Poly,
        columns: Vec<usize>,
        pivot_normal: Option<usize>,
        identity_side: Option<usize>,
    },
    EmbeddedBlowup {
        center: HistoryCenter,
        ambient_path: Vec<HistoryStep>,
        source_open: Poly,
        pivot_normal: Option<usize>,
        support_equations: Vec<usize>,
        support_columns: Vec<usize>,
    },
    PrincipalOpen {
        source_ring: Arc<Ring>,
        factor: Poly,
        semantic_side: usize,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HistoryCenter {
    BoundaryIndices(Vec<BoundaryId>),
    CoordinateIdeal {
        source_ring: Arc<Ring>,
        ideal: Ideal,
        normal_axes: Vec<usize>,
    },
    RelativeIdeal {
        source_ring: Arc<Ring>,
        ideal: Ideal,
        normals: Vec<Poly>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BirthContext {
    pub parent_chart_path: Vec<HistoryStep>,
    pub center: HistoryCenter,
}
#[derive(Clone, Debug)]
struct HistoryRoot {
    initial: Arc<VerifiedRelativeSnc>,
}
#[derive(Clone, Debug)]
pub struct ResolutionHistory {
    root: Arc<HistoryRoot>,
    ledger: Arc<VerifiedRelativeSnc>,
    stage: u64,
    births: BTreeMap<BoundaryId, u64>,
    old_snapshot: Vec<BoundaryId>,
    next_id: BoundaryId,
    path: Vec<HistoryStep>,
    birth_contexts: BTreeMap<BoundaryId, BirthContext>,
}
impl ResolutionHistory {
    pub(crate) fn advanced_relative(
        &self,
        chart: &super::general_transform::RelativeBlowupGeometry,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, chart.center().history().as_ref())
            || !Arc::ptr_eq(chart.ledger().frame(), chart.frame())
            || chart.divisors().len() != self.ledger.divisors().len()
            || (chart.pivot_normal().is_some() != chart.born().is_some())
        {
            return Err(Error::Invalid("relative history transition owner"));
        }
        let provenance = HistoryCenter::RelativeIdeal {
            source_ring: self.ledger.frame().local().ring().clone(),
            ideal: chart.center().ideal().clone(),
            normals: chart.center().normals().to_vec(),
        };
        let mut births = self.births.clone();
        let mut contexts = self.birth_contexts.clone();
        let (stage, next_id) = if let Some((id, stage)) = chart.born() {
            if (id, stage) != self.next_transition()? || births.insert(id, stage).is_some() {
                return Err(Error::Invalid("relative history birth frontier"));
            }
            contexts.insert(
                id,
                BirthContext {
                    parent_chart_path: self.path.clone(),
                    center: provenance.clone(),
                },
            );
            (
                stage,
                BoundaryId(
                    id.0.checked_add(1)
                        .ok_or(Error::ResourceIncomplete("relative divisor exhaustion"))?,
                ),
            )
        } else {
            (self.stage, self.next_id)
        };
        for (old, receipt) in self.ledger.divisors().iter().zip(chart.divisors()) {
            if old.id != receipt.id {
                return Err(Error::Invalid(
                    "relative history strict divisor association",
                ));
            }
            let actual = chart.ledger().divisors().iter().find(|d| d.id == old.id);
            if receipt.absent != actual.is_none()
                || actual.is_some_and(|d| d.equation != receipt.strict_equation)
            {
                return Err(Error::Invalid("relative history strict divisor inventory"));
            }
        }
        if chart
            .ledger()
            .divisors()
            .iter()
            .any(|d| !births.contains_key(&d.id))
        {
            return Err(Error::Invalid("relative history unissued divisor"));
        }
        budget.reserve_slots(
            self.path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("relative ancestry count"))?,
        )?;
        let mut path = self.path.clone();
        path.push(HistoryStep::RelativeBlowup {
            center: provenance,
            source_open: chart.open().source_open().clone(),
            columns: chart.open().columns().to_vec(),
            pivot_normal: chart.pivot_normal(),
            identity_side: chart.open().identity_side(),
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger: chart.ledger().clone(),
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id,
            path,
            birth_contexts: contexts,
        }))
    }
    pub(crate) fn advanced_induced(
        &self,
        origin: &super::induced::EmbeddedChildCycle,
        ambient: &super::coefficient::CarriedCompanionChart,
        support: &super::coefficient::CarriedCompanionSupport,
        center_pullback: &super::coefficient::SupportedMarkedTransform,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, origin.initial_history().as_ref())
            || !Arc::ptr_eq(origin.center(), ambient.center())
            || self.stage != 0
            || !self.path.is_empty()
            || !self.ledger.divisors().is_empty()
            || !Arc::ptr_eq(self.ledger.frame(), origin.center().child().frame())
            || !ambient
                .opens()
                .iter()
                .any(|o| std::ptr::eq(o.as_ref(), support))
        {
            return Err(Error::Invalid("induced history original auxiliary owner"));
        }
        let j = support.incidence_sum();
        let child = origin.center().child();
        if j.source().ideal() != child.source().ideal()
            || j.source().mark() != child.source().mark()
        {
            return Err(Error::Invalid("induced history original J association"));
        }
        let geometry = ambient.chart().geometry();
        let frame = j.open().frame();
        if !Arc::ptr_eq(center_pullback.support(), ambient.support())
            || !Arc::ptr_eq(center_pullback.open(), j.open())
            || center_pullback.source().ideal() != child.ideal()
            || center_pullback.source().mark() != 1
        {
            return Err(Error::Invalid("induced center pullback construction owner"));
        }
        let local = frame.local();
        if !local
            .ideal()
            .sum(center_pullback.target().ideal(), b)?
            .contains(&local.ring().one(), local.unit_relations(), b)?
        {
            return Err(Error::Invalid(
                "induced center pullback not exceptional principal",
            ));
        }
        let mut births = self.births.clone();
        let mut contexts = self.birth_contexts.clone();
        let provenance = HistoryCenter::RelativeIdeal {
            source_ring: self.ledger.frame().local().ring().clone(),
            ideal: child.ideal().clone(),
            normals: child.normals().to_vec(),
        };
        let (ledger, stage, next_id) = if let Some(e) = geometry.exceptional() {
            let (id, stage) = self.next_transition()?;
            let equation = j.open().extension().pull(e, b)?;
            let ledger = match verify_initial_relative_snc(
                frame.clone(),
                vec![InitialDivisor { id, equation }],
                b,
            )? {
                SncProduction::Verified(v) => v,
                SncProduction::Incomplete { reason, .. } => {
                    return Err(Error::ResourceIncomplete(reason));
                }
                SncProduction::Unresolved { .. } => {
                    return Err(Error::Invalid("induced exceptional SNC"));
                }
            };
            births.insert(id, stage);
            contexts.insert(
                id,
                BirthContext {
                    parent_chart_path: self.path.clone(),
                    center: provenance.clone(),
                },
            );
            (
                ledger,
                stage,
                BoundaryId(
                    id.0.checked_add(1)
                        .ok_or(Error::ResourceIncomplete("induced birth exhaustion"))?,
                ),
            )
        } else {
            let ledger = match verify_initial_relative_snc(frame.clone(), vec![], b)? {
                SncProduction::Verified(v) => v,
                SncProduction::Incomplete { reason, .. } => {
                    return Err(Error::ResourceIncomplete(reason));
                }
                SncProduction::Unresolved { .. } => {
                    return Err(Error::Invalid("induced empty SNC"));
                }
            };
            (ledger, self.stage, self.next_id)
        };
        b.reserve_slots(1)?;
        let mut path = self.path.clone();
        path.push(HistoryStep::EmbeddedBlowup {
            center: provenance,
            ambient_path: ambient.chart().history().chart_path().to_vec(),
            source_open: geometry.open().source_open().clone(),
            pivot_normal: geometry.pivot_normal(),
            support_equations: j.open().chosen_equations().to_vec(),
            support_columns: j.open().columns().to_vec(),
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger,
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id,
            path,
            birth_contexts: contexts,
        }))
    }
    pub fn initial(ledger: Arc<VerifiedRelativeSnc>) -> Result<Arc<Self>> {
        let next = ledger
            .divisors()
            .last()
            .map_or(Some(0), |v| v.id.0.checked_add(1))
            .ok_or(Error::ResourceIncomplete("divisor identity exhaustion"))?;
        let births = ledger.divisors().iter().map(|v| (v.id, 0)).collect();
        let old_snapshot = ledger.initial_old_snapshot();
        Ok(Arc::new(Self {
            root: Arc::new(HistoryRoot {
                initial: ledger.clone(),
            }),
            ledger,
            stage: 0,
            births,
            old_snapshot,
            next_id: BoundaryId(next),
            path: Vec::new(),
            birth_contexts: BTreeMap::new(),
        }))
    }
    pub fn ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.ledger
    }
    pub fn stage(&self) -> u64 {
        self.stage
    }
    pub fn births(&self) -> &BTreeMap<BoundaryId, u64> {
        &self.births
    }
    pub fn old_snapshot(&self) -> &[BoundaryId] {
        &self.old_snapshot
    }
    pub fn initial_ledger(&self) -> &Arc<VerifiedRelativeSnc> {
        &self.root.initial
    }
    pub fn same_root(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.root, &other.root)
    }
    pub fn chart_path(&self) -> &[HistoryStep] {
        &self.path
    }
    /// Compare a born identity only together with same_root and this context.
    /// Equal bare integers on distinct sibling paths do not imply identity.
    pub fn birth_context(&self, id: BoundaryId) -> Option<&BirthContext> {
        self.birth_contexts.get(&id)
    }
    pub(crate) fn next_transition(&self) -> Result<(BoundaryId, u64)> {
        Ok((
            self.next_id,
            self.stage
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("history stage exhaustion"))?,
        ))
    }
    pub(crate) fn advanced(
        &self,
        ledger: Arc<VerifiedRelativeSnc>,
        new_id: BoundaryId,
        stage: u64,
        step: HistoryChartStep,
    ) -> Result<Arc<Self>> {
        if (new_id, stage) != self.next_transition()? {
            return Err(Error::Invalid("unchecked birth transition"));
        }
        let next_id = BoundaryId(
            new_id
                .0
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("divisor identity exhaustion"))?,
        );
        let mut births = self.births.clone();
        if births.insert(new_id, stage).is_some() {
            return Err(Error::Invalid("duplicate born divisor"));
        }
        for d in ledger.divisors() {
            if !births.contains_key(&d.id) {
                return Err(Error::Invalid("unissued target divisor"));
            }
        }
        let known = |id: &BoundaryId| self.ledger.divisors().iter().any(|d| &d.id == id);
        if step.center.is_empty()
            || step.center.iter().any(|id| !known(id))
            || step.incidence.iter().any(|id| !known(id))
            || step.pivot.is_some_and(|id| !step.center.contains(&id))
        {
            return Err(Error::Invalid("birth chart provenance"));
        }
        let mut birth_contexts = self.birth_contexts.clone();
        birth_contexts.insert(
            new_id,
            BirthContext {
                parent_chart_path: self.path.clone(),
                center: HistoryCenter::BoundaryIndices(step.center.clone()),
            },
        );
        let mut path = self.path.clone();
        path.push(HistoryStep::Blowup(step));
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger,
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id,
            path,
            birth_contexts,
        }))
    }

    /// Only the verified first-coordinate producer uses this bridge. It does
    /// not encode a nonmonomial center as invented boundary identities.
    pub(crate) fn advanced_first_coordinate(
        &self,
        center: &super::super::recursive::RecursiveCenter,
        extension: &RingExtension,
        chart: &super::super::recursive::RecursiveBlowupChart,
        ledger: Arc<VerifiedRelativeSnc>,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        let transform = chart.transform();
        let (id, stage) = self.next_transition()?;
        if self.stage != 0
            || !self.path.is_empty()
            || !self.ledger.divisors().is_empty()
            || !Arc::ptr_eq(self.ledger.frame(), center.frame())
            || extension.source() != center.frame().local().ring()
            || !Arc::ptr_eq(ledger.frame(), chart.target_frame())
            || transform.born_divisor()
                != (
                    id.0,
                    usize::try_from(stage)
                        .map_err(|_| Error::ResourceIncomplete("coordinate stage conversion"))?,
                )
            || extension.ideal(center.source().ideal(), budget)? != *transform.source().ideal()
            || transform.source().mark() != center.source().mark()
        {
            return Err(Error::Invalid("first coordinate history owner"));
        }
        let source_axes = center.frame().free_axes();
        let position = source_axes
            .iter()
            .position(|i| *i == chart.pivot_source_axis())
            .ok_or(Error::Invalid("coordinate history pivot"))?;
        let expected = extension
            .target()
            .coordinate(transform.map().target().axes()[position])?;
        if ledger.divisors().len() != 1
            || ledger.divisors()[0].id != id
            || ledger.divisors()[0].equation != expected
        {
            return Err(Error::Invalid("coordinate history exceptional ledger"));
        }
        let normals = transform
            .center()
            .normal_axes()
            .iter()
            .map(|a| {
                transform
                    .center()
                    .frame()
                    .map()
                    .target()
                    .axes()
                    .iter()
                    .position(|i| i == a)
                    .map(|i| source_axes[i])
                    .ok_or(Error::Invalid("coordinate history center axes"))
            })
            .collect::<Result<Vec<_>>>()?;
        let expected_center = Ideal::new(
            extension.source().clone(),
            normals
                .iter()
                .map(|i| extension.source().coordinate(*i))
                .collect::<Result<_>>()?,
            budget,
        )?;
        for (a, z) in [
            (center.ideal(), &expected_center),
            (&expected_center, center.ideal()),
        ] {
            for f in a.generators() {
                if !z.contains(f, &[], budget)? {
                    return Err(Error::Invalid(
                        "coordinate history original center identity",
                    ));
                }
            }
        }
        let provenance = HistoryCenter::CoordinateIdeal {
            source_ring: extension.source().clone(),
            ideal: center.ideal().clone(),
            normal_axes: normals,
        };
        let mut births = self.births.clone();
        if births.insert(id, stage).is_some() {
            return Err(Error::Invalid("duplicate coordinate birth"));
        }
        let mut contexts = self.birth_contexts.clone();
        contexts.insert(
            id,
            BirthContext {
                parent_chart_path: self.path.clone(),
                center: provenance.clone(),
            },
        );
        budget.reserve_slots(
            self.path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("coordinate ancestry count"))?,
        )?;
        let mut path = self.path.clone();
        path.push(HistoryStep::CoordinateBlowup {
            center: provenance,
            pivot_axis: chart.pivot_source_axis(),
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger,
            stage,
            births,
            old_snapshot: self.old_snapshot.clone(),
            next_id: BoundaryId(
                id.0.checked_add(1)
                    .ok_or(Error::ResourceIncomplete("coordinate divisor exhaustion"))?,
            ),
            path,
            birth_contexts: contexts,
        }))
    }

    pub(crate) fn restricted(
        &self,
        open: &super::super::localization::VerifiedPrincipalOpen,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, open.source().as_ref()) {
            return Err(Error::Invalid("localization history source owner"));
        }
        self.restrict_checked(
            open.ledger(),
            open.extension(),
            open.factor(),
            open.side(),
            budget,
        )
    }
    pub(crate) fn restricted_cover(
        &self,
        open: &super::super::localization::VerifiedCoverOpen,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if !std::ptr::eq(self, open.source().as_ref()) {
            return Err(Error::Invalid("cover history source owner"));
        }
        self.restrict_checked(
            open.ledger(),
            open.extension(),
            open.factor(),
            open.index(),
            budget,
        )
    }
    fn restrict_checked(
        &self,
        ledger: &Arc<VerifiedRelativeSnc>,
        extension: &RingExtension,
        factor: &Poly,
        semantic_side: usize,
        budget: &mut Budget,
    ) -> Result<Arc<Self>> {
        if ledger.divisors().len() != self.ledger.divisors().len() {
            return Err(Error::Invalid("localization changed boundary inventory"));
        }
        for (old, target) in self.ledger.divisors().iter().zip(ledger.divisors()) {
            if old.id != target.id {
                return Err(Error::Invalid("localization changed boundary identity"));
            }
            let pulled = extension.pull(&old.equation, budget)?;
            if !ledger
                .frame()
                .local()
                .zero(&(&pulled - &target.equation), budget)?
            {
                return Err(Error::Invalid("localization boundary equation pullback"));
            }
        }
        budget.reserve_slots(
            self.path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("localization ancestry count"))?,
        )?;
        let mut path = self.path.clone();
        path.push(HistoryStep::PrincipalOpen {
            source_ring: self.ledger.frame().local().ring().clone(),
            factor: factor.clone(),
            semantic_side,
        });
        Ok(Arc::new(Self {
            root: self.root.clone(),
            ledger: ledger.clone(),
            stage: self.stage,
            births: self.births.clone(),
            old_snapshot: self.old_snapshot.clone(),
            next_id: self.next_id,
            path,
            birth_contexts: self.birth_contexts.clone(),
        }))
    }
}
