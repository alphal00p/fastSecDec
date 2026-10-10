//! Compact coefficient functions with native symbolic bodies.
//!
//! This is generation and inspection data. Symbolica owns differentiation,
//! argument substitution, and evaluator lowering; numeric sampling uses the
//! resulting ordinary saved native instructions.
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Indeterminate, Symbol, SymbolAttribute, SymbolBuilder},
    evaluate::FunctionMap,
    id::{Pattern, Replacement},
    wrap_symbol,
};

#[cfg(test)]
mod tests;

const PREFIX: &str = "fastsecdec::contour::coefficient_v1_";

#[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub struct ContourDefinition {
    function: Symbol,
    parameters: Vec<Symbol>,
    body: Atom,
}

impl ContourDefinition {
    pub fn function(&self) -> Symbol {
        self.function
    }
    pub fn parameters(&self) -> &[Symbol] {
        &self.parameters
    }
    pub fn body(&self) -> &Atom {
        &self.body
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub struct ContourDefinitions {
    entries: Vec<ContourDefinition>,
}

impl ContourDefinitions {
    pub(crate) fn from_parts(parts: Vec<(Symbol, Vec<Symbol>, Atom)>) -> Result<Self, String> {
        let result = Self {
            entries: parts
                .into_iter()
                .map(|(function, parameters, body)| ContourDefinition {
                    function,
                    parameters,
                    body,
                })
                .collect(),
        };
        result.validate(false)?;
        Ok(result)
    }

    pub fn entries(&self) -> &[ContourDefinition] {
        &self.entries
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Replace repeated, larger coefficient expressions with native functions.
    /// Every chart function has one full signature. An unused formal argument
    /// receives literal zero, so an independent coordinate remains independent
    /// during exact/stochastic classification and symbolic subtraction.
    pub(crate) fn coefficients(
        coordinates: &[Symbol],
        coefficients: &[Atom],
    ) -> Result<(Self, Vec<Atom>), String> {
        Self::with_required_bodies(coordinates, coefficients, &[])
    }

    /// The designated determinant body remains an explicit native function
    /// even in one dimension, so an evaluator can bind that complete function
    /// Atom to a composed image-derivative program before taking outer jets.
    pub(crate) fn with_required_bodies(
        coordinates: &[Symbol],
        coefficients: &[Atom],
        required: &[usize],
    ) -> Result<(Self, Vec<Atom>), String> {
        if required.iter().any(|index| *index >= coefficients.len()) {
            return Err("required compact body index out of range".into());
        }
        let mut actual = coordinates.to_vec();
        if actual.iter().copied().collect::<BTreeSet<_>>().len() != actual.len() {
            return Err("duplicate compact coefficient coordinate".into());
        }
        let mut extra = coefficients
            .iter()
            .flat_map(|a| a.get_all_symbols(false))
            .filter(|p| !coordinates.contains(p))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        extra.sort_by_key(|p| Atom::var(*p).to_canonical_string());
        actual.extend(extra);
        if actual.is_empty() {
            return Ok((Self::default(), coefficients.to_vec()));
        }
        let formal = (0..actual.len())
            .map(|i| owned_symbol(&format!("fastsecdec::contour::coefficient_argument_{i}")))
            .collect::<Result<Vec<_>, _>>()?;
        let rules = actual
            .iter()
            .zip(&formal)
            .map(|(from, to)| {
                Replacement::new(
                    Pattern::Literal(Atom::var(*from)),
                    Pattern::Literal(Atom::var(*to)),
                )
            })
            .collect::<Vec<_>>();
        let mut definitions = Self::default();
        let mut result = Vec::with_capacity(coefficients.len());
        for (index, coefficient) in coefficients.iter().enumerate() {
            let body = coefficient.replace_multiple(&rules);
            let function = owned_symbol(&format!("{PREFIX}{}", identity(&body, formal.len())))?;
            let arguments = actual.iter().map(|p| {
                if coefficient.contains_symbol(*p) {
                    Atom::var(*p)
                } else {
                    Atom::Zero
                }
            });
            let call = function.call_args(arguments);
            // A literal or short expression needs no additional representation.
            if !required.contains(&index)
                && coefficient.as_view().get_byte_size() <= call.as_view().get_byte_size()
            {
                result.push(coefficient.clone());
                continue;
            }
            if let Some(existing) = definitions
                .entries
                .iter()
                .find(|entry| entry.function == function)
            {
                if existing.parameters != formal || existing.body != body {
                    return Err(
                        "compact coefficient name has a conflicting native definition".into(),
                    );
                }
            } else {
                definitions.entries.push(ContourDefinition {
                    function,
                    parameters: formal.clone(),
                    body,
                });
            }
            result.push(call);
        }
        definitions.validate(false)?;
        Ok((definitions, result))
    }

    /// Structural admission is mandatory. Optional identity verification also
    /// checks the canonical native body against its stable definition name.
    pub fn validate(&self, verify_identity: bool) -> Result<(), String> {
        let mut functions = BTreeSet::new();
        let roots = [
            Symbol::get_symbol(wrap_symbol!("fastsecdec::contour::dynamic::strength_v1")),
            Symbol::get_symbol(wrap_symbol!(
                "fastsecdec::contour::dynamic::requested_strength_v1"
            )),
        ];
        for entry in &self.entries {
            if !entry.function.get_name().starts_with(PREFIX)
                || !entry.function.is_exportable()
                || entry.function.has_attributes()
                || entry.parameters.is_empty()
                || entry
                    .parameters
                    .iter()
                    .enumerate()
                    .any(|(index, parameter)| {
                        !parameter.is_exportable()
                            || parameter.has_attributes()
                            || parameter.get_name()
                                != format!("fastsecdec::contour::coefficient_argument_{index}")
                    })
                || !functions.insert(entry.function)
                || entry
                    .parameters
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .len()
                    != entry.parameters.len()
                || entry
                    .body
                    .get_all_symbols(false)
                    .iter()
                    .any(|p| !entry.parameters.contains(p))
                || entry.body.get_all_symbols(true).iter().any(|p| {
                    p.get_name().starts_with(PREFIX) || roots.iter().flatten().any(|root| root == p)
                })
            {
                return Err("invalid compact contour coefficient definition".into());
            }
            if verify_identity
                && entry.function.get_name()
                    != format!("{PREFIX}{}", identity(&entry.body, entry.parameters.len()))
            {
                return Err("compact contour coefficient body identity mismatch".into());
            }
        }
        Ok(())
    }

    /// Retain only definitions referenced by these native expressions. A
    /// selected program must not bind another chart's definitions implicitly.
    pub(crate) fn select<'a>(
        &self,
        expressions: impl IntoIterator<Item = &'a Atom>,
    ) -> Result<Self, String> {
        let mut functions = BTreeSet::new();
        for expression in expressions {
            let mut error = None;
            expression.visitor(&mut |term| {
                match self.call(term) {
                    Ok(Some(call)) => {
                        functions.insert(call.definition.function);
                    }
                    Ok(None) => {}
                    Err(reason) => {
                        error.get_or_insert(reason);
                    }
                }
                true
            });
            if let Some(error) = error {
                return Err(error);
            }
        }
        Ok(Self {
            entries: self
                .entries
                .iter()
                .filter(|e| functions.contains(&e.function))
                .cloned()
                .collect(),
        })
    }

    pub(crate) fn merge(&mut self, other: &Self) -> Result<(), String> {
        for incoming in &other.entries {
            if let Some(existing) = self
                .entries
                .iter()
                .find(|e| e.function == incoming.function)
            {
                if existing.parameters != incoming.parameters || existing.body != incoming.body {
                    return Err("conflicting compact contour coefficient definition".into());
                }
            } else {
                self.entries.push(incoming.clone());
            }
        }
        self.entries
            .sort_by_key(|e| e.function.get_name().to_owned());
        Ok(())
    }

    /// Resolve compact calls with native argument substitution. In particular,
    /// exact offsets call this after restriction and before native aggregation;
    /// coefficient definitions never survive into direct numeric exact binding.
    pub fn materialize(&self, expression: &Atom) -> Result<Atom, String> {
        let mut error = None;
        let mut bodies = BTreeMap::new();
        let result = expression.replace_map_bottom_up(|term, _, out| {
            let call = match self.call(term) {
                Ok(Some(call)) => call,
                Ok(None) => return,
                Err(reason) => {
                    error.get_or_insert(reason);
                    return;
                }
            };
            let body = bodies.entry(call.key()).or_insert_with(|| call.body());
            **out =
                body.replace_multiple(call.definition.parameters.iter().zip(&call.arguments).map(
                    |(from, to)| {
                        Replacement::new(
                            Pattern::Literal(Atom::var(*from)),
                            Pattern::Literal(to.clone()),
                        )
                    },
                ));
        });
        error.map_or(Ok(result), Err)
    }

    /// Simplify restricted calls without expanding surviving stochastic work.
    /// Symbolica proves literal zeros and coordinate independence from each
    /// substituted body. Repeated occurrences reuse a bounded per-expression
    /// cache keyed by the same short native call.
    pub(crate) fn simplify(
        &self,
        expression: &Atom,
        coordinates: &[Symbol],
    ) -> Result<Atom, String> {
        let mut error = None;
        let mut bodies = BTreeMap::new();
        let mut replacements = BTreeMap::<Atom, Option<Atom>>::new();
        let result = expression.replace_map_bottom_up(|term, _, out| {
            let call = match self.call(term) {
                Ok(Some(call)) => call,
                Ok(None) => return,
                Err(reason) => {
                    error.get_or_insert(reason);
                    return;
                }
            };
            let replacement = replacements.entry(term.to_owned()).or_insert_with(|| {
                let body = bodies.entry(call.key()).or_insert_with(|| call.body());
                let expanded = body.replace_multiple(
                    call.definition
                        .parameters
                        .iter()
                        .zip(&call.arguments)
                        .map(|(from, to)| {
                            Replacement::new(
                                Pattern::Literal(Atom::var(*from)),
                                Pattern::Literal(to.clone()),
                            )
                        }),
                );
                if expanded.is_zero() || !coordinates.iter().any(|p| expanded.contains_symbol(*p)) {
                    return Some(expanded);
                }
                // Correlated compound arguments can cancel a coordinate only
                // in combination. Rewriting one plain argument would break
                // that identity; keep the compact call intact in this case.
                if call.arguments.iter().any(|argument| {
                    !matches!(argument.as_view(), AtomView::Var(_) | AtomView::Num(_))
                }) {
                    return None;
                }
                // Restriction can erase only part of a call's original
                // dependence. Native symbol absence proves an inert input.
                let mut arguments = call.arguments.clone();
                let mut changed = false;
                for argument in &mut arguments {
                    if let Some(variable) = argument.as_var_view()
                        && coordinates.contains(&variable.get_symbol())
                        && !expanded.contains_symbol(variable.get_symbol())
                    {
                        *argument = Atom::Zero;
                        changed = true;
                    }
                }
                changed.then(|| {
                    if call.orders.iter().all(|order| *order == 0) {
                        call.definition.function.call_args(arguments)
                    } else {
                        Symbol::DERIVATIVE.call_args(
                            call.orders
                                .iter()
                                .map(|order| Atom::num(*order))
                                .chain([Atom::var(call.definition.function)])
                                .chain(arguments),
                        )
                    }
                })
            });
            if let Some(replacement) = replacement {
                **out = replacement.clone();
            }
        });
        error.map_or(Ok(result), Err)
    }

    /// Construct the native function map only for surviving calls. Derivative
    /// bodies are produced by native Atom differentiation, never by a separate
    /// chain-rule implementation. Always-inline is required by native Dualizer.
    pub fn function_map<'a>(
        &self,
        expressions: impl IntoIterator<Item = &'a Atom>,
    ) -> Result<FunctionMap, String> {
        let mut needed = BTreeMap::new();
        for expression in expressions {
            let mut error = None;
            expression.visitor(&mut |term| {
                match self.call(term) {
                    Ok(Some(call)) => {
                        needed.entry(call.key()).or_insert_with(|| {
                            (
                                call.registration(),
                                call.definition.parameters.clone(),
                                call.body(),
                            )
                        });
                    }
                    Ok(None) => {}
                    Err(reason) => {
                        error.get_or_insert(reason);
                    }
                }
                true
            });
            if let Some(error) = error {
                return Err(error);
            }
        }
        let arities = needed
            .values()
            .map(|(_, parameters, _)| parameters.len())
            .collect::<BTreeSet<_>>();
        if arities.len() > 1 {
            return Err(
                "one native contour program requires a uniform full-source function signature"
                    .into(),
            );
        }
        let mut native = FunctionMap::new();
        for (_, (name, parameters, body)) in needed {
            native
                .add_function(name, parameters, body)
                .map_err(|e| e.to_string())?;
        }
        Ok(native)
    }

    fn call(&self, atom: AtomView<'_>) -> Result<Option<Call<'_>>, String> {
        let Some(function) = atom.as_fun_view() else {
            // Native DERIVATIVE tags contain the function name as a Var.
            // Recursive visitors must leave that static tag untouched.
            return Ok(None);
        };
        let symbol = function.get_symbol();
        if symbol == Symbol::DERIVATIVE {
            if function.get_nargs().is_multiple_of(2) {
                return Ok(None);
            }
            let arity = (function.get_nargs() - 1) / 2;
            let Some(name) = function.get(arity).as_var_view().map(|v| v.get_symbol()) else {
                return Ok(None);
            };
            if !name.get_name().starts_with(PREFIX) {
                return Ok(None);
            }
            let definition = self.find(name)?;
            if arity != definition.parameters.len() {
                return Err("compact contour derivative arity mismatch".into());
            }
            let orders = (0..arity)
                .map(|i| {
                    u32::try_from(function.get(i))
                        .map_err(|_| "invalid compact contour derivative order".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let arguments = (arity + 1..function.get_nargs())
                .map(|i| function.get(i).to_owned())
                .collect();
            Ok(Some(Call {
                definition,
                orders,
                arguments,
            }))
        } else if symbol.get_name().starts_with(PREFIX) {
            let definition = self.find(symbol)?;
            if function.get_nargs() != definition.parameters.len() {
                return Err("compact contour function arity mismatch".into());
            }
            Ok(Some(Call {
                definition,
                orders: vec![0; definition.parameters.len()],
                arguments: function.iter().map(|a| a.to_owned()).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    fn find(&self, function: Symbol) -> Result<&ContourDefinition, String> {
        self.entries
            .iter()
            .find(|e| e.function == function)
            .ok_or_else(|| "compact contour function lacks its native definition".into())
    }
}

fn owned_symbol(name: &str) -> Result<Symbol, String> {
    let function = SymbolBuilder::new(wrap_symbol!(name))
        .with_attributes(&[] as &[SymbolAttribute])
        .build()
        .map_err(|error| format!("reserved compact contour symbol {name}: {error}"))?;
    if !function.is_exportable() || function.has_attributes() {
        return Err(format!(
            "reserved compact contour symbol {name} has existing native hooks or attributes"
        ));
    }
    Ok(function)
}

fn identity(body: &Atom, arity: usize) -> String {
    let mut digest = blake3::Hasher::new();
    digest.update(b"fastsecdec-contour-coefficient-v1\0");
    digest.update(&(arity as u64).to_le_bytes());
    digest.update(body.to_canonical_string().as_bytes());
    digest.finalize().to_hex().to_string()
}

struct Call<'a> {
    definition: &'a ContourDefinition,
    orders: Vec<u32>,
    arguments: Vec<Atom>,
}
impl Call<'_> {
    fn key(&self) -> (Symbol, Vec<u32>) {
        (self.definition.function, self.orders.clone())
    }
    fn registration(&self) -> Indeterminate {
        if self.orders.iter().all(|order| *order == 0) {
            self.definition.function.into()
        } else {
            let tags = self
                .orders
                .iter()
                .map(|order| Atom::num(*order))
                .chain([Atom::var(self.definition.function)]);
            Indeterminate::try_from(Symbol::DERIVATIVE.call_args(tags))
                .expect("native function indeterminate")
        }
    }
    fn body(&self) -> Atom {
        let mut body = self.definition.body.clone();
        for (parameter, order) in self.definition.parameters.iter().zip(&self.orders) {
            for _ in 0..*order {
                body = body.derivative(*parameter);
            }
        }
        body
    }
}
