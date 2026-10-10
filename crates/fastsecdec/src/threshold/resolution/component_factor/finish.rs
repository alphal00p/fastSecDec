use super::super::{
    BoundaryAlgebra, Budget, ComponentPattern, ComponentProduction, Error, RegularAlgebra,
    produce_component_split,
};
use super::{data::FactorData, state::*};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

pub(crate) fn leaf(
    mut data: FactorData,
    namespace: &str,
    b: &mut Budget,
) -> Result<std::result::Result<FactorLeaf, FactorPendingEvidence>> {
    b.reserve_slots(data.powers().len())?;
    let mut maximality = Vec::with_capacity(data.powers().len());
    // Unit factors are absorbed first, so every later residual proof is for
    // precisely the final source residual, not a stale intermediate vector.
    for index in 0..data.powers().len() {
        if matches!(
            RegularAlgebra::boundary(data.history().ledger().clone(), index, b)?,
            BoundaryAlgebra::Empty { .. }
        ) {
            data = data.absorb_unit(index, b)?;
        }
    }
    for index in 0..data.powers().len() {
        match RegularAlgebra::boundary(data.history().ledger().clone(), index, b)? {
            BoundaryAlgebra::Empty { .. } => {
                maximality.push(BoundaryMaximality::Unit { divisor: index })
            }
            BoundaryAlgebra::Regular(owner) => {
                let residual = data.residual(b)?;
                match produce_component_split(
                    owner,
                    residual,
                    &format!("{namespace}::final{index}"),
                    b,
                )? {
                    incomplete @ ComponentProduction::Incomplete { .. } => {
                        return Ok(Err(FactorPendingEvidence::Component(incomplete)));
                    }
                    ComponentProduction::Complete(split) => {
                        if split.pattern() != ComponentPattern::NowhereIdenticallyZero {
                            return Err(Error::Invalid("component extraction not maximal"));
                        }
                        maximality.push(BoundaryMaximality::Residual {
                            divisor: index,
                            split: Arc::new(*split),
                        });
                    }
                }
            }
        }
    }
    data.verify(b)?;
    Ok(Ok(FactorLeaf { data, maximality }))
}
