use super::*;
use crate::parametric::{FactorRole, ParametricDomain, ParametricTerm, PolynomialFactor};
use symbolica::{parse, symbol};

fn mapped(
    parameters: Vec<Symbol>,
    domain: ParametricDomain,
    polynomial: Atom,
    exponent: Atom,
    role: FactorRole,
    matrix: Vec<Vec<Integer>>,
    jacobian: Vec<Integer>,
) -> Vec<MappedTerm> {
    let input = ParametricIntegrand::new(
        parameters.clone(),
        symbol!("mapped_factor::eps"),
        domain,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; parameters.len()],
            vec![PolynomialFactor::new(polynomial, exponent, role)],
        )],
    )
    .unwrap();
    let map = SectorMap {
        fixed_parameter: None,
        exponent_matrix: matrix,
        determinant: Integer::from(1),
        jacobian_powers: jacobian,
        factor_valuations: vec![],
    };
    map.validate(domain).unwrap();
    let target = [symbol!("mapped_factor::s"), symbol!("mapped_factor::t")];
    let coordinates = coordinates(&input, &map, &target[..parameters.len()]);
    let mut supports = super::super::support::SupportCache::new(input.parameters());
    map_terms(&input, &map, &coordinates, &mut supports).unwrap()
}

#[test]
fn large_chart_monomial_keeps_the_regular_power_compact() {
    let terms = mapped(
        vec![symbol!("mapped_factor::x"), symbol!("mapped_factor::y")],
        ParametricDomain::UnitCube,
        parse!("(mapped_factor::x+mapped_factor::y)^10000"),
        Atom::one(),
        FactorRole::Polynomial,
        vec![vec![1.into(), 0.into()], vec![1.into(), 1.into()]],
        vec![1.into(), 0.into()],
    );
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].powers, [Atom::num(10001), Atom::Zero]);
    assert_eq!(terms[0].regular, parse!("(1+mapped_factor::t)^10000"));
    assert!(terms[0].regular.as_view().get_byte_size() < 128);
}

#[test]
fn orthant_infinity_stripping_retains_the_measure_and_residual() {
    let terms = mapped(
        vec![symbol!("mapped_factor::x")],
        ParametricDomain::PositiveOrthant,
        parse!("1+mapped_factor::x"),
        Atom::num(-2),
        FactorRole::Singularity,
        vec![vec![(-1).into()]],
        vec![(-2).into()],
    );
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].powers, [Atom::Zero]);
    assert_eq!(terms[0].regular, parse!("(1+mapped_factor::s)^-2"));
}

#[test]
fn hidden_polynomial_cancellation_uses_the_exact_sparse_fallback() {
    let terms = mapped(
        vec![symbol!("mapped_factor::x")],
        ParametricDomain::UnitCube,
        parse!("(1+mapped_factor::x)^2-1-2*mapped_factor::x"),
        Atom::one(),
        FactorRole::Polynomial,
        vec![vec![1.into()]],
        vec![0.into()],
    );
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].powers, [Atom::num(2)]);
    assert_eq!(terms[0].regular, Atom::one());
}

#[test]
fn one_coordinate_candidates_match_independent_full_support() {
    use symbolica::domains::atom::AtomField;
    let variables = [
        parse!("candidate::x"),
        parse!("candidate::y"),
        parse!("candidate::z"),
    ];
    let field = AtomField {
        statistical_zero_test: false,
        ..AtomField::new()
    };
    for text in [
        "x^3*(x+x*y)^7*(y+y*z)^3",
        "(x*y+x^2*y)^4*(z+z*y)^2",
        "x^2*y^3*(1+x+y+z)^4",
    ] {
        let expression = Atom::parse(text, "candidate", Default::default()).unwrap();
        let (minima, residual) = regular::common_monomial(&expression, &variables).unwrap();
        let complete = expression.to_polynomial_in_vars_with_field::<i32>(&variables, &field);
        for (axis, minimum) in minima.iter().enumerate() {
            assert_eq!(*minimum, Integer::from(complete.degree_bounds(axis).0));
        }
        let rebuilt = residual
            * variables
                .iter()
                .zip(minima)
                .map(|(v, n)| v.pow(Atom::num(n)))
                .product::<Atom>();
        assert_eq!(
            complete,
            rebuilt.to_polynomial_in_vars_with_field::<i32>(&variables, &field)
        );
    }
}

#[test]
fn hidden_coefficient_cancellation_is_only_a_common_monomial_candidate() {
    use symbolica::domains::atom::AtomField;
    let variables = [parse!("candidate::x"), parse!("candidate::y")];
    let expression =
        parse!("candidate::x*((1+candidate::y)^2-1-2*candidate::y-candidate::y^2)+candidate::x^2");
    let (minima, residual) = regular::common_monomial(&expression, &variables).unwrap();
    let field = AtomField {
        statistical_zero_test: false,
        ..AtomField::new()
    };
    let complete = expression.to_polynomial_in_vars_with_field::<i32>(&variables, &field);
    assert_eq!(
        complete,
        parse!("candidate::x^2").to_polynomial_in_vars_with_field::<i32>(&variables, &field)
    );
    for (axis, minimum) in minima.iter().enumerate() {
        assert!(*minimum <= complete.degree_bounds(axis).0);
    }
    let rebuilt = residual
        * variables
            .iter()
            .zip(minima)
            .map(|(v, n)| v.pow(Atom::num(n)))
            .product::<Atom>();
    assert_eq!(
        complete,
        rebuilt.to_polynomial_in_vars_with_field::<i32>(&variables, &field)
    );

    // Failure to recognize the quotient retains the established sparse route.
    assert!(
        regular::common_monomial(
            &parse!("(1+candidate::x)^2-1-2*candidate::x"),
            &variables[..1]
        )
        .is_none()
    );
}

#[test]
fn unsupported_one_coordinate_candidates_fall_back() {
    let variables = [parse!("candidate::x")];
    for expression in [
        Atom::Zero,
        parse!("candidate::x*sin(candidate::x)"),
        parse!("candidate::x/(1+candidate::x)"),
        parse!("candidate::x^-1"),
    ] {
        assert!(regular::common_monomial(&expression, &variables).is_none());
    }
    assert!(regular::common_monomial(&Atom::one(), &[]).is_none());
    // Stay within the native signed exponent limit. This sparse two-term
    // product exercises a near-limit degree without dense power expansion.
    let (minimum, residual) = regular::common_monomial(
        &parse!("candidate::x^1073741823*(1+candidate::x^1073741823)"),
        &variables,
    )
    .unwrap();
    assert_eq!(minimum, [Integer::from(1073741823)]);
    assert_eq!(residual, parse!("1+candidate::x^1073741823"));
}

#[test]
#[ignore = "diagnoses the pre-existing native fixed-width overflow; not a successful admission guard"]
fn native_collector_out_of_range_diagnostic() {
    let variables = [parse!("candidate::x")];
    let expression = parse!("candidate::x^1073741824*(1+candidate::x^1073741824)");
    let old = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        expression.to_polynomial_in_vars::<u32>(&variables)
    }));
    let candidate = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        regular::common_monomial(&expression, &variables)
    }));
    assert!(old.is_err() && candidate.is_err());
    eprintln!(
        "native exponent 2^31: both old full support and new candidate overflow; no production catch or admission claim"
    );
}
