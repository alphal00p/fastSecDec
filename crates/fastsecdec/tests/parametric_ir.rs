use fastsecdec::parametric::{
    FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    polynomial_support,
};
use fastsecdec::{Atom, AtomCore};
use symbolica::{parse, symbol};

fn factor(expression: Atom, exponent: Atom) -> PolynomialFactor {
    PolynomialFactor::new(expression, exponent, FactorRole::Singularity)
}

#[test]
fn projective_density_checks_the_full_degree_before_gauge_fixing() {
    let density = ParametricIntegrand::new(
        vec![symbol!("x"), symbol!("y")],
        symbol!("eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            parse!("gamma(eps)"),
            vec![Atom::Zero; 2],
            vec![
                factor(parse!("x+y"), parse!("-2+2*eps")),
                factor(parse!("-s*x*y"), parse!("-eps")),
            ],
        )],
    )
    .unwrap();
    assert_eq!(density.terms().len(), 1);
    assert_eq!(density.parameters().len(), 2);
    let wrong_degree = ParametricIntegrand::new(
        vec![symbol!("x"), symbol!("y")],
        symbol!("eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![factor(parse!("x+y"), parse!("-1"))],
        )],
    );
    assert!(wrong_degree.is_err());
}

#[test]
fn direct_domain_and_numerator_role_do_not_infer_projective_scaling() {
    let density = ParametricIntegrand::new(
        vec![symbol!("x")],
        symbol!("eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("1/eps"),
            vec![parse!("-1+eps")],
            vec![PolynomialFactor::new(
                parse!("1+eps*x+x^2"),
                Atom::one(),
                FactorRole::Polynomial,
            )],
        )],
    )
    .unwrap();
    assert_eq!(
        density.terms()[0].factors()[0].role(),
        FactorRole::Polynomial
    );
    assert!(
        (density.density() - parse!("1/eps*x^(-1+eps)*(1+eps*x+x^2)"))
            .expand()
            .is_zero()
    );
}

#[test]
fn support_adapter_refuses_hidden_nonpolynomial_dependence() {
    assert!(polynomial_support(&parse!("1+1/x"), &[symbol!("x")]).is_err());
    assert!(polynomial_support(&parse!("sin(x)"), &[symbol!("x")]).is_err());
    let support = polynomial_support(&parse!("(x+y)^2"), &[symbol!("x"), symbol!("y")]).unwrap();
    assert_eq!(support.exponents().len(), 3);
}

#[test]
fn regulator_dependent_singularities_and_parameter_dependent_exponents_are_rejected() {
    for polynomial in [parse!("eps+x"), parse!("1/(1+x)")] {
        assert!(
            ParametricIntegrand::new(
                vec![symbol!("x")],
                symbol!("eps"),
                ParametricDomain::UnitCube,
                vec![ParametricTerm::new(
                    Atom::one(),
                    vec![Atom::Zero],
                    vec![factor(polynomial, parse!("-1"))]
                )]
            )
            .is_err()
        );
    }
    assert!(
        ParametricIntegrand::new(
            vec![symbol!("x")],
            symbol!("eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(Atom::one(), vec![parse!("x")], vec![])]
        )
        .is_err()
    );
}
