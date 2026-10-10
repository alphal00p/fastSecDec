use super::*;
use crate::generation::{
    GeometryJob, RecipeFamilyJob, RecipeFamilyJobProgress, RecipeFamilyJobStage,
};

fn multi_chart_input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("family_dispatch::x"), symbol!("family_dispatch::y")],
        symbol!("family_dispatch::eps"),
        ParametricDomain::ProjectiveSimplex,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![
                PolynomialFactor::new(
                    parse!("family_dispatch::x+2*family_dispatch::y"),
                    Atom::num(-2),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}

fn build(
    width: usize,
    reverse: bool,
    mode: GenerationMode,
) -> (Vec<Vec<f64>>, Vec<(RecipeFamilyJobStage, usize)>, String) {
    let directory = tempfile::tempdir().unwrap();
    let mut session = RecipeFamilySession::new(
        multi_chart_input(),
        GenerationOptions {
            mode,
            ..Default::default()
        },
        RecipeFamily::contour(),
        directory.path().into(),
        Cursor::new(Vec::new()),
    )
    .with_evaluator(
        Default::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap();
    let mut batches = Vec::new();
    let mut geometry_dispatch = |jobs: &mut dyn ExactSizeIterator<Item = GeometryJob>| {
        let mut results = jobs
            .map(|job| job.run(|_| ControlFlow::Continue(())))
            .collect::<Vec<_>>();
        if reverse {
            results.reverse();
        }
        Ok(results)
    };
    while !session.is_complete() {
        let before = session.snapshot().completed_units;
        session
            .step_with_dispatch(
                3,
                width,
                &mut geometry_dispatch,
                &mut |jobs: &mut dyn ExactSizeIterator<Item = RecipeFamilyJob>| {
                    let count = jobs.len();
                    assert!(count <= width.min(3));
                    let mut results = std::thread::scope(|scope| {
                        jobs.map(|job| {
                            batches.push((job.id().stage, count));
                            scope.spawn(move || {
                                job.run(|_: RecipeFamilyJobProgress<'_>| ControlFlow::Continue(()))
                            })
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|task| task.join().unwrap())
                        .collect::<Result<Vec<_>, _>>()
                    })?;
                    if reverse {
                        results.reverse();
                    }
                    Ok(results)
                },
                |_| ControlFlow::Continue(()),
            )
            .unwrap();
        assert!(session.snapshot().completed_units - before <= 3);
    }
    let result = session.take_result().unwrap();
    assert_eq!(result.catalogue.recipes.len(), 4);
    assert_eq!(result.family.default_recipe(), ProgramRecipe::UndeformedV1);
    let identity = result.catalogue.content_id.clone();
    let mut reader =
        ProgramArchiveReader::from_reader(result.writer, KernelLoadOptions { validate: true })
            .unwrap();
    let values = RecipeFamily::contour()
        .recipes()
        .iter()
        .map(|recipe| {
            let mut kernels = reader.select(*recipe).unwrap().load_all().unwrap();
            let mode = match recipe {
                ProgramRecipe::UndeformedV1 => ContourMode::Off,
                ProgramRecipe::FixedV1 => ContourMode::Fixed { lambda: 0.2 },
                ProgramRecipe::DynamicPolynomialV1 => ContourMode::Dynamical {
                    safety_fraction: 0.8,
                    lambda_cap: 1.,
                    displacement_cap: 1.,
                    construction: crate::contour::DynamicConstruction::Polynomial,
                },
                ProgramRecipe::DynamicSignAwareV1 => ContourMode::dynamical(0.8),
            };
            let mut settings = ContourSettings {
                deformation: mode,
                ..Default::default()
            };
            settings.validation.policy = ContourValidation::Off;
            kernels
                .bind_parameters_with_contour(&BTreeMap::new(), &settings)
                .unwrap();
            let mut values = kernels.exact_coefficients().to_vec();
            for sector in kernels.sectors_mut() {
                let mut point = vec![0.; values.len()];
                sector
                    .evaluate(&vec![0.37; sector.dimension()], &mut point)
                    .unwrap();
                for (sum, value) in values.iter_mut().zip(point) {
                    *sum += value;
                }
            }
            values
        })
        .collect();
    (values, batches, identity)
}

#[test]
fn dispatched_family_is_ordered_bounded_and_uses_actual_caller_threads() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let (serial, _, serial_id) = build(1, false, mode);
        let (pair, _, pair_id) = build(2, true, mode);
        let (parallel, batches, parallel_id) = build(4, true, mode);
        assert_eq!(serial, pair);
        assert_eq!(serial, parallel);
        assert_eq!(serial_id, pair_id);
        assert_eq!(serial_id, parallel_id);
        assert!(
            batches
                .iter()
                .any(|(stage, count)| *stage == RecipeFamilyJobStage::Source && *count > 1)
        );
        assert!(
            batches
                .iter()
                .any(|(stage, count)| *stage == RecipeFamilyJobStage::Sector && *count > 1)
        );
    }
}

#[test]
fn incomplete_dispatch_is_terminal_without_persisting_or_exposing_a_family() {
    let directory = tempfile::tempdir().unwrap();
    let mut session = RecipeFamilySession::new(
        multi_chart_input(),
        Default::default(),
        RecipeFamily::contour(),
        directory.path().into(),
        Cursor::new(Vec::new()),
    );
    session.step(1, |_| ControlFlow::Continue(())).unwrap();
    let error = session
        .step_with_dispatch(
            3,
            3,
            &mut |_| unreachable!("geometry already prepared"),
            &mut |jobs| {
                let mut results = jobs
                    .map(|job| job.run(|_| ControlFlow::Continue(())))
                    .collect::<Result<Vec<_>, _>>()?;
                results.pop();
                Ok(results)
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap_err();
    assert!(error.to_string().contains("incomplete family"));
    assert_eq!(session.snapshot().persisted_units, 0);
    assert!(session.take_result().is_none());
    assert!(session.step(1, |_| ControlFlow::Continue(())).is_err());
}

#[test]
fn pausing_joins_the_bounded_batch_and_foreign_completions_cannot_be_replayed() {
    let directory = tempfile::tempdir().unwrap();
    let other_directory = tempfile::tempdir().unwrap();
    let make = |path: PathBuf| {
        RecipeFamilySession::new(
            multi_chart_input(),
            Default::default(),
            RecipeFamily::contour(),
            path,
            Cursor::new(Vec::new()),
        )
    };
    let mut session = make(directory.path().into());
    session.step(1, |_| ControlFlow::Continue(())).unwrap();
    let before = session.snapshot().completed_units;
    let mut joined = 0;
    let state = session
        .step_with_dispatch(
            32,
            4,
            &mut |_| unreachable!(),
            &mut |jobs| {
                let results = std::thread::scope(|scope| {
                    jobs.map(|job| scope.spawn(move || job.run(|_| ControlFlow::Continue(()))))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|job| job.join().unwrap())
                        .collect::<Result<Vec<_>, _>>()
                })?;
                joined = results.len();
                Ok(results)
            },
            |_| ControlFlow::Break(()),
        )
        .unwrap();
    assert_eq!(state, GenerationSessionState::Paused);
    assert!(joined > 1);
    assert_eq!(session.snapshot().completed_units, before + joined);

    // A genuinely executed result from another session is not a result for
    // this stage, even though the mathematical inputs are identical.
    let mut other = make(other_directory.path().into());
    other.step(1, |_| ControlFlow::Continue(())).unwrap();
    let mut foreign = None;
    assert!(
        other
            .step_with_dispatch(
                2,
                2,
                &mut |_| unreachable!(),
                &mut |jobs| {
                    let mut results = jobs
                        .map(|job| job.run(|_| ControlFlow::Continue(())))
                        .collect::<Result<Vec<_>, _>>()?;
                    foreign = results.pop();
                    Ok(results)
                },
                |_| ControlFlow::Continue(())
            )
            .is_err()
    );
    session.step(1, |_| ControlFlow::Continue(())).unwrap(); // BeginRecipe -> Map.
    let error = session
        .step_with_dispatch(
            2,
            2,
            &mut |_| unreachable!(),
            &mut |jobs| {
                let mut results = jobs
                    .map(|job| job.run(|_| ControlFlow::Continue(())))
                    .collect::<Result<Vec<_>, _>>()?;
                results[0] = foreign.take().unwrap();
                Ok(results)
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap_err();
    assert!(error.to_string().contains("foreign or duplicate"));
    assert_eq!(session.snapshot().persisted_units, 0);
    assert!(session.take_result().is_none());
}
