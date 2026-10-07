use super::super::{
    DualSector,
    subtraction::{Coordinate, Request},
};
use super::{ExactProgram, compilation};
use crate::kernel::{CompilationSettings, KernelError};
use std::collections::BTreeMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{dual::HyperDual, float::Complex, rational::Rational},
    evaluate::{Dualizer, EvaluatorComposer, Slot},
};

mod inputs;
mod shape;
use shape::Shape;

type ProgramKey = (Vec<Vec<usize>>, Vec<(usize, usize)>);
type MapKey = (Vec<Coordinate>, Vec<Vec<usize>>);
type FactorKey = (Atom, Vec<i32>, Vec<usize>, Vec<Coordinate>);
type RegularKey = (Vec<(Atom, Atom, Vec<i32>)>, Vec<usize>, Vec<Coordinate>);

pub(super) struct Requests<'a> {
    sector: &'a DualSector,
    runtime: &'a [Symbol],
    settings: CompilationSettings,
    source_inputs: Vec<Symbol>,
    map: ExactProgram,
    factor_inputs: Vec<Symbol>,
    constants: ExactProgram,
    constant_slots: Option<Vec<Slot>>,
    seed_slots: BTreeMap<MapKey, Vec<Slot>>,
    map_slots: BTreeMap<MapKey, Vec<Slot>>,
    map_programs: BTreeMap<ProgramKey, ExactProgram>,
    factor_slots: BTreeMap<FactorKey, Vec<Slot>>,
    regular_slots: BTreeMap<RegularKey, Slot>,
}
impl<'a> Requests<'a> {
    pub(super) fn append(
        &mut self,
        request: &Request,
        composer: &mut EvaluatorComposer<Complex<Rational>>,
    ) -> Result<Slot, KernelError> {
        let dimension = self.sector.parameters.len();
        if request.coordinates.len() != dimension || request.derivatives.len() != dimension {
            return Err(compilation("numerical-dual request dimension mismatch"));
        }
        let sector = self.sector;
        let term = sector
            .terms
            .get(request.term)
            .ok_or_else(|| compilation("numerical-dual request term out of range"))?;
        let desired = std::iter::once(request.epsilon_order)
            .chain(request.derivatives.iter().copied())
            .collect::<Vec<_>>();
        let regular_key = (
            term.factors
                .iter()
                .map(|factor| {
                    (
                        factor.polynomial.clone(),
                        factor.exponent.clone(),
                        factor.valuation.clone(),
                    )
                })
                .collect(),
            desired.clone(),
            request.coordinates.clone(),
        );
        if let Some(slot) = self.regular_slots.get(&regular_key) {
            return Ok(*slot);
        }
        let target = Shape::new(&desired)?;
        let mut factor_slots = Vec::new();
        let mut regular = Atom::one();
        for (factor_index, factor) in term.factors.iter().enumerate() {
            if factor.valuation.len() != dimension {
                return Err(compilation(
                    "numerical-dual factor valuation dimension mismatch",
                ));
            }
            let factor_key = (
                factor.polynomial.clone(),
                factor.valuation.clone(),
                desired.clone(),
                request.coordinates.clone(),
            );
            if let Some(slots) = self.factor_slots.get(&factor_key) {
                factor_slots.extend_from_slice(slots);
            } else {
                let shift = std::iter::once(0usize)
                    .chain(
                        factor
                            .valuation
                            .iter()
                            .zip(&request.coordinates)
                            .map(|(valuation, coordinate)| {
                                if *coordinate == Coordinate::Zero {
                                    usize::try_from(*valuation).map_err(|_| {
                                        compilation("negative valuation at numerical-dual endpoint")
                                    })
                                } else {
                                    Ok(0)
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                    .collect::<Vec<_>>();
                let maxima = desired
                    .iter()
                    .zip(&shift)
                    .map(|(a, b)| {
                        a.checked_add(*b)
                            .ok_or_else(|| compilation("numerical-dual valuation shift overflow"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let source = Shape::new(&maxima)?;
                let mapped = self.mapped(request, &source, composer)?;
                let polynomial = self.sector.programs.jets(
                    &factor.polynomial,
                    &self.source_inputs,
                    &source.components,
                    &self.source_zeros(request, &source),
                    self.settings,
                )?;
                let values = composer.append(&polynomial, &mapped).map_err(compilation)?;
                let mut selected = Vec::with_capacity(target.components.len());
                for powers in &target.components {
                    let shifted = powers
                        .iter()
                        .zip(&shift)
                        .map(|(a, b)| a + b)
                        .collect::<Vec<_>>();
                    selected.push(values[source.position(&shifted)?]);
                }
                factor_slots.extend_from_slice(&selected);
                self.factor_slots.insert(factor_key, selected);
            }
            let mut residual = Atom::var(self.factor_inputs[factor_index]);
            for ((valuation, coordinate), symbol) in factor
                .valuation
                .iter()
                .zip(&request.coordinates)
                .zip(&self.sector.parameters)
            {
                if *coordinate != Coordinate::Zero && *valuation != 0 {
                    residual *= Atom::var(*symbol).pow(Atom::num(-i64::from(*valuation)));
                }
            }
            regular *= residual.pow(&factor.exponent);
        }
        let params = self.factor_inputs[..term.factors.len()]
            .iter()
            .chain(&self.sector.parameters)
            .copied()
            .chain([self.sector.regulator])
            .chain(self.runtime.iter().copied())
            .map(Atom::var)
            .collect::<Vec<_>>();
        let regular = regular
            .evaluator(&params)
            .optimization_settings(self.settings.native())
            .build()
            .map_err(compilation)?
            .vectorize(&Dualizer::new(
                HyperDual::<Complex<Rational>>::new(target.components.clone()),
                self.seed_zeros(request, &target, term.factors.len()),
            ))
            .map_err(compilation)?;
        factor_slots.extend(self.seeds(request, &target, composer)?);
        let output = composer
            .append(&regular, &factor_slots)
            .map_err(compilation)?;
        let result = output[target.position(&desired)?];
        self.regular_slots.insert(regular_key, result);
        Ok(result)
    }
}
