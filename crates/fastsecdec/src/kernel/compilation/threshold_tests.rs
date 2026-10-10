//! Private metadata-bound v15 factory controls; indexed publication remains unsupported.
use super::*;
use crate::{generation, parametric::*, threshold};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::AtomCore,
    domains::{float::Complex, rational::Rational},
    poly::series::SeriesDepth,
    symbol,
};

fn with_bound(
    bubble: bool,
    callback: impl FnOnce(&threshold::regularization::BoundContinuation<'_>),
) {
    with_named_bound(bubble, "factory_backend_probe::t", callback)
}
fn with_named_bound(
    bubble: bool,
    unit_name: &str,
    callback: impl FnOnce(&threshold::regularization::BoundContinuation<'_>),
) {
    let (x, a, eps) = symbol!(
        "factory_backend_probe::x",
        "factory_backend_probe::a",
        "factory_backend_probe::eps"
    );
    let t = symbolica::atom::SymbolBuilder::new(symbolica::wrap_symbol!(unit_name))
        .build()
        .unwrap();
    let (f, q, numerator) = if bubble {
        (
            Atom::num(3) - Atom::num(16) * Atom::var(x) * (Atom::one() - Atom::var(x)),
            -Atom::var(eps),
            Atom::one(),
        )
    } else {
        (
            Atom::var(x) - Atom::var(a),
            -Atom::one() - Atom::var(eps),
            Atom::one() + Atom::i() * Atom::var(a) * Atom::var(x),
        )
    };
    let bodies = if unit_name.starts_with("threshold_eps_identity::") {
        vec![(
            Atom::one(),
            numerator
                + Atom::var(eps) * Atom::var(x).pow(2)
                + Atom::var(eps).pow(2) * Atom::var(x).pow(3),
        )]
    } else if unit_name.starts_with("threshold_identity::") {
        vec![
            (Atom::one(), numerator),
            (Atom::var(eps), Atom::var(x).pow(2)),
            (Atom::var(eps).pow(2), Atom::var(x).pow(3)),
        ]
    } else {
        vec![(Atom::one(), numerator)]
    };
    let terms = bodies
        .into_iter()
        .map(|(prefactor, numerator)| {
            ParametricTerm::new(
                prefactor,
                vec![Atom::Zero],
                vec![
                    PolynomialFactor::new(f.clone(), q.clone(), FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                    PolynomialFactor::new(numerator, Atom::one(), FactorRole::Polynomial),
                ],
            )
        })
        .collect();
    let input = ParametricIntegrand::new(vec![x], eps, ParametricDomain::UnitCube, terms).unwrap();
    let kinematics = if bubble {
        Default::default()
    } else {
        threshold::gcad::GcadKinematics {
            runtime_parameters: vec![a],
            strict_positive: vec![Atom::var(a), Atom::one() - Atom::var(a)],
            ..Default::default()
        }
    };
    let owner = Arc::new(
        threshold::gcad::GcadRequest::unit_cube(
            &input,
            kinematics,
            Default::default(),
            threshold::gcad::GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    let parameters = if bubble {
        BTreeMap::new()
    } else {
        BTreeMap::from([(a, Rational::from((1, 4)))])
    };
    let fiber = threshold::regularization::RegularizedFiber::admit(
        owner,
        parameters,
        t,
        Default::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_order: 1,
                max_subtractions_per_axis: 3,
                subtraction: generation::SubtractionStrategy::IntegrateByParts,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
    callback(&bound);
}

fn compile(
    bound: &threshold::regularization::BoundContinuation<'_>,
    backend: crate::kernel::EvaluatorBackend,
) -> (KernelSet, Vec<i32>) {
    let directory = tempfile::tempdir().unwrap();
    let kernels = KernelSet::compile_threshold_fiber(
        bound,
        directory.path(),
        1,
        PrecisionPolicy::default(),
        CompilationSettings {
            backend,
            ..Default::default()
        },
    )
    .unwrap();
    let mut orders = kernels.orders().to_vec();
    orders.dedup();
    assert_eq!(
        kernels.program_recipe(),
        crate::kernel::ProgramRecipe::ThresholdV1
    );
    let metadata = kernels.threshold_metadata().unwrap();
    assert!(metadata.full_original_scope().unwrap());
    assert_eq!(
        metadata.require_global_proof(),
        Err(crate::kernel::threshold_metadata::Error::ProofUnavailable)
    );
    (kernels, orders)
}

#[test]
fn prepared_native_job_and_existing_codec_preserve_complete_vectors() {
    for bubble in [false, true] {
        with_bound(bubble, |bound| {
            let mut prior: Option<Vec<Vec<f64>>> = None;
            for backend in [
                crate::kernel::EvaluatorBackend::Eager,
                crate::kernel::EvaluatorBackend::Symjit,
            ] {
                let (mut kernels, orders) = compile(bound, backend);
                assert!(kernels.runtime_parameters().is_empty());
                let bytes = kernels.to_bytes().unwrap();
                let mut restored = KernelSet::from_bytes_with_options(
                    &bytes,
                    crate::kernel::KernelLoadOptions { validate: true },
                )
                .unwrap();
                assert_eq!(restored.content_id(), kernels.content_id());
                assert_eq!(
                    restored.program_recipe(),
                    crate::kernel::ProgramRecipe::ThresholdV1
                );
                assert_eq!(
                    restored.threshold_metadata().unwrap().lineage(),
                    kernels.threshold_metadata().unwrap().lineage()
                );
                let series = bound
                    .expression()
                    .series(bound.regulators()[0], 0, SeriesDepth::absolute(2))
                    .unwrap();
                let outputs = orders
                    .iter()
                    .map(|o| series.coefficient((*o).into()).unwrap_or(Atom::Zero))
                    .collect::<Vec<_>>();
                let mut oracle =
                    Atom::evaluator_multiple(&outputs, &bound.coordinates().map(Atom::var))
                        .function_map(bound.functions().clone())
                        .optimization_settings(CompilationSettings::default().native())
                        .build()
                        .unwrap()
                        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
                let mut values = Vec::new();
                for point in [0.03125, 0.17, 0.43, 0.79, 0.96875] {
                    let mut sum = vec![0.; orders.len() * 2];
                    let mut saved_sum = sum.clone();
                    for (native, saved) in
                        kernels.sectors_mut().iter_mut().zip(restored.sectors_mut())
                    {
                        let mut a = vec![0.; sum.len()];
                        let mut b = a.clone();
                        native.evaluate(&[point], &mut a).unwrap();
                        saved.evaluate(&[point], &mut b).unwrap();
                        for i in 0..sum.len() {
                            sum[i] += a[i];
                            saved_sum[i] += b[i];
                        }
                    }
                    let mut expected = vec![Complex::new(0., 0.); orders.len()];
                    oracle.evaluate(&[Complex::new(point, 0.)], &mut expected);
                    let expected = expected
                        .into_iter()
                        .flat_map(|c| [c.re, c.im])
                        .collect::<Vec<_>>();
                    for ((actual, saved), expected) in sum.iter().zip(&saved_sum).zip(&expected) {
                        assert!((actual - expected).abs() / (1. + expected.abs()) < 2e-11);
                        assert!((actual - saved).abs() / (1. + actual.abs()) < 2e-12);
                    }
                    values.push(sum);
                }
                if let Some(prior) = &prior {
                    for (a, b) in values.iter().flatten().zip(prior.iter().flatten()) {
                        assert!((a - b).abs() / (1. + a.abs()) < 2e-12);
                    }
                }
                prior = Some(values);
            }
        });
    }
}

#[test]
fn prepared_bubble_native_kernel_qmc_matches_existing_oneloop() {
    use crate::integration::{
        CoefficientComponent::{Imag, Real},
        IntegrationProblem, Periodization, QmcSession, QmcSettings, RuleSource, SectorSpec,
    };
    with_bound(true, |bound| {
        let (mut kernels, orders) = compile(bound, crate::kernel::EvaluatorBackend::Symjit);
        assert_eq!(orders, [0, 1]);
        let problem = IntegrationProblem::new_with_components(
            "private-threshold-factory-native-codec-probe".into(),
            vec![-1, -1, 0, 0],
            vec![Real, Imag, Real, Imag],
            kernels
                .sectors()
                .iter()
                .enumerate()
                .map(|(id, k)| SectorSpec {
                    id: id as u64,
                    dimension: k.dimension(),
                })
                .collect(),
            vec![0.; 4],
        )
        .unwrap();
        let settings = QmcSettings {
            points: 16384,
            shifts: 16,
            seed: 202610106101,
            package_points: 1024,
            periodization: Periodization::Korobov3,
            rule: RuleSource::Kuo,
        };
        let mut session = QmcSession::democratic(problem, settings).unwrap();
        while let Some(task) = session.next_work().unwrap() {
            let id = task.sector_id();
            let result = session
                .worker_context(id)
                .unwrap()
                .evaluate(task, |point, out| {
                    kernels.sectors_mut()[id as usize].evaluate(point, out)
                })
                .unwrap();
            session.submit(result).unwrap();
        }
        let estimate = session.estimate().unwrap();
        let mut reference = [Complex::new(0., 0.); 3];
        oneloop::evaluate_with_backend(
            oneloop::ScalarIntegral::B0,
            &[16., 3., 3., 1.].map(|x| Complex::new(x, 0.)),
            &mut reference,
            oneloop::EvaluationBackend::Expression,
        )
        .unwrap();
        let expected = [
            reference[1].re,
            reference[1].im,
            reference[0].re,
            reference[0].im,
        ];
        for (i, expected) in expected.iter().enumerate() {
            let se = estimate.covariance_of_mean[i * 4 + i].sqrt();
            assert!(
                (estimate.mean[i] - expected).abs() <= 6. * se + 2e-10,
                "component {i}: {:?}, expected {expected}, SE {se}",
                estimate.mean
            );
        }
        println!("{}",serde_json::to_string(&serde_json::json!({"scope":"private metadata-bound threshold-v15 native backend/QMC proof; indexed publication pending",
            "mean":estimate.mean,"covariance_of_mean":estimate.covariance_of_mean,"reference":expected,"charts":kernels.sectors().len()})).unwrap());
    });
}

#[test]
fn threshold_semantic_identity_ignores_fresh_units_and_state_order() {
    let mut ids = Vec::new();
    for unit in ["threshold_identity::unit_a", "threshold_identity::unit_b"] {
        let _ = symbolica::symbol!("threshold_identity::unrelated_state_symbol");
        with_named_bound(false, unit, |bound| {
            let slots = bound
                .definitions()
                .iter()
                .filter(|d| d.derivative_order().is_none())
                .count();
            assert_eq!(slots, 3);
            assert!(
                bound
                    .definitions()
                    .iter()
                    .filter(|d| d.derivative_order().is_some())
                    .count()
                    >= 9
            );
            let (mut kernels, _) = compile(bound, crate::kernel::EvaluatorBackend::Eager);
            assert!(!kernels.contour_capable());
            assert!(
                kernels
                    .bind_parameters(&BTreeMap::from([(
                        symbol!("factory_backend_probe::a"),
                        0.3
                    )]))
                    .is_err()
            );
            assert!(
                kernels
                    .declare_program_recipe(crate::kernel::ProgramRecipe::UndeformedV1)
                    .is_err()
            );
            let cloned = kernels.try_clone().unwrap();
            assert_eq!(
                cloned.threshold_metadata().unwrap().lineage(),
                kernels.threshold_metadata().unwrap().lineage()
            );
            let manifest = crate::results::KernelResultManifest::from_kernels(&kernels);
            assert!(
                manifest
                    .threshold_scope
                    .as_ref()
                    .unwrap()
                    .is_full_original()
            );
            assert!(
                manifest
                    .canonical_scope(&crate::results::ResultScope::FullIntegral)
                    .unwrap()
                    .is_full_integral()
            );
            for stochastic_contributions in [
                vec![crate::kernel::threshold_metadata::ContributionId(0)],
                vec![],
            ] {
                let mut value = serde_json::to_value(&manifest).unwrap();
                value["threshold_scope"]["selection"] = serde_json::to_value(
                    crate::kernel::threshold_metadata::ResidentSelection::Selected {
                        stochastic_contributions,
                        exact_policy: crate::results::ExactContributionPolicy::IncludeAll,
                    },
                )
                .unwrap();
                let selected: crate::results::KernelResultManifest =
                    serde_json::from_value(value).unwrap();
                assert!(
                    !selected
                        .canonical_scope(&crate::results::ResultScope::FullIntegral)
                        .unwrap()
                        .is_full_integral()
                );
            }
            for sector_ids in [vec![0], vec![]] {
                let scope = crate::results::ResultScope::SelectedSectors {
                    sector_ids: sector_ids.clone(),
                    exact_policy: crate::results::ExactContributionPolicy::ExcludeAll,
                };
                let projected = manifest
                    .integration_problem(&scope, "threshold-selected")
                    .unwrap();
                assert_eq!(
                    projected.exact_coefficients,
                    vec![0.; kernels.components().len()]
                );
                assert_eq!(projected.components, kernels.components());
                assert_eq!(projected.sectors.len(), sector_ids.len());
                assert!(!manifest.canonical_scope(&scope).unwrap().is_full_integral());
            }
            let mut writer = std::io::Cursor::new(Vec::new());
            assert!(crate::kernel::indexed::write_unit(&mut writer, &kernels, vec![]).is_err());
            ids.push(kernels.content_id().to_owned());
        });
    }
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn threshold_complete_pole_and_zero_requested_layout_are_distinct() {
    let (x, t, eps) = symbol!(
        "threshold_exact::x",
        "threshold_exact::t",
        "threshold_exact::eps"
    );
    let input = ParametricIntegrand::new(
        vec![x],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::var(x),
                    -Atom::one() - Atom::var(eps),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    let owner = Arc::new(
        threshold::gcad::GcadRequest::unit_cube(
            &input,
            Default::default(),
            Default::default(),
            threshold::gcad::GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    let fiber = threshold::regularization::RegularizedFiber::admit(
        owner,
        BTreeMap::new(),
        t,
        Default::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_order: 0,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();
    for maximum in [-1, -2] {
        let staging = tempfile::tempdir().unwrap();
        let kernels = KernelSet::compile_threshold_fiber(
            &bound,
            staging.path(),
            maximum,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        if maximum == -2 {
            assert!(kernels.sectors().is_empty());
            assert_eq!(kernels.exact_coefficients(), [0., 0.]);
        } else {
            // Conservative unused alias bodies may retain a constant numerical
            // program. Storage partition is not a mathematical assumption.
            let mut sample = kernels.try_clone().unwrap();
            let mut total = sample.exact_coefficients().to_vec();
            for sector in sample.sectors_mut() {
                let mut value = vec![0.; 2];
                sector.evaluate(&[0.37], &mut value).unwrap();
                for (t, v) in total.iter_mut().zip(value) {
                    *t += v
                }
            }
            assert!((total[0] + 1.).abs() < 2e-12 && total[1].abs() < 2e-12);
            let coefficients = generation::threshold_expand_vector(
                &bound.chart_expressions()[0],
                &bound.coordinates(),
                eps,
                maximum,
            )
            .unwrap();
            for (order, c) in coefficients {
                println!(
                    "exact-fold diagnostic order={order}: root_has_coordinate={}, any_alias_has_coordinate={}, aliases={}",
                    c.get_root().contains_symbol(t),
                    c.get_aliases().values().any(|a| a.contains_symbol(t)),
                    c.get_aliases().len()
                );
            }
        }
        let restored = KernelSet::from_bytes_with_options(
            &kernels.to_bytes().unwrap(),
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(
            restored.threshold_metadata().unwrap().coefficient_orders(),
            [maximum]
        );
        assert_eq!(restored.exact_coefficients(), kernels.exact_coefficients());
        let kinds = &restored
            .threshold_metadata()
            .unwrap()
            .lineage()
            .contributions;
        if maximum == -2 {
            assert!(kinds.iter().all(|c| matches!(
                c.kind,
                crate::kernel::threshold_metadata::ContributionKind::CertifiedZero { .. }
            )));
        }
        assert!(kinds.iter().any(|c| matches!(
            c.kind,
            crate::kernel::threshold_metadata::ContributionKind::CertifiedZero { .. }
        )));
    }
}

#[test]
fn threshold_fresh_restore_child() {
    let Ok(path) = std::env::var("FSD_THRESHOLD_FRESH_ARTIFACT") else {
        return;
    };
    let kernels = KernelSet::from_bytes_with_options(
        &std::fs::read(path).unwrap(),
        crate::kernel::KernelLoadOptions { validate: true },
    )
    .unwrap();
    assert_eq!(
        kernels.program_recipe(),
        crate::kernel::ProgramRecipe::ThresholdV1
    );
    assert!(
        kernels
            .threshold_metadata()
            .unwrap()
            .full_original_scope()
            .unwrap()
    );
    assert_eq!(
        kernels.content_id(),
        std::env::var("FSD_THRESHOLD_FRESH_ID").unwrap()
    );
    assert_eq!(kernels.sectors().len(), 6);
}
#[test]
fn threshold_native_state_restores_in_fresh_process() {
    with_bound(true, |bound| {
        let (kernels, _) = compile(bound, crate::kernel::EvaluatorBackend::Symjit);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("threshold.fsd");
        std::fs::write(&path, kernels.to_bytes().unwrap()).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "kernel::compilation::threshold_tests::threshold_fresh_restore_child",
                "--test-threads=1",
            ])
            .env("FSD_THRESHOLD_FRESH_ARTIFACT", path)
            .env("FSD_THRESHOLD_FRESH_ID", kernels.content_id())
            .status()
            .unwrap();
        assert!(status.success());
    });
}

#[test]
fn threshold_v15_native_graph_gamma_projective_oneloop() {
    threshold_native_graph_gamma_projective_oneloop(false);
}
#[test]
fn threshold_indexed_native_graph_gamma_projective_oneloop() {
    threshold_native_graph_gamma_projective_oneloop(true);
}
fn threshold_native_graph_gamma_projective_oneloop(indexed: bool) {
    use crate::integration::{Periodization, QmcSession, QmcSettings, RuleSource};
    let (x, t, eps) = symbol!(
        "regular_bubble_qmc::x",
        "regular_bubble_qmc::t",
        "regular_bubble_qmc::eps"
    );
    let y = symbol!("regular_bubble_qmc::y");
    let model = Arc::new(
        crate::Model::from_json(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/models/scalar.json"
        )))
        .unwrap(),
    );
    let kinematics = crate::Kinematics::in_dimension(&symbolica::parse!("D"))
        .unwrap()
        .with_mass_squared(
            &feynkit_graph::symbols::external_momentum().call(1),
            Atom::num(16),
        )
        .unwrap();
    let graph = crate::input::GraphIntegral::from_dot(
        model,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/graphs/bubble.dot"
        )),
        &kinematics,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::num(3).sqrt())]))
    .unwrap();
    let gamma = symbolica::transcendental::gamma();
    let multiplier = gamma.call(Atom::one() - Atom::num(2) * Atom::var(eps))
        / (gamma.call(Atom::one() + Atom::var(eps))
            * gamma.call(Atom::one() - Atom::var(eps)).pow(2));
    let input = ParametricIntegrand::from_graph(
        &graph.with_measure_multiplier(multiplier),
        vec![x, y],
        eps,
        Atom::num(4) - Atom::num(2) * Atom::var(eps),
    )
    .unwrap();
    let preparation =
        threshold::projective::AffineProjectivePreparation::last_coordinate(&input).unwrap();
    let owner = Arc::new(
        threshold::gcad::GcadRequest::projective(
            preparation,
            threshold::gcad::GcadKinematics::default(),
            Default::default(),
            threshold::gcad::GcadRequest::default_limits(),
        )
        .unwrap()
        .solve_verified()
        .unwrap(),
    );
    assert_eq!(owner.cells().len(), 3);
    let factors = owner.request().prepared_terms()[0].factors();
    let causal = factors
        .iter()
        .filter(|f| f.semantics() == FactorSemantics::Causal)
        .collect::<Vec<_>>();
    assert_eq!(causal.len(), 1);
    assert!(
        (causal[0].polynomial()
            - (Atom::num(3) - Atom::num(16) * Atom::var(x) * (Atom::one() - Atom::var(x))))
        .expand()
        .is_zero()
    );
    eprintln!(
        "actual native graph prefactor: {}",
        owner.request().prepared_terms()[0].prefactor()
    );
    let fiber = threshold::regularization::RegularizedFiber::admit(
        owner,
        BTreeMap::new(),
        t,
        Default::default(),
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    assert_eq!(fiber.charts().len(), 6);
    let continued = fiber
        .continue_symbolically(
            &generation::GenerationOptions {
                max_subtractions_per_axis: 2,
                max_order: 1,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
    let bound = continued.bind_fiber(|_| ControlFlow::Continue(())).unwrap();

    let staging = tempfile::tempdir().unwrap();
    let bytes = if indexed {
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            &bound,
            staging.path(),
            0,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        threshold_plan_archive(&plan).0
    } else {
        KernelSet::compile_threshold_fiber(
            &bound,
            staging.path(),
            0,
            Default::default(),
            Default::default(),
        )
        .unwrap()
        .to_bytes()
        .unwrap()
    };
    let mut kernels = KernelSet::from_bytes_with_options(
        &bytes,
        crate::kernel::KernelLoadOptions { validate: true },
    )
    .unwrap();
    let lineage = kernels.threshold_metadata().unwrap().lineage();
    assert_eq!(lineage.preparation.original_coordinates.len(), 2);
    assert_eq!(lineage.patches[0].map.geometry().images.len(), 2);
    assert_eq!(lineage.patches[0].map.geometry().coordinates.len(), 1);
    let problem = crate::results::KernelResultManifest::integration_problem_from_kernels(
        &kernels,
        &crate::results::ResultScope::FullIntegral,
        "threshold-v15-native-graph-bubble",
    )
    .unwrap();
    assert_eq!(kernels.orders(), [-1, -1, 0, 0]);
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 16384,
            shifts: 16,
            seed: 202610106103,
            package_points: 1024,
            periodization: Periodization::Korobov3,
            rule: RuleSource::Kuo,
        },
    )
    .unwrap();
    while let Some(task) = session.next_work().unwrap() {
        let id = task.sector_id();
        let result = session
            .worker_context(id)
            .unwrap()
            .evaluate(task, |p, o| {
                kernels.sectors_mut()[id as usize].evaluate(p, o)
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    let mut reference = [Complex::new(0., 0.); 3];
    oneloop::evaluate_with_backend(
        oneloop::ScalarIntegral::B0,
        &[16., 3., 3., 1.].map(|x| Complex::new(x, 0.)),
        &mut reference,
        oneloop::EvaluationBackend::Expression,
    )
    .unwrap();
    let expected = [
        reference[1].re,
        reference[1].im,
        reference[0].re,
        reference[0].im,
    ];
    for (i, value) in expected.iter().enumerate() {
        assert!(
            (estimate.mean[i] - value).abs()
                <= 6. * estimate.covariance_of_mean[i * 4 + i].sqrt() + 2e-10
        );
    }
    println!(
        "{}",
        serde_json::json!({"scope":"native HEPKit graph Gamma/projective, metadata-bound restored KernelSet, ordinary QmcSession", "indexed":indexed,"mean":estimate.mean,"covariance":estimate.covariance_of_mean,"reference":expected})
    );
}

#[test]
fn threshold_factory_refuses_unavailable_coefficients_and_invalid_precision_before_staging() {
    with_bound(false, |bound| {
        let staging = tempfile::tempdir().unwrap();
        assert!(
            KernelSet::compile_threshold_fiber(
                bound,
                staging.path(),
                bound.generation_options().max_order + 1,
                Default::default(),
                Default::default()
            )
            .is_err()
        );
        assert!(staging.path().read_dir().unwrap().next().is_none());
        let precision = PrecisionPolicy {
            initial_bits: 8,
            ..Default::default()
        };
        assert!(
            KernelSet::compile_threshold_fiber(
                bound,
                staging.path(),
                0,
                precision,
                Default::default()
            )
            .is_err()
        );
        assert!(staging.path().read_dir().unwrap().next().is_none());
    });
}

#[test]
fn threshold_epsilon_numerator_slots_have_registration_independent_identity() {
    // Register unit symbols in the opposite order from construction. Each owner
    // allocates a distinct native numerator head; physical source names stay fixed.
    let _ = symbol!(
        "threshold_eps_identity::unit_b",
        "threshold_eps_identity::unit_a"
    );
    let mut ids = Vec::new();
    for name in [
        "threshold_eps_identity::unit_a",
        "threshold_eps_identity::unit_b",
    ] {
        with_named_bound(false, name, |bound| {
            assert_eq!(
                bound
                    .definitions()
                    .iter()
                    .filter(|d| d.derivative_order().is_none())
                    .count(),
                3
            );
            let (kernels, _) = compile(bound, crate::kernel::EvaluatorBackend::Eager);
            ids.push(kernels.content_id().to_owned());
        });
    }
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn threshold_detached_plan_releases_global_owner_and_preserves_v15_vectors() {
    for backend in [
        crate::kernel::EvaluatorBackend::Eager,
        crate::kernel::EvaluatorBackend::Symjit,
    ] {
        let staging = tempfile::tempdir().unwrap();
        let mut detached = None;
        with_bound(false, |bound| {
            let old = compile(bound, backend).0;
            let weak = Arc::downgrade(bound.certificate().decomposition());
            let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
                bound,
                staging.path(),
                1,
                Default::default(),
                CompilationSettings {
                    backend,
                    ..Default::default()
                },
            )
            .unwrap();
            detached = Some((weak, plan, old));
        });
        let (weak, plan, mut old) = detached.unwrap();
        assert!(
            weak.upgrade().is_none(),
            "detached job plan retained global CAD/continuation"
        );
        let mut completions = Vec::new();
        let mut partial_tested = false;
        for index in (0..plan.job_count()).rev() {
            let completion = plan.job(index).unwrap().run().unwrap();
            plan.validate_completion(&completion).unwrap();
            let kernels = completion.kernels();
            assert!(
                !kernels
                    .threshold_metadata()
                    .unwrap()
                    .full_original_scope()
                    .unwrap()
            );
            let bytes = kernels.to_bytes().unwrap();
            if backend == crate::kernel::EvaluatorBackend::Eager {
                assert!(bytes.starts_with(b"FastSecDec\0binserde\x10"));
            }
            let restored = KernelSet::from_bytes_with_options(
                &bytes,
                crate::kernel::KernelLoadOptions { validate: true },
            )
            .unwrap();
            assert_eq!(restored.content_id(), kernels.content_id());
            assert_eq!(
                restored.threshold_metadata().unwrap().resident(),
                kernels.threshold_metadata().unwrap().resident()
            );
            if !kernels.sectors().is_empty() {
                let metadata = restored.threshold_metadata().unwrap();
                let missing = metadata
                    .lineage()
                    .endpoint_charts
                    .iter()
                    .filter(|chart| {
                        let g = chart.map.geometry();
                        metadata
                            .native_atom(&g.expressions, g.positive_measure)
                            .is_err()
                    })
                    .count();
                assert!(
                    missing > 0,
                    "selected restore imported unrelated native map tables"
                );
                partial_tested = true;
            }
            completions.push(completion);
        }
        assert!(partial_tested);
        let mut assembled = plan.assemble(completions).unwrap();
        assert!(
            assembled
                .threshold_metadata()
                .unwrap()
                .full_original_scope()
                .unwrap()
        );
        assert_eq!(
            assembled.content_id(),
            old.content_id(),
            "mathematical identity depends on local native table grouping"
        );
        assert_eq!(assembled.orders(), old.orders());
        for point in [0.071, 0.317, 0.829] {
            let evaluate = |kernels: &mut KernelSet| {
                let mut values = kernels.exact_coefficients().to_vec();
                for sector in kernels.sectors_mut() {
                    let mut local = vec![0.; values.len()];
                    sector.evaluate(&[point], &mut local).unwrap();
                    for (sum, value) in values.iter_mut().zip(local) {
                        *sum += value;
                    }
                }
                values
            };
            for (actual, expected) in evaluate(&mut assembled).iter().zip(evaluate(&mut old)) {
                assert!((actual - expected).abs() / (1. + expected.abs()) < 2e-12);
            }
        }
        let bytes = assembled.to_bytes().unwrap();
        let restored = KernelSet::from_bytes_with_options(
            &bytes,
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(restored.content_id(), assembled.content_id());
        assert!(
            restored
                .threshold_metadata()
                .unwrap()
                .full_original_scope()
                .unwrap()
        );
    }
}

#[test]
fn threshold_detached_plan_refuses_missing_duplicate_and_foreign_work() {
    with_bound(true, |bound| {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let settings = CompilationSettings {
            backend: crate::kernel::EvaluatorBackend::Eager,
            ..Default::default()
        };
        let a = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            first.path(),
            1,
            Default::default(),
            settings,
        )
        .unwrap();
        let b = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            second.path(),
            1,
            Default::default(),
            settings,
        )
        .unwrap();
        assert!(a.assemble(vec![]).is_err());
        assert!(
            a.validate_completion(&b.job(0).unwrap().run().unwrap())
                .is_err()
        );
        assert!(
            a.assemble(vec![
                a.job(0).unwrap().run().unwrap(),
                a.job(0).unwrap().run().unwrap()
            ])
            .is_err()
        );
        assert!(a.job(a.job_count()).is_err());
        assert!(matches!(
            a.job(0).unwrap().kind(),
            super::threshold_plan::ThresholdJobKind::Setup {}
        ));
    });
}

fn threshold_plan_archive(
    plan: &super::threshold_plan::ThresholdCompilationPlan,
) -> (Vec<u8>, crate::kernel::indexed::ProgramArchiveCatalogue) {
    let mut writer = plan
        .archive_writer(std::io::Cursor::new(Vec::new()))
        .unwrap();
    for index in (0..plan.job_count()).rev() {
        let completed = plan.job(index).unwrap().run().unwrap();
        let mut bytes = Vec::new();
        let receipt = completed.write_record(&mut bytes).unwrap();
        if !matches!(
            completed.kind(),
            super::threshold_plan::ThresholdJobKind::Stochastic { .. }
        ) {
            assert!(
                KernelSet::from_bytes_with_options(
                    &bytes,
                    crate::kernel::KernelLoadOptions { validate: true }
                )
                .is_ok()
            );
        } else {
            assert!(
                KernelSet::from_bytes_with_options(
                    &bytes,
                    crate::kernel::KernelLoadOptions { validate: true }
                )
                .is_err(),
                "reference-only native record decoded without its parent"
            );
        }
        writer
            .append_record(
                crate::kernel::ProgramRecipe::ThresholdV1,
                &mut bytes.as_slice(),
                receipt,
            )
            .unwrap();
    }
    let (bytes, catalogue) = writer.finish().unwrap();
    (bytes.into_inner(), catalogue)
}

#[test]
fn threshold_indexed_native_parent_selected_scope_cache_and_complete_identity() {
    for backend in [
        crate::kernel::EvaluatorBackend::Eager,
        crate::kernel::EvaluatorBackend::Symjit,
    ] {
        with_bound(false, |bound| {
            let staging = tempfile::tempdir().unwrap();
            let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
                bound,
                staging.path(),
                1,
                Default::default(),
                CompilationSettings {
                    backend,
                    ..Default::default()
                },
            )
            .unwrap();
            let (bytes, catalogue) = threshold_plan_archive(&plan);
            assert_eq!(catalogue.version, 3);
            assert!(bytes.starts_with(b"FastSecDec\0indexed\x03"));
            let recipe = crate::kernel::ProgramRecipe::ThresholdV1;
            assert!(
                catalogue
                    .recipe(recipe)
                    .unwrap()
                    .threshold_scope()
                    .unwrap()
                    .unwrap()
                    .is_full_original()
            );
            let mut missing = catalogue.recipe(recipe).unwrap().clone();
            missing.records.pop();
            assert!(missing.threshold_scope().is_err());
            let mut reader = crate::kernel::indexed::ProgramArchiveReader::from_reader(
                std::io::Cursor::new(bytes.clone()),
                crate::kernel::KernelLoadOptions { validate: true },
            )
            .unwrap();
            let mut selected = reader.select(recipe).unwrap();
            let mut sector = selected.load_sector(0).unwrap();
            assert!(
                !sector
                    .threshold_metadata()
                    .unwrap()
                    .full_original_scope()
                    .unwrap()
            );
            let mut restored = KernelSet::from_bytes_with_options(
                &sector.to_bytes().unwrap(),
                crate::kernel::KernelLoadOptions { validate: true },
            )
            .unwrap();
            assert_eq!(restored.content_id(), sector.content_id());
            let mut a = vec![0.; sector.orders().len()];
            let mut b = a.clone();
            sector.sectors_mut()[0].evaluate(&[0.317], &mut a).unwrap();
            restored.sectors_mut()[0]
                .evaluate(&[0.317], &mut b)
                .unwrap();
            assert_eq!(a, b);
            let exact = selected.load_exact().unwrap();
            assert!(exact.sectors().is_empty());
            assert!(
                !exact
                    .threshold_metadata()
                    .unwrap()
                    .full_original_scope()
                    .unwrap()
            );
            let exact_saved = KernelSet::from_bytes(&exact.to_bytes().unwrap()).unwrap();
            assert_eq!(exact_saved.content_id(), exact.content_id());
            let mut all = selected.load_all().unwrap();
            assert!(
                all.threshold_metadata()
                    .unwrap()
                    .full_original_scope()
                    .unwrap()
            );
            let mut resident = compile(bound, backend).0;
            assert_eq!(all.content_id(), resident.content_id());
            assert_eq!(
                all.content_id(),
                catalogue.recipe(recipe).unwrap().content_id
            );
            let all_saved = KernelSet::from_bytes_with_options(
                &all.to_bytes().unwrap(),
                crate::kernel::KernelLoadOptions { validate: true },
            )
            .unwrap();
            assert_eq!(all_saved.content_id(), all.content_id());
            let components = all.orders().len();
            for (a, b) in all.sectors_mut().iter_mut().zip(resident.sectors_mut()) {
                let mut x = vec![0.; components];
                let mut y = x.clone();
                a.evaluate(&[0.293], &mut x).unwrap();
                b.evaluate(&[0.293], &mut y).unwrap();
                assert_eq!(x, y);
            }
            let (cached, cached_catalogue) = reader
                .write_with_native_cache(std::io::Cursor::new(Vec::new()), |_| {
                    ControlFlow::Continue(())
                })
                .unwrap();
            assert_eq!(cached_catalogue.content_id, catalogue.content_id);
            assert_eq!(
                KernelSet::from_bytes_with_options(
                    &cached.into_inner(),
                    crate::kernel::KernelLoadOptions { validate: true }
                )
                .unwrap()
                .content_id(),
                all.content_id()
            );
        });
    }
}

#[test]
fn threshold_indexed_zero_layout_and_incomplete_archive_are_distinct() {
    with_bound(true, |bound| {
        let staging = tempfile::tempdir().unwrap();
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            staging.path(),
            -2,
            Default::default(),
            CompilationSettings {
                backend: crate::kernel::EvaluatorBackend::Eager,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(plan.job_count(), 1);
        assert!(
            plan.archive_writer(std::io::Cursor::new(Vec::new()))
                .unwrap()
                .finish()
                .is_err()
        );
        let (bytes, _) = threshold_plan_archive(&plan);
        let restored = KernelSet::from_bytes_with_options(
            &bytes,
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert!(restored.sectors().is_empty());
        assert_eq!(restored.exact_coefficients(), [0., 0.]);
        assert!(
            restored
                .threshold_metadata()
                .unwrap()
                .full_original_scope()
                .unwrap()
        );
        assert!(
            restored
                .threshold_metadata()
                .unwrap()
                .lineage()
                .contributions
                .iter()
                .all(|c| matches!(
                    c.kind,
                    crate::kernel::threshold_metadata::ContributionKind::CertifiedZero { .. }
                ))
        );
    });
}

#[test]
fn threshold_indexed_historical_v15_projection_is_not_migrated_on_restore() {
    with_bound(false, |bound| {
        let (mut kernels, orders) = compile(bound, crate::kernel::EvaluatorBackend::Eager);
        let current_id = kernels.content_id().to_owned();
        let staging = tempfile::tempdir().unwrap();
        let request = bound.certificate().decomposition().request();
        let source = generation::source_identity(
            request.input(),
            &request.kinematics().runtime_parameters,
            &[],
        )
        .unwrap();
        let records = (0..bound.chart_expressions().len())
            .map(|chart| {
                crate::threshold::records::write(staging.path(), &source, bound, chart, 1).unwrap()
            })
            .collect::<Vec<_>>();
        let historical = crate::kernel::ThresholdMetadata::from_bound_historical_projection(
            bound, &records, orders,
        )
        .unwrap();
        kernels.threshold = Some(Arc::new(historical));
        kernels.initialize_artifact().unwrap();
        let historical_id = kernels.content_id().to_owned();
        assert_ne!(
            historical_id, current_id,
            "a new projection must have a versioned identity"
        );
        let bytes = kernels.to_bytes().unwrap();
        assert!(bytes.starts_with(b"FastSecDec\0binserde\x0f"));
        let restored = KernelSet::from_bytes_with_options(
            &bytes,
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(restored.content_id(), historical_id);
        assert_eq!(restored.to_bytes().unwrap(), bytes);
        let saved_again = KernelSet::from_bytes_with_options(
            &restored.to_bytes().unwrap(),
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(saved_again.content_id(), historical_id);
    });
}

#[test]
fn threshold_indexed_reads_only_carrier_and_requested_program() {
    use std::io::{Read, Seek};
    struct CountingReader {
        bytes: std::io::Cursor<Vec<u8>>,
        reads: Arc<std::sync::Mutex<Vec<(u64, u64)>>>,
    }
    impl Read for CountingReader {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let start = self.bytes.position();
            let count = self.bytes.read(output)?;
            if count != 0 {
                self.reads
                    .lock()
                    .unwrap()
                    .push((start, start + count as u64));
            }
            Ok(count)
        }
    }
    impl Seek for CountingReader {
        fn seek(&mut self, from: std::io::SeekFrom) -> std::io::Result<u64> {
            self.bytes.seek(from)
        }
    }
    with_bound(true, |bound| {
        let staging = tempfile::tempdir().unwrap();
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            staging.path(),
            1,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        let (bytes, catalogue) = threshold_plan_archive(&plan);
        let recipe = crate::kernel::ProgramRecipe::ThresholdV1;
        let records = &catalogue.recipe(recipe).unwrap().records;
        let carrier = &records[0];
        let chosen = catalogue.recipe(recipe).unwrap().sector(2).unwrap();
        let allowed = [
            (carrier.offset, carrier.offset + carrier.receipt.length),
            (chosen.offset, chosen.offset + chosen.receipt.length),
        ];
        let reads = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut reader = crate::kernel::indexed::ProgramArchiveReader::from_reader(
            CountingReader {
                bytes: std::io::Cursor::new(bytes),
                reads: reads.clone(),
            },
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        // Footer inspection is metadata-only. Measure subsequent native reads.
        reads.lock().unwrap().clear();
        let selected = reader.select(recipe).unwrap().load_sector(2).unwrap();
        assert_eq!(selected.sectors().len(), 1);
        let actual = reads.lock().unwrap();
        assert!(!actual.is_empty());
        assert!(
            actual
                .iter()
                .all(|(lo, hi)| allowed.iter().any(|(a, b)| a <= lo && hi <= b)),
            "an unrelated native record was read: {actual:?}"
        );
        assert_eq!(
            actual.iter().map(|(a, b)| b - a).sum::<u64>(),
            carrier.receipt.length + chosen.receipt.length
        );
    });
}

#[test]
fn threshold_indexed_native_state_restores_in_fresh_process() {
    with_bound(true, |bound| {
        let staging = tempfile::tempdir().unwrap();
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            staging.path(),
            1,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        let (bytes, catalogue) = threshold_plan_archive(&plan);
        let path = staging.path().join("indexed.fsd");
        std::fs::write(&path, bytes).unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "kernel::compilation::threshold_tests::threshold_fresh_restore_child",
                "--test-threads=1",
            ])
            .env("FSD_THRESHOLD_FRESH_ARTIFACT", path)
            .env(
                "FSD_THRESHOLD_FRESH_ID",
                &catalogue
                    .recipe(crate::kernel::ProgramRecipe::ThresholdV1)
                    .unwrap()
                    .content_id,
            )
            .status()
            .unwrap();
        assert!(status.success());
    });
}

#[test]
fn threshold_serial_worker_child() {
    let Ok(path) = std::env::var("FSD_THRESHOLD_WORK") else {
        return;
    };
    let work: crate::kernel::ThresholdCompilationWork =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let root = std::path::Path::new(&path).parent().unwrap();
    let mut output = std::fs::File::create(root.join("worker.fsd")).unwrap();
    let receipt = work
        .compile_record(root, 16 * 1024 * 1024, &mut output)
        .unwrap();
    std::fs::write(
        root.join("worker-receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
}

#[test]
fn threshold_serial_fresh_process_records_and_receipt_admission() {
    with_bound(false, |bound| {
        let staging = tempfile::tempdir().unwrap();
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            staging.path(),
            1,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        let mut archive = plan
            .archive_writer(std::io::Cursor::new(Vec::new()))
            .unwrap();
        for index in (0..plan.job_count()).rev() {
            let work = plan.work(index).unwrap();
            let path = staging.path().join("work.json");
            std::fs::write(&path, serde_json::to_vec(&work).unwrap()).unwrap();
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "kernel::compilation::threshold_tests::threshold_serial_worker_child",
                    "--test-threads=1",
                ])
                .env("FSD_THRESHOLD_WORK", &path)
                .status()
                .unwrap();
            assert!(status.success());
            let receipt: crate::kernel::ThresholdWorkReceipt = serde_json::from_slice(
                &std::fs::read(staging.path().join("worker-receipt.json")).unwrap(),
            )
            .unwrap();
            plan.validate_work_receipt(&receipt).unwrap();
            // Admission does not consume a receipt. A caller can retain it for
            // retry while this same native preparation authority is live.
            plan.validate_work_receipt(&receipt).unwrap();
            plan.append_work_record(
                &mut archive,
                &mut std::fs::File::open(staging.path().join("worker.fsd")).unwrap(),
                receipt,
            )
            .unwrap();
        }
        let (bytes, catalogue) = archive.finish().unwrap();
        let restored = KernelSet::from_bytes_with_options(
            &bytes.into_inner(),
            crate::kernel::KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(
            restored.content_id(),
            catalogue
                .recipe(crate::kernel::ProgramRecipe::ThresholdV1)
                .unwrap()
                .content_id
        );
        assert_eq!(
            restored.content_id(),
            compile(bound, crate::kernel::EvaluatorBackend::Auto)
                .0
                .content_id()
        );
        assert!(
            restored
                .threshold_metadata()
                .unwrap()
                .full_original_scope()
                .unwrap()
        );
    });
}

#[test]
fn threshold_serial_refuses_changed_plan_source_and_program_bytes() {
    with_bound(false, |bound| {
        let staging = tempfile::tempdir().unwrap();
        let plan = super::threshold_plan::ThresholdCompilationPlan::prepare(
            bound,
            staging.path(),
            1,
            Default::default(),
            Default::default(),
        )
        .unwrap();
        let work = plan.work(1).unwrap();
        let mut bytes = Vec::new();
        let receipt = work
            .compile_record(staging.path(), u64::MAX, &mut bytes)
            .unwrap();
        assert!(
            work.compile_record(
                staging.path(),
                work.plan_record().bytes - 1,
                &mut Vec::new()
            )
            .is_err()
        );
        let mut foreign = serde_json::to_value(&receipt).unwrap();
        foreign["work"]["plan"]["blake3"] = serde_json::json!("0".repeat(64));
        assert!(
            plan.validate_work_receipt(&serde_json::from_value(foreign).unwrap())
                .is_err()
        );
        let mut archive = plan
            .archive_writer(std::io::Cursor::new(Vec::new()))
            .unwrap();
        bytes[0] ^= 1;
        assert!(
            plan.append_work_record(&mut archive, &mut bytes.as_slice(), receipt)
                .is_err()
        );
        assert!(archive.finish().is_err());
        let path = work.plan_record().resolve(staging.path()).unwrap();
        let original = std::fs::read(&path).unwrap();
        let mut replaced = original.clone();
        *replaced.last_mut().unwrap() ^= 1;
        std::fs::write(&path, replaced).unwrap();
        assert!(
            work.compile_record(staging.path(), u64::MAX, &mut Vec::new())
                .is_err()
        );
        let mut longer = original;
        longer.push(0);
        std::fs::write(&path, longer).unwrap();
        assert!(
            work.compile_record(staging.path(), u64::MAX, &mut Vec::new())
                .is_err()
        );
    });
}
