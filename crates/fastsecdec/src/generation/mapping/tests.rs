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
    map_terms(&input, &map, &coordinates).unwrap()
}

#[test]
fn large_chart_monomial_keeps_the_regular_power_compact() {
    let terms = mapped(
        vec![symbol!("mapped_factor::x"), symbol!("mapped_factor::y")],
        ParametricDomain::UnitCube,
        parse!("(mapped_factor::x+mapped_factor::y)^128"),
        Atom::one(),
        FactorRole::Polynomial,
        vec![vec![1.into(), 0.into()], vec![1.into(), 1.into()]],
        vec![1.into(), 0.into()],
    );
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].powers, [Atom::num(129), Atom::Zero]);
    assert_eq!(terms[0].regular, parse!("(1+mapped_factor::t)^128"));
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
