//! Independent primitive slots for the native radius-coefficient combiner.
//! Symbolica assembles and collects the polynomial once during generation.
use crate::{
    contour::dynamic::{DynamicEnvelope, displacement_cap_symbol, lambda_cap_symbol},
    generation::GenerationError,
    kernel::ProgramRecipe,
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    symbol,
};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub(crate) enum CoefficientInput {
    LambdaCap,
    DisplacementCap,
    DirectionNormSquared,
    CausalSquared { order: u32 },
    PositiveSquared { factor: usize, order: u32 },
    CausalPositive { order: u32 },
    PositivePositive { factor: usize, order: u32 },
}

#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub(crate) struct CoefficientSource {
    pub parameters: Vec<Symbol>,
    pub schema: Vec<CoefficientInput>,
    pub outputs: Vec<Atom>,
}

pub(super) fn expected_schema(
    chart: &super::super::DynamicChartRecipe,
    recipe: ProgramRecipe,
) -> Result<Vec<CoefficientInput>, String> {
    let mut expected = vec![
        CoefficientInput::LambdaCap,
        CoefficientInput::DisplacementCap,
        CoefficientInput::DirectionNormSquared,
    ];
    let polynomial = match recipe {
        ProgramRecipe::DynamicPolynomialV1 => true,
        ProgramRecipe::DynamicSignAwareV1 => false,
        _ => return Err("coefficient check source requires a dynamic recipe".into()),
    };
    expected.extend(chart.causal_orders.iter().map(|order| {
        if polynomial {
            CoefficientInput::CausalSquared { order: *order }
        } else {
            CoefficientInput::CausalPositive { order: *order }
        }
    }));
    expected.extend(
        chart
            .positive_orders
            .iter()
            .enumerate()
            .flat_map(|(factor, orders)| {
                orders.iter().map(move |order| {
                    if polynomial {
                        CoefficientInput::PositiveSquared {
                            factor,
                            order: *order,
                        }
                    } else {
                        CoefficientInput::PositivePositive {
                            factor,
                            order: *order,
                        }
                    }
                })
            }),
    );
    Ok(expected)
}

impl CoefficientSource {
    pub(crate) fn validate_for(
        &self,
        chart: &super::super::DynamicChartRecipe,
        recipe: ProgramRecipe,
    ) -> Result<(), String> {
        let expected = expected_schema(chart, recipe)?;
        if self.schema != expected
            || self.parameters.len() != expected.len()
            || self.parameters.first() != Some(&lambda_cap_symbol())
            || self.parameters.get(1) != Some(&displacement_cap_symbol())
            || self
                .parameters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.parameters.len()
            || self.outputs.len() != chart.coefficient_count
        {
            return Err(
                "dynamic coefficient combiner schema differs from its full-sector descriptor"
                    .into(),
            );
        }
        for expression in &self.outputs {
            if !expression
                .get_all_symbols(false)
                .iter()
                .all(|symbol| self.parameters.contains(symbol))
            {
                return Err("dynamic coefficient combiner retains an undeclared input".into());
            }
        }
        Ok(())
    }
    pub(crate) fn new(
        envelope: &DynamicEnvelope,
        recipe: ProgramRecipe,
    ) -> Result<Self, GenerationError> {
        let mut schema = vec![
            CoefficientInput::LambdaCap,
            CoefficientInput::DisplacementCap,
            CoefficientInput::DirectionNormSquared,
        ];
        match recipe {
            ProgramRecipe::DynamicPolynomialV1 => {
                schema.extend(envelope.causal_terms().iter().map(|term| {
                    CoefficientInput::CausalSquared {
                        order: term.order(),
                    }
                }));
                schema.extend(envelope.positive_factors().iter().enumerate().flat_map(
                    |(factor, positive)| {
                        positive
                            .terms()
                            .iter()
                            .map(move |term| CoefficientInput::PositiveSquared {
                                factor,
                                order: term.order(),
                            })
                    },
                ));
            }
            ProgramRecipe::DynamicSignAwareV1 => {
                schema.extend(envelope.causal_terms().iter().map(|term| {
                    CoefficientInput::CausalPositive {
                        order: term.order(),
                    }
                }));
                schema.extend(envelope.positive_factors().iter().enumerate().flat_map(
                    |(factor, positive)| {
                        positive.terms().iter().map(move |term| {
                            CoefficientInput::PositivePositive {
                                factor,
                                order: term.order(),
                            }
                        })
                    },
                ));
            }
            _ => {
                return Err(GenerationError::Contour(
                    "coefficient check source requires a dynamic recipe".into(),
                ));
            }
        }
        let parameters = [lambda_cap_symbol(), displacement_cap_symbol()]
            .into_iter()
            .chain(
                (2..schema.len())
                    .map(|index| symbol!(format!("fastsecdec::contour::check::primitive_{index}"))),
            )
            .collect::<Vec<_>>();
        let inputs = parameters
            .iter()
            .copied()
            .map(Atom::var)
            .collect::<Vec<_>>();
        let outputs = match recipe {
            ProgramRecipe::DynamicPolynomialV1 => {
                let mut causal = inputs[3..3 + envelope.causal_terms().len()].iter();
                let mut positive = inputs[3 + envelope.causal_terms().len()..].iter();
                envelope.polynomial_coefficients_with_inputs(
                    inputs[2].clone(),
                    |_| {
                        causal
                            .next()
                            .expect("one slot per ordered causal primitive")
                            .clone()
                    },
                    |_, _| {
                        positive
                            .next()
                            .expect("one slot per ordered positive primitive")
                            .clone()
                    },
                )?
            }
            ProgramRecipe::DynamicSignAwareV1 => {
                let mut positive = inputs[3..].iter();
                envelope.sign_aware_coefficients_with_inputs(inputs[2].clone(), |_| {
                    positive
                        .next()
                        .expect("one slot per ordered smooth positive primitive")
                        .clone()
                })?
            }
            _ => unreachable!("recipe was admitted above"),
        };
        Ok(Self {
            parameters,
            schema,
            outputs,
        })
    }
}
