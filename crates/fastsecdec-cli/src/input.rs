use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use fastsecdec::{
    Atom, AtomCore, EdgeId, Kinematics, Model, ParameterCard, Symbol,
    input::GraphIntegral,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use symbolica::{atom::AtomView, id::Replacement, parser::ParseSettings};

use crate::{CliResult, config::RunCard};

pub struct LoadedInput {
    pub card: RunCard,
    pub integrand: ParametricIntegrand,
    pub label: String,
    pub loops: Option<usize>,
    pub propagators: usize,
    pub sources: Vec<crate::artifact::SourceFile>,
    pub independent_externals: Vec<String>,
    pub dependent_externals: Vec<String>,
}

pub fn expression(text: &str) -> CliResult<Atom> {
    Atom::parse(text, "feynkit_graph", ParseSettings::default())
        .map_err(|error| format!("invalid Symbolica expression {text:?}: {error}").into())
}

pub fn symbol(text: &str) -> CliResult<Symbol> {
    match expression(text)?.as_view() {
        AtomView::Var(variable) => Ok(variable.get_symbol()),
        _ => Err(format!("expected a plain symbol, got {text:?}").into()),
    }
}

fn bind(expression: &Atom, values: &BTreeMap<Symbol, Atom>) -> Atom {
    expression.replace_multiple(values.iter().map(|(symbol, value)| {
        Replacement::new(Atom::var(*symbol).to_pattern(), value.to_pattern())
    }))
}

fn resolve(mut values: BTreeMap<Symbol, Atom>) -> CliResult<BTreeMap<Symbol, Atom>> {
    for _ in 0..=values.len() {
        let next = values
            .iter()
            .map(|(symbol, value)| (*symbol, bind(value, &values)))
            .collect::<BTreeMap<_, _>>();
        if next == values {
            break;
        }
        values = next;
    }
    if values.values().any(|value| {
        values
            .keys()
            .any(|symbol| value.contains(Atom::var(*symbol).as_view()))
    }) {
        return Err("parameter definitions have a cyclic or unresolved dependency".into());
    }
    Ok(values)
}

fn value_expression(value: &toml::Value) -> CliResult<Atom> {
    match value {
        toml::Value::String(text) => expression(text),
        toml::Value::Integer(value) => Ok(Atom::num(*value)),
        toml::Value::Float(value) if value.is_finite() => expression(&value.to_string()),
        _ => Err("parameter values must be Symbolica strings or finite numbers".into()),
    }
}

fn model_values(model: &Model) -> CliResult<BTreeMap<Symbol, Atom>> {
    let mut result = BTreeMap::new();
    for parameter in model.parameters() {
        let value = if let Some(value) = parameter.value {
            expression(&format!("({})+({})*i", value.re, value.im))?
        } else if let Some(expression) = &parameter.expression {
            expression.clone()
        } else {
            continue;
        };
        result.insert(symbol(&format!("UFO::{}", parameter.name))?, value);
    }
    for coupling in model.couplings() {
        let value = if let Some(value) = coupling.value {
            expression(&format!("({})+({})*i", value.re, value.im))?
        } else {
            coupling.expression.clone()
        };
        result.insert(symbol(&format!("UFO::{}", coupling.name))?, value);
    }
    Ok(result)
}

pub fn load(path: &Path) -> CliResult<LoadedInput> {
    let mut sources = Vec::new();
    let read = |path: &Path, sources: &mut Vec<crate::artifact::SourceFile>| -> CliResult<String> {
        let text = fs::read_to_string(path)?;
        sources.push(crate::artifact::SourceFile {
            path: fs::canonicalize(path)?.to_string_lossy().into_owned(),
            blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        });
        Ok(text)
    };
    let card: RunCard = toml::from_str(&read(path, &mut sources)?)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut values = BTreeMap::new();
    for (name, value) in &card.parameters {
        values.insert(symbol(name)?, value_expression(value)?);
    }
    let regulator = symbol(&card.integral.regulator)?;
    match (&card.input, &card.direct) {
        (Some(input), None) => {
            let mut model = Model::from_json(&read(&base.join(&input.model), &mut sources)?)?;
            if let Some(parameter_card) = &input.parameter_card {
                model.apply_parameter_card(&ParameterCard::from_json(&read(
                    &base.join(parameter_card),
                    &mut sources,
                )?)?)?;
            }
            let mut all = model_values(&model)?;
            all.extend(values);
            let values = resolve(all)?;
            let mut kinematics = Kinematics::in_dimension(&expression("D")?)?;
            for product in &card.kinematics.products {
                kinematics = kinematics.with_scalar_product(
                    &feynkit_graph::symbols::external_momentum().call(product.left),
                    &feynkit_graph::symbols::external_momentum().call(product.right),
                    bind(&expression(&product.value)?, &values),
                )?;
            }
            let graph_text = read(&base.join(&input.graph), &mut sources)?;
            let powers = card
                .integral
                .powers
                .iter()
                .map(|(edge, power)| edge.parse::<usize>().map(|edge| (EdgeId(edge), *power)))
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            let graph = GraphIntegral::from_dot(Arc::new(model), &graph_text, &kinematics)?
                .with_powers(&powers)?
                .with_measure_multiplier(expression(&card.integral.measure_multiplier)?)
                .with_scalar_values(&values)?;
            let loops = Some(graph.diagram().loop_count());
            let propagators = graph.powers().len();
            let independent_externals = graph
                .family()
                .external_momenta()
                .iter()
                .map(Atom::to_canonical_string)
                .collect();
            let basis = graph.diagram().loop_momentum_basis();
            let dependent_externals = basis
                .external_edges
                .iter()
                .enumerate()
                .filter(|(_, edge)| basis.dependent_externals.contains(edge))
                .map(|(index, _)| {
                    feynkit_graph::symbols::external_momentum()
                        .call(index)
                        .to_canonical_string()
                })
                .collect();
            let parameters = (0..propagators)
                .map(|i| symbol(&format!("fastsecdec::x{i}")))
                .collect::<CliResult<Vec<_>>>()?;
            let integrand = ParametricIntegrand::from_graph(
                &graph,
                parameters,
                regulator,
                bind(&expression(&card.integral.dimension)?, &values),
            )?;
            Ok(LoadedInput {
                card,
                integrand,
                label: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                loops,
                propagators,
                sources,
                independent_externals,
                dependent_externals,
            })
        }
        (None, Some(direct)) => {
            let values = resolve(values)?;
            let domain =
                match direct.domain.as_str() {
                    "unit_cube" => ParametricDomain::UnitCube,
                    "projective_simplex" => ParametricDomain::ProjectiveSimplex,
                    "positive_orthant" => ParametricDomain::PositiveOrthant,
                    _ => return Err(
                        "direct domain must be unit_cube, projective_simplex or positive_orthant"
                            .into(),
                    ),
                };
            let parameters = direct
                .parameters
                .iter()
                .map(|name| symbol(name))
                .collect::<CliResult<Vec<_>>>()?;
            if parameters
                .iter()
                .chain(std::iter::once(&regulator))
                .any(|symbol| values.contains_key(symbol))
            {
                return Err(
                    "[parameters] cannot bind an integration parameter or its regulator".into(),
                );
            }
            let mut terms = Vec::new();
            for term in &direct.terms {
                let monomial_powers = if term.monomial_powers.is_empty() {
                    vec![Atom::Zero; parameters.len()]
                } else {
                    term.monomial_powers
                        .iter()
                        .map(|text| Ok(bind(&expression(text)?, &values)))
                        .collect::<CliResult<Vec<_>>>()?
                };
                let mut factors = Vec::new();
                for factor in &term.factors {
                    let polynomial = match (&factor.polynomial, &factor.polynomial_file) {
                        (Some(text), None) => expression(text)?,
                        (None, Some(path)) => expression(&read(&base.join(path), &mut sources)?)?,
                        _ => {
                            return Err(
                                "each factor requires exactly one of polynomial or polynomial_file"
                                    .into(),
                            );
                        }
                    };
                    let role = match factor.role.as_str() {
                        "singularity" => FactorRole::Singularity,
                        "polynomial" => FactorRole::Polynomial,
                        _ => return Err("factor role must be singularity or polynomial".into()),
                    };
                    factors.push(PolynomialFactor::new(
                        bind(&polynomial, &values),
                        bind(&expression(&factor.exponent)?, &values),
                        role,
                    ));
                }
                terms.push(ParametricTerm::new(
                    bind(
                        &(expression(&term.prefactor)?
                            * expression(&card.integral.measure_multiplier)?),
                        &values,
                    ),
                    monomial_powers,
                    factors,
                ));
            }
            let propagators = parameters.len();
            let integrand = ParametricIntegrand::new(parameters, regulator, domain, terms)?;
            Ok(LoadedInput {
                card,
                integrand,
                label: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                loops: None,
                propagators,
                sources,
                independent_externals: Vec::new(),
                dependent_externals: Vec::new(),
            })
        }
        _ => Err(
            "run card requires exactly one of [input] graph data or [direct] parametric data"
                .into(),
        ),
    }
}

pub fn artifact_path(card: &Path) -> PathBuf {
    PathBuf::from("output").join(format!(
        "{}.fsd.json",
        card.file_stem().unwrap_or_default().to_string_lossy()
    ))
}
