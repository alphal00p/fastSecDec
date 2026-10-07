use super::*;
use std::collections::BTreeSet;
use symbolica::{atom::SymbolBuilder, wrap_symbol};

impl<'a> Requests<'a> {
    pub(crate) fn new(
        sector: &'a DualSector,
        runtime: &'a [Symbol],
        settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        let dimension = sector.parameters.len();
        if sector.map.source_dimension() != sector.source_parameters.len()
            || sector.map.dimension() != dimension
            || sector
                .map
                .exponent_matrix
                .iter()
                .any(|row| row.len() != dimension)
        {
            return Err(compilation(
                "numerical-dual monomial map dimension mismatch",
            ));
        }
        let target_inputs = sector
            .parameters
            .iter()
            .copied()
            .chain([sector.regulator])
            .chain(runtime.iter().copied())
            .collect::<Vec<_>>();
        let target_atoms = target_inputs
            .iter()
            .map(|s| Atom::var(*s))
            .collect::<Vec<_>>();
        let images = sector
            .map
            .exponent_matrix
            .iter()
            .map(|row| {
                row.iter()
                    .zip(&sector.parameters)
                    .fold(Atom::one(), |value, (power, symbol)| {
                        value * Atom::var(*symbol).pow(Atom::num(power.clone()))
                    })
            })
            .chain([Atom::var(sector.regulator)])
            .chain(runtime.iter().map(|s| Atom::var(*s)))
            .collect::<Vec<_>>();
        let map = Atom::evaluator_multiple(&images, &target_atoms)
            .optimization_settings(settings.native())
            .build()
            .map_err(compilation)?;
        let source_inputs = sector
            .source_parameters
            .iter()
            .copied()
            .chain([sector.regulator])
            .chain(runtime.iter().copied())
            .collect();
        let mut occupied = target_inputs
            .iter()
            .chain(&sector.source_parameters)
            .copied()
            .collect::<BTreeSet<_>>();
        for factor in sector.terms.iter().flat_map(|term| &term.factors) {
            occupied.extend(factor.polynomial.get_all_symbols(true));
            occupied.extend(factor.exponent.get_all_symbols(true));
        }
        let max_factors = sector
            .terms
            .iter()
            .map(|term| term.factors.len())
            .max()
            .unwrap_or(0);
        let mut factor_inputs = Vec::with_capacity(max_factors);
        let mut counter = 0;
        while factor_inputs.len() < max_factors {
            let name = format!("fastsecdec::numerical_dual::factor_value{counter}");
            counter += 1;
            if Symbol::get_symbol(wrap_symbol!(&name))
                .is_some_and(|symbol| occupied.contains(&symbol))
            {
                continue;
            }
            let symbol = SymbolBuilder::new(wrap_symbol!(&name))
                .build()
                .map_err(compilation)?;
            if !symbol.is_exportable() {
                return Err(compilation(
                    "native factor placeholder has attached symbolic hooks",
                ));
            }
            occupied.insert(symbol);
            factor_inputs.push(symbol);
        }
        let constants = Atom::evaluator_multiple(&[Atom::Zero, Atom::one()], &[] as &[Atom])
            .optimization_settings(settings.native())
            .build()
            .map_err(compilation)?;
        Ok(Self {
            sector,
            runtime,
            settings,
            source_inputs,
            map,
            factor_inputs,
            constants,
            constant_slots: None,
            seed_slots: BTreeMap::new(),
            map_slots: BTreeMap::new(),
            map_programs: BTreeMap::new(),
            factor_slots: BTreeMap::new(),
            regular_slots: BTreeMap::new(),
        })
    }

    /// Seed coordinate jets at the exact formal face; regulator at zero and
    /// runtime inputs constant. These slots are numeric values, not substituted
    /// polynomial expressions.
    pub(super) fn seeds(
        &mut self,
        request: &Request,
        shape: &Shape,
        composer: &mut EvaluatorComposer<Complex<Rational>>,
    ) -> Result<Vec<Slot>, KernelError> {
        let key = (request.coordinates.clone(), shape.components.clone());
        if let Some(slots) = self.seed_slots.get(&key) {
            return Ok(slots.clone());
        }
        if self.constant_slots.is_none() {
            self.constant_slots = Some(composer.append(&self.constants, &[]).map_err(compilation)?);
        }
        let constants = self.constant_slots.as_ref().unwrap();
        let dimension = self.sector.parameters.len();
        let mut result =
            Vec::with_capacity((dimension + 1 + self.runtime.len()) * shape.components.len());
        for input in 0..dimension + 1 + self.runtime.len() {
            let scalar = if input < dimension {
                match request.coordinates[input] {
                    Coordinate::Variable(index) => {
                        if index >= dimension {
                            return Err(compilation("numerical-dual face variable out of range"));
                        }
                        Slot::Param(index)
                    }
                    Coordinate::Zero => constants[0],
                    Coordinate::One => constants[1],
                }
            } else if input == dimension {
                constants[0]
            } else {
                Slot::Param(input - 1)
            };
            for powers in &shape.components {
                let degree = powers.iter().sum::<usize>();
                result.push(if degree == 0 {
                    scalar
                } else if input <= dimension
                    && degree == 1
                    && powers[if input == dimension { 0 } else { input + 1 }] == 1
                {
                    constants[1]
                } else {
                    constants[0]
                });
            }
        }
        self.seed_slots.insert(key, result.clone());
        Ok(result)
    }

    pub(super) fn seed_zeros(
        &self,
        request: &Request,
        shape: &Shape,
        offset: usize,
    ) -> Vec<(usize, usize)> {
        let dimension = self.sector.parameters.len();
        let mut zeros = Vec::new();
        for input in 0..dimension + 1 + self.runtime.len() {
            for (component, powers) in shape.components.iter().enumerate() {
                let degree = powers.iter().sum::<usize>();
                let nonzero = if degree == 0 {
                    input > dimension
                        || (input < dimension && request.coordinates[input] != Coordinate::Zero)
                } else {
                    input <= dimension
                        && degree == 1
                        && powers[if input == dimension { 0 } else { input + 1 }] == 1
                };
                if !nonzero {
                    zeros.push((offset + input, component));
                }
            }
        }
        zeros
    }

    pub(super) fn source_zeros(&self, request: &Request, shape: &Shape) -> Vec<(usize, usize)> {
        let source_dimension = self.sector.source_parameters.len();
        let mut zeros = Vec::new();
        for input in 0..source_dimension + 1 + self.runtime.len() {
            for (component, powers) in shape.components.iter().enumerate() {
                let nonzero = if input < source_dimension {
                    powers[0] == 0
                        && self.sector.map.exponent_matrix[input]
                            .iter()
                            .zip(&powers[1..])
                            .zip(&request.coordinates)
                            .all(|((power, degree), coordinate)| {
                                let degree = symbolica::domains::integer::Integer::from(*degree);
                                if *coordinate == Coordinate::Zero {
                                    power == &degree
                                } else {
                                    power >= &degree
                                }
                            })
                } else if input == source_dimension {
                    powers[0] == 1 && powers.iter().sum::<usize>() == 1
                } else {
                    powers.iter().all(|power| *power == 0)
                };
                if !nonzero {
                    zeros.push((input, component));
                }
            }
        }
        zeros
    }

    pub(super) fn mapped(
        &mut self,
        request: &Request,
        shape: &Shape,
        composer: &mut EvaluatorComposer<Complex<Rational>>,
    ) -> Result<Vec<Slot>, KernelError> {
        let key = (request.coordinates.clone(), shape.components.clone());
        if let Some(slots) = self.map_slots.get(&key) {
            return Ok(slots.clone());
        }
        let seeds = self.seeds(request, shape, composer)?;
        let zeros = self.seed_zeros(request, shape, 0);
        let program_key = (shape.components.clone(), zeros.clone());
        if !self.map_programs.contains_key(&program_key) {
            let program = self
                .map
                .clone()
                .vectorize(&Dualizer::new(
                    HyperDual::<Complex<Rational>>::new(shape.components.clone()),
                    zeros,
                ))
                .map_err(compilation)?;
            self.map_programs.insert(program_key.clone(), program);
        }
        let slots = composer
            .append(&self.map_programs[&program_key], &seeds)
            .map_err(compilation)?;
        self.map_slots.insert(key, slots.clone());
        Ok(slots)
    }
}
