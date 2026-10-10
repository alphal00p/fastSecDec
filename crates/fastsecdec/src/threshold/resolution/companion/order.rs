use super::super::differential::relative_differential_ideal;
use super::super::{
    Budget, Error, EtaleFrame, Ideal, MarkedIdeal, OpenCoverCertificate, Poly, VerifiedOpenCover,
};
use super::factor::WholeCartierFactorization;
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct RelativeOrderLayer {
    pub order: usize,
    pub ideal: Arc<Ideal>,
    pub unit_on_cosupport: Option<bool>,
}
#[derive(Clone, Debug, Default)]
pub struct ResidualOrderProgress {
    pub cosupport_layers: Vec<Arc<Ideal>>,
    pub residual_layers: Vec<RelativeOrderLayer>,
    pub operations: usize,
    pub ideal_slots: usize,
}
#[derive(Clone, Debug)]
pub struct RestrictedResidualOrder {
    factors: Arc<WholeCartierFactorization>,
    cosupport: Arc<Ideal>,
    maximum: usize,
    upper_order_cover: VerifiedOpenCover,
    /// Exact weighted arithmetic only. Its maximal-order neighborhoods are
    /// precisely the checked upper-order opens, not the whole source chart.
    companion: Option<MarkedIdeal>,
    progress: ResidualOrderProgress,
}
impl RestrictedResidualOrder {
    pub fn factors(&self) -> &Arc<WholeCartierFactorization> {
        &self.factors
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
pub enum ResidualOrderProduction {
    Order(Box<RestrictedResidualOrder>),
    EmptyCosupport {
        factors: Arc<WholeCartierFactorization>,
        progress: ResidualOrderProgress,
    },
    TerminalParameterLocus {
        factors: Arc<WholeCartierFactorization>,
        progress: ResidualOrderProgress,
    },
    Incomplete {
        factors: Arc<WholeCartierFactorization>,
        reason: &'static str,
        progress: ResidualOrderProgress,
    },
}
pub fn produce_restricted_residual_order(
    factors: Arc<WholeCartierFactorization>,
    budget: &mut Budget,
) -> Result<ResidualOrderProduction> {
    let mut progress = ResidualOrderProgress::default();
    let result = run(&factors, budget, &mut progress);
    progress.operations = budget.operations();
    progress.ideal_slots = budget.ideal_slots();
    match result {
        Ok(Stage::Empty) => Ok(ResidualOrderProduction::EmptyCosupport { factors, progress }),
        Ok(Stage::Terminal) => {
            Ok(ResidualOrderProduction::TerminalParameterLocus { factors, progress })
        }
        Ok(Stage::Order(cosupport, maximum, upper_order_cover, companion)) => Ok(
            ResidualOrderProduction::Order(Box::new(RestrictedResidualOrder {
                factors,
                cosupport,
                maximum,
                upper_order_cover,
                companion,
                progress,
            })),
        ),
        Err(Error::ResourceIncomplete(reason)) => Ok(ResidualOrderProduction::Incomplete {
            factors,
            reason,
            progress,
        }),
        Err(e) => Err(e),
    }
}
pub(super) enum Stage {
    Empty,
    Terminal,
    Order(Arc<Ideal>, usize, VerifiedOpenCover, Option<MarkedIdeal>),
}
fn run(
    factors: &Arc<WholeCartierFactorization>,
    budget: &mut Budget,
    progress: &mut ResidualOrderProgress,
) -> Result<Stage> {
    run_input(
        OrderInput {
            frame: factors.ledger().frame(),
            source: factors.source(),
            monomial: factors.monomial(),
            residual: factors.residual(),
        },
        budget,
        progress,
    )
}
/// Shared native derivative engine. Provenance is supplied by either the
/// existing whole-equation owner or a checked component factor leaf.
pub(super) struct OrderInput<'a> {
    pub frame: &'a Arc<EtaleFrame>,
    pub source: &'a MarkedIdeal,
    pub monomial: &'a Poly,
    pub residual: &'a Arc<Ideal>,
}
pub(super) fn run_input(
    input: OrderInput<'_>,
    budget: &mut Budget,
    progress: &mut ResidualOrderProgress,
) -> Result<Stage> {
    let frame = input.frame;
    let local = frame.local();
    let source = input.source;
    if source.mark() > budget.limits.max_mark {
        return Err(Error::ResourceIncomplete("source derivative mark cap"));
    }
    budget.reserve_slots(1)?;
    progress
        .cosupport_layers
        .push(Arc::new(source.ideal().clone()));
    for _ in 1..source.mark() {
        let next = relative_differential_ideal(
            frame,
            progress.cosupport_layers.last().expect("source layer"),
            budget,
        )?;
        budget.reserve_slots(1)?;
        progress.cosupport_layers.push(Arc::new(next));
    }
    let cosupport = progress.cosupport_layers.last().expect("cosupport").clone();
    if local.ideal().sum(&cosupport, budget)?.contains(
        &local.ring().one(),
        local.unit_relations(),
        budget,
    )? {
        return Ok(Stage::Empty);
    }
    budget.reserve_slots(1)?;
    progress.residual_layers.push(RelativeOrderLayer {
        order: 0,
        ideal: input.residual.clone(),
        unit_on_cosupport: None,
    });
    for order in 0..=budget.limits.max_mark {
        let current = progress
            .residual_layers
            .last()
            .expect("current residual")
            .ideal
            .clone();
        let combined = local
            .ideal()
            .sum(&cosupport, budget)?
            .sum(&current, budget)?;
        let is_unit = combined.contains(&local.ring().one(), local.unit_relations(), budget)?;
        progress
            .residual_layers
            .last_mut()
            .expect("current residual")
            .unit_on_cosupport = Some(is_unit);
        if is_unit {
            let cover = OpenCoverCertificate {
                algebra: local.clone(),
                support: (*cosupport).clone(),
                opens: current.generators().to_vec(),
            }
            .verify(budget)?;
            let companion = if order == 0 {
                None
            } else {
                let residual = MarkedIdeal::new((**input.residual).clone(), order, budget)?;
                Some(if order < source.mark() {
                    let monomial = MarkedIdeal::new(
                        Ideal::new(local.ring().clone(), vec![input.monomial.clone()], budget)?,
                        source.mark() - order,
                        budget,
                    )?;
                    residual.sum(&monomial, budget)?
                } else {
                    residual
                })
            };
            return Ok(Stage::Order(cosupport, order, cover, companion));
        }
        if frame.free_axes().is_empty() {
            return Ok(Stage::Terminal);
        }
        if order == budget.limits.max_mark {
            break;
        }
        let next = relative_differential_ideal(frame, &current, budget)?;
        budget.reserve_slots(1)?;
        progress.residual_layers.push(RelativeOrderLayer {
            order: order + 1,
            ideal: Arc::new(next),
            unit_on_cosupport: None,
        });
    }
    Err(Error::ResourceIncomplete(
        "relative residual order unresolved at derivative cap",
    ))
}
