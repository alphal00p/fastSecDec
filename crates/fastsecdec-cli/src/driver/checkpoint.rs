use super::ResumeStatus;
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

pub(super) fn restore_checkpoint(
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

pub(super) fn save_mc_checkpoint(
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
