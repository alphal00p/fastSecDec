use super::GenerationError;
use std::collections::BTreeMap;
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    symbol,
};

#[derive(Default)]
pub(super) struct TemplateCache {
    coefficients: BTreeMap<(Atom, i32), BTreeMap<i32, Atom>>,
}

pub(super) fn expand(
    expression: &Atom,
    parameters: &[Symbol],
    regulator: Symbol,
    max_order: i32,
    cache: &mut TemplateCache,
) -> Result<BTreeMap<i32, Atom>, GenerationError> {
    // Keep dense residual polynomials opaque while expanding the small epsilon
    // template. This is the direct reference path's late-instantiation strategy:
    // native Symbolica still owns series arithmetic and absolute truncation,
    // including extra orders required by endpoint and Gamma prefactor poles.
    let epsilon = Atom::var(regulator);
    let coordinates = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let source_symbols = expression.get_all_symbols(true);
    let mut atoms = BTreeMap::<Atom, Symbol>::new();
    let mut images = BTreeMap::<Symbol, Atom>::new();
    let mut next = 0usize;
    let template = expression.replace_map(|term, _, out| {
        if !matches!(term, AtomView::Num(_))
            && !term.contains(epsilon.as_view())
            && coordinates.iter().any(|x| term.contains(x.as_view()))
        {
            let owned = term.to_owned();
            let placeholder = *atoms.entry(owned.clone()).or_insert_with(|| {
                let placeholder = loop {
                    let candidate = symbol!(format!("fastsecdec::laurent_template::c{next}"));
                    next += 1;
                    if !source_symbols.contains(&candidate) {
                        break candidate;
                    }
                };
                images.insert(placeholder, owned);
                placeholder
            });
            **out = Atom::var(placeholder);
        }
    });
    let key = (template.clone(), max_order);
    if !cache.coefficients.contains_key(&key) {
        cache.coefficients.insert(
            key.clone(),
            expand_template(&template, regulator, max_order)?,
        );
    }
    let mut coefficients = BTreeMap::new();
    for (order, coefficient) in &cache.coefficients[&key] {
        let restored = coefficient.replace_map(|term, _, out| {
            if let AtomView::Var(variable) = term
                && let Some(value) = images.get(&variable.get_symbol())
            {
                **out = value.clone();
            }
        });
        let restored = if restored.as_view().get_byte_size() <= 256 {
            restored.together()
        } else {
            restored
        };
        if !restored.is_zero() {
            coefficients.insert(*order, restored);
        }
    }
    Ok(coefficients)
}

fn expand_template(
    expression: &Atom,
    regulator: Symbol,
    max_order: i32,
) -> Result<BTreeMap<i32, Atom>, GenerationError> {
    let series = expression
        .series(regulator, 0, i64::from(max_order))
        .map_err(|error| GenerationError::Series(error.to_string()))?;
    let mut coefficients = BTreeMap::new();
    for (order, coefficient) in series.terms() {
        if coefficient.is_zero() {
            continue;
        }
        if !order.is_integer() {
            return Err(GenerationError::FractionalLaurent(order.to_string()));
        }
        let order = order
            .numerator()
            .to_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| GenerationError::FractionalLaurent(order.to_string()))?;
        if order > max_order {
            continue;
        }
        if coefficient.contains(Atom::var(regulator).as_view()) {
            return Err(GenerationError::Invariant(
                "Laurent coefficient still contains regulator".into(),
            ));
        }
        coefficients.insert(order, coefficient.clone());
    }
    Ok(coefficients)
}
