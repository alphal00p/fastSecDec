//! Model dependency closure remains owned by HEPKit's scalar binding API.

use std::collections::{BTreeMap, BTreeSet};

use feynkit_graph::FeynmanDiagram;
use feynkit_model::{ParameterCard, ParameterNature, ParameterType};
use symbolica::{
    atom::{Atom, AtomCore, Symbol, SymbolAttribute, SymbolBuilder},
    symbol, wrap_symbol,
};

use crate::{Error, Result, kernel::RuntimeMassConstraint, parametric::ParametricIntegrand};

#[derive(Clone)]
struct Parameter {
    name: String,
    symbol: Symbol,
    default: Option<f64>,
}

/// Analytic model definitions expressed in independent real evaluator inputs.
/// Only explicit fixed overrides and zero-width restrictions specialize them.
/// After preparing the integral, call `retain_used`, compile with `symbols()`,
/// and attach `mass_constraints()` to the resulting KernelSet before binding.
#[derive(Clone)]
pub struct RuntimeModelBindings {
    values: BTreeMap<Symbol, Atom>,
    parameters: Vec<Parameter>,
    masses: Vec<RuntimeMassConstraint>,
}

impl RuntimeModelBindings {
    pub fn new(
        diagram: &FeynmanDiagram,
        card: Option<&ParameterCard>,
        fixed: &BTreeMap<Symbol, Atom>,
    ) -> Result<Self> {
        let model = diagram.model();
        let defaults = model.scalar_bindings(card, fixed)?;
        let mut overrides = fixed.clone();
        let mut widths = BTreeSet::new();
        let mut mass_names = BTreeSet::new();
        for (edge_id, ends, edge) in diagram.edges() {
            if ends.source.is_none()
                || ends.target.is_none()
                || edge.external.is_some()
                || edge.is_dummy
            {
                continue;
            }
            let width = model.particle_width(edge.particle)?;
            let key = symbol!(&format!("UFO::{}", width.name));
            if !defaults.get(&key).is_some_and(Atom::is_zero) {
                return Err(Error::UnsupportedWidth {
                    edge: edge_id,
                    parameter: width.name.clone(),
                });
            }
            widths.insert(key);
            overrides.insert(key, Atom::Zero);
            mass_names.insert(model.particle_mass(edge.particle)?.name.clone());
        }
        let mut parameters = Vec::new();
        let mut names = BTreeSet::new();
        // Match HEPKit's independent known-parameter boundary, including an
        // expressionless internal constant rather than freezing it by accident.
        let mut external = model
            .parameters()
            .iter()
            .filter(|p| p.nature == ParameterNature::External || p.expression.is_none())
            .collect::<Vec<_>>();
        external.sort_by(|a, b| a.name.cmp(&b.name));
        for parameter in external {
            let key = symbol!(&format!("UFO::{}", parameter.name));
            if parameter.name == "ZERO" || widths.contains(&key) || fixed.contains_key(&key) {
                continue;
            }
            let value = defaults.get(&key).and_then(|value| {
                value
                    .evaluate(&std::collections::HashMap::<
                        Atom,
                        numerica::domains::float::Complex<f64>,
                    >::new())
                    .ok()
            });
            let mut input = |suffix: &str, default: Option<f64>| -> Result<Atom> {
                let name = format!("model::{}{suffix}", parameter.name);
                let symbol = SymbolBuilder::new(wrap_symbol!(&name))
                    .with_attributes(&[SymbolAttribute::Real])
                    .build()
                    .map_err(|error| Error::ScalarBindings(error.to_string()))?;
                if !names.insert(name.clone()) || fixed.contains_key(&symbol) {
                    return Err(Error::ScalarBindings(format!(
                        "duplicate or fixed runtime model input {name}"
                    )));
                }
                parameters.push(Parameter {
                    name,
                    symbol,
                    default: default.filter(|v| v.is_finite()),
                });
                Ok(Atom::var(symbol))
            };
            let expression = if parameter.parameter_type == ParameterType::Real {
                input("", value.as_ref().filter(|v| v.im == 0.0).map(|v| v.re))?
            } else {
                input("_re", value.as_ref().map(|v| v.re))?
                    + Atom::i() * input("_im", value.as_ref().map(|v| v.im))?
            };
            overrides.insert(key, expression);
        }
        // Passing no card here retains internal analytic definitions; only the
        // explicitly recorded width/fixed overrides are numerical restrictions.
        let values = model.scalar_bindings(None, &overrides)?;
        let symbols = parameters.iter().map(|p| p.symbol).collect::<BTreeSet<_>>();
        let masses = mass_names
            .into_iter()
            .filter_map(|name| {
                let expression = values.get(&symbol!(&format!("UFO::{name}")))?;
                expression
                    .get_all_symbols(false)
                    .iter()
                    .any(|s| symbols.contains(s))
                    .then(|| RuntimeMassConstraint {
                        name,
                        expression: expression.clone(),
                    })
            })
            .collect();
        Ok(Self {
            values,
            parameters,
            masses,
        })
    }

    pub fn values(&self) -> &BTreeMap<Symbol, Atom> {
        &self.values
    }

    /// Stable model-name order, with real then imaginary components for a
    /// complex independent input. No hash iteration determines evaluator layout.
    pub fn symbols(&self) -> Vec<Symbol> {
        self.parameters.iter().map(|p| p.symbol).collect()
    }

    pub fn mass_constraints(&self) -> &[RuntimeMassConstraint] {
        &self.masses
    }

    /// Metadata only: defaults are not bound to the compiled evaluator.
    pub fn defaults(&self) -> BTreeMap<String, f64> {
        self.parameters
            .iter()
            .filter_map(|p| p.default.map(|value| (p.name.clone(), value)))
            .collect()
    }

    pub fn retain_used(&mut self, integrand: &ParametricIntegrand) {
        let mut used = BTreeSet::new();
        for term in integrand.terms() {
            for expression in std::iter::once(term.prefactor())
                .chain(term.monomial_powers())
                .chain(
                    term.factors()
                        .iter()
                        .flat_map(|f| [f.polynomial(), f.exponent()]),
                )
            {
                used.extend(expression.get_all_symbols(false));
            }
        }
        for mass in &self.masses {
            used.extend(mass.expression.get_all_symbols(false));
        }
        self.parameters
            .retain(|parameter| used.contains(&parameter.symbol));
    }
}
