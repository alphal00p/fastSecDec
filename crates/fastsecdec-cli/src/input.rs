use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use fastsecdec::{
    Atom, AtomCore, EdgeId, Kinematics, Model, ParameterCard, Symbol,
    input::{GraphIntegral, RuntimeModelBindings},
    parametric::{
        FactorRole, FamilyPreparationPolicy, FamilyPreparationReport, ParametricDomain,
        ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use feynkit_model::resolve_scalar_bindings;
use symbolica::{
    atom::AtomView,
    domains::rational::Rational,
    id::{Pattern, Replacement},
    parser::ParseSettings,
};

use crate::{
    CliResult,
    config::{MomentumName, RunCard},
};

impl MomentumName {
    fn atom(&self) -> CliResult<Atom> {
        match self {
            Self::External(index) => Ok(feynkit_graph::symbols::external_momentum().call(*index)),
            Self::Expression(text) => expression(text),
        }
    }
}

pub struct LoadedInput {
    pub card: RunCard,
    pub integrand: ParametricIntegrand,
    pub runtime_parameters: Vec<Symbol>,
    pub runtime_mass_constraints: Vec<fastsecdec::kernel::RuntimeMassConstraint>,
    pub model_parameter_defaults: BTreeMap<String, f64>,
    pub label: String,
    pub loops: Option<usize>,
    pub propagators: usize,
    pub sources: Vec<crate::artifact::SourceFile>,
    pub family_preparation: Option<FamilyPreparationReport>,
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

pub fn value_expression(value: &toml::Value) -> CliResult<Atom> {
    match value {
        toml::Value::String(text) => expression(text),
        toml::Value::Integer(value) => Ok(Atom::num(*value)),
        toml::Value::Float(value) if value.is_finite() => {
            Ok(Atom::num(Rational::try_from(*value)?))
        }
        _ => Err("parameter values must be Symbolica strings or finite numbers".into()),
    }
}

pub fn load(path: &Path) -> CliResult<LoadedInput> {
    let started = Instant::now();
    let mut sources = Vec::new();
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let read = |path: &Path, sources: &mut Vec<crate::artifact::SourceFile>| -> CliResult<String> {
        let text = fs::read_to_string(path)?;
        sources.push(crate::artifact::SourceFile {
            path: crate::artifact::relative_path(path, base)?
                .to_string_lossy()
                .into_owned(),
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
            let graph_text = read(&base.join(&input.graph), &mut sources)?;
            let diagram = feynkit_graph::FeynmanDiagram::from_dot(Arc::new(model), &graph_text)?;
            let mut runtime_model = match input.model_parameters {
                crate::config::ModelParameters::Runtime => Some(RuntimeModelBindings::new(
                    &diagram,
                    Some(&restriction),
                    &values,
                )?),
                crate::config::ModelParameters::Fixed => None,
            };
            let values = match &runtime_model {
                Some(runtime) => runtime.values().clone(),
                None => diagram
                    .model()
                    .scalar_bindings(Some(&restriction), &values)?,
            };
            let model_symbols = runtime_model
                .as_ref()
                .map_or_else(Vec::new, RuntimeModelBindings::symbols);
            if model_symbols.contains(&regulator) {
                return Err("runtime model inputs must differ from the regulator".into());
            }
            let mut kinematics = Kinematics::in_dimension(&expression("D")?)?;
            let mut runtime_parameters = Vec::new();
            for product in &card.kinematics.products {
                let product_value = match (&product.value, &product.symbol) {
                    (Some(value), None) => bind(&expression(value)?, &values),
                    (None, Some(name)) => {
                        let parameter = symbol(name)?;
                        if parameter == regulator
                            || values.contains_key(&parameter)
                            || model_symbols.contains(&parameter)
                        {
                            return Err(format!("runtime kinematic symbol {name} conflicts with a fixed scalar or regulator").into());
                        }
                        if !runtime_parameters.contains(&parameter) {
                            runtime_parameters.push(parameter);
                        }
                        Atom::var(parameter)
                    }
                    _ => {
                        return Err(
                            "each kinematic product requires exactly one of symbol or value".into(),
                        );
                    }
                };
                kinematics = kinematics.with_scalar_product(
                    &product.left.atom()?,
                    &product.right.atom()?,
                    product_value,
                )?;
            }
            let powers = card
                .integral
                .powers
                .iter()
                .map(|(edge, power)| edge.parse::<usize>().map(|edge| (EdgeId(edge), *power)))
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            let graph = GraphIntegral::new_with_runtime_scalar_values(
                Arc::new(diagram),
                &kinematics,
                &values,
                &model_symbols,
            )?
            .with_auxiliary_external_momenta(
                &card
                    .kinematics
                    .auxiliary_momenta
                    .iter()
                    .map(|name| expression(name))
                    .collect::<CliResult<Vec<_>>>()?,
            )?
            .with_powers(&powers)?
            .with_measure_multiplier(expression(&card.integral.measure_multiplier)?);
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
            if runtime_parameters
                .iter()
                .any(|parameter| parameters.contains(parameter))
            {
                return Err(
                    "runtime kinematic symbols must differ from integration coordinates".into(),
                );
            }
            let input_seconds = started.elapsed().as_secs_f64();
            let parametrization_started = Instant::now();
            let policy = card.generation.family_preparation;
            let dimension = bind(&expression(&card.integral.dimension)?, &values);
            let (integrand, family_preparation) = if policy == FamilyPreparationPolicy::Original {
                (
                    ParametricIntegrand::from_graph(&graph, parameters, regulator, dimension)?,
                    None,
                )
            } else {
                let (integrand, report) = ParametricIntegrand::from_graph_prepared(
                    &graph, parameters, regulator, dimension, policy,
                )?;
                (integrand, Some(report))
            };
            let (runtime_mass_constraints, model_parameter_defaults) =
                if let Some(runtime) = &mut runtime_model {
                    runtime.retain_used(&integrand);
                    runtime_parameters.extend(runtime.symbols());
                    (runtime.mass_constraints().to_vec(), runtime.defaults())
                } else {
                    (Vec::new(), BTreeMap::new())
                };
            Ok(LoadedInput {
                card,
                integrand,
                runtime_parameters,
                runtime_mass_constraints,
                model_parameter_defaults,
                label: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                loops,
                propagators,
                sources,
                family_preparation,
                independent_externals,
                dependent_externals,
                input_seconds,
                parametrization_seconds: parametrization_started.elapsed().as_secs_f64(),
            })
        }
        (None, Some(direct)) => {
            if card.generation.family_preparation != FamilyPreparationPolicy::Original {
                return Err(
                    "family preparation requires native graph input, not direct parametric data"
                        .into(),
                );
            }
            let values = resolve_scalar_bindings(values)?;
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
                runtime_parameters: Vec::new(),
                runtime_mass_constraints: Vec::new(),
                model_parameter_defaults: BTreeMap::new(),
                loops: None,
                propagators,
                sources,
                family_preparation: None,
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
        "{}.fsd",
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
    fn numeric_model_parameters_keep_native_imaginary_coefficients() {
        let mut json: serde_json::Value =
            serde_json::from_str(include_str!("../../../examples/models/scalar.json")).unwrap();
        json["parameters"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name":"complex_external", "nature":"external", "parameter_type":"complex",
            "value":[2.5,3.25], "expression":null,
            }));
        let model = Model::from_json(&serde_json::to_string(&json).unwrap()).unwrap();
        let values = model.scalar_bindings(None, &BTreeMap::new()).unwrap();
        assert_eq!(
            values[&symbol("UFO::complex_external").unwrap()],
            Atom::num((5, 2)) + Atom::num((13, 4)) * Atom::i(),
        );
        assert_eq!(
            value_expression(&toml::Value::Float(172.5)).unwrap(),
            Atom::num((345, 2)),
        );
    }

    #[test]
    fn inline_zero_width_is_admitted_before_the_default_model_point() {
        let directory = tempfile::tempdir().unwrap();
        let mut model: serde_json::Value =
            serde_json::from_str(include_str!("../../../examples/models/massless_phi3.json"))
                .unwrap();
        model["parameters"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "name":"width", "nature":"external", "parameter_type":"real",
                "value":[0.1,0.0], "expression":null,
            }));
        model["particles"][0]["width"] = "width".into();
        fs::write(directory.path().join("model.json"), model.to_string()).unwrap();
        fs::write(
            directory.path().join("bubble.dot"),
            include_str!("../../../examples/graphs/bubble.dot"),
        )
        .unwrap();
        let card = r#"
[input]
model = 'model.json'
graph = 'bubble.dot'
[kinematics]
products = [{left=1,right=1,value='-1'}]
[parameters]
'UFO::width' = '0'
"#;
        let path = directory.path().join("input.toml");
        fs::write(&path, card).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.propagators, 2);
        assert!(!loaded.integrand.terms().is_empty());
        for card in [
            card.replace("'UFO::width' = '0'", ""),
            card.replace("'UFO::width' = '0'", "'UFO::width' = '1'"),
        ] {
            fs::write(&path, card).unwrap();
            match load(&path) {
                Err(error) => assert!(error.to_string().contains("width")),
                Ok(_) => panic!("unsupported width was silently omitted"),
            }
        }
    }

    #[test]
    fn numerator_vectors_use_native_expression_labels_and_scalar_bindings() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("model.json"),
            include_str!("../../../examples/models/massless_phi3.json"),
        )
        .unwrap();
        let model = Arc::new(
            Model::from_json(include_str!("../../../examples/models/massless_phi3.json")).unwrap(),
        );
        let kin = Kinematics::in_dimension(&expression("D").unwrap()).unwrap();
        let k = feynkit_graph::symbols::loop_momentum().call(0);
        let e = Atom::var(spenso::vector_symbol!("helicity::e"));
        let numerator = kin.scalar_product(&k, &e).unwrap().pow(2);
        let diagram = feynkit_graph::FeynmanDiagram::from_dot(
            model,
            include_str!("../../../examples/graphs/bubble.dot"),
        )
        .unwrap()
        .with_numerator(numerator)
        .unwrap();
        fs::write(
            directory.path().join("graph.dot"),
            diagram.to_dot().unwrap(),
        )
        .unwrap();
        let card = r#"
[input]
graph = "graph.dot"
model = "model.json"
[kinematics]
auxiliary_momenta = ["helicity::e"]
products = [
  {left=1, right=1, value="-1"},
  {left=1, right="helicity::e", value="0"},
  {left="helicity::e", right="helicity::e", value="norm"},
]
[parameters]
norm = "-1"
"#;
        // Native canonical spelling retains the tensor/rank-one metadata in
        // a fresh process, before any graph has registered the vector head.
        let encoded = serde_json::to_string(&e.to_canonical_string()).unwrap();
        let card = card.replace("\"helicity::e\"", &encoded);
        let path = directory.path().join("input.toml");
        fs::write(&path, &card).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.loops, Some(1));
        assert_eq!(loaded.propagators, 2);
        assert!(
            loaded
                .independent_externals
                .contains(&e.to_canonical_string())
        );
        let density = loaded.integrand.density();
        assert!(!density.is_zero());
        assert!(!density.contains(expression("norm").unwrap().as_view()));
        assert!(
            !density
                .get_all_symbols(true)
                .contains(&feynkit_graph::symbols::loop_momentum())
        );

        // The numerator must not silently lose an undeclared vector.
        fs::write(
            &path,
            card.replace(&format!("auxiliary_momenta = [{encoded}]"), ""),
        )
        .unwrap();
        assert!(load(&path).is_err());
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
