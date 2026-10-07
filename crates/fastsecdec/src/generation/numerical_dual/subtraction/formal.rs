//! Decode native formal-function derivatives; no regular-function algebra.
use super::{Coordinate, Recipe, Request};
use crate::generation::{GenerationError, mapping::MappedTerm};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol, SymbolAttribute, SymbolBuilder},
    coefficient::Coefficient,
    domains::integer::Integer,
    wrap_symbol,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    term: usize,
    epsilon_order: usize,
    derivatives: Vec<usize>,
    coordinates: Vec<Coordinate>,
}

pub(super) struct Functions {
    parameters: Vec<Symbol>,
    regulator: Symbol,
    occupied: BTreeSet<Symbol>,
    functions: BTreeMap<Symbol, usize>,
    next_function: usize,
    next_request: usize,
}

impl Functions {
    pub(super) fn new(
        terms: &[MappedTerm],
        parameters: &[Symbol],
        regulator: Symbol,
        reserved: &[Symbol],
    ) -> Result<Self, GenerationError> {
        let mut occupied = reserved
            .iter()
            .chain(parameters)
            .copied()
            .collect::<BTreeSet<_>>();
        occupied.insert(regulator);
        for term in terms {
            for expression in [&term.regular, &term.prefactor]
                .into_iter()
                .chain(&term.powers)
            {
                occupied.extend(expression.get_all_symbols(true));
            }
        }
        Ok(Self {
            parameters: parameters.to_vec(),
            regulator,
            occupied,
            functions: BTreeMap::new(),
            next_function: 0,
            next_request: 0,
        })
    }

    fn fresh(&mut self, function: bool) -> Result<Symbol, GenerationError> {
        loop {
            let (kind, next) = if function {
                ("f", &mut self.next_function)
            } else {
                ("r", &mut self.next_request)
            };
            let name = format!("fastsecdec::numerical_dual::{kind}{}", *next);
            *next = next
                .checked_add(1)
                .ok_or(GenerationError::ResourceLimit("dual recipe symbols"))?;
            if Symbol::get_symbol(wrap_symbol!(&name)).is_some_and(|s| self.occupied.contains(&s)) {
                continue;
            }
            let symbol = SymbolBuilder::new(wrap_symbol!(&name))
                .with_attributes(&[] as &[SymbolAttribute])
                .build()
                .map_err(|error| GenerationError::ReservedSymbolConflict {
                    name: name.clone(),
                    reason: error.to_string(),
                })?;
            if !symbol.is_exportable() {
                return Err(GenerationError::ReservedSymbolConflict {
                    name,
                    reason: "an existing native hook is attached to the symbol".into(),
                });
            }
            if self.occupied.insert(symbol) {
                return Ok(symbol);
            }
        }
    }

    pub(super) fn function(&mut self, term: usize) -> Result<Atom, GenerationError> {
        let symbol = self.fresh(true)?;
        self.functions.insert(symbol, term);
        Ok(symbol.call(
            &std::iter::once(Atom::var(self.regulator))
                .chain(self.parameters.iter().copied().map(Atom::var))
                .collect::<Vec<_>>(),
        ))
    }

    fn request(&self, expression: AtomView<'_>) -> Result<Option<Key>, GenerationError> {
        let AtomView::Fun(function) = expression else {
            return Ok(None);
        };
        let (term, depths, arguments) =
            if let Some(term) = self.functions.get(&function.get_symbol()) {
                (
                    *term,
                    vec![0; self.parameters.len() + 1],
                    function.iter().collect::<Vec<_>>(),
                )
            } else if function.get_symbol() == Symbol::DERIVATIVE
                && function.get_nargs() >= 3
                && function.get_nargs() % 2 == 1
            {
                let arguments = function.iter().collect::<Vec<_>>();
                let arity = (arguments.len() - 1) / 2;
                let AtomView::Var(name) = arguments[arity] else {
                    return Ok(None);
                };
                let Some(term) = self.functions.get(&name.get_symbol()) else {
                    return Ok(None);
                };
                let depths = arguments[..arity]
                    .iter()
                    .copied()
                    .map(depth)
                    .collect::<Result<Vec<_>, _>>()?;
                (*term, depths, arguments[arity + 1..].to_vec())
            } else {
                return Ok(None);
            };
        if arguments.len() != self.parameters.len() + 1 || depths.len() != arguments.len() {
            return Err(GenerationError::Invariant(
                "dual formal function arity".into(),
            ));
        }
        if !arguments[0].is_zero() {
            return Err(GenerationError::Invariant(
                "dual formal regulator is not centered at zero".into(),
            ));
        }
        let coordinates = arguments[1..]
            .iter()
            .map(|value| {
                if value.is_zero() {
                    Ok(Coordinate::Zero)
                } else if value.is_one() {
                    Ok(Coordinate::One)
                } else if let AtomView::Var(value) = value {
                    self.parameters
                        .iter()
                        .position(|p| *p == value.get_symbol())
                        .map(Coordinate::Variable)
                        .ok_or_else(|| {
                            GenerationError::Invariant(
                                "unknown coordinate in dual formal function".into(),
                            )
                        })
                } else {
                    Err(GenerationError::Invariant(
                        "unsupported dual formal face argument".into(),
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(Key {
            term,
            epsilon_order: depths[0],
            derivatives: depths[1..].to_vec(),
            coordinates,
        }))
    }

    pub(super) fn lower(
        mut self,
        coefficients: BTreeMap<i32, AliasedAtom>,
        maximum: Option<usize>,
        poll: &mut impl FnMut() -> Result<(), GenerationError>,
    ) -> Result<Recipe, GenerationError> {
        let mut keys = BTreeSet::new();
        let mut aliases = BTreeMap::new();
        for coefficient in coefficients.values() {
            for (handle, body) in coefficient.get_aliases() {
                if let Some(old) = aliases.insert(handle.clone(), body.clone())
                    && old != *body
                {
                    return Err(GenerationError::Invariant(
                        "inconsistent dual recipe aliases".into(),
                    ));
                }
            }
        }
        // Assign placeholders in semantic tuple order, independent of the
        // native process-local symbol IDs or expression traversal order.
        for expression in coefficients
            .values()
            .map(AliasedAtom::get_root)
            .chain(aliases.values())
        {
            poll()?;
            let mut error = None;
            let _ = expression.replace_map(|value, _, _| {
                if error.is_some() {
                    return;
                }
                match poll().and_then(|_| self.request(value)) {
                    Ok(Some(key)) => {
                        keys.insert(key);
                        if maximum.is_some_and(|limit| keys.len() > limit) {
                            error = Some(GenerationError::ResourceLimit("dual unique requests"));
                        }
                    }
                    Ok(None) => (),
                    Err(failure) => error = Some(failure),
                }
            });
            if let Some(error) = error {
                return Err(error);
            }
        }
        let mut replacements = BTreeMap::new();
        let mut requests = Vec::new();
        for key in keys {
            poll()?;
            let placeholder = self.fresh(false)?;
            let mut scale = Integer::from(1);
            for order in std::iter::once(key.epsilon_order).chain(key.derivatives.iter().copied()) {
                for factor in 2..=order {
                    scale *= Integer::from(factor);
                }
            }
            replacements.insert(key.clone(), Atom::num(scale) * Atom::var(placeholder));
            requests.push(Request {
                placeholder,
                term: key.term,
                epsilon_order: key.epsilon_order,
                derivatives: key.derivatives,
                coordinates: key.coordinates,
            });
        }
        let mut replace = |expression: &Atom| -> Result<Atom, GenerationError> {
            poll()?;
            let mut error = None;
            let result = expression.replace_map(|value, _, out| {
                if error.is_some() {
                    return;
                }
                match poll().and_then(|_| self.request(value)) {
                    Ok(Some(key)) => match replacements.get(&key) {
                        Some(value) => **out = value.clone(),
                        None => {
                            error = Some(GenerationError::Invariant("missing dual request".into()))
                        }
                    },
                    Ok(None) => (),
                    Err(failure) => error = Some(failure),
                }
            });
            if let Some(error) = error {
                return Err(error);
            }
            let symbols = result.get_all_symbols(true);
            if self.functions.keys().any(|symbol| symbols.contains(symbol)) {
                return Err(GenerationError::Invariant(
                    "unresolved dual formal function".into(),
                ));
            }
            Ok(result)
        };
        let aliases = aliases
            .into_iter()
            .map(|(handle, body)| Ok((handle, replace(&body)?)))
            .collect::<Result<BTreeMap<_, _>, GenerationError>>()?;
        let mut output = BTreeMap::new();
        for (order, coefficient) in coefficients {
            let mut coefficient = AliasedAtom::from(replace(coefficient.get_root())?);
            for (handle, body) in &aliases {
                coefficient.register_alias(handle.clone(), body.clone());
            }
            output.insert(order, coefficient);
        }
        Ok(Recipe {
            coefficients: output,
            requests,
        })
    }
}

fn depth(value: AtomView<'_>) -> Result<usize, GenerationError> {
    if let AtomView::Num(value) = value
        && let Coefficient::Complex(value) = value.get_coeff_view().to_owned()
        && value.im.is_zero()
        && value.re.is_integer()
        && value.re >= 0
    {
        return value
            .re
            .numerator()
            .to_i64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or(GenerationError::ResourceLimit("dual derivative degree"));
    }
    Err(GenerationError::Invariant(
        "invalid native dual derivative degree".into(),
    ))
}
