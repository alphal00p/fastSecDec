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

#[test]
fn native_polynomial_predicate_distinguishes_factored_coordinates_and_coefficients() {
    let compact = parse!("(x+y)^10000");
    let coordinates = compact.is_polynomial(true, false).unwrap();
    assert_eq!(coordinates.len(), 2);
    assert!(coordinates.contains(&parse!("x").as_view()));
    assert!(coordinates.contains(&parse!("y").as_view()));

    // Native polynomial variables may be compound indeterminates. Admission
    // must verify that these do not hide coordinate dependence.
    for expression in [parse!("sin(x)"), parse!("x^eps"), parse!("(1+x)^(-2)")] {
        assert!(
            expression
                .is_polynomial(true, false)
                .is_none_or(|variables| {
                    variables.iter().any(|variable| {
                        variable.contains(parse!("x").as_view())
                            && *variable != parse!("x").as_view()
                    })
                })
        );
    }
    // The general native predicate treats s and sin(s) as conflicting
    // indeterminates even though both are harmless scalar coefficients in x.
    let coefficient = parse!("x*(sin(s)+s)");
    assert!(coefficient.is_polynomial(true, false).is_none());
    assert!(polynomial_support(&coefficient, &[symbol!("x")]).is_ok());
}

#[test]
fn nonprojective_admission_keeps_large_powers_factored_with_exact_fallback() {
    for domain in [
        ParametricDomain::UnitCube,
        ParametricDomain::PositiveOrthant,
    ] {
        for polynomial in [parse!("(x+y)^10000"), parse!("x*(sin(s)+s)")] {
            let integral = ParametricIntegrand::new(
                vec![symbol!("x"), symbol!("y")],
                symbol!("eps"),
                domain,
                vec![ParametricTerm::new(
                    Atom::one(),
                    vec![Atom::Zero; 2],
                    vec![PolynomialFactor::new(
                        polynomial.clone(),
                        Atom::one(),
                        FactorRole::Polynomial,
                    )],
                )],
            )
            .unwrap();
            assert_eq!(integral.terms()[0].factors()[0].polynomial(), &polynomial);
        }
        for polynomial in [parse!("sin(x)"), parse!("x^eps"), parse!("(1+x)^(-2)")] {
            assert!(
                ParametricIntegrand::new(
                    vec![symbol!("x"), symbol!("y")],
                    symbol!("eps"),
                    domain,
                    vec![ParametricTerm::new(
                        Atom::one(),
                        vec![Atom::Zero; 2],
                        vec![PolynomialFactor::new(
                            polynomial,
                            Atom::one(),
                            FactorRole::Polynomial
                        )],
                    )],
                )
                .is_err()
            );
        }
    }
}

fn projective_numerator(
    polynomial: Atom,
    degree: i64,
) -> Result<ParametricIntegrand, fastsecdec::parametric::ParametricError> {
    ParametricIntegrand::new(
        vec![symbol!("x"), symbol!("y")],
        symbol!("eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![
                factor(parse!("x+y"), Atom::num(-2 - degree)),
                PolynomialFactor::new(polynomial, Atom::one(), FactorRole::Polynomial),
            ],
        )],
    )
}

#[test]
fn projective_regular_admission_keeps_large_numerators_factored() {
    let polynomial = parse!("(x+y)^10000");
    let integral = projective_numerator(polynomial.clone(), 10000).unwrap();
    assert_eq!(integral.terms()[0].factors()[1].polynomial(), &polynomial);
    assert!(projective_numerator(polynomial, 9999).is_err());
}

#[test]
fn projective_regular_admission_rejects_hidden_dependence_and_keeps_exact_fallback() {
    for polynomial in [parse!("x/y"), parse!("sin(x)"), parse!("x+y^2")] {
        assert!(projective_numerator(polynomial, 0).is_err());
    }
    // The scaling conversion retains both formal degrees with exact coefficient
    // zero tests. Existing sparse support resolves the cancelling degree two.
    let cancelling = parse!("x^2-y^2-(x-y)*(x+y)+x");
    let integral = projective_numerator(cancelling.clone(), 1).unwrap();
    assert_eq!(integral.terms()[0].factors()[1].polynomial(), &cancelling);
    assert!((cancelling - parse!("x")).expand().is_zero());

    // Native general polynomial recognition can be conservative about scalar
    // functions; support admission still accepts parameter-independent ones.
    assert!(projective_numerator(parse!("x*(sin(s)+s)"), 1).is_ok());
}

#[test]
fn projective_regular_scaling_does_not_claim_nonzero() {
    let literal = projective_numerator(Atom::Zero, 0).unwrap();
    assert!(literal.terms().is_empty());
    let disguised = parse!("x^2-y^2-(x-y)*(x+y)");
    assert!(!disguised.is_zero());
    assert!(disguised.expand().is_zero());
    // An identically zero regular numerator obeys the scaling identity too.
    // Admission neither declares it nonzero nor manufactures a zero result.
    let integral = projective_numerator(disguised.clone(), 2).unwrap();
    assert_eq!(integral.terms()[0].factors()[1].polynomial(), &disguised);
}

#[test]
fn exact_support_keeps_a_small_coefficient_hidden_by_cancellation() {
    // Native sampled zero testing is inconclusive here because the exact
    // residual is far below the roundoff of its unexpanded representation.
    let coefficient =
        parse!("(1+support_exact::a)^2-support_exact::a^2-2*support_exact::a-1+1/10^30");
    assert_eq!(coefficient.expand(), parse!("1/10^30"));
    let expression = coefficient * parse!("support_exact::x^2") + parse!("support_exact::x^3");
    let support = polynomial_support(&expression, &[symbol!("support_exact::x")]).unwrap();
    assert_eq!(
        support.exponents(),
        &[
            vec![symbolica::domains::integer::Integer::from(2)],
            vec![symbolica::domains::integer::Integer::from(3)]
        ]
    );
}
