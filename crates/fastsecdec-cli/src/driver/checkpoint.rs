use super::{ResumeStatus, replay::AcceptedReplay};
use crate::{
    CliResult,
    artifact::{Artifact, atomic_write},
    config::IntegrationInput,
};
use fastsecdec::{
    integration::mc::HavanaSession,
    status::{EvaluationDiagnostics, IntegrationStage},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub(super) struct Checkpoint {
    format_version: u32,
    content_id: String,
    settings: serde_json::Value,
    pub(super) round_index: usize,
    session: serde_json::Value,
    #[serde(default)]
    diagnostics: EvaluationDiagnostics,
    replay: AcceptedReplay,
}

pub(super) struct RestoredCheckpoint {
    pub round_index: usize,
    pub diagnostics: EvaluationDiagnostics,
    pub replay: AcceptedReplay,
    pub session: Vec<u8>,
}

pub(super) fn settings_identity(settings: &IntegrationInput) -> CliResult<serde_json::Value> {
    let mut value = serde_json::to_value(settings)?;
    value.as_object_mut().unwrap().remove("workers");
    Ok(value)
}

pub(super) fn save_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
    round: usize,
    session: Vec<u8>,
    diagnostics: &EvaluationDiagnostics,
    replay: &AcceptedReplay,
) -> CliResult<()> {
    atomic_write(
        path,
        &serde_json::to_vec(&Checkpoint {
            format_version: 3,
            content_id: artifact.content_id.clone(),
            settings: settings_identity(settings)?,
            round_index: round,
            session: serde_json::from_slice(&session)?,
            diagnostics: diagnostics.clone(),
            replay: replay.clone(),
        })?,
    )
}

pub(super) fn restore_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
) -> CliResult<RestoredCheckpoint> {
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    if value
        .get("format_version")
        .and_then(serde_json::Value::as_u64)
        != Some(3)
    {
        return Err("unsupported checkpoint version; restart integration to create a weighted-replay checkpoint".into());
    }
    let checkpoint: Checkpoint = serde_json::from_value(value)?;
    let mut historical: IntegrationInput = serde_json::from_value(checkpoint.settings.clone())?;
    if checkpoint.settings.get("stability").is_none() {
        historical.stability = fastsecdec::kernel::StabilitySettings::validated();
    }
    if checkpoint.content_id != artifact.content_id
        || settings_identity(&historical)? != settings_identity(settings)?
        || checkpoint.round_index >= settings.max_rounds
    {
        return Err("checkpoint input identity or integration settings differ; only the worker count may change during resume".into());
    }
    Ok(RestoredCheckpoint {
        round_index: checkpoint.round_index,
        diagnostics: checkpoint.diagnostics,
        replay: checkpoint.replay,
        session: serde_json::to_vec(&checkpoint.session)?,
    })
}

pub(super) fn save_mc_checkpoint(
    path: &Path,
    artifact: &Artifact,
    settings: &IntegrationInput,
    round: usize,
    session: &HavanaSession,
    diagnostics: &EvaluationDiagnostics,
    replay: &AcceptedReplay,
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
        replay,
    )?;
    Ok(ResumeStatus::CheckpointSaved)
}
