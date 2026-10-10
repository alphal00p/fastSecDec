//! Explicit inspection only: restore full Symbolica expressions on demand.
//! Neither generation, compilation nor artifact serialization calls this code.
use super::{DualSector, subtraction::Coordinate};
use symbolica::{
    atom::{Atom, AtomCore},
    domains::atom::AtomField,
    id::{Pattern, Replacement},
};

impl DualSector {
    pub(crate) fn materialized_coefficients(&self) -> Vec<Atom> {
        let variables = self
            .parameters
            .iter()
            .map(|p| Atom::var(*p))
            .collect::<Vec<_>>();
        let images = self
            .map
            .exponent_matrix
            .iter()
            .map(|row| {
                row.iter()
                    .zip(&variables)
                    .map(|(power, variable)| variable.pow(Atom::num(power.clone())))
                    .product::<Atom>()
            })
            .collect::<Vec<_>>();
        let regular = self.mapped_regular.clone().unwrap_or_else(|| {
            self.terms
                .iter()
                .map(|term| {
                    term.factors
                        .iter()
                        .map(|factor| {
                            let mapped = factor.polynomial.replace_multiple(
                                self.source_parameters.iter().zip(&images).map(
                                    |(source, target)| {
                                        Replacement::new(
                                            Pattern::Literal(Atom::var(*source)),
                                            Pattern::Literal(target.clone()),
                                        )
                                    },
                                ),
                            );
                            let residual = if factor.valuation.iter().all(|value| *value == 0) {
                                mapped
                            } else {
                                mapped
                                    .to_polynomial_in_vars_with_field::<i32>(
                                        &variables,
                                        &AtomField {
                                            statistical_zero_test: false,
                                            ..AtomField::new()
                                        },
                                    )
                                    .mul_exp(
                                        &factor
                                            .valuation
                                            .iter()
                                            .map(|value| -*value)
                                            .collect::<Vec<_>>(),
                                    )
                                    .flatten(false)
                            };
                            residual.pow(&factor.exponent)
                        })
                        .product::<Atom>()
                })
                .collect::<Vec<_>>()
        });
        let definitions = self
            .recipe
            .requests
            .iter()
            .map(|request| {
                let mut value = regular[request.term].clone();
                for order in 1..=request.epsilon_order {
                    value = value.derivative(self.regulator) / Atom::num(order);
                }
                value = value
                    .replace(Pattern::Literal(Atom::var(self.regulator)))
                    .with(Atom::Zero);
                for (symbol, depth) in self.parameters.iter().zip(&request.derivatives) {
                    for order in 1..=*depth {
                        value = value.derivative(*symbol) / Atom::num(order);
                    }
                }
                value = value.replace_multiple(variables.iter().zip(&request.coordinates).map(
                    |(variable, center)| {
                        let center = match center {
                            Coordinate::Variable(axis) => variables[*axis].clone(),
                            Coordinate::Zero => Atom::Zero,
                            Coordinate::One => Atom::one(),
                        };
                        Replacement::new(
                            Pattern::Literal(variable.clone()),
                            Pattern::Literal(center),
                        )
                    },
                ));
                (Atom::var(request.placeholder), value)
            })
            .collect::<Vec<_>>();
        self.orders
            .iter()
            .map(|order| {
                self.recipe
                    .coefficients
                    .get(order)
                    .map(|coefficient| {
                        coefficient
                            .clone()
                            .into_inner()
                            .replace_multiple(definitions.iter().map(|(source, target)| {
                                Replacement::new(
                                    Pattern::Literal(source.clone()),
                                    Pattern::Literal(target.clone()),
                                )
                            }))
                    })
                    .unwrap_or(Atom::Zero)
            })
            .map(|coefficient| {
                self.contour_definitions
                    .materialize(&coefficient)
                    .expect("admitted compact contour definitions")
            })
            .collect()
    }
}
