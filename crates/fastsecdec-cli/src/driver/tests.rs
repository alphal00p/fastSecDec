use super::*;
use super::{
    checkpoint::{Checkpoint, restore_checkpoint, save_checkpoint, save_mc_checkpoint},
    execution::problem,
    refinement::{adaptive_budget, qmc_design},
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
    assert_eq!(qmc_design(1 << 19, 8, 0).unwrap(), (1 << 19, 8));
    assert_eq!(qmc_design(1 << 19, 8, 1).unwrap(), (1 << 20, 8));
    assert_eq!(qmc_design(1 << 19, 8, 2).unwrap(), (1 << 20, 16));
    assert_eq!(qmc_design(1 << 19, 8, 3).unwrap(), (1 << 20, 32));
    assert!(qmc_design(1 << 20, u32::MAX, 1).is_err());
    assert!(qmc_design(3, 8, 0).is_err());
    assert!(qmc_design(1 << 21, 8, 0).is_err());
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
fn completed_checkpoint_resume_does_not_refine_or_repeat_work() {
    let (dir, artifact, kernels) = fixture();
    for method in ["qmc", "mc"] {
        let settings = settings(method);
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
    }
}

#[test]
fn cancelled_partial_qmc_preserves_complete_replica_diagnostics() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("qmc");
    let mut session = QmcSession::democratic(
        problem(&artifact, &kernels).unwrap(),
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
    assert_eq!(report.snapshot.completed_points, 3072);
    assert_eq!(report.snapshot.stop_reason, Some(StoppingReason::Cancelled));
    assert_eq!(report.resume_status, ResumeStatus::CheckpointSaved);
}

#[test]
fn pilot_mc_checkpoint_is_skipped_and_cancellation_requires_restart() {
    let (dir, artifact, kernels) = fixture();
    let settings = settings("adaptive_mc");
    let checkpoint = dir.path().join("pilot.json");
    let pilot = HavanaSession::pilot(
        problem(&artifact, &kernels).unwrap(),
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
            &EvaluationDiagnostics::default()
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
        problem(&artifact, &kernels).unwrap(),
        QmcSettings::default(),
    )
    .unwrap();
    save_checkpoint(
        &checkpoint,
        &artifact,
        &settings,
        0,
        session.checkpoint().unwrap(),
        &EvaluationDiagnostics::default(),
    )
    .unwrap();
    let mut changed = settings.clone();
    changed.workers = 3;
    assert!(restore_checkpoint(&checkpoint, &artifact, &changed).is_ok());
    changed.points *= 2;
    assert!(restore_checkpoint(&checkpoint, &artifact, &changed).is_err());
}
