//! Complete local factor inventory before admitting a supported residual drop.
use super::*;
#[derive(Clone, Debug)]
pub enum SupportedDropRecord {
    Empty(ComponentResidualProduction),
    ZeroMaximum(Arc<ComponentResidualOrder>),
    Positive(Arc<ComponentResidualOrder>),
    Pending(ComponentResidualProduction),
}
#[derive(Clone, Debug)]
pub struct SupportedDropInventory {
    prior: Arc<SupportedProblemChart>,
    open: Arc<SupportedProblemOpen>,
    factors: Arc<CompletedComponentFactors>,
    records: Vec<SupportedDropRecord>,
    zero_component: bool,
    maximum: Option<usize>,
}
impl SupportedDropInventory {
    pub fn prove(
        prior: Arc<SupportedProblemChart>,
        open: Arc<SupportedProblemOpen>,
        factors: Arc<CompletedComponentFactors>,
        b: &mut Budget,
    ) -> Result<Arc<Self>> {
        prior.check_open(&open)?;
        let root = factors
            .nodes()
            .get(&Vec::new())
            .ok_or(Error::Invalid("supported drop missing factor root"))?;
        if !Arc::ptr_eq(root.data().source(), open.source().target())
            || !Arc::ptr_eq(root.data().history(), open.history())
        {
            return Err(Error::Invalid(
                "supported drop actual complete factor inventory",
            ));
        }
        if !open.companion_cosupport().empty()
            || !Arc::ptr_eq(
                open.companion_cosupport().source(),
                open.companion().target(),
            )
            || !Arc::ptr_eq(
                open.companion_cosupport().frame(),
                open.source().open().frame(),
            )
        {
            return Err(Error::Invalid(
                "supported residual drop lacks old companion completion",
            ));
        }
        let mut records = Vec::new();
        let mut zero_component = false;
        let mut maximum = Some(0usize);
        b.reserve_slots(factors.nodes().len())?;
        for node in factors.nodes().values() {
            if node.zero_ideal().is_some() {
                zero_component = true;
                maximum = None;
            }
            let Some(leaf) = node.leaf() else { continue };
            let data = leaf.data();
            if !Arc::ptr_eq(data.source(), open.source().target())
                || !data.history().same_root(open.history())
                || !data
                    .history()
                    .chart_path()
                    .starts_with(open.history().chart_path())
                || data.history().births() != open.history().births()
            {
                return Err(Error::Invalid("supported residual drop component ancestry"));
            }
            let p = produce_component_residual_order(leaf.clone(), b)?;
            match p {
                ComponentResidualProduction::Order(order) => {
                    let order = Arc::new(*order);
                    let m = order.algebraic_maximum_on_cosupport();
                    if let Some(current) = &mut maximum {
                        *current = (*current).max(m);
                    }
                    records.push(if m == 0 {
                        SupportedDropRecord::ZeroMaximum(order)
                    } else {
                        SupportedDropRecord::Positive(order)
                    });
                }
                p @ ComponentResidualProduction::EmptyCosupport { .. } => {
                    records.push(SupportedDropRecord::Empty(p))
                }
                p => {
                    maximum = None;
                    records.push(SupportedDropRecord::Pending(p));
                }
            }
        }
        if let Some(m) = maximum {
            let old = prior
                .prepared()
                .center()
                .coefficient()
                .source()
                .order()
                .algebraic_maximum_on_cosupport();
            if m >= old {
                return Err(Error::Invalid(
                    "supported residual maximum did not strictly decrease",
                ));
            }
        }
        Ok(Arc::new(Self {
            prior,
            open,
            factors,
            records,
            zero_component,
            maximum,
        }))
    }
    pub fn prior(&self) -> &Arc<SupportedProblemChart> {
        &self.prior
    }
    pub fn open(&self) -> &Arc<SupportedProblemOpen> {
        &self.open
    }
    pub fn factors(&self) -> &Arc<CompletedComponentFactors> {
        &self.factors
    }
    pub fn records(&self) -> &[SupportedDropRecord] {
        &self.records
    }
    /// Whole selected supported open, never inferred from one chosen component.
    pub fn proved_maximum(&self) -> Option<usize> {
        self.maximum
    }
    pub fn unresolved_zero_component(&self) -> bool {
        self.zero_component
    }
    pub fn positive_drop(self: &Arc<Self>, index: usize) -> Result<Arc<SupportedResidualDrop>> {
        if self.maximum.is_none() {
            return Err(Error::Invalid("supported residual inventory incomplete"));
        }
        let Some(SupportedDropRecord::Positive(current)) = self.records.get(index) else {
            return Err(Error::Invalid(
                "supported residual cycle requires positive component",
            ));
        };
        Ok(Arc::new(SupportedResidualDrop {
            inventory: self.clone(),
            current: current.clone(),
        }))
    }
}
#[derive(Clone, Debug)]
pub struct SupportedResidualDrop {
    inventory: Arc<SupportedDropInventory>,
    current: Arc<ComponentResidualOrder>,
}
impl SupportedResidualDrop {
    pub fn inventory(&self) -> &Arc<SupportedDropInventory> {
        &self.inventory
    }
    pub fn current(&self) -> &Arc<ComponentResidualOrder> {
        &self.current
    }
}
