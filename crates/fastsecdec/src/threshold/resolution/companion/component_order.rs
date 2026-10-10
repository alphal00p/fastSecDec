use super::super::component_factor::FactorLeaf;
use super::super::{Budget, Error, Ideal, MarkedIdeal, Poly, VerifiedOpenCover};
use super::order::{OrderInput, ResidualOrderProgress, Stage, run_input};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct ComponentResidualOrder {
    factor: Arc<FactorLeaf>,
    current_source: Arc<MarkedIdeal>,
    residual: Arc<Ideal>,
    monomial: Poly,
    cosupport: Arc<Ideal>,
    maximum: usize,
    upper_order_cover: VerifiedOpenCover,
    companion: Option<MarkedIdeal>,
    progress: ResidualOrderProgress,
}
impl ComponentResidualOrder {
    pub fn factor(&self) -> &Arc<FactorLeaf> {
        &self.factor
    }
    /// Actual localized source, not the original root-ring ideal.
    pub fn current_source(&self) -> &Arc<MarkedIdeal> {
        &self.current_source
    }
    pub fn residual(&self) -> &Arc<Ideal> {
        &self.residual
    }
    pub fn monomial(&self) -> &Poly {
        &self.monomial
    }
    pub fn cosupport(&self) -> &Arc<Ideal> {
        &self.cosupport
    }
    pub fn algebraic_maximum_on_cosupport(&self) -> usize {
        self.maximum
    }
    pub fn upper_order_cover(&self) -> &VerifiedOpenCover {
        &self.upper_order_cover
    }
    pub fn companion_arithmetic(&self) -> Option<&MarkedIdeal> {
        self.companion.as_ref()
    }
    pub fn progress(&self) -> &ResidualOrderProgress {
        &self.progress
    }
}
#[derive(Clone, Debug)]
pub enum ComponentResidualProduction {
    Order(Box<ComponentResidualOrder>),
    EmptyCosupport {
        factor: Arc<FactorLeaf>,
        progress: ResidualOrderProgress,
    },
    TerminalParameterLocus {
        factor: Arc<FactorLeaf>,
        progress: ResidualOrderProgress,
    },
    Incomplete {
        factor: Arc<FactorLeaf>,
        reason: &'static str,
        progress: ResidualOrderProgress,
    },
}
struct Prepared {
    source: Arc<MarkedIdeal>,
    residual: Arc<Ideal>,
    monomial: Poly,
}
fn prepare(factor: &FactorLeaf, b: &mut Budget) -> Result<Prepared> {
    let data = factor.data();
    data.verify(b)?;
    b.reserve_slots(data.originals().len())?;
    let source = Arc::new(MarkedIdeal::new(
        Ideal::new(
            data.history().ledger().frame().local().ring().clone(),
            data.originals().to_vec(),
            b,
        )?,
        data.source().mark(),
        b,
    )?);
    Ok(Prepared {
        source,
        residual: data.residual(b)?,
        monomial: data.monomial(b)?,
    })
}
/// Componentwise maximality is inherited only from an actual private-state
/// factor-frontier leaf. Relative order is restricted to its marked cosupport;
/// the upper-order opens, not the whole chart, carry the companion bound.
pub fn produce_component_residual_order(
    factor: Arc<FactorLeaf>,
    budget: &mut Budget,
) -> Result<ComponentResidualProduction> {
    let mut progress = ResidualOrderProgress::default();
    let result = (|| {
        let input = prepare(&factor, budget)?;
        let stage = run_input(
            OrderInput {
                frame: factor.data().history().ledger().frame(),
                source: &input.source,
                monomial: &input.monomial,
                residual: &input.residual,
            },
            budget,
            &mut progress,
        )?;
        Ok((input, stage))
    })();
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok((input, Stage::Order(cosupport, maximum, upper_order_cover, companion))) => Ok(
            ComponentResidualProduction::Order(Box::new(ComponentResidualOrder {
                factor,
                current_source: input.source,
                residual: input.residual,
                monomial: input.monomial,
                cosupport,
                maximum,
                upper_order_cover,
                companion,
                progress,
            })),
        ),
        Ok((_, Stage::Empty)) => {
            Ok(ComponentResidualProduction::EmptyCosupport { factor, progress })
        }
        Ok((_, Stage::Terminal)) => {
            Ok(ComponentResidualProduction::TerminalParameterLocus { factor, progress })
        }
        Err(Error::ResourceIncomplete(reason)) => Ok(ComponentResidualProduction::Incomplete {
            factor,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
