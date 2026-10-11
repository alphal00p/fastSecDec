//! Carry the ORIGINAL companion coefficient and incidence sum on the actual
//! strict contact support. No new derivative/coefficient construction occurs.
use super::super::*;
use super::{
    general::RecursiveCenterOrigin,
    general_transform::{RelativeMarkedTransform, RelativeRecursiveChart, controlled_marked},
    helpers::unit,
    support::{StrictContactSupport, StrictSupportOpen, transport_contact_support},
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct SupportedGenerator {
    pub source: Poly,
    pub clearing: UnitClearing,
    pub numerator: Poly,
    pub denominator: Poly,
    pub total: Poly,
    pub quotient: Poly,
}
#[derive(Clone, Debug)]
pub struct SupportedMarkedTransform {
    support: Arc<StrictContactSupport>,
    open: Arc<StrictSupportOpen>,
    source: Arc<MarkedIdeal>,
    target: Arc<MarkedIdeal>,
    generators: Vec<SupportedGenerator>,
    exceptional_ledger: Option<Arc<VerifiedRelativeSnc>>,
}
impl SupportedMarkedTransform {
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn open(&self) -> &Arc<StrictSupportOpen> {
        &self.open
    }
    pub fn source(&self) -> &Arc<MarkedIdeal> {
        &self.source
    }
    pub fn target(&self) -> &Arc<MarkedIdeal> {
        &self.target
    }
    pub fn generators(&self) -> &[SupportedGenerator] {
        &self.generators
    }
    pub fn exceptional_ledger(&self) -> Option<&Arc<VerifiedRelativeSnc>> {
        self.exceptional_ledger.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct CarriedCompanionSupport {
    coefficient: Arc<SupportedMarkedTransform>,
    incidence_sum: Arc<SupportedMarkedTransform>,
}
impl CarriedCompanionSupport {
    pub fn coefficient(&self) -> &Arc<SupportedMarkedTransform> {
        &self.coefficient
    }
    pub fn incidence_sum(&self) -> &Arc<SupportedMarkedTransform> {
        &self.incidence_sum
    }
}
#[derive(Clone, Debug)]
pub struct CarriedCompanionChart {
    center: Arc<CompanionCenter>,
    chart: Arc<RelativeRecursiveChart>,
    companion: Arc<RelativeMarkedTransform>,
    companion_order: OrderProduction,
    support: Arc<StrictContactSupport>,
    opens: Vec<Arc<CarriedCompanionSupport>>,
}
impl CarriedCompanionChart {
    pub fn center(&self) -> &Arc<CompanionCenter> {
        &self.center
    }
    pub fn chart(&self) -> &Arc<RelativeRecursiveChart> {
        &self.chart
    }
    pub fn companion(&self) -> &Arc<RelativeMarkedTransform> {
        &self.companion
    }
    pub fn companion_order(&self) -> &OrderProduction {
        &self.companion_order
    }
    pub fn support(&self) -> &Arc<StrictContactSupport> {
        &self.support
    }
    pub fn opens(&self) -> &[Arc<CarriedCompanionSupport>] {
        &self.opens
    }
}
/// Consume the actual next-center chart; retained G, C and J all originate in
/// its checked CompanionCenter. This never accepts a supplied replacement C.
pub fn carry_companion_chart(
    chart: Arc<RelativeRecursiveChart>,
    namespace: &str,
    b: &mut Budget,
) -> Result<Arc<CarriedCompanionChart>> {
    let RecursiveCenterOrigin::Companion(center) = chart.geometry().center().origin() else {
        return Err(Error::Invalid(
            "carried companion needs actual companion center",
        ));
    };
    let center = center.clone();
    let q = center.coefficient().contact();
    if !Arc::ptr_eq(
        q.source().frame(),
        chart.geometry().center().history().ledger().frame(),
    ) {
        return Err(Error::Invalid("carried companion contact owner"));
    }
    let g = Arc::new(MarkedIdeal::new(
        (**q.source().source()).clone(),
        q.source().algebraic_maximum_order(),
        b,
    )?);
    let companion = controlled_marked(chart.geometry().clone(), g, &format!("{namespace}_G"), b)?;
    let companion_order = produce_ordinary_contact_cover(
        chart.geometry().frame().clone(),
        Arc::new(companion.target().ideal().clone()),
        b,
    )?;
    if let OrderProduction::Incomplete { reason, .. } = &companion_order {
        return Err(Error::ResourceIncomplete(reason));
    }
    let support = transport_contact_support(
        chart.geometry().clone(),
        q.clone(),
        &format!("{namespace}_support"),
        b,
    )?;
    let coefficient = Arc::new(q.differential_coefficient().clone());
    let incidence_sum = Arc::new(center.coefficient().coefficient().coefficient().clone());
    let mut opens = Vec::new();
    b.reserve_slots(support.opens().len())?;
    for (i, open) in support.opens().iter().enumerate() {
        let c = controlled_supported(
            support.clone(),
            open.clone(),
            coefficient.clone(),
            &format!("{namespace}_C{i}"),
            b,
        )?;
        let j = controlled_supported(
            support.clone(),
            open.clone(),
            incidence_sum.clone(),
            &format!("{namespace}_J{i}"),
            b,
        )?;
        opens.push(Arc::new(CarriedCompanionSupport {
            coefficient: c,
            incidence_sum: j,
        }));
    }
    Ok(Arc::new(CarriedCompanionChart {
        center,
        chart,
        companion,
        companion_order,
        support,
        opens,
    }))
}
pub(super) fn controlled_supported(
    support: Arc<StrictContactSupport>,
    open: Arc<StrictSupportOpen>,
    source: Arc<MarkedIdeal>,
    namespace: &str,
    b: &mut Budget,
) -> Result<Arc<SupportedMarkedTransform>> {
    if !support.opens().iter().any(|o| Arc::ptr_eq(o, &open))
        || source.ideal().ring() != support.source().contact().local().ring()
    {
        return Err(Error::Invalid("supported marked transform owner"));
    }
    let geometry = support.geometry();
    let local = open.frame().local();
    let exceptional = geometry
        .exceptional()
        .map(|e| open.extension().pull(e, b))
        .transpose()?;
    let exceptional_ledger = if let Some(e) = &exceptional {
        let id = geometry
            .born()
            .ok_or(Error::Invalid("support exceptional birth"))?
            .0;
        match verify_initial_relative_snc(
            open.frame().clone(),
            vec![InitialDivisor {
                id,
                equation: e.clone(),
            }],
            b,
        )? {
            SncProduction::Verified(v) => Some(v),
            SncProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            SncProduction::Unresolved { .. } => {
                return Err(Error::ResourceIncomplete(
                    "exceptional restriction not smooth on support",
                ));
            }
        }
    } else {
        None
    };
    let original = support.source().source().frame().local().ring();
    let pull = |p: &Poly, b: &mut Budget| -> Result<Poly> {
        if (original.len()..p.nvars()).any(|i| p.degree(i) > 0) {
            return Err(Error::ResourceIncomplete(
                "supported coefficient has a foreign slot",
            ));
        }
        let p = super::super::elimination::remap(p, original, b)?;
        open.extension().pull(&geometry.pull(&p, b)?, b)
    };
    let mut generators = Vec::new();
    b.reserve_slots(source.ideal().generators().len())?;
    for (i, f) in source.ideal().generators().iter().enumerate() {
        let clearing = clear_units(support.source().contact().local(), f, b)?;
        let numerator = pull(&clearing.numerator, b)?;
        let denominator = pull(&clearing.denominator, b)?;
        if !unit(local, &denominator, b)? {
            return Err(Error::Invalid("supported clearing denominator lost unit"));
        }
        let total = super::super::companion::divide_regular_in_localization(
            local,
            &denominator,
            &numerator,
            &format!("{namespace}_denom{i}"),
            b,
        )?
        .0;
        let mut quotient = total.clone();
        if let Some(e) = &exceptional {
            for j in 0..source.mark() {
                quotient = super::super::companion::divide_regular_in_localization(
                    local,
                    e,
                    &quotient,
                    &format!("{namespace}_controlled{i}_{j}"),
                    b,
                )?
                .0;
            }
            let factor = b.power(e, source.mark())?;
            if !local.zero(&(&total - &b.mul(&factor, &quotient)?), b)? {
                return Err(Error::Invalid("supported full controlled recombination"));
            }
        }
        generators.push(SupportedGenerator {
            source: f.clone(),
            clearing,
            numerator,
            denominator,
            total,
            quotient,
        });
    }
    let target = Arc::new(MarkedIdeal::new(
        Ideal::new(
            local.ring().clone(),
            generators.iter().map(|g| g.quotient.clone()).collect(),
            b,
        )?,
        source.mark(),
        b,
    )?);
    Ok(Arc::new(SupportedMarkedTransform {
        support,
        open,
        source,
        target,
        generators,
        exceptional_ledger,
    }))
}
