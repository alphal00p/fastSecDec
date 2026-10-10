//! One local Laurent vector using the existing native Atom/State record codec.
//! A record is untrusted transport, not independent continuation authority.
use crate::{
    generation::{
        self,
        streaming::{
            RecordRef, StreamingError,
            codec::{self, Atoms, invalid},
        },
    },
    threshold::regularization::BoundContinuation,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, Symbol},
    evaluate::{FunctionMap, FunctionRegistrationOptions, InliningPolicy},
};

const KIND: &str = "threshold-local-laurent-v1";
type Result<T> = std::result::Result<T, StreamingError>;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Aliased {
    root: usize,
    aliases: Vec<(usize, usize)>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    head: usize,
    tags: Vec<usize>,
    formals: Vec<usize>,
    body: usize,
    derivative_order: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum VectorKind {
    Stochastic {},
    Exact {},
    ZeroInLayout {},
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    parent: String,
    chart: usize,
    maximum: i32,
    coordinates: Vec<usize>,
    kind: VectorKind,
    coefficients: Vec<(i32, Aliased)>,
    definitions: Vec<Definition>,
    profiles: Vec<generation::EndpointProfileRow>,
}
pub(crate) struct StagedVector {
    pub record: RecordRef,
    pub chart: usize,
    pub minimum: i32,
    pub maximum: i32,
    pub kind: VectorKind,
}
pub(crate) struct LocalVector {
    pub coordinates: Vec<Symbol>,
    pub coefficients: BTreeMap<i32, AliasedAtom>,
    pub functions: FunctionMap,
    pub profiles: Vec<generation::EndpointProfileRow>,
    pub kind: VectorKind,
}
fn symbol_index(symbols: &mut Vec<Symbol>, symbol: Symbol) -> usize {
    if let Some(index) = symbols.iter().position(|s| *s == symbol) {
        index
    } else {
        let index = symbols.len();
        symbols.push(symbol);
        index
    }
}
fn alias(value: &AliasedAtom, atoms: &mut Atoms) -> Aliased {
    let mut aliases = value.get_aliases().iter().collect::<Vec<_>>();
    aliases.sort_by_cached_key(|(a, _)| a.to_canonical_string());
    Aliased {
        root: atoms.push(value.get_root()),
        aliases: aliases
            .into_iter()
            .map(|(a, b)| (atoms.push(a), atoms.push(b)))
            .collect(),
    }
}

pub(crate) fn write(
    root: &Path,
    parent: &str,
    bound: &BoundContinuation<'_>,
    chart: usize,
    maximum: i32,
) -> Result<StagedVector> {
    let expression = bound
        .chart_expressions()
        .get(chart)
        .ok_or_else(|| invalid("continued chart index"))?;
    let mut coefficients = generation::threshold_expand_vector(
        expression,
        &bound.coordinates(),
        bound.regulators()[0],
        maximum,
    )?;
    let kind = if coefficients.is_empty() {
        VectorKind::ZeroInLayout {}
    } else if coefficients
        .values()
        .all(|c| generation::coordinate_independent(c, &bound.coordinates()))
    {
        for coefficient in coefficients.values_mut() {
            let materialized = bound
                .materialize_exact(&coefficient.clone().into_inner())
                .map_err(invalid)?;
            *coefficient = AliasedAtom::from(materialized);
        }
        VectorKind::Exact {}
    } else {
        VectorKind::Stochastic {}
    };
    let minimum = coefficients
        .keys()
        .next()
        .copied()
        .unwrap_or(maximum)
        .min(maximum.min(0));
    let mut atoms = Atoms::default();
    let mut symbols = Vec::new();
    let coordinates = bound
        .coordinates()
        .iter()
        .map(|s| symbol_index(&mut symbols, *s))
        .collect();
    let coefficients = coefficients
        .iter()
        .map(|(order, value)| (*order, alias(value, &mut atoms)))
        .collect();
    let mut definitions = Vec::new();
    if kind == (VectorKind::Stochastic {}) {
        for definition in bound.definitions() {
            definitions.push(Definition {
                head: symbol_index(&mut symbols, definition.head()),
                tags: definition.tags().iter().map(|a| atoms.push(a)).collect(),
                formals: definition
                    .formals()
                    .iter()
                    .map(|s| symbol_index(&mut symbols, *s))
                    .collect(),
                body: atoms.push(definition.body()),
                derivative_order: definition.derivative_order(),
            });
        }
    }
    let record = codec::write(
        root,
        &format!("threshold-chart-{chart}"),
        KIND,
        &Stored {
            parent: parent.into(),
            chart,
            maximum,
            coordinates,
            kind,
            coefficients,
            definitions,
            profiles: bound
                .profiles()
                .get(chart)
                .ok_or_else(|| invalid("continued chart profiles"))?
                .clone(),
        },
        atoms,
        symbols,
    )?;
    Ok(StagedVector {
        record,
        chart,
        minimum,
        maximum,
        kind,
    })
}

pub(crate) fn read(
    root: &Path,
    record: &StagedVector,
    parent: &str,
    maximum_bytes: u64,
) -> Result<LocalVector> {
    if record.record.bytes > maximum_bytes {
        return Err(invalid("local vector transport limit"));
    }
    let (stored, atoms, symbols): (Stored, _, _) = codec::read(root, &record.record, KIND)?;
    if stored.parent != parent
        || stored.chart != record.chart
        || stored.maximum != record.maximum
        || stored.kind != record.kind
    {
        return Err(invalid("local vector parent/chart/layout association"));
    }
    let symbol = |index: usize| {
        symbols
            .get(index)
            .copied()
            .ok_or_else(|| invalid("local vector symbol index"))
    };
    let coordinates = stored
        .coordinates
        .into_iter()
        .map(symbol)
        .collect::<Result<Vec<_>>>()?;
    if coordinates.is_empty()
        || coordinates.iter().copied().collect::<BTreeSet<_>>().len() != coordinates.len()
    {
        return Err(invalid("local vector coordinate layout"));
    }
    let mut coefficients = BTreeMap::new();
    for (order, value) in stored.coefficients {
        if order > stored.maximum {
            return Err(invalid("local coefficient exceeds requested maximum"));
        }
        let mut coefficient = AliasedAtom::from(atoms.take(value.root)?);
        let mut seen = BTreeSet::new();
        for (handle, body) in value.aliases {
            let handle = atoms.take(handle)?;
            if !seen.insert(handle.clone()) {
                return Err(invalid("duplicate local alias"));
            }
            coefficient.register_alias(handle, atoms.take(body)?);
        }
        if coefficients.insert(order, coefficient).is_some() {
            return Err(invalid("duplicate local Laurent order"));
        }
    }
    if coefficients
        .keys()
        .next()
        .copied()
        .unwrap_or(stored.maximum)
        .min(stored.maximum.min(0))
        != record.minimum
    {
        return Err(invalid("local Laurent minimum"));
    }
    if (stored.kind == (VectorKind::ZeroInLayout {})) != coefficients.is_empty() {
        return Err(invalid("zero-layout record mismatch"));
    }
    if stored.kind != (VectorKind::Stochastic {}) && !stored.definitions.is_empty() {
        return Err(invalid("nonstochastic helper definitions"));
    }
    let mut functions = FunctionMap::new();
    for definition in stored.definitions {
        let head = symbol(definition.head)?;
        let tags = definition
            .tags
            .into_iter()
            .map(|i| atoms.take(i))
            .collect::<Result<Vec<_>>>()?;
        let formals = definition
            .formals
            .into_iter()
            .map(symbol)
            .collect::<Result<Vec<_>>>()?;
        let body = atoms.take(definition.body)?;
        if let Some(order) = definition.derivative_order {
            if head != Symbol::DERIVATIVE
                || order == 0
                || tags.len() != 4
                || !tags[0].is_zero()
                || tags[1] != Atom::num(order)
            {
                return Err(invalid("local derivative definition signature"));
            }
            functions
                .add_tagged_function_with_options(
                    head,
                    tags,
                    formals,
                    body,
                    FunctionRegistrationOptions::new().inlining(InliningPolicy::Always),
                )
                .map_err(invalid)?;
        } else {
            functions
                .add_tagged_function(head, tags, formals, body)
                .map_err(invalid)?;
        }
    }
    Ok(LocalVector {
        coordinates,
        coefficients,
        functions,
        profiles: stored.profiles,
        kind: stored.kind,
    })
}
