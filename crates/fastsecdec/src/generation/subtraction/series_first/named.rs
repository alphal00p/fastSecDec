//! Test-only names for native regular-series coefficients. Symbolica owns
//! derivatives, face substitutions, series arithmetic and alias definitions.
mod tests;

use super::*;
use std::{cell::RefCell, collections::BTreeSet};
use symbolica::{
    atom::{AliasedAtom, AtomView},
    coefficient::Coefficient,
    symbol,
};

#[derive(Clone)]
struct Body {
    value: Atom,
    parameters: Vec<Symbol>,
}

type Request = (Symbol, Vec<usize>, Vec<Atom>);

#[derive(Default)]
struct State {
    occupied: BTreeSet<Symbol>,
    next_name: BTreeMap<&'static str, usize>,
    ids: BTreeMap<Atom, Symbol>,
    bodies: BTreeMap<Symbol, Body>,
    partials: BTreeMap<(Symbol, Vec<usize>), Atom>,
    faces: BTreeMap<Request, Atom>,
}

pub(super) struct Coefficients {
    parameters: Vec<Symbol>,
    regulator: Symbol,
    state: RefCell<State>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Statistics {
    pub source_bodies: usize,
    pub derivative_requests: usize,
    pub face_requests: usize,
    pub aliases: usize,
    pub root_bytes: usize,
    pub definition_bytes: usize,
    pub resolution_seconds: f64,
}

type NamedExpansion = (BTreeMap<i32, AliasedAtom>, Vec<Attempt>, Statistics);

impl Coefficients {
    fn new(terms: &[MappedTerm], parameters: &[Symbol], regulator: Symbol) -> Self {
        let mut occupied = parameters.iter().copied().collect::<BTreeSet<_>>();
        occupied.insert(regulator);
        for term in terms {
            for expression in [&term.prefactor, &term.regular]
                .into_iter()
                .chain(&term.powers)
            {
                occupied.extend(expression.get_all_symbols(true));
            }
        }
        Self {
            parameters: parameters.to_vec(),
            regulator,
            state: RefCell::new(State {
                occupied,
                ..State::default()
            }),
        }
    }

    fn fresh(state: &mut State, kind: &'static str) -> Symbol {
        loop {
            let next = state.next_name.entry(kind).or_default();
            let index = *next;
            *next = next
                .checked_add(1)
                .expect("test-only name counter overflow");
            let candidate = symbol!(format!("fastsecdec::named_regular::{kind}{index}"));
            if state.occupied.insert(candidate) {
                return candidate;
            }
        }
    }

    fn wrap(&self, coefficient: &Atom) -> Atom {
        assert!(!coefficient.contains(Atom::var(self.regulator).as_view()));
        let parameters = self
            .parameters
            .iter()
            .copied()
            .filter(|p| coefficient.contains(Atom::var(*p).as_view()))
            .collect::<Vec<_>>();
        if parameters.is_empty() {
            return coefficient.clone();
        }
        let mut state = self.state.borrow_mut();
        let name = match state.ids.get(coefficient) {
            Some(name) => *name,
            None => {
                let name = Self::fresh(&mut state, "f");
                state.ids.insert(coefficient.clone(), name);
                state.bodies.insert(
                    name,
                    Body {
                        value: coefficient.clone(),
                        parameters: parameters.clone(),
                    },
                );
                name
            }
        };
        name.call(&parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>())
    }

    pub(super) fn wrap_series(&self, series: &NativeSeries) -> NativeSeries {
        let named = series.map_coeff(|coefficient| self.wrap(coefficient));
        assert_eq!(named.absolute_order(), series.absolute_order());
        let restored = named.map_coeff(|coefficient| self.restore(coefficient));
        assert_eq!(restored.absolute_order(), series.absolute_order());
        assert_eq!(
            restored.terms().collect::<Vec<_>>(),
            series.terms().collect::<Vec<_>>(),
            "exact coefficient naming identity"
        );
        named
    }

    fn request(&self, term: AtomView) -> Option<Request> {
        let AtomView::Fun(function) = term else {
            return None;
        };
        let state = self.state.borrow();
        if let Some(body) = state.bodies.get(&function.get_symbol()) {
            assert_eq!(function.get_nargs(), body.parameters.len());
            return Some((
                function.get_symbol(),
                vec![0; body.parameters.len()],
                function.iter().map(|a| a.to_owned()).collect(),
            ));
        }
        if function.get_symbol() != Symbol::DERIVATIVE
            || function.get_nargs() < 3
            || function.get_nargs() % 2 == 0
        {
            return None;
        }
        let n = (function.get_nargs() - 1) / 2;
        let args = function.iter().collect::<Vec<_>>();
        let AtomView::Var(source) = args[n] else {
            return None;
        };
        let body = state.bodies.get(&source.get_symbol())?;
        assert_eq!(n, body.parameters.len());
        let depths = args[..n]
            .iter()
            .map(|a| {
                let AtomView::Num(number) = a else {
                    panic!("native derivative depth");
                };
                let Coefficient::Complex(value) = number.get_coeff_view().to_owned() else {
                    panic!("native rational depth");
                };
                assert!(value.im.is_zero() && value.re.is_integer());
                usize::try_from(value.re.numerator().to_i64().unwrap()).unwrap()
            })
            .collect();
        Some((
            source.get_symbol(),
            depths,
            args[n + 1..].iter().map(|a| a.to_owned()).collect(),
        ))
    }

    fn resolve(&self, request: Request) -> Atom {
        let mut state = self.state.borrow_mut();
        if let Some(value) = state.faces.get(&request) {
            return value.clone();
        }
        let (source, depths, arguments) = &request;
        let body = state.bodies[source].clone();
        let partial = state
            .partials
            .entry((*source, depths.clone()))
            .or_insert_with(|| {
                let mut value = body.value.clone();
                for (parameter, count) in body.parameters.iter().zip(depths) {
                    for _ in 0..*count {
                        value = value.derivative(*parameter);
                    }
                }
                value
            });
        let replacements = body
            .parameters
            .iter()
            .copied()
            .zip(arguments.iter())
            .collect::<BTreeMap<_, _>>();
        let value = partial.replace_map(|term, _, out| {
            if let AtomView::Var(variable) = term
                && let Some(value) = replacements.get(&variable.get_symbol())
            {
                **out = (*value).clone();
            }
        });
        state.faces.insert(request, value.clone());
        value
    }

    fn restore(&self, expression: &Atom) -> Atom {
        expression.replace_map(|term, _, out| {
            if let Some(request) = self.request(term) {
                **out = self.resolve(request);
            }
        })
    }

    pub(super) fn assert_independent(&self, expression: &Atom) {
        let state = self.state.borrow();
        let symbols = expression.get_all_symbols(true);
        assert!(
            state.bodies.keys().all(|name| !symbols.contains(name)),
            "formal coefficient used as a scalar multiplier/nonlinear input"
        );
    }

    fn assert_resolved(&self, expression: &Atom) {
        self.assert_independent(expression);
        expression.visitor(&mut |term| {
            if let AtomView::Fun(function) = term {
                assert_ne!(
                    function.get_symbol(),
                    Symbol::DERIVATIVE,
                    "unresolved native derivative request at executable boundary"
                );
            }
            true
        });
    }

    fn finish(
        &self,
        coefficients: BTreeMap<i32, Atom>,
    ) -> (BTreeMap<i32, AliasedAtom>, Statistics) {
        let started = Instant::now();
        let mut aliases = BTreeMap::<Atom, Atom>::new();
        let mut body_handles = BTreeMap::<Atom, Atom>::new();
        let mut requests = BTreeMap::<Atom, Atom>::new();
        let roots = coefficients
            .into_iter()
            .map(|(order, coefficient)| {
                let root = coefficient.replace_map(|term, _, out| {
                    let Some(request) = self.request(term) else {
                        return;
                    };
                    if let Some(value) = requests.get(&term.to_owned()) {
                        **out = value.clone();
                        return;
                    }
                    let body = self.resolve(request);
                    self.assert_resolved(&body);
                    let value = if !self
                        .parameters
                        .iter()
                        .any(|p| body.contains(Atom::var(*p).as_view()))
                    {
                        body
                    } else if let Some(handle) = body_handles.get(&body) {
                        handle.clone()
                    } else {
                        let handle = Atom::var(Self::fresh(&mut self.state.borrow_mut(), "a"));
                        aliases.insert(handle.clone(), body.clone());
                        body_handles.insert(body, handle.clone());
                        handle
                    };
                    requests.insert(term.to_owned(), value.clone());
                    **out = value;
                });
                self.assert_resolved(&root);
                (order, root)
            })
            .collect::<BTreeMap<_, _>>();
        let state = self.state.borrow();
        let statistics = Statistics {
            source_bodies: state.bodies.len(),
            derivative_requests: state.partials.len(),
            face_requests: state.faces.len(),
            aliases: aliases.len(),
            root_bytes: roots.values().map(|a| a.as_view().get_byte_size()).sum(),
            definition_bytes: aliases.values().map(|a| a.as_view().get_byte_size()).sum(),
            resolution_seconds: started.elapsed().as_secs_f64(),
        };
        let coefficients = roots
            .into_iter()
            .map(|(order, root)| {
                let mut coefficient = AliasedAtom::from(root);
                for (handle, body) in &aliases {
                    coefficient.register_alias(handle.clone(), body.clone());
                }
                (order, coefficient)
            })
            .collect();
        (coefficients, statistics)
    }
}

pub(crate) fn expand_named(
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
    options: &GenerationOptions,
) -> Result<NamedExpansion, GenerationError> {
    let names = Coefficients::new(terms, parameters, regulator);
    let (coefficients, attempts) =
        expand_with_names(terms, parameters, regulator, options, Some(&names))?;
    let (coefficients, statistics) = names.finish(coefficients);
    Ok((coefficients, attempts, statistics))
}
