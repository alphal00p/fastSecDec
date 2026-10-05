//! Attempt-local native coefficient requests and flat alias lowering.
mod interleaved;
#[cfg(test)]
mod tests;

use super::{
    LoweredRequests, NativeSeries, RequestCounts, RequestLimits, RequestProgress, RequestStage,
};
use crate::generation::{GenerationError, mapping::MappedTerm};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ops::ControlFlow,
};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol, SymbolAttribute, SymbolBuilder},
    coefficient::Coefficient,
    wrap_symbol,
};

#[derive(Clone)]
struct Body {
    value: Atom,
    parameters: Vec<Symbol>,
}

type Request = (Symbol, Vec<usize>, Vec<Atom>);

#[derive(Clone, Copy)]
enum NameKind {
    Body,
    Alias,
}

pub(super) struct Requests {
    parameters: Vec<Symbol>,
    regulator: Symbol,
    limits: RequestLimits,
    occupied: BTreeSet<Symbol>,
    next_body: usize,
    next_alias: usize,
    ids: BTreeMap<Atom, Symbol>,
    bodies: BTreeMap<Symbol, Body>,
    partials: BTreeMap<(Symbol, Vec<usize>), Atom>,
    faces: BTreeMap<Request, Atom>,
    counts: RequestCounts,
    #[cfg(test)]
    original_only: bool,
}

fn next_count(
    current: usize,
    maximum: Option<usize>,
    label: &'static str,
) -> Result<usize, GenerationError> {
    current
        .checked_add(1)
        .filter(|next| maximum.is_none_or(|maximum| *next <= maximum))
        .ok_or(GenerationError::ResourceLimit(label))
}

fn poll(
    counts: RequestCounts,
    stage: RequestStage,
    observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
) -> Result<(), GenerationError> {
    if observer(RequestProgress { stage, counts }).is_break() {
        Err(GenerationError::Cancelled)
    } else {
        Ok(())
    }
}

impl Requests {
    pub(super) fn new(
        terms: &[MappedTerm],
        parameters: &[Symbol],
        regulator: Symbol,
        limits: RequestLimits,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<Self, GenerationError> {
        let counts = RequestCounts::default();
        poll(counts, RequestStage::ReserveSymbols, observer)?;
        let mut occupied = parameters.iter().copied().collect::<BTreeSet<_>>();
        occupied.insert(regulator);
        for term in terms {
            for expression in [&term.prefactor, &term.regular]
                .into_iter()
                .chain(&term.powers)
            {
                poll(counts, RequestStage::ReserveSymbols, observer)?;
                occupied.extend(expression.get_all_symbols(true));
                poll(counts, RequestStage::ReserveSymbols, observer)?;
            }
        }
        Ok(Self {
            parameters: parameters.to_vec(),
            regulator,
            limits,
            occupied,
            next_body: 0,
            next_alias: 0,
            ids: BTreeMap::new(),
            bodies: BTreeMap::new(),
            partials: BTreeMap::new(),
            faces: BTreeMap::new(),
            counts,
            #[cfg(test)]
            original_only: false,
        })
    }

    fn fresh(
        &mut self,
        kind: NameKind,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<Symbol, GenerationError> {
        loop {
            let (prefix, stage) = match kind {
                NameKind::Body => ("f", RequestStage::NameCoefficients),
                NameKind::Alias => ("a", RequestStage::LowerRequests),
            };
            poll(self.counts, stage, observer)?;
            let next = match kind {
                NameKind::Body => &mut self.next_body,
                NameKind::Alias => &mut self.next_alias,
            };
            let index = *next;
            *next = next_count(index, None, "named symbol counter")?;
            let name = format!("fastsecdec::named_regular::{prefix}{index}");
            if Symbol::get_symbol(wrap_symbol!(&name)).is_some_and(|s| self.occupied.contains(&s)) {
                continue;
            }
            let candidate = SymbolBuilder::new(wrap_symbol!(&name))
                .with_attributes(&[] as &[SymbolAttribute])
                .build()
                .map_err(|error| GenerationError::ReservedSymbolConflict {
                    name: name.clone(),
                    reason: error.to_string(),
                })?;
            // Native empty metadata requests retain existing hooks. The public
            // predicate is the native owner of all five hook-absence checks.
            if !candidate.is_exportable() {
                return Err(GenerationError::ReservedSymbolConflict {
                    name,
                    reason: "an existing native hook is attached to the symbol".into(),
                });
            }
            if self.occupied.insert(candidate) {
                return Ok(candidate);
            }
        }
    }

    fn wrap(
        &mut self,
        coefficient: &Atom,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<Atom, GenerationError> {
        poll(self.counts, RequestStage::NameCoefficients, observer)?;
        if coefficient.contains(Atom::var(self.regulator).as_view()) {
            return Err(GenerationError::Invariant(
                "named regular coefficient contains regulator".into(),
            ));
        }
        let parameters = self
            .parameters
            .iter()
            .copied()
            .filter(|p| coefficient.contains(Atom::var(*p).as_view()))
            .collect::<Vec<_>>();
        if parameters.is_empty() {
            return Ok(coefficient.clone());
        }
        let name = if let Some(name) = self.ids.get(coefficient) {
            *name
        } else {
            let count = next_count(self.counts.source_bodies, None, "named source bodies")?;
            let name = self.fresh(NameKind::Body, observer)?;
            self.ids.insert(coefficient.clone(), name);
            self.bodies.insert(
                name,
                Body {
                    value: coefficient.clone(),
                    parameters: parameters.clone(),
                },
            );
            self.counts.source_bodies = count;
            name
        };
        Ok(name.call(&parameters.into_iter().map(Atom::var).collect::<Vec<_>>()))
    }

    /// An error invalidates this entire attempt; callers must drop the registry.
    pub(super) fn wrap_series(
        &mut self,
        series: &NativeSeries,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<NativeSeries, GenerationError> {
        let state = RefCell::new((self, observer, None));
        let named = series.map_coeff(|coefficient| {
            let mut state = state.borrow_mut();
            let (requests, observer, first_error) = &mut *state;
            if first_error.is_some() {
                return coefficient.clone();
            }
            match requests.wrap(coefficient, *observer) {
                Ok(value) => value,
                Err(error) => {
                    *first_error = Some(error);
                    coefficient.clone()
                }
            }
        });
        let (requests, observer, first_error) = state.into_inner();
        if let Some(error) = first_error {
            return Err(error);
        }
        if named.absolute_order() != series.absolute_order() {
            return Err(GenerationError::Invariant(
                "native coefficient naming changed series coverage".into(),
            ));
        }
        poll(requests.counts, RequestStage::NameCoefficients, observer)?;
        Ok(named)
    }

    pub(super) fn check_scalar(&self, scalar: &Atom) -> Result<(), GenerationError> {
        let symbols = scalar.get_all_symbols(true);
        if self.bodies.keys().any(|name| symbols.contains(name)) {
            Err(GenerationError::Invariant(
                "formal coefficient used as a scalar or unresolved at executable boundary".into(),
            ))
        } else {
            Ok(())
        }
    }

    fn depth(value: AtomView) -> Result<usize, GenerationError> {
        if let AtomView::Num(number) = value
            && let Coefficient::Complex(value) = number.get_coeff_view().to_owned()
            && value.im.is_zero()
            && value.re.is_integer()
            && value.re >= 0
        {
            return value
                .re
                .numerator()
                .to_i64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or(GenerationError::ResourceLimit("named derivative depth"));
        }
        Err(GenerationError::Invariant(
            "malformed owned native derivative depth".into(),
        ))
    }

    fn request(&self, term: AtomView) -> Result<Option<Request>, GenerationError> {
        let AtomView::Fun(function) = term else {
            return Ok(None);
        };
        if let Some(body) = self.bodies.get(&function.get_symbol()) {
            if function.get_nargs() != body.parameters.len() {
                return Err(GenerationError::Invariant(
                    "owned coefficient function arity".into(),
                ));
            }
            return Ok(Some((
                function.get_symbol(),
                vec![0; body.parameters.len()],
                function.iter().map(|a| a.to_owned()).collect(),
            )));
        }
        if function.get_symbol() != Symbol::DERIVATIVE
            || function.get_nargs() < 3
            || function.get_nargs() % 2 == 0
        {
            return Ok(None);
        }
        let n = (function.get_nargs() - 1) / 2;
        let args = function.iter().collect::<Vec<_>>();
        let AtomView::Var(source) = args[n] else {
            return Ok(None);
        };
        let Some(body) = self.bodies.get(&source.get_symbol()) else {
            return Ok(None);
        };
        if n != body.parameters.len() {
            return Err(GenerationError::Invariant(
                "owned native derivative arity".into(),
            ));
        }
        Ok(Some((
            source.get_symbol(),
            args[..n]
                .iter()
                .copied()
                .map(Self::depth)
                .collect::<Result<_, _>>()?,
            args[n + 1..].iter().map(|a| a.to_owned()).collect(),
        )))
    }

    fn resolve(
        &mut self,
        request: Request,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<Atom, GenerationError> {
        poll(self.counts, RequestStage::LowerRequests, observer)?;
        if let Some(value) = self.faces.get(&request) {
            return Ok(value.clone());
        }
        let next = next_count(
            self.counts.unique_requests,
            self.limits.max_unique_requests,
            "named unique requests",
        )?;
        let (source, depths, arguments) = &request;
        let body =
            self.bodies.get(source).cloned().ok_or_else(|| {
                GenerationError::Invariant("unknown owned coefficient source".into())
            })?;
        if depths.len() != body.parameters.len() || arguments.len() != body.parameters.len() {
            return Err(GenerationError::Invariant("owned request shape".into()));
        }
        let interleave = {
            #[cfg(test)]
            {
                !self.original_only
            }
            #[cfg(not(test))]
            {
                true
            }
        };
        let mut value = None;
        if interleave {
            let counts = self.counts;
            value = interleaved::resolve(&body, depths, arguments, &mut || {
                poll(counts, RequestStage::LowerRequests, observer)
            })?;
        }
        let value = if let Some(value) = value {
            self.counts.interleaved_requests = next_count(
                self.counts.interleaved_requests,
                None,
                "named interleaved requests",
            )?;
            value
        } else {
            let key = (*source, depths.clone());
            let partial = if let Some(value) = self.partials.get(&key) {
                value.clone()
            } else {
                let mut value = body.value.clone();
                for (parameter, count) in body.parameters.iter().zip(depths) {
                    for _ in 0..*count {
                        poll(self.counts, RequestStage::LowerRequests, observer)?;
                        value = value.derivative(*parameter);
                        poll(self.counts, RequestStage::LowerRequests, observer)?;
                    }
                }
                let count = next_count(self.counts.cached_partials, None, "named partial cache")?;
                self.partials.insert(key, value.clone());
                self.counts.cached_partials = count;
                value
            };
            let replacements = body
                .parameters
                .iter()
                .copied()
                .zip(arguments)
                .collect::<BTreeMap<_, _>>();
            poll(self.counts, RequestStage::LowerRequests, observer)?;
            let value = partial.replace_map(|term, _, out| {
                if let AtomView::Var(variable) = term
                    && let Some(value) = replacements.get(&variable.get_symbol())
                {
                    **out = (*value).clone();
                }
            });
            poll(self.counts, RequestStage::LowerRequests, observer)?;
            self.counts.fallback_requests = next_count(
                self.counts.fallback_requests,
                None,
                "named fallback requests",
            )?;
            value
        };
        self.faces.insert(request, value.clone());
        self.counts.unique_requests = next;
        Ok(value)
    }

    pub(super) fn lower(
        mut self,
        coefficients: BTreeMap<i32, Atom>,
        observer: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<LoweredRequests, GenerationError> {
        let mut aliases = BTreeMap::<Atom, Atom>::new();
        let mut body_handles = BTreeMap::<Atom, Atom>::new();
        let mut lowered = BTreeMap::<Atom, Atom>::new();
        let mut roots = BTreeMap::new();
        for (order, coefficient) in coefficients {
            poll(self.counts, RequestStage::LowerRequests, observer)?;
            let mut first_error = None;
            let root = coefficient.replace_map(|term, _, out| {
                if first_error.is_some() {
                    return;
                }
                let result = (|| {
                    let Some(request) = self.request(term)? else {
                        return Ok(None);
                    };
                    if let Some(value) = lowered.get(&term.to_owned()) {
                        poll(self.counts, RequestStage::LowerRequests, observer)?;
                        return Ok(Some(value.clone()));
                    }
                    let body = self.resolve(request, observer)?;
                    self.check_scalar(&body)?;
                    let value = if !self
                        .parameters
                        .iter()
                        .any(|p| body.contains(Atom::var(*p).as_view()))
                    {
                        body
                    } else if let Some(handle) = body_handles.get(&body) {
                        handle.clone()
                    } else {
                        let count = next_count(self.counts.aliases, None, "named aliases")?;
                        let handle = Atom::var(self.fresh(NameKind::Alias, observer)?);
                        aliases.insert(handle.clone(), body.clone());
                        body_handles.insert(body, handle.clone());
                        self.counts.aliases = count;
                        handle
                    };
                    lowered.insert(term.to_owned(), value.clone());
                    Ok(Some(value))
                })();
                match result {
                    Ok(Some(value)) => **out = value,
                    Ok(None) => (),
                    Err(error) => first_error = Some(error),
                }
            });
            if let Some(error) = first_error {
                return Err(error);
            }
            self.check_scalar(&root)?;
            roots.insert(order, root);
        }
        poll(self.counts, RequestStage::LowerRequests, observer)?;
        let mut coefficients = BTreeMap::new();
        for (order, root) in roots {
            let mut coefficient = AliasedAtom::from(root);
            for (handle, body) in &aliases {
                poll(self.counts, RequestStage::LowerRequests, observer)?;
                coefficient.register_alias(handle.clone(), body.clone());
            }
            coefficients.insert(order, coefficient);
        }
        poll(self.counts, RequestStage::LowerRequests, observer)?;
        Ok(LoweredRequests {
            coefficients,
            counts: self.counts,
        })
    }
}
