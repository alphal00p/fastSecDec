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
    let kernels = KernelSet::compile_threshold_fiber(
        &bound,
        staging.path(),
        0,
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let mut kernels = KernelSet::from_bytes_with_options(
        &kernels.to_bytes().unwrap(),
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
        serde_json::json!({"scope":"native HEPKit graph Gamma/projective, metadata-bound threshold-v15 restored KernelSet, ordinary QmcSession","mean":estimate.mean,"covariance":estimate.covariance_of_mean,"reference":expected})
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
