use super::*;

#[test]
fn composed_epsilon_numerators() {
    let (x, t, a, b, eps) = symbol!(
        "epsilon_bridge::x",
        "epsilon_bridge::t",
        "epsilon_bridge::a",
        "epsilon_bridge::b",
        "epsilon_bridge::eps"
    );
    let e = Atom::var(eps);
    let x_a = Atom::var(x);
    let coefficients = [
        (Atom::one() + Atom::i() * Atom::var(a) * &x_a) * (Atom::one() + &x_a).pow(3),
        Atom::var(b) * (Atom::num(2) + &x_a),
        Atom::one() + x_a.pow(2),
    ];
    let numerator = coefficients
        .iter()
        .enumerate()
        .map(|(k, c)| e.pow(k) * c)
        .sum::<Atom>();
    let term = |prefactor: Atom, numerator: Atom| {
        ParametricTerm::new(
            prefactor,
            vec![-Atom::num(2) - &e],
            vec![
                PolynomialFactor::new(
                    &x_a - Atom::var(a),
                    -Atom::one() - &e,
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(numerator, Atom::one(), FactorRole::Polynomial),
            ],
        )
    };
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![term(Atom::one(), numerator)],
    )
    .unwrap();
    let control = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        coefficients
            .iter()
            .enumerate()
            .map(|(k, c)| term(e.pow(k), c.clone()))
            .collect(),
    )
    .unwrap();
    let kinematics = GcadKinematics {
        exact_values: BTreeMap::from([(b, Rational::from((7, 3)))]),
        runtime_parameters: vec![a],
        strict_positive: vec![Atom::var(a), Atom::one() - Atom::var(a)],
    };
    let owner = solve(&input, kinematics.clone());
    let reference = solve(&control, kinematics);
    for a_value in [Rational::from((1, 4)), Rational::from((2, 3))] {
        let parameters = BTreeMap::from([(a, a_value)]);
        let fiber = RegularizedFiber::admit(
            owner.clone(),
            parameters.clone(),
            t,
            Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let baseline = threshold::regularization::RegularizedFiber::admit(
            reference.clone(),
            parameters,
            t,
            threshold::regularization::Limits::default(),
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(fiber.convergence_strip().upper(), Some(&Rational::from(-1)));
        assert_eq!(
            fiber.convergence_strip().upper(),
            baseline.convergence_strip().upper()
        );
        for strategy in [
            generation::SubtractionStrategy::Taylor,
            generation::SubtractionStrategy::IntegrateByParts,
        ] {
            let options = generation::GenerationOptions {
                max_subtractions_per_axis: 3,
                max_order: 1,
                subtraction: strategy,
                ..Default::default()
            };
            let continued = fiber
                .continue_symbolically(&options, |_| ControlFlow::Continue(()))
                .unwrap();
            let expected = baseline
                .continue_symbolically(&options, |_| ControlFlow::Continue(()))
                .unwrap();
            let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
            let reference = expected.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
            assert!(
                bound
                    .definitions()
                    .iter()
                    .all(|d| !d.body().contains_symbol(eps)
                        && !d.body().contains_symbol(a)
                        && !d.body().contains_symbol(b))
            );
            let left = bound
                .expression()
                .series(eps, 0, SeriesDepth::absolute(2))
                .unwrap();
            let right = reference
                .expression()
                .series(eps, 0, SeriesDepth::absolute(2))
                .unwrap();
            let coeffs =
                |series: &symbolica::poly::series::Series<symbolica::domains::atom::AtomField>| {
                    (-2..=1)
                        .map(|k| series.coefficient(k.into()).unwrap_or(Atom::Zero))
                        .collect::<Vec<_>>()
                };
            let mut actual = restored(&coeffs(&left), &[Atom::var(t)], bound.functions().clone());
            let mut expected = restored(
                &coeffs(&right),
                &[Atom::var(t)],
                reference.functions().clone(),
            );
            for point in [0.1, 0.37, 0.9] {
                let mut av = [Complex::new(0., 0.); 4];
                let mut ev = av;
                actual.evaluate(&[Complex::new(point, 0.)], &mut av);
                expected.evaluate(&[Complex::new(point, 0.)], &mut ev);
                for (a, e) in av.iter().zip(ev) {
                    let delta = (a.re - e.re).abs() + (a.im - e.im).abs();
                    assert!(
                        delta < 2e-9 * (1. + e.re.abs() + e.im.abs()),
                        "{av:?} != {ev:?}"
                    );
                }
            }
        }
    }
    println!(
        "PASS epsilon-polynomial complex numerator, exact+runtime bindings, common strip and native IBP/Taylor restored coefficient parity against separate native source terms"
    );
}

#[test]
fn positive_scales() {
    use threshold::regularization::meromorphic::*;
    let eps = symbol!("positive_scale_bridge::eps");
    let e = Atom::var(eps);
    let lim = meromorphic::Limits::default();
    let gamma = symbolica::transcendental::gamma();
    let original = gamma.call(e.clone()) * gamma.call(Atom::one() - Atom::num(2) * &e)
        / (gamma.call(Atom::one() + &e) * gamma.call(Atom::one() - &e).pow(2));
    let unscaled = MeromorphicPrefactor::admit(original.clone(), eps, lim).unwrap();
    let scaled =
        MeromorphicPrefactor::admit(&original * Atom::num(4).pow(e.clone()), eps, lim).unwrap();
    assert_eq!(scaled.positive_scales().len(), 1);
    assert_eq!(scaled.positive_scales()[0].base(), &Rational::from(4));
    let upper = Rational::one();
    let witness =
        MeromorphicWitness::construct(vec![unscaled.clone()], None, Some(&upper), lim, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    let scale_witness =
        MeromorphicWitness::construct(vec![scaled], None, Some(&upper), lim, |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
    assert_eq!(
        witness.rational_witness().epsilon(),
        scale_witness.rational_witness().epsilon()
    );
    let a = witness.gamma_witnesses();
    let b = scale_witness.gamma_witnesses();
    assert!(
        a.iter()
            .zip(b)
            .all(|(a, b)| a.pole_distance() == b.pole_distance())
    );
    for exponent in [
        Atom::num((1, 2)),
        Atom::num((2, 3)) + &e,
        -Atom::num(3) + Atom::num((5, 7)) * &e,
    ] {
        assert!(MeromorphicPrefactor::admit(Atom::num(2).pow(exponent), eps, lim).is_ok());
    }
    for bad in [
        Atom::num(-2).pow(e.clone()),
        Atom::var(symbol!("positive_scale_bridge::mu")).pow(e.clone()),
        Atom::num(2).pow(e.pow(2)),
        Atom::num(2).pow(e.exp()),
        Atom::num(2.5f64).pow(e.clone()),
    ] {
        assert!(
            MeromorphicPrefactor::admit(bad.clone(), eps, lim).is_err(),
            "accepted {bad}"
        );
    }
    println!(
        "PASS positive exact scale admission, unchanged common witness/Gamma pole distances and unsupported branch/source guards"
    );
}
