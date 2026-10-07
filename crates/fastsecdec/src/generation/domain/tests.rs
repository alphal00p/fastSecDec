use super::*;
use symbolica::symbol;

fn atom(value: &str) -> Atom {
    Atom::parse(value, "domain_assessment_test", Default::default()).unwrap()
}

#[test]
fn all_coefficient_signs_and_branches_remain_explicitly_uncertified() {
    for polynomial in [
        atom("(177775/2-6000*sqrt(11))+(1+sqrt(2))*x"),
        atom("-1-x"),
        atom("sqrt(2)*(1-2*x)"),
        atom("sqrt(2)*(1-x)"),
        atom("1-x+x^2"),
        atom("a+x"),
        atom("exp(1)+x"),
        atom("1+x") + Atom::i(),
        atom("1-(sqrt(2)-1)^64*x"),
    ] {
        for asserted in [false, true] {
            let result = check_factors(
                &[symbol!("domain_assessment_test::x")],
                ParametricDomain::UnitCube,
                &[(
                    2,
                    3,
                    PolynomialFactor::new(
                        polynomial.clone(),
                        atom("-1/2"),
                        FactorRole::Singularity,
                    ),
                )],
                asserted,
            )
            .unwrap();
            assert_eq!(result.branch_policy(), BranchPolicy::UserResponsible);
            assert_eq!(result.caller_asserted(), asserted);
            assert!(!result.relies_on_assertion());
            assert_eq!(result.factors.len(), 1);
            let factor = &result.factors[0];
            assert_eq!(factor.polynomial(), &polynomial);
            assert_eq!(factor.exponent(), &atom("-1/2"));
            assert_eq!((factor.term_index(), factor.factor_index()), (2, 3));
            assert_eq!(
                factor.certificate(),
                FactorCertificate::UncheckedUserResponsibility
            );
        }
    }
}

#[test]
fn resolved_factor_checks_only_its_nonzero_constant_term() {
    let parameter = symbol!("domain_assessment_test::x");
    for polynomial in [
        atom("1+x"),
        atom("-1-x"),
        atom("a+x"),
        atom("exp(1)+x"),
        atom("1+x") + Atom::i(),
    ] {
        check_residual(&polynomial, &[parameter]).unwrap();
    }
    for polynomial in [Atom::Zero, atom("x"), atom("x+x^2")] {
        assert!(matches!(
            check_residual(&polynomial, &[parameter]),
            Err(GenerationError::Invariant(_))
        ));
    }
}

#[test]
fn nonnegative_polynomial_powers_need_no_singularity_record() {
    for exponent in [Atom::Zero, Atom::one(), Atom::num(2)] {
        let factor = PolynomialFactor::new(atom("1-2*x"), exponent, FactorRole::Singularity);
        assert!(!is_singular(&factor));
        let result = check_factors(
            &[symbol!("domain_assessment_test::x")],
            ParametricDomain::UnitCube,
            &[(0, 0, factor)],
            false,
        )
        .unwrap();
        assert!(result.factors().is_empty());
    }
}
