use std::{collections::BTreeMap, fs, path::Path, time::Instant};

use fastsecdec::{
    integration::{
        IntegrationProblem, Periodization, QmcSession, QmcSettings, QmcWorker, RuleSource,
        SectorSpec, Tolerance, VectorEstimate,
        mc::{HavanaSession, HavanaSettings, HavanaWorker},
    },
    kernel::{KernelSet, SectorKernel},
    status::{EvaluationDiagnostics, IntegrationSnapshot, IntegrationStage, StoppingReason},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    CliResult,
    artifact::{Artifact, atomic_write},
    config::IntegrationInput,
    display::Dashboard,
};

#[derive(Serialize)]
pub struct IntegrationReport {
    pub content_id: String,
    pub elapsed_seconds: f64,
    pub converged: bool,
    pub stopping_reason: String,
    pub estimate: Option<VectorEstimate>,
    pub snapshot: IntegrationSnapshot,
    pub resume_status: ResumeStatus,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeStatus {
    CheckpointSaved,
    /// Havana does not export mutable pilot training state. Restart the pilot.
    PilotRestartRequired,
}

struct QmcSlot {
    kernels: Vec<SectorKernel>,
    workers: BTreeMap<u64, QmcWorker>,
}
struct McSlot {
    kernels: Vec<SectorKernel>,
    workers: BTreeMap<u64, HavanaWorker>,
}

fn evaluate_tracked(
    kernel: &mut SectorKernel,
    point: &[f64],
    output: &mut [f64],
    diagnostics: &mut EvaluationDiagnostics,
) -> Result<(), String> {
    match kernel.evaluate_with_diagnostics(point, output) {
        Ok(report) => diagnostics
            .record(report)
            .map_err(|error| error.to_string()),
        Err(error) => {
            diagnostics
                .record_failure()
                .map_err(|error| error.to_string())?;
            Err(error.to_string())
        }
    }
}

fn with_diagnostics(
    mut snapshot: IntegrationSnapshot,
    diagnostics: &EvaluationDiagnostics,
) -> IntegrationSnapshot {
    snapshot.evaluation_diagnostics = Some(diagnostics.clone());
    snapshot
}

fn stopped(
    mut snapshot: IntegrationSnapshot,
    cancelled: bool,
    converged: bool,
) -> IntegrationSnapshot {
    snapshot.stop_reason = Some(if cancelled {
        StoppingReason::Cancelled
    } else if converged {
        StoppingReason::TargetReached
    } else {
        StoppingReason::WorkLimit
    });
    snapshot
}

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    format_version: u32,
    content_id: String,
    settings: serde_json::Value,
    round_index: usize,
    session: serde_json::Value,
    #[serde(default)]
    diagnostics: EvaluationDiagnostics,
}

fn settings_identity(settings: &IntegrationInput) -> CliResult<serde_json::Value> {
    let mut value = serde_json::to_value(settings)?;
    value.as_object_mut().unwrap().remove("workers");
    Ok(value)
}

fn save_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
    round: usize,
    session: Vec<u8>,
    diagnostics: &EvaluationDiagnostics,
) -> CliResult<()> {
    atomic_write(
        path,
        &serde_json::to_vec(&Checkpoint {
            format_version: 2,
            content_id: artifact.content_id.clone(),
            settings: settings_identity(settings)?,
            round_index: round,
            session: serde_json::from_slice(&session)?,
            diagnostics: diagnostics.clone(),
        })?,
    )
}

fn restore_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
) -> CliResult<(usize, EvaluationDiagnostics, Vec<u8>)> {
    let checkpoint: Checkpoint = serde_json::from_slice(&fs::read(path)?)?;
    if checkpoint.format_version != 2
        || checkpoint.content_id != artifact.content_id
        || checkpoint.settings != settings_identity(settings)?
        || checkpoint.round_index >= settings.max_rounds
    {
        return Err("checkpoint input identity or integration settings differ; only the worker count may change during resume".into());
    }
    Ok((
        checkpoint.round_index,
        checkpoint.diagnostics,
        serde_json::to_vec(&checkpoint.session)?,
    ))
}

// The bundled lattice stops at 2^20 points; beyond that size, add independent
// shifts without requesting an unavailable rule.
fn qmc_design(points: u64, shifts: u32, round: usize) -> CliResult<(u64, u32)> {
    const MAX_POINTS: u64 = 1 << 20;
    if !(1024..=MAX_POINTS).contains(&points) || !points.is_power_of_two() {
        return Err("bundled QMC points must be a power of two in 1024..=2^20".into());
    }
    let (mut points, mut shifts) = (points, shifts);
    for _ in 0..round {
        if points < MAX_POINTS {
            points *= 2;
        } else {
            shifts = shifts
                .checked_mul(2)
                .ok_or("independent shift growth overflow")?;
        }
    }
    Ok((points, shifts))
}

fn mc_points(points: u64, round: usize) -> CliResult<u64> {
    points
        .checked_mul(
            1u64.checked_shl(round.try_into()?)
                .ok_or("MC growth overflow")?,
        )
        .ok_or_else(|| "MC growth overflow".into())
}

fn adaptive_budget(settings: &IntegrationInput, round: usize) -> CliResult<(f64, u32)> {
    let (_, shifts) = qmc_design(settings.points, settings.shifts, round)?;
    let multiplier = shifts
        .checked_div(settings.shifts)
        .ok_or("at least two shifts are required")?;
    let seconds = settings.production_seconds * f64::from(multiplier);
    let minimum_shifts = 2u32
        .checked_mul(multiplier)
        .ok_or("production shift growth overflow")?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("adaptive production budget must be finite and positive".into());
    }
    Ok((seconds, minimum_shifts))
}

fn save_mc_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
    round: usize,
    session: &HavanaSession,
    diagnostics: &EvaluationDiagnostics,
) -> CliResult<ResumeStatus> {
    if session.stage() == IntegrationStage::Pilot {
        return Ok(ResumeStatus::PilotRestartRequired);
    }
    save_checkpoint(
        path,
        artifact,
        settings,
        round,
        session.checkpoint()?,
        diagnostics,
    )?;
    Ok(ResumeStatus::CheckpointSaved)
}

fn problem(artifact: &Artifact, kernels: &KernelSet) -> CliResult<IntegrationProblem> {
    Ok(IntegrationProblem::new_with_components(
        artifact.content_id.clone(),
        kernels.orders().to_vec(),
        kernels.components().to_vec(),
        kernels
            .sectors()
            .iter()
            .enumerate()
            .map(|(id, kernel)| SectorSpec {
                id: id as u64,
                dimension: kernel.dimension(),
            })
            .collect(),
        kernels.exact_coefficients().to_vec(),
    )?)
}

fn cloned_kernels(kernels: &KernelSet) -> CliResult<Vec<SectorKernel>> {
    Ok(kernels
        .sectors()
        .iter()
        .map(SectorKernel::try_clone)
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn integrate(
    artifact: &Artifact,
    kernels: &KernelSet,
    settings: &IntegrationInput,
    checkpoint: &Path,
    resume: bool,
    dashboard: &mut Dashboard,
) -> CliResult<IntegrationReport> {
    if settings.workers == 0 || settings.max_rounds == 0 {
        return Err("workers and max_rounds must be positive".into());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(settings.workers)
        .build()?;
    let problem = problem(artifact, kernels)?;
    let tolerance = Tolerance::new(settings.absolute_tolerance, settings.relative_tolerance)?;
    let started = Instant::now();
    let mut last_checkpoint = Instant::now();
    let method = settings.method.replace('-', "_");
    let restored = if resume {
        Some(restore_checkpoint(checkpoint, artifact, settings)?)
    } else {
        None
    };
    let mut diagnostics = restored
        .as_ref()
        .map(|(_, diagnostics, _)| diagnostics.clone())
        .unwrap_or_default();
    if method == "mc" || method == "adaptive_mc" {
        let options = HavanaSettings {
            points_per_batch: settings.points.try_into()?,
            batches: settings.shifts,
            seed: settings.seed,
            ..HavanaSettings::default()
        };
        let mut session = if let Some((_, _, bytes)) = &restored {
            HavanaSession::restore(bytes, &problem)?
        } else if method == "adaptive_mc" {
            HavanaSession::pilot(problem.clone(), options.clone())?
        } else {
            HavanaSession::production(problem.clone(), options.clone())?
        };
        let mut cancelled = false;
        let mut round = restored.as_ref().map_or(0, |(round, _, _)| *round);
        let mut current_points = mc_points(settings.points, round)?;
        loop {
            let mut slots = (0..settings.workers)
                .map(|_| {
                    Ok(McSlot {
                        kernels: cloned_kernels(kernels)?,
                        workers: session
                            .problem()
                            .sectors
                            .iter()
                            .map(|sector| Ok((sector.id, session.worker_context(sector.id)?)))
                            .collect::<CliResult<_>>()?,
                    })
                })
                .collect::<CliResult<Vec<_>>>()?;
            while !session.is_complete() {
                let tasks = (0..settings.workers)
                    .filter_map(|_| session.next_work())
                    .collect::<Vec<_>>();
                if tasks.is_empty() {
                    return Err("MC scheduler has no work before completion".into());
                }
                let returns = pool.install(|| {
                    slots
                        .par_iter_mut()
                        .zip(tasks.into_par_iter())
                        .map(|(slot, task)| {
                            let id = task.sector_id();
                            let kernel = &mut slot.kernels[id as usize];
                            let mut local = EvaluationDiagnostics::default();
                            let result = slot
                                .workers
                                .get_mut(&id)
                                .unwrap()
                                .evaluate(task, |point, output| {
                                    evaluate_tracked(kernel, point, output, &mut local)
                                });
                            (result, local)
                        })
                        .collect::<Vec<_>>()
                });
                let mut failure = None;
                for (result, local) in returns {
                    diagnostics.merge(&local)?;
                    match result {
                        Ok(result) => session.submit(result)?,
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    }
                }
                dashboard.integration(
                    &with_diagnostics(session.snapshot()?, &diagnostics),
                    started.elapsed().as_secs_f64(),
                )?;
                if let Some(error) = failure {
                    save_mc_checkpoint(
                        checkpoint,
                        artifact,
                        settings,
                        round,
                        &session,
                        &diagnostics,
                    )?;
                    return Err(error.into());
                }
                if last_checkpoint.elapsed().as_secs() >= 5 {
                    save_mc_checkpoint(
                        checkpoint,
                        artifact,
                        settings,
                        round,
                        &session,
                        &diagnostics,
                    )?;
                    last_checkpoint = Instant::now();
                }
                if dashboard.cancelled() {
                    cancelled = true;
                    break;
                }
            }
            if cancelled {
                break;
            }
            if session.stage() == IntegrationStage::Pilot {
                session.freeze_production(0.5, current_points.try_into()?, settings.shifts)?;
                continue;
            }
            if session.estimate()?.meets(tolerance)? || round + 1 >= settings.max_rounds {
                break;
            }
            round += 1;
            current_points = mc_points(settings.points, round)?;
            let mut next = options.clone();
            next.points_per_batch = current_points.try_into()?;
            session = if method == "adaptive_mc" {
                HavanaSession::pilot(problem.clone(), next)?
            } else {
                HavanaSession::production(problem.clone(), next)?
            };
        }
        let resume_status = save_mc_checkpoint(
            checkpoint,
            artifact,
            settings,
            round,
            &session,
            &diagnostics,
        )?;
        let snapshot = with_diagnostics(session.snapshot()?, &diagnostics);
        let estimate = snapshot.estimate.clone();
        let converged = !cancelled
            && estimate
                .as_ref()
                .is_some_and(|estimate| estimate.meets(tolerance).unwrap_or(false));
        return Ok(IntegrationReport {
            content_id: artifact.content_id.clone(),
            elapsed_seconds: started.elapsed().as_secs_f64(),
            converged,
            stopping_reason: if cancelled && resume_status == ResumeStatus::PilotRestartRequired {
                "cancelled during MC pilot; restart the pilot to continue"
            } else if cancelled {
                "cancelled"
            } else if converged {
                "accuracy reached"
            } else {
                "work limit"
            }
            .into(),
            estimate,
            snapshot: stopped(snapshot, cancelled, converged),
            resume_status,
        });
    }
    if method != "qmc" && method != "adaptive_qmc" {
        return Err("integration method must be qmc, adaptive_qmc, mc or adaptive_mc".into());
    }
    qmc_design(settings.points, settings.shifts, 0)?;
    let options = QmcSettings {
        points: settings.points,
        shifts: settings.shifts,
        seed: settings.seed,
        package_points: settings.package_points,
        periodization: match settings.periodization.as_str() {
            "none" => Periodization::None,
            "korobov3" => Periodization::Korobov3,
            _ => return Err("periodization must be none or korobov3".into()),
        },
        rule: RuleSource::Kuo,
    };
    let mut session = if let Some((_, _, bytes)) = &restored {
        QmcSession::restore(bytes, &problem)?
    } else if method == "adaptive_qmc" {
        QmcSession::adaptive(problem.clone(), options.clone())?
    } else {
        QmcSession::democratic(problem.clone(), options.clone())?
    };
    let mut cancelled = false;
    let mut round = restored.as_ref().map_or(0, |(round, _, _)| *round);
    loop {
        let mut slots = (0..settings.workers)
            .map(|_| {
                Ok(QmcSlot {
                    kernels: cloned_kernels(kernels)?,
                    workers: session
                        .problem()
                        .sectors
                        .iter()
                        .map(|sector| Ok((sector.id, session.worker_context(sector.id)?)))
                        .collect::<CliResult<_>>()?,
                })
            })
            .collect::<CliResult<Vec<_>>>()?;
        while !session.is_complete() {
            let tasks = (0..settings.workers)
                .map(|_| session.next_work())
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            if tasks.is_empty() {
                return Err("QMC scheduler has no work before completion".into());
            }
            let returns = pool.install(|| {
                slots
                    .par_iter_mut()
                    .zip(tasks.into_par_iter())
                    .map(|(slot, task)| {
                        let id = task.sector_id();
                        let kernel = &mut slot.kernels[id as usize];
                        let mut local = EvaluationDiagnostics::default();
                        let result = slot
                            .workers
                            .get_mut(&id)
                            .unwrap()
                            .evaluate(task, |point, output| {
                                evaluate_tracked(kernel, point, output, &mut local)
                            });
                        (result, local)
                    })
                    .collect::<Vec<_>>()
            });
            let mut failure = None;
            for (result, local) in returns {
                diagnostics.merge(&local)?;
                match result {
                    Ok(result) => session.submit(result)?,
                    Err(error) => {
                        failure.get_or_insert(error);
                    }
                }
            }
            dashboard.integration(
                &with_diagnostics(session.snapshot()?, &diagnostics),
                started.elapsed().as_secs_f64(),
            )?;
            if let Some(error) = failure {
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
                    &diagnostics,
                )?;
                return Err(error.into());
            }
            if last_checkpoint.elapsed().as_secs() >= 5 {
                save_checkpoint(
                    checkpoint,
                    artifact,
                    settings,
                    round,
                    session.checkpoint()?,
                    &diagnostics,
                )?;
                last_checkpoint = Instant::now();
            }
            if dashboard.cancelled() {
                cancelled = true;
                break;
            }
        }
        if cancelled {
            break;
        }
        if session.stage() == IntegrationStage::Pilot {
            let (seconds, minimum_shifts) = adaptive_budget(settings, round)?;
            let allocation = session.recommend_allocation(
                seconds,
                minimum_shifts,
                &vec![1.0; kernels.orders().len()],
            )?;
            session.freeze_production(allocation)?;
            continue;
        }
        if session.estimate()?.meets(tolerance)? || round + 1 >= settings.max_rounds {
            break;
        }
        round += 1;
        let mut next = options.clone();
        (next.points, next.shifts) = qmc_design(settings.points, settings.shifts, round)?;
        session = if method == "adaptive_qmc" {
            QmcSession::adaptive(problem.clone(), next)?
        } else {
            QmcSession::democratic(problem.clone(), next)?
        };
    }
    save_checkpoint(
        checkpoint,
        artifact,
        settings,
        round,
        session.checkpoint()?,
        &diagnostics,
    )?;
    let snapshot = with_diagnostics(session.snapshot()?, &diagnostics);
    let estimate = snapshot.estimate.clone();
    let converged = !cancelled
        && estimate
            .as_ref()
            .is_some_and(|estimate| estimate.meets(tolerance).unwrap_or(false));
    Ok(IntegrationReport {
        content_id: artifact.content_id.clone(),
        elapsed_seconds: started.elapsed().as_secs_f64(),
        converged,
        stopping_reason: if cancelled {
            "cancelled"
        } else if converged {
            "accuracy reached"
        } else {
            "work limit"
        }
        .into(),
        estimate,
        snapshot: stopped(snapshot, cancelled, converged),
        resume_status: ResumeStatus::CheckpointSaved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let state: Checkpoint =
                serde_json::from_slice(&fs::read(&checkpoint).unwrap()).unwrap();
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
}
