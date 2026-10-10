//! Shared exact multiplicity, zero-dimensional folding and Laurent assembly.
use super::{
    ChartRecord, DomainAssessment, GeneratedIntegral, GeneratedSector, GenerationError,
    GenerationMetadata, coefficients, conditioning::Profile, program::ProgramData,
};
use fastsecdec_sectors::SectorMap;
use std::collections::BTreeMap;
use symbolica::atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol};

/// Canonicalize numerical weights after summing exact contributions. Native
/// addition alone leaves `a + b - (a + b)` uncollected. Distributing only
/// numerical coefficients exposes those cancellations without expanding general
/// products or touching opaque function arguments (including saved root keys).
/// This also applies when selected saved records are assembled on loading; it
/// does not rebuild a smooth density or an optimized evaluator.
pub(crate) fn normalize_exact_coefficient(expression: &Atom) -> Atom {
    expression.expand_num()
}

type Pending = (
    SectorMap,
    Vec<Symbol>,
    BTreeMap<i32, AliasedAtom>,
    Profile,
    ProgramData,
);
pub(super) struct Assembly {
    pending: Vec<Pending>,
    exact: BTreeMap<i32, Atom>,
    minimum: i32,
    kernel_indices: BTreeMap<usize, usize>,
    program: ProgramData,
}
impl Assembly {
    pub fn new(max_order: i32) -> Self {
        Self {
            pending: Vec::new(),
            exact: BTreeMap::new(),
            minimum: max_order.min(0),
            kernel_indices: BTreeMap::new(),
            program: ProgramData::default(),
        }
    }
    pub fn push(
        &mut self,
        representative_index: usize,
        map: SectorMap,
        parameters: Vec<Symbol>,
        multiplicity: usize,
        output: coefficients::Output,
        program: ProgramData,
    ) -> Result<(), GenerationError> {
        let needs_plan = program.contour_jacobian == crate::contour::ContourJacobian::Dual
            && !parameters.is_empty()
            && program
                .descriptor
                .as_ref()
                .is_some_and(|owner| owner.recipe().is_contour());
        if needs_plan || !program.jacobians.is_empty() {
            let [(_, plan)] = program.jacobians.as_slice() else {
                return Err(GenerationError::Invariant(
                    "symbolic sector requires one local contour Jacobian plan".into(),
                ));
            };
            if plan.parameters != parameters
                || program.contour_jacobian != crate::contour::ContourJacobian::Dual
            {
                return Err(GenerationError::Invariant(
                    "symbolic contour Jacobian source/context mismatch".into(),
                ));
            }
        }
        self.program.merge(&program)?;
        let definitions = program.contour_definitions()?;
        let coefficients = output
            .coefficients
            .into_iter()
            .map(|(order, mut coefficient)| {
                if !definitions.is_empty() {
                    let root = definitions
                        .simplify(coefficient.get_root(), &parameters)
                        .map_err(GenerationError::Contour)?;
                    let mut simplified = AliasedAtom::from(root);
                    for (handle, body) in coefficient.get_aliases() {
                        simplified.register_alias(
                            handle.clone(),
                            definitions
                                .simplify(body, &parameters)
                                .map_err(GenerationError::Contour)?,
                        );
                    }
                    coefficient = simplified;
                }
                Ok((
                    order,
                    coefficient.map_root(|root| root * Atom::num(multiplicity)),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, GenerationError>>()?;
        let conditioning = output.conditioning;
        if let Some(order) = coefficients.keys().next() {
            self.minimum = self.minimum.min(*order);
        }
        if coefficients.values().all(|coefficient| {
            let symbols = coefficient.get_root().get_all_symbols(true);
            parameters.iter().all(|p| {
                let parameter = Atom::var(*p);
                !symbols.contains(p)
                    && coefficient.get_aliases().iter().all(|(handle, body)| {
                        let AtomView::Var(handle) = handle.as_view() else {
                            unreachable!("Laurent images are native symbols")
                        };
                        !symbols.contains(&handle.get_symbol())
                            || !body.contains(parameter.as_view())
                    })
            })
        }) {
            for (order, coefficient) in coefficients {
                let value = definitions
                    .materialize(&coefficient.into_inner())
                    .map_err(GenerationError::Contour)?;
                *self.exact.entry(order).or_insert(Atom::Zero) += value;
            }
        } else {
            self.kernel_indices
                .insert(representative_index, self.pending.len());
            self.pending
                .push((map, parameters, coefficients, conditioning, program));
        }
        Ok(())
    }
    pub fn finish(
        self,
        domain: DomainAssessment,
        mut charts: Vec<ChartRecord>,
        max_order: i32,
    ) -> GeneratedIntegral {
        for chart in &mut charts {
            chart.kernel_sector = self.kernel_indices.get(&chart.representative).copied();
        }
        let orders = (self.minimum..=max_order).collect::<Vec<_>>();
        let sectors = self
            .pending
            .into_iter()
            .enumerate()
            .map(
                |(index, (map, parameters, coefficients, conditioning, program))| GeneratedSector {
                    symbolic_jacobian: program.jacobians.first().map(|(_, plan)| {
                        let faces = charts
                            .iter()
                            .find(|chart| {
                                chart.kernel_sector == Some(index)
                                    && chart.source_index == chart.representative
                            })
                            .and_then(|chart| chart.contour.as_ref())
                            .map(|contour| contour.validation_faces().to_vec())
                            .unwrap_or_default();
                        crate::contour::SymbolicContourJacobian {
                            plan: plan.clone(),
                            faces,
                        }
                    }),
                    contour_jacobian: program.contour_jacobian,
                    contour_definitions: program
                        .contour_definitions()
                        .expect("previously merged contour definitions"),
                    program_descriptor: program.descriptor,
                    dynamic_check_sources: program.checks,
                    deferred: None,
                    cancellation_degree: conditioning.degree,
                    cancellation_terms: conditioning.rows,
                    endpoint_profiles: conditioning.endpoint_profiles,
                    conditioning_basis: conditioning.basis,
                    parameters,
                    map,
                    materialized: Default::default(),
                    coefficients: orders
                        .iter()
                        .map(|order| coefficients.get(order).cloned().unwrap_or_default())
                        .collect(),
                },
            )
            .collect();
        let exact_coefficients = orders
            .iter()
            .map(|order| {
                self.exact
                    .get(order)
                    .map(normalize_exact_coefficient)
                    .unwrap_or(Atom::Zero)
            })
            .collect();
        GeneratedIntegral {
            contour_jacobian: self.program.contour_jacobian,
            program_descriptor: self.program.descriptor,
            dynamic_check_sources: self.program.checks,
            metadata: GenerationMetadata {
                domain,
                charts,
                source_scope: None,
            },
            orders,
            sectors,
            exact_coefficients,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contour::ContourDefinitions, generation::GenerationPhase};
    use std::{sync::Arc, time::Instant};
    use symbolica::{id::Pattern, symbol};

    #[test]
    fn restricted_compact_calls_fold_before_exact_aggregation() {
        let x = symbol!("compact_assembly::x");
        let y = symbol!("compact_assembly::y");
        let polynomial = Atom::one()
            + (1..=24)
                .map(|power| Atom::var(x).pow(Atom::num(power)))
                .sum::<Atom>();
        let (definitions, calls) =
            ContourDefinitions::coefficients(&[x, y], &[polynomial]).unwrap();
        assert!(!definitions.is_empty());
        assert!(!calls[0].contains_symbol(y));
        let restricted = calls[0]
            .replace(Pattern::Literal(Atom::var(x)))
            .with(Atom::Zero);
        assert!(!restricted.is_one()); // Still a native function call here.
        let mut assembly = Assembly::new(0);
        let map = SectorMap {
            fixed_parameter: None,
            exponent_matrix: vec![vec![1.into(), 0.into()], vec![0.into(), 1.into()]],
            determinant: 1.into(),
            jacobian_powers: vec![0.into(), 0.into()],
            factor_valuations: vec![],
        };
        let output = coefficients::Output {
            coefficients: BTreeMap::from([(0, AliasedAtom::from(restricted))]),
            conditioning: Profile::retained(vec![], vec![], 2).unwrap(),
            phase: GenerationPhase::CoefficientExpansion,
            phase_started: Instant::now(),
        };
        assembly
            .push(
                0,
                map,
                vec![x, y],
                3,
                output,
                ProgramData {
                    definitions: vec![(0, Arc::new(definitions))],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(assembly.pending.is_empty());
        assert_eq!(assembly.exact[&0], Atom::num(3));
    }
}
