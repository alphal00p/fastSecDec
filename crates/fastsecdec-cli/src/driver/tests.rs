use super::*;
use super::{
    checkpoint::{Checkpoint, restore_checkpoint, save_checkpoint, save_mc_checkpoint},
    execution::{problem, submit_package},
    refinement::{adaptive_budget, qmc_design},
    replay::AcceptedReplay,
};
use crate::{artifact::Artifact, config::IntegrationInput, display::Dashboard};
use fastsecdec::{
    integration::{
        QmcSession, QmcSettings,
        mc::{HavanaSession, HavanaSettings},
    },
    kernel::KernelSet,
    status::{EvaluationDiagnostics, IntegrationStage, StoppingReason},
};
use std::fs;

fn fixture() -> (tempfile::TempDir, Artifact, KernelSet) {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("input.toml");
    fs::write(
        &card,
        r#"
[direct]
domain = "unit_cube"
parameters = ["x"]
[[direct.terms]]
monomial_powers = ["1"]
"#,
    )
    .unwrap();
    let (artifact, kernels) = crate::generate::generate(
        &card,
        &dir.path().join("integral.json"),
        &mut Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    (dir, artifact, kernels)
}

fn settings(method: &str) -> IntegrationInput {
    IntegrationInput {
        method: method.into(),
        points: 1024,
        shifts: 4,
        package_points: 1024,
        absolute_tolerance: 0.0,
        relative_tolerance: 0.0,
        max_rounds: 2,
        ..IntegrationInput::default()
    }
}

#[test]
fn lattice_refinement_caps_points_then_grows_independent_shifts() {
    let design = |points, shifts, round| {
        qmc_design(
            &IntegrationInput {
                points,
                shifts,
                ..Default::default()
            },
            round,
        )
    };
    assert_eq!(design(1 << 19, 8, 0).unwrap(), (1 << 19, 8));
    assert_eq!(design(1 << 19, 8, 1).unwrap(), (1 << 20, 8));
    assert_eq!(design(1 << 19, 8, 2).unwrap(), (1 << 20, 16));
    assert_eq!(design(1 << 19, 8, 3).unwrap(), (1 << 20, 32));
    assert!(design(1 << 20, u32::MAX, 1).is_err());
    assert!(design(3, 8, 0).is_err());
    assert!(design(1 << 21, 8, 0).is_err());
    let settings = IntegrationInput {
        points: 1 << 19,
        shifts: 8,
        production_seconds: 1.25,
        ..IntegrationInput::default()
    };
    assert_eq!(adaptive_budget(&settings, 1).unwrap(), (1.25, 2));
    assert_eq!(adaptive_budget(&settings, 2).unwrap(), (2.5, 4));
    assert_eq!(adaptive_budget(&settings, 3).unwrap(), (5.0, 8));
}

#[test]
fn published_catalogue_uses_native_bounds_and_preserves_legacy_settings_identity() {
    use fastsecdec::integration::{PublishedLattice, RuleSource};
    let legacy = IntegrationInput::default();
    let old_identity = checkpoint::settings_identity(&legacy).unwrap();
    assert!(old_identity.get("lattice").is_none());
    let restored: IntegrationInput = serde_json::from_value(old_identity.clone()).unwrap();
    assert_eq!(restored.lattice, "kuo33002");
    assert_eq!(restored.qmc_settings().unwrap().rule, RuleSource::Kuo);
    assert_eq!(
        checkpoint::settings_identity(&restored).unwrap(),
        old_identity
    );
    for (name, catalogue) in [
        ("kuo38005", PublishedLattice::Kuo38005),
        ("kuo39101", PublishedLattice::Kuo39101),
        ("hkkn-alpha3", PublishedLattice::HkknAlpha3),
    ] {
        let selected = IntegrationInput {
            lattice: name.into(),
            points: catalogue.min_points(),
            ..Default::default()
        };
        assert_eq!(
            selected.qmc_settings().unwrap().rule,
            RuleSource::Published(catalogue)
        );
        assert_ne!(
            checkpoint::settings_identity(&selected).unwrap(),
            old_identity
        );
        assert_eq!(
            qmc_design(&selected, 1).unwrap(),
            (catalogue.min_points() * 2, selected.shifts)
        );
        let capped = IntegrationInput {
            points: catalogue.max_points(),
            ..selected
        };
        assert_eq!(
            qmc_design(&capped, 1).unwrap(),
            (catalogue.max_points(), capped.shifts * 2)
        );
    }
    assert!(
        IntegrationInput {
            points: 2,
            ..legacy
        }
        .qmc_settings()
        .is_err()
    );
}

#[test]
fn completed_checkpoint_resume_does_not_refine_or_repeat_work() {
    let (dir, artifact, kernels) = fixture();
    for method in ["qmc", "mc", "adaptive_qmc", "adaptive_mc"] {
        let mut settings = settings(method);
        settings.production_seconds = 1.0;
        let checkpoint = dir.path().join(format!("{method}.json"));
        let original = integrate(
            &artifact,
            &kernels,
            &settings,
            &checkpoint,
            false,
            &mut Dashboard::new(false, false).unwrap(),
        )
        .unwrap();
        let state: Checkpoint = serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
        assert_eq!(state.round_index, 1);
        let accepted = restore_checkpoint(&checkpoint, &artifact, &settings)
            .unwrap()
            .replay;
        assert!(accepted.state(0).verified());
        let diagnostics = original.snapshot.evaluation_diagnostics.as_ref().unwrap();
        assert!(diagnostics.weighted_checks > 0);
        assert!(diagnostics.additional_replays > 0);
        settings.workers = 3;
        let resumed = integrate(
            &artifact,
            &kernels,
            &settings,
            &checkpoint,
            true,
            &mut Dashboard::new(false, false).unwrap(),
        )
        .unwrap();
        assert_eq!(
            original.snapshot.completed_points,
            resumed.snapshot.completed_points
        );
        assert_eq!(
            original.snapshot.planned_points,
            resumed.snapshot.planned_points
        );
        assert_eq!(original.estimate, resumed.estimate);
        assert_eq!(
            original.snapshot.evaluation_diagnostics,
            resumed.snapshot.evaluation_diagnostics
        );
        assert_eq!(
            original.snapshot.worker_seconds,
            resumed.snapshot.worker_seconds
        );
        assert_eq!(original.qmc_design, resumed.qmc_design);
        if method.ends_with("qmc") {
            let design = original.qmc_design.as_ref().unwrap();
            assert_eq!(design.settings.points, 2048);
            assert!(
                design
                    .allocations
                    .iter()
                    .all(|allocation| allocation.points == 2048)
            );
        } else {
            assert!(original.qmc_design.is_none());
        }
        assert_eq!(
            accepted,
            restore_checkpoint(&checkpoint, &artifact, &settings)
                .unwrap()
                .replay
        );
    }
}

#[test]
fn failed_prefix_and_rejected_submission_never_advance_replay_state() {
    let (_dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let mut accepted = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let initial = accepted.clone();
    let mut context = accepted
        .contexts(
            &kernels,
            &fastsecdec::results::KernelResultManifest::from_kernels(&kernels).sectors,
        )
        .unwrap()
        .remove(&0)
        .unwrap();
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        QmcSettings {
            points: 1024,
            shifts: 2,
            package_points: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let task = session.next_work().unwrap().unwrap();
    let mut worker = session.worker_context(0).unwrap();
    let mut count = 0;
    let failed = worker.evaluate_weighted(task.clone(), |point, weight, output| {
        count += 1;
        if count == 3 {
            return Err("injected package evaluation failure".to_owned());
        }
        context
            .evaluate_weighted(point, weight, output)
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    assert!(
        context.state().verified(),
        "prefix must actually advance local state"
    );
    assert!(
        submit_package(
            failed,
            0,
            Some(context.state().clone()),
            &mut accepted,
            |result| session.submit(result)
        )
        .is_err()
    );
    assert_eq!(accepted, initial);
    assert_eq!(session.snapshot().unwrap().completed_points, 0);

    // Retry from accepted state, then reject a duplicate numerical return with
    // a deliberately larger local envelope. Neither failed submission commits.
    let mut context = accepted
        .contexts(
            &kernels,
            &fastsecdec::results::KernelResultManifest::from_kernels(&kernels).sectors,
        )
        .unwrap()
        .remove(&0)
        .unwrap();
    let returned = worker
        .evaluate_weighted(task, |point, weight, output| {
            context.evaluate_weighted(point, weight, output).map(|_| ())
        })
        .unwrap();
    submit_package(
        Ok(returned.clone()),
        0,
        Some(context.state().clone()),
        &mut accepted,
        |result| session.submit(result),
    )
    .unwrap();
    let prior = accepted.clone();
    context
        .evaluate_weighted(&[1.0], 100.0, &mut [0.0])
        .unwrap();
    assert!(context.state().maxima()[0] > prior.state(0).maxima()[0]);
    assert!(
        submit_package(
            Ok(returned),
            0,
            Some(context.state().clone()),
            &mut accepted,
            |result| session.submit(result)
        )
        .is_err()
    );
    assert_eq!(accepted, prior);
}

#[test]
fn partial_weighted_checkpoint_resumes_new_work_with_different_worker_count() {
    let (dir, artifact, kernels) = fixture();
    let mut settings = settings("qmc");
    settings.max_rounds = 1;
    let mut accepted = AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap();
    let mut context = accepted
        .contexts(
            &kernels,
            &fastsecdec::results::KernelResultManifest::from_kernels(&kernels).sectors,
        )
        .unwrap()
        .remove(&0)
        .unwrap();
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        QmcSettings {
            points: 1024,
            shifts: 4,
            package_points: 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let mut worker = session.worker_context(0).unwrap();
    let mut diagnostics = EvaluationDiagnostics::default();
    for _ in 0..2 {
        let task = session.next_work().unwrap().unwrap();
        let result = worker.evaluate_weighted(task, |point, weight, output| {
            let report = context
                .evaluate_weighted(point, weight, output)
                .map_err(|e| e.to_string())?;
            diagnostics.record_replay(report).map_err(|e| e.to_string())
        });
        submit_package(
            result,
            0,
            Some(context.state().clone()),
            &mut accepted,
            |result| session.submit(result),
        )
        .unwrap();
    }
    let prior_maxima = accepted.state(0).maxima().to_vec();
    let checkpoint = dir.path().join("partial-weighted.json");
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
    let estimate = report.estimate.unwrap();
    assert!(estimate.production_complete);
    assert_eq!(report.snapshot.completed_points, 4096);
    assert_eq!(report.snapshot.planned_points, 4096);
    assert!((estimate.mean[0] - 0.5).abs() < 2e-5);
    let restored = restore_checkpoint(&checkpoint, &artifact, &settings).unwrap();
    assert_eq!(restored.diagnostics.evaluations, 4096);
    assert!(restored.diagnostics.weighted_checks >= diagnostics.weighted_checks);
    assert!(restored.diagnostics.additional_replays >= diagnostics.additional_replays);
    for (current, prior) in restored.replay.state(0).maxima().iter().zip(prior_maxima) {
        assert!(*current >= prior);
    }
}

#[test]
fn checkpoint_rejects_missing_or_incompatible_replay_state_and_old_version() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let checkpoint = dir.path().join("replay.json");
    let session = QmcSession::democratic(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        settings.qmc_settings().unwrap(),
    )
    .unwrap();
    save_checkpoint(
        &checkpoint,
        &artifact,
        &settings,
        0,
        session.checkpoint().unwrap(),
        &EvaluationDiagnostics::default(),
        &AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap(),
    )
    .unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
    let mutations: [fn(&mut serde_json::Value); 7] = [
        |v| v["format_version"] = 2.into(),
        |v| v["replay"]["states"] = serde_json::json!([]),
        |v| v["replay"]["policy"]["growth_factor"] = 17.0.into(),
        |v| v["replay"]["states"][0]["content_id"] = "another-kernel".into(),
        |v| v["replay"]["states"][0]["sector"] = 1.into(),
        |v| v["replay"]["states"][0]["maximum_absolute_weighted"] = serde_json::json!([]),
        |v| v["replay"]["states"][0]["policy"]["growth_factor"] = 17.0.into(),
    ];
    for (case, mutate) in mutations.into_iter().enumerate() {
        let mut value = original.clone();
        mutate(&mut value);
        fs::write(&checkpoint, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            integrate(
                &artifact,
                &kernels,
                &settings,
                &checkpoint,
                true,
                &mut Dashboard::new(false, false).unwrap()
            )
            .is_err(),
            "tamper case {case}"
        );
    }
}

#[test]
fn cancelled_partial_qmc_preserves_complete_replica_diagnostics() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        QmcSettings {
            points: 1024,
            shifts: 4,
            package_points: 1024,
            ..QmcSettings::default()
        },
    )
    .unwrap();
    let mut worker = session.worker_context(0).unwrap();
    for _ in 0..2 {
        let task = session.next_work().unwrap().unwrap();
        session
            .submit(
                worker
                    .evaluate(task, |x, out| {
                        out[0] = x[0];
                        Ok::<_, std::convert::Infallible>(())
                    })
                    .unwrap(),
            )
            .unwrap();
    }
    let checkpoint = dir.path().join("partial.json");
    save_checkpoint(
        &checkpoint,
        &artifact,
        &settings,
        0,
        session.checkpoint().unwrap(),
        &EvaluationDiagnostics::default(),
        &AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap(),
    )
    .unwrap();
    let mut dashboard = Dashboard::new(false, false).unwrap();
    dashboard.request_cancel();
    let report = integrate(
        &artifact,
        &kernels,
        &settings,
        &checkpoint,
        true,
        &mut dashboard,
    )
    .unwrap();
    assert!(!report.converged);
    assert!(report.estimate.is_some());
    assert!(!report.estimate.unwrap().production_complete);
    // Cancellation is checked before dispatch, so a pre-cancelled resume does
    // not issue one additional package beyond the accepted checkpoint.
    assert_eq!(report.snapshot.completed_points, 2048);
    assert_eq!(report.snapshot.stop_reason, Some(StoppingReason::Cancelled));
    assert_eq!(report.resume_status, ResumeStatus::CheckpointSaved);
}

#[test]
fn pilot_mc_checkpoint_is_skipped_and_cancellation_requires_restart() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("adaptive_mc");
    let checkpoint = dir.path().join("pilot.json");
    let pilot = HavanaSession::pilot(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        HavanaSettings::default(),
    )
    .unwrap();
    assert_eq!(
        save_mc_checkpoint(
            &checkpoint,
            &artifact,
            &settings,
            0,
            &pilot,
            &EvaluationDiagnostics::default(),
            &AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap(),
        )
        .unwrap(),
        ResumeStatus::PilotRestartRequired
    );
    assert!(!checkpoint.exists());
    let mut dashboard = Dashboard::new(false, false).unwrap();
    dashboard.request_cancel();
    let report = integrate(
        &artifact,
        &kernels,
        &settings,
        &checkpoint,
        false,
        &mut dashboard,
    )
    .unwrap();
    assert_eq!(report.resume_status, ResumeStatus::PilotRestartRequired);
    assert_eq!(report.snapshot.stage, IntegrationStage::Pilot);
    assert!(report.stopping_reason.contains("restart the pilot"));
    assert!(!checkpoint.exists());
}

#[test]
fn checkpoint_settings_allow_only_worker_count_changes() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let checkpoint = dir.path().join("identity.json");
    let session = QmcSession::democratic(
        problem(&artifact, &kernels, &Default::default()).unwrap(),
        settings.qmc_settings().unwrap(),
    )
    .unwrap();
    save_checkpoint(
        &checkpoint,
        &artifact,
        &settings,
        0,
        session.checkpoint().unwrap(),
        &EvaluationDiagnostics::default(),
        &AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap(),
    )
    .unwrap();
    let mut changed = settings.clone();
    changed.workers = 3;
    assert!(restore_checkpoint(&checkpoint, &artifact, &changed).is_ok());
    changed.points *= 2;
    assert!(restore_checkpoint(&checkpoint, &artifact, &changed).is_err());
}

#[test]
fn checkpoint_binds_native_catalogue_method_and_refinement_to_outer_settings() {
    use fastsecdec::integration::{PublishedLattice, RuleSource};
    let (dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let expected_problem = problem(&artifact, &kernels, &Default::default()).unwrap();
    let mut alternative = settings.qmc_settings().unwrap();
    alternative.rule = RuleSource::Published(PublishedLattice::HkknAlpha3);
    let sessions = [
        (
            "catalogue",
            0,
            QmcSession::democratic(expected_problem.clone(), alternative).unwrap(),
        ),
        (
            "method",
            0,
            QmcSession::adaptive(expected_problem.clone(), settings.qmc_settings().unwrap())
                .unwrap(),
        ),
        (
            "round",
            1,
            QmcSession::democratic(expected_problem, settings.qmc_settings().unwrap()).unwrap(),
        ),
    ];
    for (case, round, session) in sessions {
        let path = dir.path().join(format!("mismatch-{case}.json"));
        save_checkpoint(
            &path,
            &artifact,
            &settings,
            round,
            session.checkpoint().unwrap(),
            &EvaluationDiagnostics::default(),
            &AcceptedReplay::new(&kernels, settings.replay.clone()).unwrap(),
        )
        .unwrap();
        let saved = fs::read(&path).unwrap();
        let error = match integrate(
            &artifact,
            &kernels,
            &settings,
            &path,
            true,
            &mut Dashboard::new(false, false).unwrap(),
        ) {
            Ok(_) => panic!("{case}: incompatible native checkpoint accepted"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("native QMC design"),
            "{case}: {error}"
        );
        assert_eq!(
            fs::read(path).unwrap(),
            saved,
            "mismatched checkpoints must not advance work"
        );
    }
}
