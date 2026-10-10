use super::*;

fn input(coefficient: Atom) -> Arc<ParametricIntegrand> {
    let (x, eps) = symbolica::symbol!("represented_values::x", "represented_values::eps");
    Arc::new(
        ParametricIntegrand::new(
            vec![x],
            eps,
            crate::parametric::ParametricDomain::UnitCube,
            vec![ParametricTerm::new(coefficient, vec![Atom::Zero], vec![])],
        )
        .unwrap(),
    )
}

fn convert(source: Arc<ParametricIntegrand>, limits: Limits) -> Result<ExactRepresentedInput> {
    ExactRepresentedInput::prepare(source, NumericalMeaning::RepresentedValues, limits, |_| {
        ControlFlow::Continue(())
    })
}

#[test]
fn multiprecision_complex_and_subnormal_values_keep_their_exact_representations() {
    let high = Float::parse("0.1000000000000000000000000000000000000001", Some(192)).unwrap();
    let imaginary = Float::with_val(53, -0.1f64);
    let source = input(Atom::num(Complex::new(high.clone(), imaginary.clone())));
    let prepared = convert(source.clone(), Limits::default()).unwrap();
    assert!(Arc::ptr_eq(prepared.original(), &source));
    assert_eq!(
        prepared.exact().terms()[0].prefactor(),
        &Atom::num(Complex::new(
            high.try_to_rational().unwrap(),
            imaginary.try_to_rational().unwrap()
        ))
    );
    assert!(
        prepared
            .conversions()
            .iter()
            .any(|row| row.precision_bits[0] == 192)
    );
    assert_ne!(
        imaginary.try_to_rational().unwrap(),
        Rational::from((-1, 10))
    );
    let tiny = f64::from_bits(1);
    let prepared = convert(input(Atom::num(tiny)), Limits::default()).unwrap();
    assert_eq!(
        prepared.exact().terms()[0].prefactor(),
        &Atom::num(Rational::try_from(tiny).unwrap())
    );
    assert!(!prepared.exact().terms()[0].prefactor().is_zero());
    let again = convert(prepared.exact().clone(), Limits::default()).unwrap();
    assert!(again.conversions().is_empty());
    assert_eq!(again.exact(), prepared.exact());
}

#[test]
fn coefficient_allocation_limits_and_interpretation_are_enforced_before_admission() {
    let high = Float::parse("0.1000000000000000000000000000000000000001", Some(192)).unwrap();
    for limits in [
        Limits {
            precision_bits: 64,
            ..Default::default()
        },
        Limits {
            rational_bits: 128,
            ..Default::default()
        },
        Limits {
            nodes: 0,
            ..Default::default()
        },
        Limits {
            converted_literals: 0,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            convert(input(Atom::num(high.clone())), limits),
            Err(Error::ResourceIncomplete(_))
        ));
    }
    let huge = Float::parse("1e100000", Some(64)).unwrap();
    assert!(matches!(
        convert(input(Atom::num(huge)), Limits::default()),
        Err(Error::ResourceIncomplete(
            "represented Float ratio allocation bound"
        ))
    ));
    let source = input(Atom::num(0.1f64));
    assert!(matches!(
        ExactRepresentedInput::prepare(
            source.clone(),
            NumericalMeaning::UncertaintyBounds,
            Limits::default(),
            |_| ControlFlow::Continue(())
        ),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        ExactRepresentedInput::prepare(
            source,
            NumericalMeaning::RepresentedValues,
            Limits::default(),
            |_| ControlFlow::Break(())
        ),
        Err(Error::Cancelled)
    ));
}
