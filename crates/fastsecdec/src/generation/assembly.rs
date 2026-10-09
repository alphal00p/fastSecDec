//! Shared exact multiplicity, zero-dimensional folding and Laurent assembly.
use super::{
    ChartRecord, DomainAssessment, GeneratedIntegral, GeneratedSector, GenerationError,
    GenerationMetadata, coefficients, conditioning::Profile, program::ProgramData,
};
use fastsecdec_sectors::SectorMap;
use std::collections::BTreeMap;
use symbolica::atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol};
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
        self.program.merge(&program)?;
        let coefficients = output
            .coefficients
            .into_iter()
            .map(|(order, coefficient)| {
                (
                    order,
                    coefficient.map_root(|root| root * Atom::num(multiplicity)),
                )
            })
            .collect::<BTreeMap<_, _>>();
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
                *self.exact.entry(order).or_insert(Atom::Zero) += coefficient.into_inner();
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
            .map(
                |(map, parameters, coefficients, conditioning, program)| GeneratedSector {
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
            .map(|order| self.exact.get(order).cloned().unwrap_or(Atom::Zero))
            .collect();
        GeneratedIntegral {
            program_descriptor: self.program.descriptor,
            dynamic_check_sources: self.program.checks,
            metadata: GenerationMetadata { domain, charts },
            orders,
            sectors,
            exact_coefficients,
        }
    }
}
