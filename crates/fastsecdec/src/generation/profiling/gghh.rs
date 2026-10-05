//! Ignored attribution on the unchanged generated ggHH fixture. JSON is only a
//! data translation of its TOML card; native owners construct every expression.

use super::*;
use crate::{Atom, ParameterCard, Symbol};
use feynkit_model::ParameterNature;
use symbolica::{
    atom::AtomView,
    domains::rational::Rational,
    id::{Pattern, Replacement},
};

fn expression(text: &str) -> Atom {
    Atom::parse(text, "feynkit_graph", Default::default()).unwrap()
}

fn scalar_symbol(text: &str) -> Symbol {
    match expression(text).as_view() {
        AtomView::Var(variable) => variable.get_symbol(),
        _ => panic!("expected a scalar symbol"),
    }
}

fn bind(value: &Atom, values: &BTreeMap<Symbol, Atom>) -> Atom {
    value.replace_multiple(values.iter().map(|(symbol, value)| {
        Replacement::new(
            Pattern::Literal(Atom::var(*symbol)),
            Pattern::Literal(value.clone()),
        )
    }))
}

// Same native model-value and recursive binding boundary as CLI input::load.
// This diagnostic is deliberately limited to the supplied fixed fixture.
fn bindings(model: &Model, restriction: &ParameterCard) -> BTreeMap<Symbol, Atom> {
    let mut values = BTreeMap::new();
    for parameter in model.parameters() {
        let analytic = parameter.expression.as_ref().filter(|_| {
            parameter.nature == ParameterNature::Internal
                && !restriction.contains_key(&parameter.name)
        });
        let value = if let Some(expression) = analytic {
            expression.clone()
        } else if let Some(value) = parameter.value {
            Atom::num(Rational::try_from(value.re).unwrap())
                + Atom::num(Rational::try_from(value.im).unwrap()) * Atom::i()
        } else if let Some(expression) = &parameter.expression {
            expression.clone()
        } else {
            continue;
        };
        values.insert(scalar_symbol(&format!("UFO::{}", parameter.name)), value);
    }
    for coupling in model.couplings() {
        values.insert(
            scalar_symbol(&format!("UFO::{}", coupling.name)),
            coupling.expression.clone(),
        );
    }
    for _ in 0..=values.len() {
        let next = values.iter().map(|(s, v)| (*s, bind(v, &values))).collect();
        if next == values {
            break;
        }
        values = next;
    }
    assert!(values.values().all(|value| {
        values
            .keys()
            .all(|symbol| !value.contains(Atom::var(*symbol).as_view()))
    }));
    values
}

#[test]
#[ignore = "explicit original ggHH input, output trace and external resource bound required"]
fn original_gghh_mapping_stage_attribution() {
    let card_path = std::env::var("FASTSECDEC_GGHH_CARD_JSON").unwrap();
    let base = std::env::var("FASTSECDEC_GGHH_INPUT_DIR").unwrap();
    let mapping_trace = std::env::var("FASTSECDEC_GGHH_MAPPING_TRACE").ok();
    let symmetry_trace = std::env::var("FASTSECDEC_GGHH_SYMMETRY_TRACE").ok();
    assert!(mapping_trace.is_some() || symmetry_trace.is_some());
    let card: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(card_path).unwrap()).unwrap();
    assert!(card.get("parameters").is_none());
    assert!(card.get("direct").is_none());
    assert!(card["integral"].get("powers").is_none());
    assert!(card["generation"].get("family_preparation").is_none());
    let read = |field: &str| {
        std::fs::read_to_string(Path::new(&base).join(card["input"][field].as_str().unwrap()))
            .unwrap()
    };
    let mut model = Model::from_json(&read("model")).unwrap();
    let restriction = ParameterCard::from_json(&read("parameter_card")).unwrap();
    model.apply_parameter_card(&restriction).unwrap();
    let values = bindings(&model, &restriction);
    let mut kinematics = Kinematics::in_dimension(&expression("D")).unwrap();
    for product in card["kinematics"]["products"].as_array().unwrap() {
        kinematics = kinematics
            .with_scalar_product(
                &expression(product["left"].as_str().unwrap()),
                &expression(product["right"].as_str().unwrap()),
                bind(&expression(product["value"].as_str().unwrap()), &values),
            )
            .unwrap();
    }
    let auxiliary = card["kinematics"]["auxiliary_momenta"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| expression(v.as_str().unwrap()))
        .collect::<Vec<_>>();
    let graph = GraphIntegral::from_dot(Arc::new(model), &read("graph"), &kinematics)
        .unwrap()
        .with_auxiliary_external_momenta(&auxiliary)
        .unwrap()
        .with_powers(&BTreeMap::new())
        .unwrap()
        .with_measure_multiplier(expression(
            card["integral"]["measure_multiplier"].as_str().unwrap(),
        ))
        .with_scalar_values(&values)
        .unwrap();
    let input = ParametricIntegrand::from_graph(
        &graph,
        (0..graph.powers().len())
            .map(|i| scalar_symbol(&format!("fastsecdec::x{i}")))
            .collect(),
        scalar_symbol(card["integral"]["regulator"].as_str().unwrap()),
        bind(
            &expression(card["integral"]["dimension"].as_str().unwrap()),
            &values,
        ),
    )
    .unwrap();
    assert_eq!((input.parameters().len(), input.terms().len()), (7, 179));
    let mut options = GenerationOptions {
        max_order: card["generation"]["order"]
            .as_i64()
            .unwrap()
            .try_into()
            .unwrap(),
        assume_no_threshold: card["generation"]["assume_no_threshold"].as_bool().unwrap(),
        coefficient_expansion: serde_json::from_value(
            card["generation"]["coefficient_expansion"].clone(),
        )
        .unwrap(),
        ..Default::default()
    };
    // Exact absent-card CLI defaults, not the independently evolving library defaults.
    options.decomposition.max_sectors = card["generation"]["max_sectors"]
        .as_u64()
        .unwrap_or(1_000_000)
        .try_into()
        .unwrap();
    options.decomposition.max_support_pairs = card["generation"]["max_support_pairs"]
        .as_u64()
        .unwrap_or(10_000_000)
        .try_into()
        .unwrap();
    assert_eq!(options.max_order, 0);
    assert!(!options.assume_no_threshold);
    eprintln!("original ggHH: 7 parameters, 179 terms; unchanged native generation options");
    if let Some(trace) = mapping_trace {
        super::super::mapping::profile::begin_trace(Path::new(&trace));
    }
    if let Some(trace) = symmetry_trace {
        symmetry::profile::begin_trace(Path::new(&trace));
    }
    let result = generate(&input, &options, |progress| {
        eprintln!("{progress:?}");
        if let GenerationProgress::Factorization { total, .. } = progress {
            assert_eq!(*total, 30);
        }
        ControlFlow::Continue(())
    });
    eprintln!("generation completed: {}", result.is_ok());
}
