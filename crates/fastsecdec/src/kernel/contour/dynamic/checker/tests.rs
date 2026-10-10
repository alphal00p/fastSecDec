use super::*;
use crate::{
    contour::{ContourMode, DynamicConstruction, functions::dynamic},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, ProgramRecipe, compilation},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::ops::ControlFlow;
use symbolica::{
    atom::{Atom, AtomCore},
    domains::float::Complex,
    symbol,
};

struct Fixture {
    source: Arc<DynamicCheckProgram>,
    level: ExactProgram,
    strength: ExactProgram,
    owner: Arc<crate::kernel::NativeProgramDescriptor>,
}

fn fixture(construction: DynamicConstruction) -> Fixture {
    let x = symbol!("dynamic_certificate_numeric::x");
    let eps = symbol!("dynamic_certificate_numeric::eps");
    let variable = Atom::var(x);
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![-Atom::one() - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    (Atom::num((1, 4)) - &variable) * (Atom::one() + variable.pow(2)),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(
                    Atom::one() + variable.pow(2),
                    Atom::Zero,
                    FactorRole::Polynomial,
                )
                .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap();
    let recipe = match construction {
        DynamicConstruction::Polynomial => ProgramRecipe::DynamicPolynomialV1,
        DynamicConstruction::SignAware => ProgramRecipe::DynamicSignAwareV1,
    };
    let generated = generate(
        &input,
        &GenerationOptions {
            program_recipe: recipe,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let owner = generated.program_descriptor().unwrap().clone();
    let source = &generated.dynamic_check_sources()[0];
    let runtime = compilation::runtime_inputs(&generated, &[]);
    let saved = DynamicCheckProgram::build(
        source,
        &owner.charts()[0],
        generated.metadata().charts()[0].contour().unwrap(),
        &runtime,
        CompilationSettings::default(),
    )
    .unwrap();
    let level = owner
        .root_helper(&saved.structure.helper_digest)
        .unwrap()
        .exact_program()
        .clone();
    let inputs = source
        .parameters
        .iter()
        .chain(&runtime)
        .map(|symbol| Atom::var(*symbol))
        .collect::<Vec<_>>();
    let strength = source.full_strength.evaluator(&inputs).build().unwrap();
    Fixture {
        source: Arc::new(saved),
        level,
        strength,
        owner,
    }
}

fn mode(
    construction: DynamicConstruction,
    safety_fraction: f64,
    lambda_cap: f64,
    displacement_cap: f64,
) -> ContourMode {
    ContourMode::Dynamical {
        safety_fraction,
        lambda_cap,
        displacement_cap,
        construction,
    }
}
fn runtime(deformation: ContourMode) -> Vec<f64> {
    let ContourMode::Dynamical {
        safety_fraction,
        lambda_cap,
        displacement_cap,
        ..
    } = deformation
    else {
        unreachable!()
    };
    vec![safety_fraction, lambda_cap, displacement_cap]
}
fn check(fixture: &Fixture, deformation: ContourMode) -> Check {
    Check::prepare(
        fixture.source.clone(),
        fixture.level.clone(),
        runtime(deformation),
        deformation,
        Rational::from((1, 1_000_000_000)),
    )
    .unwrap()
}
fn candidate(fixture: &Fixture, x: f64, deformation: ContourMode, bits: u32) -> Rational {
    let _owner = fixture.owner.enter();
    let _plain = dynamic::requested::Mode::default().enter();
    let (value, failure) = dynamic::isolated_attempt(|| {
        dynamic::with_precision(bits, || {
            let mut evaluator = fixture.strength.clone().map_coeff_with_prec(
                &|value| {
                    Complex::new(
                        value.re.to_multi_prec_float(bits),
                        value.im.to_multi_prec_float(bits),
                    )
                },
                bits,
            );
            let input = std::iter::once(x)
                .chain(runtime(deformation))
                .map(|value| Complex::new(Float::with_val(bits, value), Float::with_val(bits, 0)))
                .collect::<Vec<_>>();
            let mut output = [Complex::new(
                Float::with_val(bits, 0),
                Float::with_val(bits, 0),
            )];
            evaluator.evaluate(&input, &mut output);
            assert!(output[0].im.is_fully_zero());
            output[0].re.to_rational()
        })
    });
    assert!(failure.is_none(), "{failure:?}");
    value
}

#[test]
fn independent_native_certificates_accept_actual_candidates_and_reject_wrong_roots() {
    for construction in [
        DynamicConstruction::Polynomial,
        DynamicConstruction::SignAware,
    ] {
        let fixture = fixture(construction);
        let deformation = mode(construction, 0.8, 1., 1.);
        let mut check = check(&fixture, deformation);
        for x in [0., 0.25, 0.5, 0.9, 1.] {
            let actual = candidate(&fixture, x, deformation, 192);
            let result = check.evaluate(&[x], &[], &actual, 192).unwrap();
            assert!(result.actual_level_upper < 1);
            assert!(matches!(
                check.evaluate(&[x], &[], &(&actual / &Rational::from(2)), 192),
                Err(RadiusFailure::Unresolved(
                    "intended-root approximation accuracy is not certified"
                ))
            ));
        }
        // These are restrictions of the full saved map and schema, not new
        // envelopes rebuilt on the zero-dimensional faces.
        for endpoint in [0, 1] {
            let actual = candidate(&fixture, f64::from(endpoint), deformation, 192);
            check
                .evaluate(&[0.37], &[(0, endpoint)], &actual, 192)
                .unwrap();
        }
    }
}

#[test]
fn certificate_binding_rejects_mismatched_caps_safety_and_recipe() {
    let fixture = fixture(DynamicConstruction::Polynomial);
    let deformation = mode(DynamicConstruction::Polynomial, 0.8, 1., 2.);
    for index in 0..3 {
        let mut values = runtime(deformation);
        values[index] *= 0.5;
        assert!(
            Check::prepare(
                fixture.source.clone(),
                fixture.level.clone(),
                values,
                deformation,
                Rational::from((1, 1000))
            )
            .is_err()
        );
    }
    let wrong = mode(DynamicConstruction::SignAware, 0.8, 1., 2.);
    assert!(
        Check::prepare(
            fixture.source.clone(),
            fixture.level.clone(),
            runtime(wrong),
            wrong,
            Rational::from((1, 1000))
        )
        .is_err()
    );
}

#[test]
fn near_unit_safety_and_extreme_caps_preserve_native_precision_and_explicit_slack() {
    for construction in [
        DynamicConstruction::Polynomial,
        DynamicConstruction::SignAware,
    ] {
        let fixture = fixture(construction);
        for (safety, cap, displacement) in [
            (f64::from_bits(1f64.to_bits() - 1), 1., 1.),
            (0.8, 1e-100, 1e100),
            (0.8, 1e100, 1e-100),
        ] {
            let deformation = mode(construction, safety, cap, displacement);
            let mut check = check(&fixture, deformation);
            let actual = candidate(&fixture, 0.37, deformation, 256);
            assert!(actual > 0);
            let proof = check.evaluate(&[0.37], &[], &actual, 256).unwrap();
            assert!(proof.actual_level_upper < 1);
            if safety > 0.99 {
                // A low-precision certificate may straddle one. It may only
                // accept if it independently proves strict causal slack.
                if let Ok(proof) = check.evaluate(&[0.37], &[], &actual, 53) {
                    assert!(proof.actual_level_upper < 1);
                }
            }
        }
    }
}
