use super::*;
use crate::generation::SubtractionStrategy;
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{atom::AtomView, domains::integer::Integer, id::Pattern, parse, symbol};

fn coefficient(
    request: &Request,
    terms: &[MappedTerm],
    parameters: &[Symbol],
    regulator: Symbol,
) -> Atom {
    let mut value = terms[request.term].regular.clone();
    let mut factorial = Integer::from(1);
    for (parameter, degree) in std::iter::once((regulator, request.epsilon_order)).chain(
        parameters
            .iter()
            .copied()
            .zip(request.derivatives.iter().copied()),
    ) {
        for index in 0..degree {
            value = value.derivative(parameter);
            factorial *= Integer::from(index + 1);
        }
    }
    value = value
        .replace(Pattern::Literal(Atom::var(regulator)))
        .with(Atom::Zero);
    let replacements = parameters
        .iter()
        .copied()
        .zip(&request.coordinates)
        .map(|(symbol, coordinate)| {
            let value = match coordinate {
                Coordinate::Variable(index) => Atom::var(parameters[*index]),
                Coordinate::Zero => Atom::Zero,
                Coordinate::One => Atom::one(),
            };
            (symbol, value)
        })
        .collect::<BTreeMap<_, _>>();
    value.replace_map(|term, _, out| {
        if let AtomView::Var(term) = term
            && let Some(value) = replacements.get(&term.get_symbol())
        {
            **out = value.clone();
        }
    }) / Atom::num(factorial)
}

fn parity(regular: Atom, strategy: SubtractionStrategy) -> Recipe {
    let parameters = [
        symbol!("dual_recipe_test::x"),
        symbol!("dual_recipe_test::y"),
    ];
    let regulator = symbol!("dual_recipe_test::eps");
    let terms = vec![MappedTerm {
        powers: vec![
            parse!("-3-2*dual_recipe_test::eps"),
            parse!("-2+dual_recipe_test::eps"),
        ],
        prefactor: parse!("1/(dual_recipe_test::eps*(1+dual_recipe_test::eps))"),
        regular,
    }];
    let options = GenerationOptions {
        subtraction: strategy,
        max_order: 1,
        ..Default::default()
    };
    let recipe = expand(&terms, &parameters, regulator, &options, &[], &mut |_| {
        ControlFlow::Continue(())
    })
    .unwrap();
    let expected = coefficient_first::expand(
        &terms,
        &parameters,
        regulator,
        &options,
        coefficient_first::Limits::default(),
        &mut TemplateCache::default(),
        &mut |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let replacements = recipe
        .requests
        .iter()
        .map(|request| {
            (
                request.placeholder,
                coefficient(request, &terms, &parameters, regulator),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let orders = recipe
        .coefficients
        .keys()
        .chain(expected.coefficients.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for order in orders {
        let actual = recipe
            .coefficients
            .get(&order)
            .cloned()
            .unwrap_or_else(|| AliasedAtom::from(Atom::Zero))
            .into_inner()
            .replace_map(|term, _, out| {
                if let AtomView::Var(term) = term
                    && let Some(value) = replacements.get(&term.get_symbol())
                {
                    **out = value.clone();
                }
            });
        let expected = expected
            .coefficients
            .get(&order)
            .cloned()
            .unwrap_or_else(|| AliasedAtom::from(Atom::Zero))
            .into_inner();
        assert!(
            (actual - expected).together().expand().is_zero(),
            "{strategy:?}, epsilon order {order}"
        );
    }
    recipe
}

#[test]
fn normalized_formal_jets_preserve_taylor_ibp_mixed_faces_and_laurent_poles() {
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let recipe = parity(
            parse!(
                "(1+dual_recipe_test::x)^3*(1+2*dual_recipe_test::y+dual_recipe_test::y^2)*(1+dual_recipe_test::eps+dual_recipe_test::eps^2+dual_recipe_test::eps^3)"
            ),
            strategy,
        );
        assert!(
            recipe
                .requests
                .iter()
                .any(|request| request.epsilon_order >= 2)
        );
        assert!(
            recipe
                .requests
                .iter()
                .any(|request| request.derivatives[0] >= 2)
        );
        assert!(recipe.requests.iter().any(|request| {
            request
                .coordinates
                .iter()
                .all(|face| *face == Coordinate::Zero)
        }));
        if strategy == SubtractionStrategy::IntegrateByParts {
            assert!(
                recipe
                    .requests
                    .iter()
                    .any(|request| request.coordinates.contains(&Coordinate::One))
            );
        }
    }
}

#[test]
fn opaque_regular_epsilon_power_matches_native_symbolic_coefficients() {
    for strategy in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        parity(
            parse!("(1+dual_recipe_test::x+dual_recipe_test::y)^(-1-dual_recipe_test::eps)"),
            strategy,
        );
    }
}

#[test]
fn exact_admission_cancellation_and_request_limits_are_explicit() {
    let parameters = [symbol!("dual_recipe_admission::x")];
    let regulator = symbol!("dual_recipe_admission::eps");
    let terms = vec![MappedTerm {
        powers: vec![Atom::num(-2)],
        prefactor: Atom::one(),
        regular: parse!("dual_recipe_admission::x^2"),
    }];
    assert!(matches!(
        expand(
            &terms,
            &parameters,
            regulator,
            &GenerationOptions::default(),
            &[],
            &mut |_| ControlFlow::Continue(())
        ),
        Err(RecipeError::NeedsExactAdmission { term: 0, axis: 0 })
    ));
    assert!(matches!(
        expand(
            &terms,
            &parameters,
            regulator,
            &GenerationOptions::default(),
            &[],
            &mut |_| ControlFlow::Break(())
        ),
        Err(RecipeError::Generation(GenerationError::Cancelled))
    ));
    let terms = vec![MappedTerm {
        powers: vec![Atom::Zero],
        prefactor: Atom::one(),
        regular: Atom::one(),
    }];
    let mut options = GenerationOptions::default();
    options.coefficient_expansion.max_unique_requests = Some(0);
    assert!(matches!(
        expand(&terms, &parameters, regulator, &options, &[], &mut |_| {
            ControlFlow::Continue(())
        }),
        Err(RecipeError::Generation(GenerationError::ResourceLimit(_)))
    ));
}

#[test]
fn request_identity_is_deterministic_and_avoids_source_symbols() {
    let parameters = [symbol!("dual_recipe_identity::x")];
    let regulator = symbol!("dual_recipe_identity::eps");
    let reserved = [
        symbol!("fastsecdec::numerical_dual::r0"),
        symbol!("fastsecdec::numerical_dual::f0"),
    ];
    let terms = vec![MappedTerm {
        powers: vec![parse!("-2+dual_recipe_identity::eps")],
        prefactor: Atom::one(),
        regular: Atom::one(),
    }];
    let run = || {
        expand(
            &terms,
            &parameters,
            regulator,
            &GenerationOptions::default(),
            &reserved,
            &mut |_| ControlFlow::Continue(()),
        )
        .unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first.requests, second.requests);
    assert!(
        first
            .requests
            .iter()
            .all(|request| !reserved.contains(&request.placeholder))
    );
    for (order, first) in first.coefficients {
        assert_eq!(first.get_root(), second.coefficients[&order].get_root());
        assert_eq!(
            first.get_aliases(),
            second.coefficients[&order].get_aliases()
        );
    }
}
