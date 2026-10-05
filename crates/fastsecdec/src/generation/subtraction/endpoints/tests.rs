use super::*;
use crate::generation::{
    GenerationOptions, SubtractionStrategy, mapping::MappedTerm, subtraction::subtract,
};
use symbolica::{parse, symbol};

#[test]
fn affine_rational_endpoints_keep_the_exact_integrability_threshold() {
    let epsilon = symbol!("endpoint_admission::eps_");
    for (constant, expected) in [
        (Rational::from((-1, 2)), 0),
        (Rational::from(-1), 1),
        (Rational::from((-3, 2)), 1),
        (Rational::from(-2), 2),
        (Rational::from((-5, 2)), 2),
    ] {
        let slope = Rational::from((2, 3));
        let expression =
            Atom::num(constant.clone()) + Atom::num(slope.clone()) * Atom::var(epsilon);
        let endpoint = admit(&expression, epsilon, expected).unwrap();
        assert_eq!(endpoint.constant, constant);
        assert_eq!(endpoint.slope, slope);
        assert_eq!(endpoint.subtractions, expected);
    }
    let factored = parse!("-3/2*(1+endpoint_admission::eps_)");
    let endpoint = admit(&factored, epsilon, 1).unwrap();
    assert_eq!(endpoint.constant, Rational::from((-3, 2)));
    assert_eq!(endpoint.slope, Rational::from((-3, 2)));
    assert_eq!(endpoint.subtractions, 1);
}

#[test]
fn invalid_affine_power_precedes_limits_and_zero_slope_is_not_a_divergence_proof() {
    let epsilon = symbol!("endpoint_admission::eps_");
    for expression in [
        parse!("-2+endpoint_admission::eps_^2"),
        parse!("-2+endpoint_admission::x*endpoint_admission::eps_"),
    ] {
        assert!(matches!(
            admit(&expression, epsilon, 0),
            Err(GenerationError::EndpointExponent(value)) if value == expression
        ));
    }
    assert!(matches!(
        admit(&Atom::num(-2), epsilon, 1),
        Err(GenerationError::ResourceLimit("Taylor subtraction degree"))
    ));
    let endpoint = admit(&Atom::num(-2), epsilon, 2).unwrap();
    assert!(endpoint.slope.is_zero());
    assert_eq!(endpoint.subtractions, 2);
    assert!(matches!(
        admit(&parse!("-9223372036854775808"), epsilon, usize::MAX),
        Err(GenerationError::ResourceLimit("Taylor subtraction degree"))
    ));
}

#[test]
fn cancellation_degree_handles_empty_rows_exact_limits_and_overflow() {
    assert_eq!(checked_cancellation_degree(&[]).unwrap(), 0);
    assert_eq!(checked_cancellation_degree(&[vec![]]).unwrap(), 0);
    assert_eq!(
        checked_cancellation_degree(&[vec![1, 2], vec![0, 4]]).unwrap(),
        4
    );
    assert_eq!(
        checked_cancellation_degree(&[vec![usize::MAX, 0]]).unwrap(),
        usize::MAX
    );
    assert!(matches!(
        checked_cancellation_degree(&[vec![usize::MAX, 1]]),
        Err(GenerationError::ResourceLimit(
            "endpoint cancellation degree"
        ))
    ));
}

#[test]
fn fractional_endpoint_continuation_retains_boundary_and_remainder_density() {
    let x = symbol!("endpoint_density::x");
    let epsilon = symbol!("endpoint_density::eps");
    let expected =
        parse!("1/(endpoint_density::eps-1/2)+endpoint_density::x^(endpoint_density::eps-1/2)");
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let (density, pieces, rows) = subtract(
            vec![MappedTerm {
                powers: vec![parse!("-3/2+endpoint_density::eps")],
                prefactor: Atom::one(),
                regular: Atom::one() + Atom::var(x),
            }],
            &[x],
            epsilon,
            &GenerationOptions {
                subtraction,
                max_subtractions_per_axis: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!((density - &expected).expand().is_zero());
        assert_eq!(pieces, 2);
        assert_eq!(rows, [vec![0], vec![1]]);
    }
}

#[test]
fn physical_zero_boundaries_preserve_unregulated_pruning_and_error_precedence() {
    let x = symbol!("endpoint_pruning::x");
    let epsilon = symbol!("endpoint_pruning::eps");
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let run = |regular, maximum| {
            subtract(
                vec![MappedTerm {
                    powers: vec![Atom::num(-2)],
                    prefactor: Atom::one(),
                    regular,
                }],
                &[x],
                epsilon,
                &GenerationOptions {
                    subtraction,
                    max_subtractions_per_axis: maximum,
                    ..Default::default()
                },
            )
        };
        let (density, pieces, rows) = run(Atom::var(x).pow(Atom::num(2)), 2).unwrap();
        assert_eq!(density, Atom::one());
        assert_eq!(pieces, 1);
        assert_eq!(rows, [vec![0]]);
        let (density, pieces, rows) = run(Atom::Zero, 2).unwrap();
        assert!(density.is_zero() && pieces == 0 && rows.is_empty());
        assert!(matches!(
            run(Atom::one(), 1),
            Err(GenerationError::ResourceLimit("Taylor subtraction degree"))
        ));
        assert!(
            matches!(run(Atom::one(), 2), Err(GenerationError::UnregulatedEndpoint { parameter }) if parameter == x)
        );
    }
}
