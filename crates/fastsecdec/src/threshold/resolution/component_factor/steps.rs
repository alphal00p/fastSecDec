use super::super::{
    BoundaryAlgebra, Budget, CartierDivision, ComponentPattern, ComponentProduction, Error,
    LocalizationProduction, RegularAlgebra, divide_cartier, localize_component_open,
    produce_component_split,
};
use super::{data::FactorData, state::*};
use std::sync::Arc;
type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Step {
    pub data: FactorData,
    pub state: State,
    pub child: Option<(FactorPath, FactorNode)>,
    pub evidence: Option<FactorPendingEvidence>,
}
impl Step {
    fn plain(data: FactorData, state: State) -> Self {
        Self {
            data,
            state,
            child: None,
            evidence: None,
        }
    }
}
pub(crate) struct StepContext<'a> {
    pub path: &'a [usize],
    pub namespace: &'a str,
    pub limits: &'a ComponentFactorLimits,
    pub node_count: usize,
}
fn split_result(
    data: FactorData,
    state: State,
    phase: SplitPhase,
    out: ComponentProduction,
) -> Result<Step> {
    match out {
        incomplete @ ComponentProduction::Incomplete { .. } => Ok(Step {
            data,
            state,
            child: None,
            evidence: Some(FactorPendingEvidence::Component(incomplete)),
        }),
        ComponentProduction::Complete(split) => {
            let split = Arc::new(*split);
            let next = match (phase, split.pattern()) {
                (SplitPhase::Ambient, ComponentPattern::EverywhereIdenticallyZero) => {
                    State::ZeroIdeal(split)
                }
                (SplitPhase::Ambient, ComponentPattern::NowhereIdenticallyZero) => {
                    State::Boundary(0)
                }
                (SplitPhase::Boundary(i), ComponentPattern::EverywhereIdenticallyZero) => {
                    State::Divide(i)
                }
                (SplitPhase::Boundary(i), ComponentPattern::NowhereIdenticallyZero) => {
                    State::Boundary(
                        i.checked_add(1)
                            .ok_or(Error::ResourceIncomplete("boundary index overflow"))?,
                    )
                }
                (_, ComponentPattern::Mixed) => State::Split {
                    split,
                    phase,
                    children: Vec::new(),
                },
            };
            Ok(Step::plain(data, next))
        }
    }
}
pub(crate) fn prepare(
    data: FactorData,
    state: State,
    c: &StepContext<'_>,
    b: &mut Budget,
) -> Result<Step> {
    match state {
        State::Ambient => {
            let owner = RegularAlgebra::ambient(data.history().ledger().frame().clone());
            let source = data.residual(b)?;
            let out =
                produce_component_split(owner, source, &format!("{}::ambient", c.namespace), b)?;
            split_result(data, State::Ambient, SplitPhase::Ambient, out)
        }
        State::Boundary(index) => {
            if index == data.powers().len() {
                return Ok(Step::plain(data, State::Finalize));
            }
            match RegularAlgebra::boundary(data.history().ledger().clone(), index, b)? {
                BoundaryAlgebra::Empty { .. } => {
                    let data = data.absorb_unit(index, b)?;
                    Ok(Step::plain(
                        data,
                        State::Boundary(
                            index
                                .checked_add(1)
                                .ok_or(Error::ResourceIncomplete("boundary index overflow"))?,
                        ),
                    ))
                }
                BoundaryAlgebra::Regular(owner) => {
                    let source = data.residual(b)?;
                    let out = produce_component_split(
                        owner,
                        source,
                        &format!("{}::boundary{index}_p{}", c.namespace, data.powers()[index]),
                        b,
                    )?;
                    split_result(
                        data,
                        State::Boundary(index),
                        SplitPhase::Boundary(index),
                        out,
                    )
                }
            }
        }
        State::Divide(index) => {
            if data.powers()[index] >= c.limits.max_power {
                return Err(Error::ResourceIncomplete("component factor power cap"));
            }
            b.reserve_slots(data.quotients().len())?;
            let mut quotients = Vec::with_capacity(data.quotients().len());
            for (i, q) in data.quotients().iter().enumerate() {
                match divide_cartier(
                    data.history().ledger().clone(),
                    index,
                    q.clone(),
                    &format!(
                        "{}::divide{index}_p{}_g{i}",
                        c.namespace,
                        data.powers()[index]
                    ),
                    b,
                )? {
                    CartierDivision::Quotient(q) => quotients.push(q.quotient().clone()),
                    _ => {
                        return Err(Error::Invalid(
                            "component division contradicted complete boundary split",
                        ));
                    }
                }
            }
            Ok(Step::plain(
                data.divided(index, quotients, b)?,
                State::Boundary(index),
            ))
        }
        State::Split {
            split,
            phase,
            mut children,
        } => {
            let side = children.len();
            if side > 1 {
                return Err(Error::Invalid("component split child count"));
            }
            if c.node_count >= c.limits.max_nodes {
                return Err(Error::ResourceIncomplete("component factor node cap"));
            }
            let depth = c
                .path
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("component factor depth overflow"))?;
            if depth > c.limits.max_depth {
                return Err(Error::ResourceIncomplete("component factor depth cap"));
            }
            let result = localize_component_open(
                data.history().clone(),
                split.clone(),
                side,
                &format!("{}::side{side}", c.namespace),
                b,
            )?;
            let mut child = None;
            match result {
                incomplete @ LocalizationProduction::Incomplete { .. } => {
                    return Ok(Step {
                        data,
                        state: State::Split {
                            split,
                            phase,
                            children,
                        },
                        child: None,
                        evidence: Some(FactorPendingEvidence::Localization(incomplete)),
                    });
                }
                empty @ LocalizationProduction::Empty { .. } => children.push(FactorChild::Empty {
                    side,
                    evidence: empty,
                }),
                LocalizationProduction::Complete(open) => {
                    let restriction = Arc::new(*open);
                    let child_data = data.restricted(&restriction, b)?;
                    let mut path = c.path.to_vec();
                    path.push(side);
                    let state = match phase {
                        SplitPhase::Ambient => State::Ambient,
                        SplitPhase::Boundary(i) => State::Boundary(i),
                    };
                    child = Some((
                        path.clone(),
                        FactorNode {
                            data: child_data,
                            parent: Some(restriction.clone()),
                            state,
                            evidence: None,
                        },
                    ));
                    children.push(FactorChild::Nonempty { path, restriction });
                }
            }
            let state = if children.len() == 2 {
                State::Expanded { split, children }
            } else {
                State::Split {
                    split,
                    phase,
                    children,
                }
            };
            Ok(Step {
                data,
                state,
                child,
                evidence: None,
            })
        }
        State::Finalize => {
            let out = super::finish::leaf(data.checked_clone(b)?, c.namespace, b)?;
            match out {
                Ok(leaf) => Ok(Step::plain(
                    leaf.data.clone(),
                    State::Complete(Arc::new(leaf)),
                )),
                Err(evidence) => Ok(Step {
                    data,
                    state: State::Finalize,
                    child: None,
                    evidence: Some(evidence),
                }),
            }
        }
        _ => Err(Error::Invalid(
            "completed component factor node resubmitted",
        )),
    }
}
