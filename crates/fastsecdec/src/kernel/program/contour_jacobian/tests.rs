use super::*;
use crate::{
    contour::{
        ContourJacobian, ContourJacobianPlan, ContourMode, ContourSettings, ContourValidation,
        ContourValidationOptions, DynamicConstruction, functions::dynamic,
    },
    generation::{GenerationMode, GenerationOptions, SubtractionStrategy, generate},
    kernel::{EvaluatorBackend, KernelSet, indexed::ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
};
use std::{ops::ControlFlow, sync::Arc};
use symbolica::domains::{
    float::{Complex, Float, RealLike},
    rational::Rational,
};

fn source() -> ParametricIntegrand {
    let x = symbol!("symbolic_contour_ibp::x");
    let eps = symbol!("symbolic_contour_ibp::eps");
    let positive = Atom::one() + Atom::var(x).pow(2);
    ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one() + Atom::i(),
            vec![Atom::num(-3) - Atom::var(eps)],
            vec![
                PolynomialFactor::new(
                    (Atom::num((1, 4)) - Atom::var(x)) * &positive,
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
                PolynomialFactor::new(positive, Atom::Zero, FactorRole::Singularity)
                    .with_semantics(FactorSemantics::Positive),
            ],
        )],
    )
    .unwrap()
}

fn numeric(program: ExactProgram, recipe: ProgramRecipe) -> Vec<Vec<f64>> {
    let runtime = recipe
        .recipe_parameters()
        .iter()
        .map(|name| symbol!(*name))
        .collect::<Vec<_>>();
    let output_count = program.get_output_len();
    let mut evaluator = dynamic::with_precision(192, || {
        program.map_coeff_with_prec(
            &|c: &Complex<Rational>| {
                Complex::new(c.re.to_multi_prec_float(192), c.im.to_multi_prec_float(192))
            },
            192,
        )
    });
    [0.13, 0.37, 0.71]
        .into_iter()
        .map(|x| {
            let point = std::iter::once(x)
                .chain(runtime.iter().map(|s| {
                    if *s == crate::contour::dynamic::safety_fraction_symbol() {
                        0.8
                    } else if *s == crate::contour::dynamic::displacement_cap_symbol() {
                        1.
                    } else {
                        0.125
                    }
                }))
                .map(|v| Complex::new(Float::with_val(192, v), Float::with_val(192, 0)))
                .collect::<Vec<_>>();
            let mut values =
                vec![Complex::new(Float::with_val(192, 0), Float::with_val(192, 0)); output_count];
            let (_, failure) =
                dynamic::isolated_attempt(|| evaluator.evaluate(&point, &mut values));
            assert!(failure.is_none(), "{recipe:?}: {failure:?}");
            values
                .into_iter()
                .flat_map(|v| [v.re.to_f64(), v.im.to_f64()])
                .collect()
        })
        .collect()
}

fn bind(bytes: &[u8], recipe: ProgramRecipe) -> KernelSet {
    let mut kernels = KernelSet::from_bytes(bytes).unwrap();
    kernels
        .bind_parameters_with_contour(
            &BTreeMap::new(),
            &ContourSettings {
                deformation: if recipe == ProgramRecipe::FixedV1 {
                    ContourMode::Fixed { lambda: 0.125 }
                } else {
                    ContourMode::Dynamical {
                        safety_fraction: 0.8,
                        lambda_cap: 0.125,
                        displacement_cap: 1.,
                        construction: if recipe == ProgramRecipe::DynamicPolynomialV1 {
                            DynamicConstruction::Polynomial
                        } else {
                            DynamicConstruction::SignAware
                        },
                    }
                },
                validation: ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 1,
                },
            },
        )
        .unwrap();
    for chart in kernels.contour_validation_charts() {
        kernels
            .validate_contour_point(chart.chart_index, &[0.37], true)
            .unwrap();
    }
    kernels.finish_contour_pilot().unwrap();
    kernels
        .validate_integration_readiness(&crate::results::ResultScope::FullIntegral)
        .unwrap();
    kernels
}

#[test]
fn symbolic_ibp_uses_only_contour_first_partials_and_restores_complete_vectors() {
    for recipe in [
        ProgramRecipe::FixedV1,
        ProgramRecipe::DynamicPolynomialV1,
        ProgramRecipe::DynamicSignAwareV1,
    ] {
        for subtraction in [
            SubtractionStrategy::Taylor,
            SubtractionStrategy::IntegrateByParts,
        ] {
            let options = GenerationOptions {
                mode: GenerationMode::Symbolic,
                program_recipe: recipe,
                subtraction,
                ..Default::default()
            };
            let reference = generate(&source(), &options, |_| ControlFlow::Continue(())).unwrap();
            let generated = generate(
                &source(),
                &GenerationOptions {
                    contour_jacobian: ContourJacobian::Dual,
                    ..options
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
            assert_eq!(generated.orders(), reference.orders());
            assert_eq!(generated.sectors().len(), 1);
            assert_eq!(reference.sectors().len(), 1);
            // This fixture retains its endpoint terms in the single sector;
            // separate public tests cover surviving exact offsets.
            assert!(generated.exact_coefficients().iter().all(|v| v.is_zero()));
            assert!(reference.exact_coefficients().iter().all(|v| v.is_zero()));
            let sector = &generated.sectors()[0];
            assert_eq!(sector.generation_mode(), GenerationMode::Symbolic);
            assert!(
                sector.deferred.is_none(),
                "no endpoint/full-density Dualizer plan"
            );
            let retained = sector.symbolic_jacobian.as_ref().unwrap();
            assert!(retained.faces.contains(&vec![(0, 0)]));
            let coefficients = sector.aliased_coefficients();
            let roots = coefficients
                .iter()
                .map(|c| c.get_root().clone())
                .collect::<Vec<_>>();
            let aliases = coefficients
                .iter()
                .find(|c| !c.get_aliases().is_empty())
                .map(|c| {
                    c.get_aliases()
                        .iter()
                        .map(|(a, b)| (a.clone(), b.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let runtime = recipe
                .recipe_parameters()
                .iter()
                .map(|name| symbol!(*name))
                .collect::<Vec<_>>();
            let inputs = sector
                .parameters()
                .iter()
                .chain(&runtime)
                .copied()
                .collect::<Vec<_>>();
            let settings = CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            };
            let (exact, bindings) = {
                let _owner = generated.program_descriptor().map(|owner| owner.enter());
                build(
                    &roots,
                    &aliases,
                    &inputs,
                    sector.contour_definitions(),
                    retained,
                    None,
                    settings,
                )
                .unwrap()
            };
            assert!(
                bindings > 0,
                "must actually dualize surviving contour entries"
            );
            assert_eq!(
                exact.get_input_len(),
                inputs.len(),
                "private slots never become runtime parameters"
            );
            let actual = {
                let _owner = generated.program_descriptor().map(|owner| owner.enter());
                numeric(exact, recipe)
            };
            let expected = {
                let _owner = reference.program_descriptor().map(|owner| owner.enter());
                numeric(
                    super::super::build_sector_with_lowering(
                        &reference.sectors()[0],
                        &runtime,
                        settings,
                        None,
                    )
                    .unwrap()
                    .exact,
                    recipe,
                )
            };
            for (a, b) in actual.iter().flatten().zip(expected.iter().flatten()) {
                assert!(
                    a.is_finite() && b.is_finite() && (a - b).abs() < 2e-12 * (1. + b.abs()),
                    "{recipe:?}/{subtraction:?}: {a} vs {b}"
                );
            }
            let mut backends = vec![EvaluatorBackend::Eager];
            if cfg!(feature = "native") {
                backends.push(EvaluatorBackend::Symjit);
            }
            for backend in backends {
                let original = generated
                    .compile_with_settings(CompilationSettings {
                        backend,
                        ..Default::default()
                    })
                    .unwrap();
                assert!(
                    original.sectors()[0]
                        .statistics()
                        .symbolic_endpoint_contour_partials
                        .unwrap()
                        > 0
                );
                let bytes = original.to_bytes().unwrap();
                drop(original);
                let mut restored = bind(&bytes, recipe);
                assert_eq!(
                    restored.sectors()[0]
                        .statistics()
                        .symbolic_endpoint_contour_partials,
                    None
                );
                assert_eq!(restored.exact_coefficients(), vec![0.; expected[0].len()]);
                for (x, expected) in [0.13, 0.37, 0.71].into_iter().zip(&expected) {
                    let mut actual = vec![0.; expected.len()];
                    restored.sectors_mut()[0]
                        .evaluate(&[x], &mut actual)
                        .unwrap();
                    for (a, b) in actual.iter().zip(expected) {
                        assert!(
                            (a - b).abs() < 2e-9 * (1. + b.abs()),
                            "{recipe:?}/{subtraction:?}/{backend:?}: {a} vs {b}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn cancelled_and_unreachable_image_partials_build_no_prefix() {
    let x = symbol!("unused_contour_prefix::x");
    let unknown = symbol!("unused_contour_prefix::unregistered").call(Atom::var(x));
    let entry = unknown.derivative(x);
    let handle = Atom::var(symbol!("unused_contour_prefix::alias"));
    let retained = SymbolicContourJacobian {
        plan: Arc::new(ContourJacobianPlan {
            parameters: vec![x],
            images: vec![unknown],
            jacobian: entry.clone(),
        }),
        faces: vec![vec![], vec![(0, 0)]],
    };
    let (program, bindings) = build(
        &[&entry - &entry + Atom::one()],
        &[(handle, entry)],
        &[x],
        &ContourDefinitions::default(),
        &retained,
        None,
        CompilationSettings::default(),
    )
    .unwrap();
    assert_eq!(bindings, 0);
    assert_eq!(program.get_input_len(), 1);
    assert_eq!(program.get_output_len(), 1);
    let mut evaluator = program.map_coeff(&|r| Complex::new(r.re.to_f64(), r.im.to_f64()));
    assert_eq!(
        evaluator.evaluate_single(&[Complex::new(0.4, 0.)]),
        Complex::new(1., 0.)
    );
}

#[test]
fn native_zero_derivative_body_prunes_its_private_image_input() {
    let x = symbol!("pruned_contour_prefix::x");
    let m = symbol!("pruned_contour_prefix::m");
    let image = symbol!("pruned_contour_prefix::unregistered").call(Atom::var(x));
    let entry = image.derivative(x);
    let (definitions, calls) =
        ContourDefinitions::with_required_bodies(&[x, m], &[Atom::var(m).pow(2)], &[0]).unwrap();
    // The unresolved native function derivative retains its argument until
    // the native function-map builder differentiates the actual quadratic.
    let zero = calls[0]
        .derivative(m)
        .derivative(m)
        .derivative(m)
        .replace(Atom::var(m))
        .with(entry.clone());
    assert!(!zero.is_zero());
    assert!(definitions.materialize(&zero).unwrap().is_zero());
    let retained = SymbolicContourJacobian {
        plan: Arc::new(ContourJacobianPlan {
            parameters: vec![x],
            images: vec![image],
            jacobian: entry,
        }),
        faces: vec![vec![], vec![(0, 0)]],
    };
    let (program, active) = build(
        &[zero + Atom::one()],
        &[],
        &[x],
        &definitions,
        &retained,
        None,
        CompilationSettings::default(),
    )
    .unwrap();
    assert_eq!(
        active, 0,
        "native function-body cancellation must remove the prefix"
    );
    assert!(program.export_instructions().constant_functions.is_empty());
    let mut evaluator = program.map_coeff(&|r| Complex::new(r.re.to_f64(), r.im.to_f64()));
    assert_eq!(
        evaluator.evaluate_single(&[Complex::new(0.3, 0.)]),
        Complex::new(1., 0.)
    );
}

#[test]
fn symbolic_control_flow_does_not_eagerly_hoist_image_partials() {
    let x = symbol!("conditional_contour_prefix::x");
    let image = Atom::var(x).pow(2);
    let entry = image.derivative(x);
    let expression = Symbol::IF.call_args([Atom::var(x), entry.clone(), Atom::one()]);
    let retained = SymbolicContourJacobian {
        plan: Arc::new(ContourJacobianPlan {
            parameters: vec![x],
            images: vec![image],
            jacobian: entry,
        }),
        faces: vec![vec![]],
    };
    let error = build(
        std::slice::from_ref(&expression),
        &[],
        &[x],
        &ContourDefinitions::default(),
        &retained,
        None,
        CompilationSettings::default(),
    )
    .err()
    .unwrap();
    assert!(error.contains("control flow"));
    // The ordinary symbolic path remains valid and retains branch execution.
    let mut reference = expression
        .evaluator(&[Atom::var(x)])
        .build()
        .unwrap()
        .map_coeff(&|r| Complex::new(r.re.to_f64(), r.im.to_f64()));
    assert_eq!(
        reference.evaluate_single(&[Complex::new(0., 0.)]),
        Complex::new(1., 0.)
    );
    assert_eq!(
        reference.evaluate_single(&[Complex::new(0.25, 0.)]),
        Complex::new(0.5, 0.)
    );
}

#[test]
fn private_entry_symbols_do_not_capture_runtime_inputs_or_coordinate_order() {
    let x = symbol!("private_entry_collision::x");
    let p = symbol!("fastsecdec::private_contour_partial_0");
    let image = Atom::var(p) * Atom::var(x).pow(2);
    let entry = image.derivative(x);
    let expression = entry.sin() + Atom::var(p);
    let retained = SymbolicContourJacobian {
        plan: Arc::new(ContourJacobianPlan {
            parameters: vec![x],
            images: vec![image],
            jacobian: entry,
        }),
        faces: vec![vec![], vec![(0, 0)]],
    };
    let (program, active) = build(
        &[expression],
        &[],
        &[p, x],
        &ContourDefinitions::default(),
        &retained,
        None,
        CompilationSettings::default(),
    )
    .unwrap();
    assert_eq!(active, 1);
    assert_eq!(program.get_input_len(), 2);
    let mut evaluator = program.map_coeff(&|r| Complex::new(r.re.to_f64(), r.im.to_f64()));
    let actual = evaluator.evaluate_single(&[Complex::new(3., 0.), Complex::new(0.25, 0.)]);
    assert!((actual.re - (1.5f64.sin() + 3.)).abs() < 1e-14);
    assert_eq!(actual.im, 0.);
}
