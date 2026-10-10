use super::*;
use crate::{
    contour::{ContourMode, DynamicConstruction},
    generation::{GenerationOptions, generate},
    kernel::{CompilationSettings, EvaluatorBackend, ProgramRecipe},
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::ExactContributionPolicy,
};
use std::ops::ControlFlow;
use symbolica::{atom::Atom, symbol};

fn fixture() -> (KernelSet, BTreeMap<Symbol, f64>, ContourSettings) {
    let x = symbol!("dynamic_pilot_binding::x");
    let input = ParametricIntegrand::new(
        vec![x],
        symbol!("dynamic_pilot_binding::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::num((1, 4)) - Atom::var(x),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            program_recipe: ProgramRecipe::DynamicPolynomialV1,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut kernels = generated
        .compile_with_settings(CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(kernels.sectors.len(), 1);
    // Reuse the actual native program in two resident owners to exercise a
    // shared source; no numerical reference or integration claim is made here.
    kernels
        .sectors
        .push(kernels.sectors[0].try_clone().unwrap());
    let settings = ContourSettings {
        deformation: ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 1.,
            displacement_cap: 1.,
            construction: DynamicConstruction::Polynomial,
        },
        validation: ContourValidationOptions {
            policy: ContourValidation::Always,
            pilot_points: 1,
        },
    };
    let point = kernels
        .runtime_parameters
        .iter()
        .enumerate()
        .map(|(index, symbol)| (*symbol, if index == 0 { 0.8 } else { 1. }))
        .collect();
    (kernels, point, settings)
}

#[test]
fn shared_source_pilot_requires_every_sector_and_cloned_readiness_is_independent() {
    let (kernels, point, settings) = fixture();
    let mut binding = DynamicBinding::new(&kernels, &point, settings).unwrap();
    let chart = binding.charts().pop().unwrap();
    assert_eq!(chart.kernel_sectors, [0, 1]);
    assert_eq!(chart.kernel_sector, None);
    assert!(!chart.includes_exact);
    let mut readiness = binding.for_sector(1).unwrap();
    assert!(readiness.validate(&[0.3]).is_err());
    // The numerical checker has separate scientific gates. These receipts
    // exercise lifecycle fencing with the actual saved request identities.
    let bundles = binding
        .sectors
        .iter()
        .map(|sector| sector.bundles.clone())
        .collect::<Vec<_>>();
    let make_receipt = |plan: &PilotPlan| {
        plan.execute(|work| match work {
            PilotWork::Sector(work) => Ok(Coverage {
                bundles: bundles[work.sector_index].clone(),
                ..Default::default()
            }),
            PilotWork::Exact(_) => panic!("fixture has no exact root"),
        })
        .unwrap()
    };
    let plan = binding.plan(chart.chart_index, &[0.3], true).unwrap();
    assert_eq!(plan.sectors.len(), 2);
    let mut missing = make_receipt(&plan);
    missing.sectors.pop();
    assert!(binding.accept(plan, missing).is_err());
    assert_eq!(binding.report().accepted_pilot_points, 0);
    let mut clone = binding.clone();
    let plan = binding.plan(chart.chart_index, &[0.3], true).unwrap();
    let receipt = make_receipt(&plan);
    assert!(clone.accept(plan, receipt).is_err());
    let plan = binding.plan(chart.chart_index, &[0.3], false).unwrap();
    let receipt = make_receipt(&plan);
    binding.accept(plan, receipt).unwrap();
    assert!(
        binding.finish().is_err(),
        "nonpreflight evaluations must not unlock the pilot"
    );
    let plan = binding.plan(chart.chart_index, &[0.3], true).unwrap();
    let receipt = make_receipt(&plan);
    binding.accept(plan, receipt).unwrap();
    assert!(binding.finish().unwrap().pilot_complete);
    readiness.validate(&[0.3]).unwrap();
    assert!(clone.require_ready(&ResultScope::FullIntegral).is_err());
    let disabled = binding
        .with_policy(
            &kernels,
            ContourValidationOptions {
                policy: ContourValidation::Off,
                pilot_points: 1,
            },
        )
        .unwrap();
    assert_eq!(disabled.report().accepted_pilot_points, 1);
    assert!(disabled.report().pilot_complete);
    assert!(disabled.for_sector(0).is_none());
    let restored = disabled
        .with_policy(
            &kernels,
            ContourValidationOptions {
                policy: ContourValidation::Pilot,
                pilot_points: 1,
            },
        )
        .unwrap();
    restored.require_ready(&ResultScope::FullIntegral).unwrap();
    assert!(
        restored
            .with_policy(
                &kernels,
                ContourValidationOptions {
                    policy: ContourValidation::Always,
                    pilot_points: 2
                }
            )
            .unwrap()
            .require_ready(&ResultScope::FullIntegral)
            .is_err()
    );
}

#[test]
fn pilot_receipt_is_bound_to_one_plan_point_and_binding() {
    let (kernels, point, settings) = fixture();
    let mut binding = DynamicBinding::new(&kernels, &point, settings).unwrap();
    let chart = binding.charts().pop().unwrap();
    let execute = |plan: &PilotPlan| {
        plan.execute(|work| match work {
            PilotWork::Sector(work) => {
                // These are state-machine controls; numerical validation is tested
                // separately. The callback receives the immutable planned point
                // and the same cached native specification used for that owner.
                assert_eq!(work.point, plan.sectors[work.sector_index].point);
                assert!(Arc::ptr_eq(
                    &work.specification,
                    &binding
                        .sector_specification(work.sector_index, true)
                        .unwrap()
                        .unwrap()
                ));
                Ok(Coverage {
                    bundles: binding.sectors[work.sector_index].bundles.clone(),
                    ..Default::default()
                })
            }
            PilotWork::Exact(_) => panic!("fixture has no exact root"),
        })
        .unwrap()
    };
    let first = binding.plan(chart.chart_index, &[0.2], true).unwrap();
    let foreign = binding.plan(chart.chart_index, &[0.8], true).unwrap();
    let foreign_receipt = execute(&foreign);
    assert!(binding.accept(first, foreign_receipt).is_err());
    assert_eq!(binding.report().accepted_pilot_points, 0);

    let plan = binding.plan(chart.chart_index, &[0.3], true).unwrap();
    let mut calls = 0;
    assert!(
        plan.execute(|_| {
            calls += 1;
            if calls == 2 {
                Err(invalid("injected second-owner failure"))
            } else {
                Ok(Coverage::default())
            }
        })
        .is_err()
    );
    assert_eq!(binding.report().accepted_pilot_points, 0);

    let execute = |plan: &PilotPlan| {
        plan.execute(|work| match work {
            PilotWork::Sector(work) => Ok(Coverage {
                bundles: binding.sectors[work.sector_index].bundles.clone(),
                ..Default::default()
            }),
            PilotWork::Exact(_) => panic!("fixture has no exact root"),
        })
        .unwrap()
    };
    let receipt = execute(&plan);
    let replay = execute(&plan);
    binding.accept(plan, receipt).unwrap();
    let next = binding.plan(chart.chart_index, &[0.3], true).unwrap();
    assert!(binding.accept(next, replay).is_err());
    assert_eq!(binding.report().accepted_pilot_points, 1);
}

#[test]
fn native_scope_helper_preserves_legacy_and_complete_associations() {
    let selected = |ids: Vec<u64>, exact_policy| ResultScope::SelectedSectors {
        sector_ids: ids,
        exact_policy,
    };
    let old: ContourValidationChart =
        serde_json::from_str(r#"{"chart_index":0,"kernel_sector":2,"dimension":1}"#).unwrap();
    assert!(old.required_by_scope(&selected(vec![2], ExactContributionPolicy::ExcludeAll)));
    assert!(!old.required_by_scope(&selected(vec![1], ExactContributionPolicy::IncludeAll)));
    let old_exact: ContourValidationChart =
        serde_json::from_str(r#"{"chart_index":1,"kernel_sector":null,"dimension":0}"#).unwrap();
    assert!(old_exact.required_by_scope(&selected(vec![], ExactContributionPolicy::IncludeAll)));
    assert!(!old_exact.required_by_scope(&selected(vec![], ExactContributionPolicy::ExcludeAll)));
    let chart = ContourValidationChart {
        chart_index: 3,
        kernel_sector: None,
        kernel_sectors: vec![1, 3],
        includes_exact: false,
        dimension: 1,
    };
    assert!(chart.required_by_scope(&selected(vec![3], ExactContributionPolicy::ExcludeAll)));
    assert!(!chart.required_by_scope(&selected(vec![], ExactContributionPolicy::IncludeAll)));
    let both = ContourValidationChart {
        includes_exact: true,
        ..chart
    };
    assert!(both.required_by_scope(&selected(vec![], ExactContributionPolicy::IncludeAll)));
}

#[test]
fn stochastic_callback_association_is_structural_even_when_validation_is_off() {
    let (kernels, point, mut settings) = fixture();
    settings.validation.policy = ContourValidation::Off;
    let binding = DynamicBinding::new(&kernels, &point, settings).unwrap();
    assert!(binding.sector_specification(0, false).unwrap().is_none());
    let callbacks = || program::callbacks(kernels.sectors[0].exact_program()).unwrap();
    assert!(!sector_requests(callbacks()).unwrap().is_empty());
    // Mutate the actual exported native callback metadata; admission must not
    // depend on constructing an optional checked numerical specification.
    let index = callbacks()
        .iter()
        .position(|callback| callback.symbol == requested::symbol())
        .unwrap();
    let mut plain = callbacks();
    plain[index].symbol = crate::contour::functions::dynamic::symbol();
    assert!(
        sector_requests(plain)
            .unwrap_err()
            .to_string()
            .contains("unassociated")
    );
    let mut surplus = callbacks();
    surplus[index].tags.push(Atom::Zero);
    assert!(sector_requests(surplus).is_err());
    let mut constant = callbacks();
    constant[index].fixed_args = Some(vec![]);
    assert!(sector_requests(constant).is_err());
    let mut foreign = binding.sectors[0].bundles.iter().next().unwrap().clone();
    foreign.0[0].namespace.push_str("_foreign");
    assert!(
        binding
            .descriptor
            .validate_request_coverage([&foreign])
            .is_err()
    );
    let mut absent_face = binding.sectors[0].bundles.iter().next().unwrap().clone();
    absent_face.0[0].face = vec![(0, 1)];
    assert!(
        binding
            .descriptor
            .validate_request_coverage([&absent_face])
            .is_err()
    );
}

#[test]
fn pilot_coverage_totals_and_source_projections_are_consistent() {
    use super::super::validation::ContextCoverage;
    let (kernels, point, settings) = fixture();
    let mut binding = DynamicBinding::new(&kernels, &point, settings).unwrap();
    let chart = binding.charts().pop().unwrap();
    for wrong_projection in [false, true] {
        let plan = binding.plan(chart.chart_index, &[0.3], true).unwrap();
        let mut receipt = plan
            .execute(|work| match work {
                PilotWork::Sector(work) => {
                    let context = &binding.contexts[0];
                    Ok(Coverage {
                        bundles: binding.sectors[work.sector_index].bundles.clone(),
                        checked_arguments: 1,
                        maximum_bits: 96,
                        contexts: vec![ContextCoverage {
                            certificate_index: context.certificate_index,
                            namespace: context.namespace.clone(),
                            chart_index: binding.descriptor.certificates().unwrap()
                                [context.certificate_index]
                                .chart_index,
                            checked_arguments: 1,
                            maximum_bits: 96,
                        }],
                    })
                }
                PilotWork::Exact(_) => unreachable!(),
            })
            .unwrap();
        if wrong_projection {
            receipt.sectors[0].1.contexts[0].chart_index = Some(usize::MAX);
        } else {
            receipt.sectors[0].1.maximum_bits = 53;
        }
        assert!(binding.accept(plan, receipt).is_err());
        assert_eq!(binding.report().accepted_pilot_points, 0);
        assert_eq!(binding.report().checked_arguments, 0);
    }
}
