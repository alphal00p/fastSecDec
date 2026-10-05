use super::*;
use symbolica::symbol;

fn atom(value: &str) -> Atom {
    Atom::parse(value, "domain_sign_test", Default::default()).unwrap()
}

fn assess(
    polynomial: Atom,
    exponent: &str,
    asserted: bool,
) -> Result<DomainAssessment, GenerationError> {
    check_factors(
        &[symbol!("domain_sign_test::x")],
        ParametricDomain::UnitCube,
        &[(
            0,
            0,
            PolynomialFactor::new(polynomial, atom(exponent), FactorRole::Singularity),
        )],
        asserted,
    )
}

#[test]
fn algebraic_coefficients_preserve_real_branch_policy() {
    let positive = atom("(177775/2-6000*sqrt(11))+(1+sqrt(2))*x");
    let result = assess(positive.clone(), "-1/2", false).unwrap();
    assert!(matches!(
        result.factors[0].certificate,
        FactorCertificate::PositiveCoefficients
    ));
    check_residual(&positive, &[symbol!("domain_sign_test::x")]).unwrap();

    let negative = -positive;
    let result = assess(negative.clone(), "-1", false).unwrap();
    assert!(matches!(
        result.factors[0].certificate,
        FactorCertificate::NegativeCoefficientsIntegerPower
    ));
    for asserted in [false, true] {
        assert!(matches!(
            assess(negative.clone(), "-1/2", asserted),
            Err(GenerationError::ComplexBranch(_))
        ));
    }
}

#[test]
fn unsupported_and_nonreal_coefficients_remain_inconclusive() {
    for polynomial in [atom("a+x"), atom("exp(1)+x"), atom("1+x") + Atom::i()] {
        assert!(matches!(
            assess(polynomial.clone(), "-1", false),
            Err(GenerationError::UnknownDomain(_))
        ));
        assert!(check_residual(&polynomial, &[symbol!("domain_sign_test::x")]).is_err());
    }
    // Even a very small exact negative coefficient must reach native sign
    // admission; statistical zero removal would invent a positive-coefficient
    // certificate. The conservative mixed-sign rule still needs an assertion.
    let tiny = atom("1-(sqrt(2)-1)^64*x");
    assert_eq!(
        uniform_sign(&tiny, &[atom("x")], &mut CoefficientSigns::default()),
        None
    );
    assert!(matches!(
        assess(tiny, "-1", false),
        Err(GenerationError::UnknownDomain(_))
    ));
}

#[test]
fn algebraic_witnesses_faces_and_residuals_keep_existing_boundary_rules() {
    for asserted in [false, true] {
        assert!(matches!(
            assess(atom("sqrt(2)*(1-2*x)"), "-1", asserted),
            Err(GenerationError::Threshold(_))
        ));
        assert!(matches!(
            assess(atom("sqrt(2)*(1-x)"), "-1", asserted),
            Err(GenerationError::UpperBoundary(_))
        ));
    }
    let mixed = atom("sqrt(2)*(1-x+x^2)");
    assert!(matches!(
        assess(mixed.clone(), "-1", false),
        Err(GenerationError::UnknownDomain(_))
    ));
    let accepted = assess(mixed.clone(), "-1", true).unwrap();
    assert!(matches!(
        accepted.factors[0].certificate,
        FactorCertificate::ExplicitInteriorAssertion
    ));
    check_residual(&mixed, &[symbol!("domain_sign_test::x")]).unwrap();
}

#[test]
fn exact_zero_and_repeated_coefficients_use_the_native_embedding() {
    let mut signs = CoefficientSigns::default();
    let positive = atom("177775/2-6000*sqrt(11)");
    assert_eq!(signs.get(&positive), Some(Ordering::Greater));
    assert_eq!(signs.get(&positive), Some(Ordering::Greater));
    assert_eq!(signs.get(&(-positive)), Some(Ordering::Less));
    let zero = atom("1/(sqrt(2)+1)-(sqrt(2)-1)");
    assert!(!zero.is_zero());
    assert_eq!(signs.get(&zero), Some(Ordering::Equal));
    assert_eq!(signs.get(&Atom::i()), None);
    assert_eq!(signs.get(&atom("exp(1)")), None);
}
