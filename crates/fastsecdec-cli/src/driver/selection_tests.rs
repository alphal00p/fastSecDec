use super::{
    checkpoint::{restore_checkpoint, save_checkpoint, settings_identity},
    execution::{integrate, problem, submit_package},
    replay::AcceptedReplay,
};
use crate::{config::IntegrationInput, display::Dashboard};
use fastsecdec::{
    integration::QmcSession,
    results::{ExactContributionPolicy, ResultScope},
    status::EvaluationDiagnostics,
};

#[test]
fn native_toml_scope_and_legacy_missing_field_share_expected_settings_identity() {
    let old = serde_json::to_value(IntegrationInput::default()).unwrap();
    assert!(old.get("scope").is_none());
    let restored: IntegrationInput = serde_json::from_value(old).unwrap();
    assert_eq!(restored.scope, ResultScope::FullIntegral);
    assert_eq!(
        settings_identity(&restored).unwrap(),
        settings_identity(&IntegrationInput::default()).unwrap()
    );
    let selected: IntegrationInput =
        toml::from_str("[scope.SelectedSectors]\nsector_ids=[7,2]\nexact_policy='ExcludeAll'")
            .unwrap();
    assert_eq!(
        selected.scope,
        ResultScope::SelectedSectors {
            sector_ids: vec![7, 2],
            exact_policy: ExactContributionPolicy::ExcludeAll
        }
    );
}

#[test]
fn partial_selected_checkpoint_resumes_only_selected_contexts_with_original_id() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    std::fs::write(&card, "[direct]\ndomain='unit_cube'\nparameters=['x','y']\n[[direct.terms]]\nmonomial_powers=['1','0']\n[[direct.terms.factors]]\npolynomial='x+y'\nexponent='-1'\n").unwrap();
    let (artifact, kernels) = crate::generate::generate(
        &card,
        &dir.path().join("artifact.fsd"),
        &mut Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(kernels.sectors().len(), 2);
    let mut settings = IntegrationInput {
        scope: ResultScope::SelectedSectors {
            sector_ids: vec![1],
            exact_policy: ExactContributionPolicy::ExcludeAll,
        },
        points: 1024,
        shifts: 8,
        package_points: 1024,
        lattice: "hkkn-alpha3".into(),
        max_rounds: 1,
        ..Default::default()
    };
    let projected = problem(&artifact, &kernels, &settings.scope).unwrap();
    let mut session =
        QmcSession::democratic(projected.clone(), settings.qmc_settings().unwrap()).unwrap();
    let mut accepted = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let mut contexts = accepted.contexts(&kernels, &projected.sectors).unwrap();
    assert_eq!(contexts.keys().copied().collect::<Vec<_>>(), [1]);
    let kernel = contexts.get_mut(&1).unwrap();
    let mut worker = session.worker_context(1).unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    for _ in 0..2 {
        let task = session.next_work().unwrap().unwrap();
        assert_eq!(task.sector_id(), 1);
        let result = worker.evaluate_weighted(task, |point, weight, output| {
            let report = kernel
                .evaluate_weighted(point, weight, output)
                .map_err(|e| e.to_string())?;
            diagnostics.record_replay(report).map_err(|e| e.to_string())
        });
        submit_package(
            result,
            1,
            Some(kernel.state().clone()),
            &mut accepted,
            |result| session.submit(result),
        )
        .unwrap();
    }
    let prior = accepted.state(1).clone();
    assert!(!accepted.state(0).verified());
    let checkpoint = dir.path().join("partial.json");
    save_checkpoint(
        &checkpoint,
        &artifact,
        &settings,
        0,
        session.checkpoint().unwrap(),
        &diagnostics,
        &accepted,
    )
    .unwrap();
    settings.workers = 3;
    let report = integrate(
        &artifact,
        &kernels,
        &settings,
        &checkpoint,
        true,
        &mut Dashboard::new(false, false).unwrap(),
    )
    .unwrap();
    assert_eq!(report.snapshot.completed_points, 8192);
    assert_eq!(report.snapshot.sectors.len(), 1);
    assert_eq!(report.snapshot.sectors[0].id, 1);
    let value = report.estimate.unwrap().mean[0];
    assert!(
        [(1.0 - 2.0_f64.ln()) / 2.0, 2.0_f64.ln() / 2.0]
            .iter()
            .any(|expected| (value - expected).abs() < 1e-4)
    );
    assert!(!report.converged);
    let restored = restore_checkpoint(&checkpoint, &artifact, &settings).unwrap();
    assert_eq!(restored.diagnostics.evaluations, 8192);
    assert!(!restored.replay.state(0).verified());
    assert!(
        restored
            .replay
            .state(1)
            .maxima()
            .iter()
            .zip(prior.maxima())
            .all(|(current, prior)| current >= prior)
    );
}
