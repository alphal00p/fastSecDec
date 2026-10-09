//! Repeated endpoint subtraction with the same nonlinear full-sector radius.
use super::*;

fn source() -> ParametricIntegrand {
    let original = super::cubic::source();
    ParametricIntegrand::new(
        original.parameters().to_vec(),
        original.regulator(),
        original.domain(),
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::num(-3) - Atom::var(original.regulator())],
            original.terms()[0].factors().to_vec(),
        )],
    )
    .unwrap()
}

#[test]
fn repeated_endpoint_reference_has_native_causal_primitive() {
    let input = source();
    let parameter = input.parameters()[0];
    let x = Atom::var(parameter);
    let original = Atom::one() / (x.pow(3) * input.terms()[0].factors()[0].polynomial());
    let endpoints = Atom::num(4) / x.pow(3) + Atom::num(16) / x.pow(2) + Atom::num(60) / &x;
    let regular = Atom::num((1024, 17)) / (Atom::num((1, 4)) - &x)
        + (Atom::num((4, 17)) * &x - Atom::num((16, 17))) / (Atom::one() + x.pow(2));
    assert!(
        (original.apart(parameter) - endpoints - &regular)
            .together()
            .cancel()
            .is_zero()
    );
    // The log identity for arctan has no cut on this real interval. The causal
    // log supplies the lower lip of the physical pole separately.
    let arctangent = ((Atom::one() + Atom::i() * &x).log() - (Atom::one() - Atom::i() * &x).log())
        / (Atom::num(2) * Atom::i());
    let primitive = -Atom::num((1024, 17))
        * crate::contour::functions::causal_log(&(Atom::num((1, 4)) - &x))
        + Atom::num((2, 17)) * (Atom::one() + x.pow(2)).log()
        - Atom::num((16, 17)) * arctangent;
    assert!(
        (primitive.derivative(parameter) - regular)
            .together()
            .cancel()
            .is_zero()
    );
    let mut evaluator = primitive
        .evaluator(&[x])
        .build()
        .unwrap()
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
    let boundary = evaluator.evaluate_single(&[Complex::new(1., 0.)])
        - evaluator.evaluate_single(&[Complex::new(0., 0.)]);
    let expected = expected_finite();
    // Integrating the first two endpoint monomials gives -4/2-16=-18.
    assert!((boundary.re - 18. - expected.re).abs() < 1e-12);
    assert!((boundary.im - expected.im).abs() < 1e-12);
}

fn expected_finite() -> Complex<f64> {
    let pi = std::f64::consts::PI;
    Complex::new(
        -18. - 1024. / 17. * 3f64.ln() + 2. / 17. * 2f64.ln() - 4. * pi / 17.,
        1024. * pi / 17.,
    )
}

#[test]
fn sign_aware_higher_endpoint_jets_preserve_pole_and_finite_part() {
    let input = source();
    let expected = expected_finite();
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let generated = generate(
                &input,
                &GenerationOptions {
                    program_recipe: ProgramRecipe::DynamicSignAwareV1,
                    mode,
                    subtraction,
                    ..Default::default()
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.orders(), &[-1, 0]);
            for sector in generated.sectors() {
                assert_eq!(sector.generation_mode(), mode);
                if let Some(deferred) = &sector.deferred {
                    assert!(
                        deferred
                            .recipe
                            .requests
                            .iter()
                            .any(|request| request.derivatives[0] >= 2)
                    );
                }
            }
            let faces = generated.metadata().charts()[0]
                .contour()
                .unwrap()
                .validation_faces();
            assert!(faces.contains(&vec![(0, 0)]));
            assert_eq!(
                faces.contains(&vec![(0, 1)]),
                subtraction == SubtractionStrategy::IntegrateByParts
            );
            let values = super::cubic::integrate(&generated, ProgramRecipe::DynamicSignAwareV1);
            assert!(
                (values[0].re + 60.).abs() < 1e-8,
                "{mode:?}/{subtraction:?}: {values:?}"
            );
            assert!(
                values[0].im.abs() < 1e-8,
                "{mode:?}/{subtraction:?}: {values:?}"
            );
            assert!(
                (values[1].re - expected.re).abs() < 2e-4,
                "{mode:?}/{subtraction:?}: {values:?}"
            );
            assert!(
                (values[1].im - expected.im).abs() < 2e-4,
                "{mode:?}/{subtraction:?}: {values:?}"
            );
        }
    }
}
