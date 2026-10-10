use super::super::contact::clear_units;
use super::super::{
    Budget, Error, EtaleFrame, Ideal, LocalizedAlgebra, UnitClearing, VerifiedRelativeSnc,
};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

/// Actual smoothness owner; arbitrary supplied quotient rings cannot construct it.
#[derive(Clone, Debug)]
pub struct RegularAlgebra {
    origin: RegularOrigin,
    local: Arc<LocalizedAlgebra>,
}
#[derive(Clone, Debug)]
pub enum RegularOrigin {
    Ambient(Arc<EtaleFrame>),
    Boundary {
        ledger: Arc<VerifiedRelativeSnc>,
        divisor: usize,
        equation: UnitClearing,
    },
}
#[derive(Clone, Debug)]
pub enum BoundaryAlgebra {
    Regular(Arc<RegularAlgebra>),
    Empty {
        ledger: Arc<VerifiedRelativeSnc>,
        divisor: usize,
    },
}
impl RegularAlgebra {
    pub fn ambient(frame: Arc<EtaleFrame>) -> Arc<Self> {
        Arc::new(Self {
            local: frame.local().clone(),
            origin: RegularOrigin::Ambient(frame),
        })
    }
    pub fn boundary(
        ledger: Arc<VerifiedRelativeSnc>,
        divisor: usize,
        budget: &mut Budget,
    ) -> Result<BoundaryAlgebra> {
        let h = &ledger
            .divisors()
            .get(divisor)
            .ok_or(Error::Invalid("regular boundary index"))?
            .equation;
        let singleton = ledger
            .intersection(&[divisor])
            .ok_or(Error::Invalid("missing checked singleton boundary"))?;
        if singleton.empty {
            return Ok(BoundaryAlgebra::Empty { ledger, divisor });
        }
        let source = ledger.frame().local();
        let equation = clear_units(source, h, budget)?;
        budget.reserve_slots(1)?;
        let ideal = source.ideal().sum(
            &Ideal::new(
                source.ring().clone(),
                vec![equation.numerator.clone()],
                budget,
            )?,
            budget,
        )?;
        if ideal.contains(&source.ring().one(), source.unit_relations(), budget)? {
            return Err(Error::Invalid("nonempty checked boundary became empty"));
        }
        budget.reserve_slots(source.axes().len())?;
        budget.reserve_slots(source.guards().len())?;
        let local = LocalizedAlgebra::new(
            ideal,
            source.axes().to_vec(),
            source.guards().to_vec(),
            budget,
        )?;
        Ok(BoundaryAlgebra::Regular(Arc::new(Self {
            local,
            origin: RegularOrigin::Boundary {
                ledger,
                divisor,
                equation,
            },
        })))
    }
    pub fn origin(&self) -> &RegularOrigin {
        &self.origin
    }
    pub fn local(&self) -> &Arc<LocalizedAlgebra> {
        &self.local
    }
    /// The source before imposing a boundary equation. All open covers use this.
    pub fn ambient_local(&self) -> &Arc<LocalizedAlgebra> {
        match &self.origin {
            RegularOrigin::Ambient(frame) => frame.local(),
            RegularOrigin::Boundary { ledger, .. } => ledger.frame().local(),
        }
    }
}
