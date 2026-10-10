//! Executable lowering receives already selected requests, before native jets.
use super::*;
use crate::{
    generation::numerical_dual::native::{self, Coordinate},
    kernel::{CompilationSettings, EvaluatorBackend, KernelError},
};
use std::collections::BTreeSet;
use symbolica::domains::{
    float::{DoubleFloat, RealLike},
    rational::Rational,
};

mod requested;

fn repeated_source() -> ParametricIntegrand {
    let input = super::cubic::source();
    ParametricIntegrand::new(
        input.parameters().to_vec(),
        input.regulator(),
        input.domain(),
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::num(-3) - Atom::var(input.regulator())],
            input.terms()[0].factors().to_vec(),
        )],
    )
    .unwrap()
}

#[test]
fn late_lowering_receives_full_faces_without_changing_mathematical_sources() {
    for subtraction in [
        SubtractionStrategy::Taylor,
        SubtractionStrategy::IntegrateByParts,
    ] {
        let generated = generate(
            &repeated_source(),
            &GenerationOptions {
                program_recipe: ProgramRecipe::DynamicSignAwareV1,
                mode: GenerationMode::NumericalDual,
                subtraction,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let runtime = ProgramRecipe::DynamicSignAwareV1
            .recipe_parameters()
            .iter()
            .map(|name| symbol!(*name))
            .collect::<Vec<_>>();
        let settings = CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        };
        let scope = generated.program_descriptor().unwrap().enter();
        for sector in generated.sectors() {
            let deferred = sector.deferred.as_ref().unwrap();
            let original = deferred.mapped_regular.clone().unwrap();
            assert!(
                deferred
                    .recipe
                    .requests
                    .iter()
                    .any(|request| request.derivatives[0] >= 2)
            );
            let expected_faces = deferred
                .recipe
                .requests
                .iter()
                .map(|request| request.coordinates.clone())
                .collect::<BTreeSet<_>>();
            let mut actual_faces = BTreeSet::new();
            let lowered =
                native::build_with_lowering(deferred, &runtime, settings, &mut |body, face| {
                    // Face restriction has not changed the physical body;
                    // native seeds must apply it after differentiation.
                    assert!(original.contains(body));
                    assert!(body.contains_symbol(sector.parameters()[0]));
                    actual_faces.insert(face.to_vec());
                    Ok(body.clone())
                })
                .unwrap();
            assert_eq!(actual_faces, expected_faces);
            assert!(actual_faces.contains(&vec![Coordinate::Variable(0)]));
            assert!(actual_faces.contains(&vec![Coordinate::Zero]));
            assert_eq!(
                actual_faces.contains(&vec![Coordinate::One]),
                subtraction == SubtractionStrategy::IntegrateByParts
            );
            assert_eq!(deferred.mapped_regular.as_ref().unwrap(), &original);
            assert!(sector.materialized.get().is_none());

            let ordinary = native::build(deferred, &runtime, settings).unwrap();
            let map = |value: &Complex<Rational>| {
                Complex::new(DoubleFloat::from(&value.re), DoubleFloat::from(&value.im))
            };
            let mut ordinary = ordinary.map_coeff(&map);
            let mut lowered = lowered.map_coeff(&map);
            let number = |value: f64| Complex::new(DoubleFloat::from(value), DoubleFloat::from(0.));
            for x in [0.01, 0.1, 0.24, 0.25, 0.26, 0.8, 0.99] {
                let point = [number(x), number(0.8), number(0.2), number(1.)];
                let mut expected = vec![number(0.); generated.orders().len()];
                let mut actual = expected.clone();
                ordinary.evaluate(&point, &mut expected);
                lowered.evaluate(&point, &mut actual);
                assert_eq!(actual, expected, "{subtraction:?} at {x}");
                assert!(actual.iter().all(|value| {
                    value.re.to_f64().is_finite() && value.im.to_f64().is_finite()
                }));
            }
            let error = native::build_with_lowering(deferred, &runtime, settings, &mut |_, _| {
                Err(KernelError::Compilation("unrecognized face".into()))
            })
            .unwrap_err();
            assert!(error.to_string().contains("unrecognized face"));
        }
        drop(scope);
    }
}

#[test]
fn opaque_undeformed_jets_do_not_invoke_contour_lowering() {
    let generated = generate(
        &super::pole(),
        &GenerationOptions {
            mode: GenerationMode::NumericalDual,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    for sector in generated.sectors() {
        let deferred = sector.deferred.as_ref().unwrap();
        assert!(deferred.mapped_regular.is_none());
        native::build_with_lowering(
            deferred,
            &[],
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
            &mut |_, _| panic!("undeformed native jets invoked contour metadata lowering"),
        )
        .unwrap();
    }
}
