use super::*;
use crate::generation::SubtractionStrategy;
use symbolica::{parse, symbol};

fn terms() -> Vec<MappedTerm> {
    vec![
        MappedTerm {
            powers: vec![parse!("-2+formula_key::eps"), parse!("2*formula_key::eps")],
            prefactor: parse!("(1+formula_key::eps)/formula_key::eps"),
            regular: parse!("(1+formula_key::x)^2*(1+formula_key::y)"),
        },
        MappedTerm {
            powers: vec![parse!("formula_key::eps"), parse!("-1-formula_key::eps")],
            prefactor: parse!("-1/formula_key::eps"),
            regular: Atom::one(),
        },
    ]
}
fn key(terms: &[MappedTerm], options: &GenerationOptions) -> Key {
    Key::discover(terms, 2, symbol!("formula_key::eps"), options)
        .unwrap()
        .unwrap()
}

#[test]
fn exact_signature_preserves_ordered_powers_prefactors_strategy_and_limits() {
    let options = GenerationOptions::default();
    let baseline = key(&terms(), &options);
    let mut changed = terms();
    changed[0].regular = parse!("7+formula_key::x^4");
    assert_eq!(
        baseline,
        key(&changed, &options),
        "native body is supplied at instantiation"
    );
    changed[0].powers.swap(0, 1);
    assert_ne!(
        baseline,
        key(&changed, &options),
        "coordinate ordering matters"
    );
    changed = terms();
    changed.swap(0, 1);
    assert_ne!(
        baseline,
        key(&changed, &options),
        "request term indices are ordered"
    );
    changed = terms();
    changed[0].prefactor += Atom::one();
    assert_ne!(
        baseline,
        key(&changed, &options),
        "complete epsilon prefactor remains exact"
    );
    changed = terms();
    changed[0].powers[0] = parse!("-2+2*formula_key::eps");
    assert_ne!(baseline, key(&changed, &options), "epsilon slope matters");
    for variant in [
        GenerationOptions {
            max_order: 1,
            ..options.clone()
        },
        GenerationOptions {
            subtraction: SubtractionStrategy::IntegrateByParts,
            ..options.clone()
        },
        GenerationOptions {
            max_subtractions_per_axis: 3,
            ..options.clone()
        },
        GenerationOptions {
            max_subtraction_terms: 7,
            ..options.clone()
        },
        {
            let mut value = options.clone();
            value.coefficient_expansion.initial_relative_width = 2;
            value
        },
        {
            let mut value = options.clone();
            value.coefficient_expansion.max_series_attempts = Some(2);
            value
        },
        {
            let mut value = options.clone();
            value.coefficient_expansion.max_relative_width = Some(3);
            value
        },
        {
            let mut value = options.clone();
            value.coefficient_expansion.max_unique_requests = Some(7);
            value
        },
    ] {
        let changed = key(&terms(), &variant);
        assert_ne!(baseline, changed);
        assert_ne!(baseline.lookup_key(), changed.lookup_key());
    }
    let rebuilt = baseline.terms(symbol!("formula_key::eps"));
    for (original, rebuilt) in terms().iter().zip(rebuilt) {
        assert_eq!(original.powers, rebuilt.powers);
        assert_eq!(original.prefactor, rebuilt.prefactor);
    }
}

#[test]
fn discovery_admits_only_regulated_native_endpoint_signatures() {
    let options = GenerationOptions::default();
    let mut source = terms();
    source[0].powers[0] = Atom::num(-1);
    assert!(
        Key::discover(&source, 2, symbol!("formula_key::eps"), &options)
            .unwrap()
            .is_none()
    );
    source[0].powers[0] = parse!("formula_key::eps^2-1");
    assert!(matches!(
        Key::discover(&source, 2, symbol!("formula_key::eps"), &options),
        Err(GenerationError::EndpointExponent(_))
    ));
    assert!(Key::discover(&terms(), 1, symbol!("formula_key::eps"), &options).is_err());
}

#[test]
fn reconstructed_complete_recipe_preserves_native_laurent_composition_and_reserved_symbols() {
    use crate::{
        generation::numerical_dual::pipeline::Context,
        parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
    };
    // Reserved f0/r0 symbols must not be confused with the formal function or
    // request slots, including when absent from the small signature skeleton.
    let f0 = symbol!("fastsecdec::numerical_dual::f0");
    let r0 = symbol!("fastsecdec::numerical_dual::r0");
    let parameters = vec![symbol!("formula_key::x"), symbol!("formula_key::y")];
    let regulator = symbol!("formula_key::eps");
    let input = ParametricIntegrand::new(
        parameters.clone(),
        regulator,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::var(f0) + Atom::var(r0),
            vec![Atom::Zero; 2],
            vec![],
        )],
    )
    .unwrap();
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let options = GenerationOptions {
            subtraction,
            max_order: 1,
            ..Default::default()
        };
        let context = Context::new(&input, &options, parameters.clone());
        let expected = subtraction::expand(
            &terms(),
            &parameters,
            regulator,
            &options,
            &context.reserved_symbols,
            &mut |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let actual = build(
            &key(&terms(), &options),
            &context,
            0,
            Counts {
                total: 1,
                sectors: 2,
            },
            &mut |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(
            actual.coefficients.keys().collect::<Vec<_>>(),
            expected.coefficients.keys().collect::<Vec<_>>()
        );
        for (order, coefficient) in &actual.coefficients {
            assert_eq!(
                coefficient.get_root(),
                expected.coefficients[order].get_root()
            );
            assert_eq!(
                coefficient.get_aliases(),
                expected.coefficients[order].get_aliases()
            );
        }
        assert_eq!(actual.requests, expected.requests);
        assert!(
            actual
                .requests
                .iter()
                .all(|request| request.placeholder != r0 && request.placeholder != f0)
        );
        assert!(
            actual
                .requests
                .iter()
                .any(|request| request.epsilon_order > 0)
        );
    }
}
