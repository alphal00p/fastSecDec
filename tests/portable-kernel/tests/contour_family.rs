//! Real file-backed native family execution, also run unchanged in Emscripten.
//! This does not exercise a Python wheel, browser event loop or native threads.
use fastsecdec::{
    Atom,
    contour::{ContourMode, ContourSettings, ContourValidation, ContourValidationOptions},
    generation::{
        GenerationMode, GenerationOptions, GenerationSessionState, RecipeFamily,
        RecipeFamilySession,
    },
    integration::{Periodization, QmcSession, QmcSettings, RuleSource},
    kernel::{
        CompilationSettings, EvaluatorBackend, KernelLoadOptions, KernelSet, ProgramRecipe,
        indexed::ProgramArchiveReader,
    },
    parametric::{
        FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
        PolynomialFactor,
    },
    results::{KernelResultManifest, ResultScope},
    status::CoefficientComponent,
};
use std::{collections::BTreeMap, fs::File, ops::ControlFlow};
use symbolica::{parse, symbol};

fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("portable_family::x")],
        symbol!("portable_family::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("2+3𝑖"),
            vec![parse!("-1+portable_family::eps")],
            vec![
                PolynomialFactor::new(
                    parse!("1+portable_family::x"),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

fn settings(recipe: ProgramRecipe) -> ContourSettings {
    let deformation = match recipe {
        ProgramRecipe::UndeformedV1 => ContourMode::Off,
        ProgramRecipe::FixedV1 => ContourMode::Fixed { lambda: 0.2 },
        ProgramRecipe::DynamicPolynomialV1 => ContourMode::Dynamical {
            safety_fraction: 0.8,
            lambda_cap: 1.,
            displacement_cap: 1.,
            construction: fastsecdec::contour::DynamicConstruction::Polynomial,
        },
        ProgramRecipe::DynamicSignAwareV1 => ContourMode::dynamical(0.8),
        ProgramRecipe::ThresholdV1 => {
            unreachable!("this fixture only generates the explicit contour recipe family")
        }
    };
    ContourSettings {
        deformation,
        validation: ContourValidationOptions {
            policy: ContourValidation::Always,
            pilot_points: 2,
        },
    }
}

fn bind_and_pilot(kernels: &mut KernelSet) {
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings(kernels.program_recipe()))
        .unwrap();
    if kernels.program_recipe() != ProgramRecipe::UndeformedV1 {
        assert!(
            kernels
                .validate_integration_readiness(&ResultScope::FullIntegral)
                .is_err()
        );
        for chart in kernels.contour_validation_charts() {
            for x in [0.25, 0.75] {
                kernels
                    .validate_contour_point(chart.chart_index, &vec![x; chart.dimension], true)
                    .unwrap();
            }
        }
        assert!(kernels.finish_contour_pilot().unwrap().pilot_complete);
        let identity = kernels.content_id().to_owned();
        kernels
            .set_contour_validation(ContourValidationOptions {
                policy: ContourValidation::Pilot,
                pilot_points: 2,
            })
            .unwrap();
        assert_eq!(kernels.content_id(), identity);
    }
    kernels
        .validate_integration_readiness(&ResultScope::FullIntegral)
        .unwrap();
}

fn integrate(mut kernels: KernelSet) {
    bind_and_pilot(&mut kernels);
    let problem = KernelResultManifest::integration_problem_from_kernels(
        &kernels,
        &ResultScope::FullIntegral,
        kernels.content_id(),
    )
    .unwrap();
    let recipe = kernels.program_recipe();
    let mut contexts = (0..kernels.sectors().len())
        .map(|index| {
            kernels
                .evaluation_context(index, Default::default())
                .unwrap()
        })
        .collect::<Vec<_>>();
    drop(kernels);
    let mut session = QmcSession::democratic(
        problem,
        QmcSettings {
            points: 1024,
            shifts: 4,
            package_points: 128,
            seed: 294371,
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
            .evaluate_weighted_batch(task, 32, |points, weights, values| {
                contexts[id as usize]
                    .evaluate_weighted_batch(points, weights, values)
                    .map(|_| ())
            })
            .unwrap();
        session.submit(result).unwrap();
    }
    let estimate = session.estimate().unwrap();
    estimate.validate().unwrap();
    assert!(estimate.production_complete);
    assert_eq!(estimate.orders, [-1, -1, 0, 0]);
    assert_eq!(estimate.covariance_of_mean.len(), 16);
    for i in 0..4 {
        let scale = match estimate.components[i] {
            CoefficientComponent::Real => 2.,
            CoefficientComponent::Imag => 3.,
        };
        // Integral x^(-1+eps)/(1+x) = 1/eps - log(2) + O(eps).
        let reference = if estimate.orders[i] == -1 {
            scale
        } else {
            -scale * 2f64.ln()
        };
        assert!(estimate.standard_error[i] < 2e-5);
        assert!(
            (estimate.mean[i] - reference).abs() < (8. * estimate.standard_error[i]).max(2e-5),
            "{recipe:?}: component {i}, {} +/- {}, expected {reference}",
            estimate.mean[i],
            estimate.standard_error[i]
        );
    }
    if recipe != ProgramRecipe::UndeformedV1 {
        for context in contexts {
            assert_eq!(
                context
                    .contour_validation_report()
                    .unwrap()
                    .checked_arguments,
                0
            );
        }
    }
}

fn file_backed_family(mode: GenerationMode, resident: Option<ProgramRecipe>) {
    let staging = tempfile::tempdir().unwrap();
    // Transfer the native file owner directly. Emscripten's Rust File::try_clone
    // is unsupported; family generation itself does not require descriptor dup.
    let (writer, archive) = tempfile::NamedTempFile::new().unwrap().into_parts();
    let mut session = RecipeFamilySession::new(
        input(),
        GenerationOptions {
            mode,
            ..Default::default()
        },
        RecipeFamily::contour(),
        staging.path().to_owned(),
        writer,
    )
    .with_resident_recipe(resident)
    .unwrap()
    .with_evaluator(
        Default::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            horner_iterations: 0,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(std::fs::read_dir(staging.path()).unwrap().count(), 0);
    assert_eq!(std::fs::metadata(&archive).unwrap().len(), 0);
    assert!(session.take_result().is_none());
    let mut elapsed = 0.;
    while !session.is_complete() {
        let before = session.snapshot().completed_units;
        let state = session
            .step(8, |snapshot| {
                assert!(snapshot.generation.elapsed_seconds >= elapsed);
                elapsed = snapshot.generation.elapsed_seconds;
                ControlFlow::Break(())
            })
            .unwrap();
        assert_eq!(session.snapshot().completed_units, before + 1);
        assert!(matches!(
            state,
            GenerationSessionState::Paused | GenerationSessionState::Complete
        ));
    }
    assert_eq!(session.snapshot().prepared_sources, 1);
    assert_eq!(session.snapshot().completed_recipes, 4);
    let result = session.take_result().unwrap();
    assert!(session.take_result().is_none());
    assert_eq!(result.family.default_recipe(), ProgramRecipe::UndeformedV1);
    assert_eq!(
        result.resident.as_ref().map(KernelSet::program_recipe),
        resident
    );
    assert_eq!(result.catalogue.recipes.len(), 4);
    result.writer.sync_all().unwrap();
    drop(result.writer);
    drop(session);
    // File storage survives the session. Completed programs must not need the
    // source spool, including deferred native-dual functions and root owners.
    let staging_path = staging.path().to_owned();
    staging.close().unwrap();
    assert!(!staging_path.exists());
    if let Some(kernels) = result.resident {
        assert_eq!(
            kernels.content_id(),
            result
                .catalogue
                .recipe(kernels.program_recipe())
                .unwrap()
                .content_id
        );
        integrate(kernels);
    }
    let saved = tempfile::NamedTempFile::new().unwrap();
    std::fs::copy(&archive, saved.path()).unwrap();
    let original_path = archive.to_path_buf();
    archive.close().unwrap();
    assert!(!original_path.exists());
    for recipe in RecipeFamily::contour().recipes() {
        let mut reader = ProgramArchiveReader::from_reader(
            File::open(saved.path()).unwrap(),
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(reader.catalogue().content_id, result.catalogue.content_id);
        let mut selected = reader.select(*recipe).unwrap();
        let kernels = selected.load_all().unwrap();
        assert_eq!(kernels.program_recipe(), *recipe);
        assert_eq!(kernels.sectors().len(), 1);
        let mut sector = selected.load_sector(0).unwrap();
        assert_eq!(sector.program_recipe(), *recipe);
        drop(reader);
        bind_and_pilot(&mut sector);
        let mut values = vec![0.; sector.orders().len()];
        sector.sectors_mut()[0]
            .evaluate(&[0.37], &mut values)
            .unwrap();
        assert!(values.iter().all(|value| value.is_finite()));
        integrate(kernels);
    }
    let saved_path = saved.path().to_owned();
    saved.close().unwrap();
    assert!(!saved_path.exists());
}

#[test]
fn symbolic_family_uses_real_tempfiles_without_retaining_a_resident_recipe() {
    file_backed_family(GenerationMode::Symbolic, None);
}

#[test]
fn numerical_dual_family_restores_all_recipes_and_retains_requested_resident() {
    file_backed_family(
        GenerationMode::NumericalDual,
        Some(ProgramRecipe::DynamicSignAwareV1),
    );
}
