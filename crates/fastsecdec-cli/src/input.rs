use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use fastsecdec::{
    Atom, AtomCore, EdgeId, Kinematics, Model, ParameterCard, Symbol,
    input::GraphIntegral,
    parametric::{
        FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use feynkit_model::ParameterNature;
use symbolica::{
    atom::AtomView,
    id::{Pattern, Replacement},
    parser::ParseSettings,
};

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
    pub input_seconds: f64,
    pub parametrization_seconds: f64,
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
        Replacement::new(
            Pattern::Literal(Atom::var(*symbol)),
            Pattern::Literal(value.clone()),
        )
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

fn model_values(model: &Model, restriction: &ParameterCard) -> CliResult<BTreeMap<Symbol, Atom>> {
    let mut result = BTreeMap::new();
    for parameter in model.parameters() {
        // Cached numeric dependents belong to the model's original point. Keep
        // native analytic definitions for exact inline values; explicit internal
        // restriction-card values remain authoritative, as in native recomputation.
        let analytic = parameter.expression.as_ref().filter(|_| {
            parameter.nature == ParameterNature::Internal
                && !restriction.contains_key(&parameter.name)
        });
        let value = if let Some(expression) = analytic {
            expression.clone()
        } else if let Some(value) = parameter.value {
            expression(&format!("({})+({})*i", value.re, value.im))?
        } else if let Some(expression) = &parameter.expression {
            expression.clone()
        } else {
            continue;
        };
        result.insert(symbol(&format!("UFO::{}", parameter.name))?, value);
    }
    for coupling in model.couplings() {
        result.insert(
            symbol(&format!("UFO::{}", coupling.name))?,
            coupling.expression.clone(),
        );
    }
    Ok(result)
}

pub fn load(path: &Path) -> CliResult<LoadedInput> {
    let started = Instant::now();
    let mut sources = Vec::new();
    let read = |path: &Path, sources: &mut Vec<crate::artifact::SourceFile>| -> CliResult<String> {
        let text = fs::read_to_string(path)?;
        sources.push(crate::artifact::SourceFile {
            path: fs::canonicalize(path)?.to_string_lossy().into_owned(),
            blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
            fingerprint: crate::artifact::SourceFingerprint::Bytes,
        });
        Ok(text)
    };
    let text = read(path, &mut sources)?;
    sources[0].fingerprint = crate::artifact::SourceFingerprint::RunCardWithoutReference;
    sources[0].blake3 = sources[0].fingerprint.hash(text.as_bytes())?;
    let card: RunCard = toml::from_str(&text)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut values = BTreeMap::new();
    for (name, value) in &card.parameters {
        values.insert(symbol(name)?, value_expression(value)?);
    }
    let regulator = symbol(&card.integral.regulator)?;
    match (&card.input, &card.direct) {
        (Some(input), None) => {
            let mut model = Model::from_json(&read(&base.join(&input.model), &mut sources)?)?;
            let restriction = if let Some(parameter_card) = &input.parameter_card {
                let restriction =
                    ParameterCard::from_json(&read(&base.join(parameter_card), &mut sources)?)?;
                model.apply_parameter_card(&restriction)?;
                restriction
            } else {
                ParameterCard::new()
            };
            let mut all = model_values(&model, &restriction)?;
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
            let input_seconds = started.elapsed().as_secs_f64();
            let parametrization_started = Instant::now();
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
                input_seconds,
                parametrization_seconds: parametrization_started.elapsed().as_secs_f64(),
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
            let input_seconds = started.elapsed().as_secs_f64();
            let parametrization_started = Instant::now();
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
                input_seconds,
                parametrization_seconds: parametrization_started.elapsed().as_secs_f64(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_parameter_names_are_literal_not_pattern_wildcards() {
        let values = BTreeMap::from([(symbol("user_value_").unwrap(), expression("2").unwrap())]);
        assert_eq!(
            bind(&expression("user_value_+untouched").unwrap(), &values),
            expression("2+untouched").unwrap()
        );
    }

    #[test]
    fn native_model_cached_dependents_follow_exact_overrides_and_restrictions() {
        let directory = tempfile::tempdir().unwrap();
        let mut model: serde_json::Value =
            serde_json::from_str(include_str!("../../../examples/models/scalar.json")).unwrap();
        model["parameters"].as_array_mut().unwrap().extend([
            serde_json::json!({"name":"a","nature":"external","parameter_type":"real","value":[2.0,0.0],"expression":null}),
            serde_json::json!({"name":"b","nature":"internal","parameter_type":"real","value":[4.0,0.0],"expression":"2*UFO::a"}),
            serde_json::json!({"name":"fixed","nature":"internal","parameter_type":"real","value":[11.0,0.0],"expression":null}),
        ]);
        model["couplings"] = serde_json::json!([
            {"name":"GC_check","expression":"UFO::b","orders":[],"value":[4.0,0.0]}
        ]);
        fs::write(
            directory.path().join("model.json"),
            serde_json::to_vec(&model).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.path().join("bubble.dot"),
            include_str!("../../../examples/graphs/bubble.dot"),
        )
        .unwrap();
        for (inline, restriction, expected) in [
            ("'UFO::a' = '3'", None, "23*gamma(eps)"),
            (
                "'UFO::a' = 'symbolica::pi'",
                None,
                "(11+4*symbolica::pi)*gamma(eps)",
            ),
            ("'UFO::a' = '3'", Some(r#"{"b":[5,0]}"#), "21*gamma(eps)"),
            (
                "'UFO::a' = '3'\n'UFO::b' = '7'",
                Some(r#"{"b":[5,0]}"#),
                "25*gamma(eps)",
            ),
        ] {
            let restriction_field = if let Some(restriction) = restriction {
                fs::write(directory.path().join("restriction.json"), restriction).unwrap();
                "parameter_card = 'restriction.json'"
            } else {
                ""
            };
            let card = format!(
                r#"
[input]
graph = 'bubble.dot'
model = 'model.json'
{restriction_field}
[kinematics]
products = [{{left=1,right=1,value='-1'}}]
[parameters]
{inline}
[integral]
measure_multiplier = 'UFO::b+UFO::GC_check+UFO::fixed'
"#
            );
            let path = directory.path().join("input.toml");
            fs::write(&path, card).unwrap();
            let loaded = load(&path).unwrap();
            assert_eq!(
                loaded.integrand.terms()[0].prefactor(),
                &expression(expected).unwrap(),
                "inline {inline}, restriction {restriction:?}"
            );
        }
    }
}
